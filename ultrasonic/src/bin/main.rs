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
use esp_hal::ledc::channel::ChannelIFace;
use esp_hal::ledc::timer::TimerIFace;
use esp_hal::main;
use esp_hal::rtc_cntl::Rtc;
// use esp_hal::time::{Duration, Instant};
use esp_hal::time::{Rate};
use log::info;

use esp_hal::ledc::{Ledc, LSGlobalClkSource, LowSpeed, timer, channel};
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig};

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

    let led = peripherals.GPIO10;
    let echo = peripherals.GPIO4;
    let trig = peripherals.GPIO20;

    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    lstimer0
        .configure(timer::config::Config { 
            duty: timer::config::Duty::Duty5Bit, 
            clock_source: timer::LSClockSource::APBClk, 
            frequency: Rate::from_khz(24) })
        .unwrap();

    let mut channel0 = ledc.channel(channel::Number::Channel0, led);
    channel0
        .configure(channel::config::Config { 
            timer: &lstimer0, 
            duty_pct: 10, 
            drive_mode: esp_hal::gpio::DriveMode::PushPull })
        .unwrap();

    let mut trig = Output::new(trig, Level::Low, OutputConfig::default());
    let echo = Input::new(echo, InputConfig::default().with_pull(esp_hal::gpio::Pull::Down));


    let delay = Delay::new();
    
    const THRESHOLD: f64 = 30.0;

    let rtc = Rtc::new(peripherals.LPWR);

    loop {
        delay.delay_millis(5);

        trig.set_low();
        delay.delay_micros(2);
        trig.set_high();
        delay.delay_micros(10);
        trig.set_low();

        while echo.is_low() {}
        let time1 = rtc.current_time_us();
        while echo.is_high() {}
        let time2 = rtc.current_time_us();
        let pulse_width = time2 - time1;

        let distance = (pulse_width as f64 * 0.0343) / 2.0;
        info!("Distance: {}", distance);

        let duty_pct: u8 = if distance < THRESHOLD {
            let ratio = (THRESHOLD - distance) / THRESHOLD;
            let p = (ratio * 100.0) as u8;
            p.min(100)
        } else {
            0
        };

        if let Err(e) = channel0.set_duty(duty_pct) {
            info!("Failed to set duty cycle: {:?}", e);
        }

        delay.delay_millis(60);
    }
}
