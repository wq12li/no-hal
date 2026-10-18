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
use esp_hal::delay::Delay;
use esp_hal::main;
use esp_hal::time::{Duration, Instant, Rate};
use log::{info, error};
use shtcx;

use esp_hal::gpio::{Output, OutputConfig, Level};
use hd44780_driver::bus::FourBitBusPins;
use hd44780_driver::{HD44780};
use hd44780_driver::memory_map::MemoryMap1602;
use hd44780_driver::setup::DisplayOptions4Bit;

use heapless::String;
use core::fmt::Write;

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

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // The following pins are used to bootstrap the chip. They are available
    // for use, but check the datasheet of the module for more information on them.
    // - GPIO2
    // - GPIO8
    // - GPIO9
    // These GPIO pins are in use by some feature of the module and should not be used.
    let _ = peripherals.GPIO11;
    let _ = peripherals.GPIO12;
    let _ = peripherals.GPIO13;
    let _ = peripherals.GPIO14;
    let _ = peripherals.GPIO15;
    let _ = peripherals.GPIO16;
    let _ = peripherals.GPIO17;

    let mut delay = Delay::new();

    let mut display = {
        let mut options = DisplayOptions4Bit::new(MemoryMap1602::new()).with_pins(FourBitBusPins {
            rs: Output::new(peripherals.GPIO0, Level::Low, OutputConfig::default()),
            en: Output::new(peripherals.GPIO1, Level::Low, OutputConfig::default()),
            d4: Output::new(peripherals.GPIO3, Level::Low, OutputConfig::default()),
            d5: Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default()),
            d6: Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default()),
            d7: Output::new(peripherals.GPIO6, Level::Low, OutputConfig::default()),
        });
    
        loop {
            match HD44780::new(options, &mut delay) {
                Err((options_back, error)) => {
                    error!("Error: {error}");
                    options = options_back;
                    delay.delay_millis(500);
                }
                Ok(display) => break display,
            }
        }
    };

    let mut sht = {
        let i2c_bus = esp_hal::i2c::master::I2c::new(
            peripherals.I2C0, 
            esp_hal::i2c::master::Config::default().with_frequency(Rate::from_khz(400)))
        .unwrap()
        .with_scl(peripherals.GPIO8)
        .with_sda(peripherals.GPIO7);
        
        shtcx::shtc3(i2c_bus)
    };

    {
        let device_id = sht.device_identifier().unwrap();
        let raw_id = sht.raw_id_register().unwrap();
    
        info!("device id: {}", device_id);
        info!("raw id: {}", raw_id);
    }

    loop {
        display.clear(&mut delay).unwrap();
        display.reset(&mut delay).unwrap();

        let measurement = sht.measure(shtcx::PowerMode::NormalMode, &mut delay).unwrap();

        let mut s: String<128> = String::new();
        write!(&mut s, "Temp.: {} F",
            measurement.temperature.as_degrees_celsius() * 9.0 / 5.0 + 32.0)
        .unwrap();
        display.write_str(&s, &mut delay).unwrap();
        
        display.set_cursor_xy((0,1), &mut delay).unwrap();
        let mut s: String<128> = String::new();
        write!(&mut s, "Humd.: {}%", measurement.humidity.as_percent()).unwrap();
        display.write_str(&s, &mut delay).unwrap();

        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(1000) {}
    }

}
