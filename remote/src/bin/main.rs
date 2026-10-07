#![no_std]
#![no_main]

use defmt::{info, warn};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::Peri;
use embassy_nrf::gpio::{AnyPin, Input, Level, Output, OutputDrive, Pull};
use embassy_nrf::radio::ieee802154::Radio;
use embassy_nrf::rng::Rng;
use embassy_nrf::saadc::{self, ChannelConfig, Saadc, VddhDiv5Input};
use embassy_nrf::{bind_interrupts, peripherals, radio, rng};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use panic_probe as _;
use remote::knoppen::{Keypad, Message};
use remote::radio::{Battery, Hfxo, Led, NrfLink, Sender};

bind_interrupts!(struct Irqs {
    RADIO => radio::InterruptHandler<peripherals::RADIO>;
    RNG => rng::InterruptHandler<peripherals::RNG>;
    SAADC => saadc::InterruptHandler;
});

static MESSAGES: Channel<CriticalSectionRawMutex, Message, 8> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    // Switches the board's 3.3 V output pin, which nothing is connected to.
    Output::new(p.P0_13, Level::Low, OutputDrive::Standard).persist();

    let button = |pin: Peri<'static, AnyPin>| Input::new(pin, Pull::Up);
    let keypad = Keypad::new([
        [
            button(p.P0_17.into()),
            button(p.P0_20.into()),
            button(p.P1_00.into()),
            button(p.P0_11.into()),
        ],
        [
            button(p.P0_22.into()),
            button(p.P0_24.into()),
            button(p.P1_04.into()),
            button(p.P1_06.into()),
        ],
    ]);

    let mut rng = Rng::new(p.RNG, Irqs);
    let mut first_sequence = [0];
    rng.fill_bytes(&mut first_sequence).await;

    let saadc = Saadc::new(
        p.SAADC,
        Irqs,
        saadc::Config::default(),
        [ChannelConfig::single_ended(VddhDiv5Input)],
    );
    let battery = Battery::new(saadc).await;
    let sender = Sender::new(NrfLink::new(Radio::new(p.RADIO, Irqs)), first_sequence[0]);
    let led = Led::new(Output::new(p.P0_15, Level::Low, OutputDrive::Standard));

    info!("Remote started");
    spawner.spawn(buttons_task(keypad).unwrap());
    spawner.spawn(radio_task(sender, battery, led).unwrap());
}

#[embassy_executor::task]
async fn buttons_task(mut keypad: Keypad<'static>) {
    loop {
        let message = keypad.next_message().await;
        info!(
            "Button: {} for blind {:#06x}",
            message.command, message.blind
        );
        MESSAGES.send(message).await;
    }
}

#[embassy_executor::task]
async fn radio_task(
    mut sender: Sender<NrfLink<'static>>,
    mut battery: Battery<'static>,
    mut led: Led<'static>,
) {
    loop {
        let message = MESSAGES.receive().await;
        let percent = battery.percent().await;

        let result = {
            let _hfxo = Hfxo::start();
            sender.send(message, percent).await
        };

        if let Err(error) = result {
            warn!("Sending {} failed: {}", message.command, error);
            led.blink().await;
        }
    }
}
