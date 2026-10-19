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

use esp_hal::gpio::{Output, OutputConfig, Level};
use hd44780_driver::bus::FourBitBusPins;
use hd44780_driver::{HD44780};
use hd44780_driver::memory_map::MemoryMap1602;
use hd44780_driver::setup::DisplayOptions4Bit;

use heapless::String;
use wonky_hal::delay_us;
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

    let mut i2c_bus = esp_hal::i2c::master::I2c::new(
        peripherals.I2C0, 
        esp_hal::i2c::master::Config::default().with_frequency(Rate::from_khz(400)))
    .unwrap()
    .with_scl(peripherals.GPIO8)
    .with_sda(peripherals.GPIO7);

    // (from section 5.1)
    const SHTC3_I2C_ADDRESS: u8 = 0x70;

    // Assume clock stretching is disabled and temperature is measured first (section 5.3).
    const SHTC3_READ_COMMAND: [u8; 2] = [0x78, 0x66];

    // (section 5.2)
    const SHTC3_SLEEP_COMMAND: [u8; 2] = [0xB0, 0x98];
    const SHTC3_WAKEUP_COMMAND: [u8; 2] = [0x35, 0x17];

    // Maximum power up time is 240 microseconds (section 3.1).
    const T_PU: u64 = 240;

    // Maximum measurement duration is 12.1 milliseconds (section 3.1).
    const T_MEAS: u64 = 12100;

    loop {
        display.clear(&mut delay).unwrap();
        display.reset(&mut delay).unwrap();
        
        i2c_bus.write(SHTC3_I2C_ADDRESS, &SHTC3_READ_COMMAND).unwrap();
        unsafe { delay_us(T_MEAS); }

        // A read produces six bytes of info (section 5.6).
        // First two bytes are temperature, third is CRC; 4/5 are humidity, 6th is CRC.
        // (assuming temperature is measured first)
        let mut buf = [0; 6];
        i2c_bus.read(SHTC3_I2C_ADDRESS, &mut buf).unwrap();

        // Ignore the CRC check for simplicity.
        // Convert output based on formulas in 5.11.
        let temp = u16::from_be_bytes([buf[0], buf[1]]);
        let hum = u16::from_be_bytes([buf[3], buf[4]]);

        let temp = temp as f32 * 175.0 / (1 << 16) as f32 - 45.0;
        let hum = hum as f32 * 100.0 / (1 << 16) as f32;
        
        let mut s: String<128> = String::new();
        write!(&mut s, "Temp.: {:.1} C", temp).unwrap();
        display.write_str(&s, &mut delay).unwrap();
        
        display.set_cursor_xy((0,1), &mut delay).unwrap();
        let mut s: String<128> = String::new();
        write!(&mut s, "Humd.: {:.1}%", hum).unwrap();
        display.write_str(&s, &mut delay).unwrap();

        unsafe { delay_us(1_000_000); }
    }

}
