#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

#[cfg(all(feature = "radio", feature = "mqtt"))]
compile_error!(
    "esp-radio can't run Wi-Fi and IEEE 802.15.4 at the same time, enable either `radio` or `mqtt`"
);

use defmt::{error, info, warn};
use driver::motor::tmc2209::Tmc2209;
use driver::motor::{Blind, Stepper};
use driver::mqtt::Status;
use driver::opslag::Storage;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::signal::Signal;
use embassy_time::Duration;
use esp_bootloader_esp_idf::partitions::{
    self, DataPartitionSubType, FlashRegion, PARTITION_TABLE_MAX_LEN, PartitionType,
};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::peripherals::FLASH;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::uart::{self, Uart};
use esp_storage::FlashStorage;
use panic_rtt_target as _;
use protocol::Command;
use static_cell::{ConstStaticCell, StaticCell};

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

const BLIND_ID: u8 = match env!("BLIND_ID").as_bytes() {
    b"1" => 1,
    b"2" => 2,
    _ => panic!("BLIND_ID must be 1 or 2"),
};

const RUN_CURRENT_MA: u16 = 800;
const MICROSTEPS: u16 = 16;
/// About one revolution per second, with a 200 step motor at 16 microsteps.
const STEP_INTERVAL: Duration = Duration::from_micros(300);

type Flash = FlashRegion<'static, FlashStorage<'static>>;
type BlindModule = Blind<Tmc2209<'static>, Input<'static>, Flash>;

static COMMANDS: Channel<CriticalSectionRawMutex, Command, 4> = Channel::new();
static STATUS: Signal<CriticalSectionRawMutex, Status> = Signal::new();
static BATTERY: Signal<CriticalSectionRawMutex, u8> = Signal::new();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // EN high keeps the motor without current, and lets the chain move by hand (FE7).
    let enable = Output::new(peripherals.GPIO20, Level::High, OutputConfig::default());

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 65536);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    info!("Blind module {} starting", BLIND_ID);

    let uart = Uart::new(
        peripherals.UART1,
        uart::Config::default().with_baudrate(115_200),
    )
    .unwrap()
    .with_tx(peripherals.GPIO22)
    .with_rx(peripherals.GPIO23)
    .into_async();
    let step = Output::new(peripherals.GPIO18, Level::Low, OutputConfig::default());
    let dir = Output::new(peripherals.GPIO19, Level::Low, OutputConfig::default());
    let endstop = Input::new(
        peripherals.GPIO21,
        InputConfig::default().with_pull(Pull::Up),
    );

    let mut driver = Tmc2209::new(uart, step, dir, enable, STEP_INTERVAL);
    match driver.configure(RUN_CURRENT_MA, MICROSTEPS).await {
        Ok(()) => info!("TMC2209 configured"),
        Err(error) => error!("TMC2209 configuration failed: {}", error),
    }
    driver.disable();

    let blind = Blind::new(
        driver,
        endstop,
        Storage::new(nvs_partition(peripherals.FLASH)),
    );
    info!("Loaded positions {}", blind.positions());
    STATUS.signal(blind.status());
    spawner.spawn(motor_task(blind).unwrap());

    #[cfg(feature = "radio")]
    radio::start(&spawner, peripherals.IEEE802154);

    #[cfg(feature = "mqtt")]
    mqtt::start(&spawner, peripherals.WIFI);

    loop {
        core::future::pending::<()>().await;
    }
}

/// The positions are stored in the NVS partition, which nothing else uses.
fn nvs_partition(flash: FLASH<'static>) -> Flash {
    static FLASH: StaticCell<FlashStorage> = StaticCell::new();
    static TABLE: ConstStaticCell<[u8; PARTITION_TABLE_MAX_LEN]> =
        ConstStaticCell::new([0; PARTITION_TABLE_MAX_LEN]);

    let flash = FLASH.init(FlashStorage::new(flash));
    let table =
        partitions::read_partition_table(flash, TABLE.take()).expect("Unreadable partition table");
    table
        .find_partition(PartitionType::Data(DataPartitionSubType::Nvs))
        .expect("Unreadable partition table")
        .expect("No NVS partition")
        .as_embedded_storage(flash)
}

#[embassy_executor::task]
async fn motor_task(mut blind: BlindModule) {
    let mut last_status = blind.status();
    loop {
        let result = if blind.is_moving() {
            let event = select(COMMANDS.receive(), blind.step()).await;
            match event {
                Either::First(command) => {
                    info!("Command {}", command);
                    blind.handle(command)
                }
                Either::Second(result) => result,
            }
        } else {
            let command = COMMANDS.receive().await;
            info!("Command {}", command);
            blind.handle(command)
        };

        if let Err(error) = result {
            warn!("Saving the positions failed: {}", error);
        }

        let status = blind.status();
        if status != last_status {
            last_status = status;
            STATUS.signal(status);
        }
    }
}

#[cfg(feature = "radio")]
mod radio {
    use defmt::info;
    use driver::radio::Receiver;
    use embassy_executor::Spawner;
    use esp_hal::peripherals::IEEE802154;
    use esp_radio::ieee802154::Ieee802154;
    use protocol::BLIND_ADDRESSES;

    use super::{BATTERY, BLIND_ID, COMMANDS};

    pub fn start(spawner: &Spawner, radio: IEEE802154<'static>) {
        let address = BLIND_ADDRESSES[usize::from(BLIND_ID) - 1];
        let receiver = Receiver::new(Ieee802154::new(radio), address);
        info!("Listening on 802.15.4 address {:#06x}", address);
        spawner.spawn(radio_task(receiver).unwrap());
    }

    #[embassy_executor::task]
    async fn radio_task(mut receiver: Receiver<'static>) {
        loop {
            let frame = receiver.receive().await;
            info!("Remote sent {} (battery {}%)", frame.command, frame.battery);
            BATTERY.signal(frame.battery);
            COMMANDS.send(frame.command).await;
        }
    }
}

#[cfg(feature = "mqtt")]
mod mqtt {
    use core::net::Ipv4Addr;

    use defmt::{info, warn};
    use driver::mqtt::{Buffers, Credentials, Error, HomeAssistant, KEEP_ALIVE_SECS};
    use embassy_executor::Spawner;
    use embassy_futures::select::{Either4, select4};
    use embassy_net::tcp::TcpSocket;
    use embassy_net::{Runner, Stack, StackResources};
    use embassy_time::{Duration, Ticker, Timer};
    use esp_hal::peripherals::WIFI;
    use esp_hal::rng::Rng;
    use esp_radio::wifi::sta::StationConfig;
    use esp_radio::wifi::{self, Config, Interface, WifiController};
    use static_cell::ConstStaticCell;

    use super::{BATTERY, BLIND_ID, COMMANDS, STATUS};

    const WIFI_SSID: &str = env!("WIFI_SSID");
    const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");
    const MQTT_BROKER: &str = env!("MQTT_BROKER");
    const MQTT_PORT: u16 = 1883;
    const CREDENTIALS: Credentials = Credentials {
        username: non_empty(env!("MQTT_USERNAME")),
        password: non_empty(env!("MQTT_PASSWORD")),
    };
    const RETRY_DELAY: Duration = Duration::from_secs(5);

    const fn non_empty(text: &'static str) -> Option<&'static str> {
        if text.is_empty() { None } else { Some(text) }
    }

    pub fn start(spawner: &Spawner, wifi: WIFI<'static>) {
        static RESOURCES: ConstStaticCell<StackResources<3>> =
            ConstStaticCell::new(StackResources::new());

        let (controller, interfaces) =
            wifi::new(wifi, Default::default()).expect("Failed to initialize Wi-Fi controller");

        let rng = Rng::new();
        let seed = u64::from(rng.random()) << 32 | u64::from(rng.random());
        let (stack, runner) = embassy_net::new(
            interfaces.station,
            embassy_net::Config::dhcpv4(Default::default()),
            RESOURCES.take(),
            seed,
        );

        spawner.spawn(wifi_task(controller).unwrap());
        spawner.spawn(net_task(runner).unwrap());
        spawner.spawn(mqtt_task(stack).unwrap());
    }

    #[allow(
        clippy::large_stack_frames,
        reason = "the state of a task's future is stored statically by the executor"
    )]
    #[embassy_executor::task]
    async fn wifi_task(mut controller: WifiController<'static>) {
        let config = Config::Station(
            StationConfig::default()
                .with_ssid(WIFI_SSID)
                .with_password(WIFI_PASSWORD.into()),
        );
        if let Err(error) = controller.set_config(&config) {
            warn!("Invalid Wi-Fi config: {}", error);
            return;
        }

        loop {
            match controller.connect_async().await {
                Ok(_) => {
                    info!("Wi-Fi connected to {}", WIFI_SSID);
                    let _ = controller.wait_for_disconnect_async().await;
                    warn!("Wi-Fi disconnected");
                }
                Err(error) => warn!("Wi-Fi connection failed: {}", error),
            }
            Timer::after(RETRY_DELAY).await;
        }
    }

    #[embassy_executor::task]
    async fn net_task(mut runner: Runner<'static, Interface<'static>>) {
        runner.run().await
    }

    #[embassy_executor::task]
    async fn mqtt_task(stack: Stack<'static>) {
        let broker: Ipv4Addr = MQTT_BROKER
            .parse()
            .expect("MQTT_BROKER must be an IPv4 address");
        static SOCKET_RX: ConstStaticCell<[u8; 1024]> = ConstStaticCell::new([0; 1024]);
        static SOCKET_TX: ConstStaticCell<[u8; 1024]> = ConstStaticCell::new([0; 1024]);
        static MQTT_BUFFERS: ConstStaticCell<Buffers> = ConstStaticCell::new(Buffers::new());
        let rx_buffer = SOCKET_RX.take();
        let tx_buffer = SOCKET_TX.take();
        let buffers = MQTT_BUFFERS.take();

        loop {
            stack.wait_config_up().await;

            let mut socket = TcpSocket::new(stack, rx_buffer, tx_buffer);
            socket.set_timeout(Some(Duration::from_secs(u64::from(KEEP_ALIVE_SECS) * 2)));
            match socket.connect((broker, MQTT_PORT)).await {
                Ok(()) => {
                    info!("Connected to the MQTT broker");
                    if let Err(error) = run_session(socket, buffers).await {
                        warn!("MQTT session ended: {}", error);
                    }
                }
                Err(error) => warn!("MQTT broker unreachable: {}", error),
            }
            Timer::after(RETRY_DELAY).await;
        }
    }

    #[allow(
        clippy::large_stack_frames,
        reason = "the state of a task's future is stored statically by the executor"
    )]
    async fn run_session(
        socket: TcpSocket<'_>,
        buffers: &mut Buffers,
    ) -> Result<(), Error<embassy_net::tcp::Error>> {
        let mut home_assistant =
            HomeAssistant::connect(socket, buffers, BLIND_ID, &CREDENTIALS).await?;
        home_assistant.announce().await?;

        let mut ping = Ticker::every(Duration::from_secs(u64::from(KEEP_ALIVE_SECS) / 2));
        loop {
            let event = select4(
                home_assistant.next_command(),
                STATUS.wait(),
                BATTERY.wait(),
                ping.next(),
            )
            .await;
            match event {
                Either4::First(command) => COMMANDS.send(command?).await,
                Either4::Second(status) => home_assistant.publish(status).await?,
                Either4::Third(percent) => home_assistant.publish_battery(percent).await?,
                Either4::Fourth(()) => home_assistant.ping().await?,
            }
        }
    }
}
