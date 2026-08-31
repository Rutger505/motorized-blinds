#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::mcpwm::operator::PwmPinConfig;
use esp_hal::time::{Duration, Instant};
use esp_println::println;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}



// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    // generator version: 1.3.0
    // generator parameters: --chip esp32 -o unstable-hal -o alloc -o wifi

    let peripherals = esp_hal::init(esp_hal::Config::default());

    // esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);
    let mut led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());
    loop {
        led.toggle();
        println!("HI!");

        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}
    }

    // esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);
    //
    // let timg0 = TimerGroup::new(peripherals.TIMG0);
    // let sw_interrupt =
    //     esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    // esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);
    //
    // let wifi = peripherals.WIFI;
    // let (controller, interfaces) = esp_radio::wifi::new(wifi, Default::default()).unwrap();
    //
    // let mut esp_now = interfaces.esp_now();
    // esp_now.set_channel(11).unwrap();

    // let (mut _wifi_controller, _interfaces) =
    //     esp_radio::wifi::new(peripherals.WIFI, Default::default())
    //         .expect("Failed to initialize Wi-Fi controller");

    // loop {
        //     let delay_start = Instant::now();
        //     while delay_start.elapsed() < Duration::from_millis(500) {}
    // }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.1.0/examples
}
