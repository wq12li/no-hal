#![no_std]
#![no_main]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

use wonky_hal::*;
use esp_hal::__macro_implementation::__entry;
// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[__entry]
fn main() -> ! {
    init();

    let led = 10;

    set_enable(led);
    pad_driver(led);
    init_io_mux(led);
    set_high(led);

    loop {
        set_high(led);
        delay_us(500_000);
        set_low(led);
        delay_us(500_000);
    }

}