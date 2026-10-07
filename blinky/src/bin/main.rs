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
use esp_hal::time::{Duration, Instant};
use log::info;

use esp_hal::gpio::{Output, Level, OutputConfig};

const GPIO_BASE: usize = 0x6000_4000;

const OUT_REG: usize = 0x04;
const OUT_W1TS: usize = 0x08;
const OUT_W1TC: usize = 0x0C;


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

    let mut _led = Output::new(peripherals.GPIO10, Level::High, OutputConfig::default());

    // esp32c3::gpio::PIN::modify(&self, f)

    // esp32c3::generic::Reg

    // esp32c3::gpio::out::DATA_ORIG_R::bits(&self);
    // esp32c3::gpio::out_w1ts::W.out_w1ts();

    // type OUT_W1TS = esp32c3::generic::Reg<esp32c3::gpio::out_w1ts::OUT_W1TS_SPEC>;

    // let hmm = OUT_W1TS::

    // let mut a = esp32c3::gpio::out_w1ts::OUT_W1TS_SPEC;
    // let mut b = esp32c3::gpio::out_w1ts::W::out_w1ts(&mut self);
    // a.reset();

    let out = (GPIO_BASE + OUT_REG) as *mut u32;
    let w1ts = (GPIO_BASE + OUT_W1TS) as *mut u32;
    let w1tc = (GPIO_BASE + OUT_W1TC) as *mut u32;

    // unsafe {
    //     w1ts.write_volatile(1 << 10);
    //     let v = out.read_volatile();
    //     info!("v: {:#x}", v);
    //     w1tc.write_volatile(1 << 10);
    //     let v = out.read_volatile();
    //     info!("v: {:#x}", v);

    // }

    loop {
        info!("Hello world!");
        unsafe {
            w1ts.write_volatile(1 << 10);
        }
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}

        unsafe {
            w1tc.write_volatile(1 << 10);
        }

        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}

    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.1.0/examples
}
