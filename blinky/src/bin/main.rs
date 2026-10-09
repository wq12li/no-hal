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

const GPIO_BASE: usize = 0x6000_4000;
const IO_MUX_BASE: usize = 0x6000_9000;

const OUT_REG: usize = 0x04;
const OUT_W1TS: usize = 0x08;
const OUT_W1TC: usize = 0x0C;

const ENABLE_W1TS: usize = 0x24;

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

    let led = 10;

    unsafe {
        set_enable(led);
        pad_driver(led);
        init_io_mux(led);
        set_high(led);
    }

    loop {
        info!("Hello world!");
        unsafe { set_high(led); }
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}

        unsafe { set_low(led); }
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}
    }

}

unsafe fn set_enable(gpio: usize) {
    // Normally configuring GPIO_FUNCx_OUT_SEL_CFG_REG might be needed here,
    // but not for "simple GPIO output" (5.5.3).
    let reg = (GPIO_BASE + ENABLE_W1TS) as *mut u32;
    unsafe { reg.write_volatile(1 << gpio); }
}

// Probably unnecessary, because it looks like 0 is the default?
unsafe fn pad_driver(gpio: usize) {
    let reg = (GPIO_BASE + 0x74 + 4 * gpio) as *mut u32;
    unsafe {reg.write_volatile(0);}
}

unsafe fn init_io_mux(gpio: usize) {
    let io_mux_gpio_reg: usize = IO_MUX_BASE + 0x04 + 4 * gpio;
    let reg = (io_mux_gpio_reg) as *mut u32;
    
    // https://documentation.espressif.com/esp32-c3_technical_reference_manual_en.pdf
    // Apparently for GPIOs 2, 3, 5, 18, 19, it's
    // 0: 5 mA
    // 1: 20 mA
    // 2: 10 mA
    // 3: 40 mA
    //
    // But for other GPIOs it's
    // 0: 5 mA
    // 1: 10 mA
    // 2: 20 mA
    // 3: 40 mA
    
    // Turn off FUN_WPD (bit 7), FUN_WPU (bit 8), and set drive strength (10-11)
    unsafe {reg.write_volatile(2 << 10);}
}

unsafe fn read_gpio(gpio: usize) -> bool {
    let out = (GPIO_BASE + OUT_REG) as *mut u32;
    unsafe { out.read_volatile() & (1 << gpio) != 0 }
}

unsafe fn set_high(gpio: usize) {
    let w1ts = (GPIO_BASE + OUT_W1TS) as *mut u32;
    unsafe { w1ts.write_volatile(1 << gpio); }
}

unsafe fn set_low(gpio: usize) {
    let w1tc = (GPIO_BASE + OUT_W1TC) as *mut u32;
    unsafe { w1tc.write_volatile(1 << gpio); }
}