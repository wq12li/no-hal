#![no_std]

const GPIO_BASE: usize = 0x6000_4000;
const IO_MUX_BASE: usize = 0x6000_9000;
const SYSTIMER_BASE: usize = 0x6002_3000;

const OUT_REG: usize = 0x04;
const OUT_W1TS: usize = 0x08;
const OUT_W1TC: usize = 0x0C;

const ENABLE_W1TS: usize = 0x24;

pub unsafe fn init() {
    // Skipping some stuff here from the HAL; presumably it's to be more generic
    // but they don't seem relevant to the esp32c3.
    // - pre_init() call; is empty for esp32c3
    // - branch predictor init
    // - something about making sure the stack pointer is in range?
    //   - don't quite understand this one but it seems more like an assertion than a thing that does something
    // - stack guard monitoring
    //   - sets a breakpoint, apparently
    // 
}

// This is slightly more hairy than expected because the
// counter might wrap around.
// Not sure if this is really as robust as the HAL version.
pub unsafe fn delay_ticks(ticks: u64) {
    let mut start = unsafe { read_unit0_count() };
    let mut end = start.wrapping_add(ticks);

    if start > end { core::mem::swap(&mut start, &mut end);}

    loop {
        let current = unsafe { read_unit0_count() };
        if current >= end {
            return;
        }
    }
}

pub unsafe fn read_unit0_count() -> u64 {
    // man why is the clock stuff so arcane

    // Section 10.5
    const SYSTIMER_TIMER_UNIT0_OP_REG: usize = SYSTIMER_BASE + 0x04;
    let reg = SYSTIMER_TIMER_UNIT0_OP_REG as *mut u32;

    // "update": reads count into VALUE_HI and VALUE_LO
    unsafe { reg.write_volatile(1 << 30); }

    // "valid": poll until true, meaning the update finished
    loop {
        let valid = unsafe { reg.read_volatile() & (1 << 29) != 0 };
        if valid { break; }
    }
    
    // NOTE: The HAL accomodates for an edge case here where an interrupt happens
    // between reading hi and lo, but lazy.
    let hi = (SYSTIMER_BASE + 0x40) as *mut u32;
    let lo = (SYSTIMER_BASE + 0x44) as *mut u32;

    let hi = unsafe { hi.read_volatile() };
    let lo = unsafe { lo.read_volatile() };

    (hi as u64) << 32 | lo as u64
}

pub unsafe fn set_enable(gpio: usize) {
    // Normally configuring GPIO_FUNCx_OUT_SEL_CFG_REG might be needed here,
    // but not for "simple GPIO output" (5.5.3).
    let reg = (GPIO_BASE + ENABLE_W1TS) as *mut u32;
    unsafe { reg.write_volatile(1 << gpio); }
}

// Probably unnecessary, because it looks like 0 is the default?
pub unsafe fn pad_driver(gpio: usize) {
    let reg = (GPIO_BASE + 0x74 + 4 * gpio) as *mut u32;
    unsafe { reg.write_volatile(0); }
}

pub unsafe fn init_io_mux(gpio: usize) {
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

pub unsafe fn read_gpio(gpio: usize) -> bool {
    let out = (GPIO_BASE + OUT_REG) as *mut u32;
    unsafe { out.read_volatile() & (1 << gpio) != 0 }
}

pub unsafe fn set_high(gpio: usize) {
    let w1ts = (GPIO_BASE + OUT_W1TS) as *mut u32;
    unsafe { w1ts.write_volatile(1 << gpio); }
}

pub unsafe fn set_low(gpio: usize) {
    let w1tc = (GPIO_BASE + OUT_W1TC) as *mut u32;
    unsafe { w1tc.write_volatile(1 << gpio); }
}