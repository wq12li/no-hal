#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::main;

use wonky_hal::*;

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
    // generator parameters: --chip esp32c3 -o esp32c3-mini-1 -o unstable-hal -o log -o esp-backtrace

    // HAL stuff that hasn't been successfully replaced yet.
    {
        esp_println::logger::init_logger_from_env();
        let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
        let _peripherals = esp_hal::init(config);
    }

    let led = 10;

    unsafe {
        set_enable(led);
        pad_driver(led);
        init_io_mux(led);
        set_high(led);
    }

    loop {
        unsafe {
            set_high(led);
            delay_us(500_000);
            set_low(led);
            delay_us(500_000);
        }
    }

}