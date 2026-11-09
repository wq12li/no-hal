#![no_std]

const LEDC_LS_SIG_OUT0: u32 = 45;

// GPIO register block.
const GPIO_BASE: usize = 0x6000_4000;
    const GPIO_OUT_REG: usize = 0x04;
    const GPIO_OUT_W1TS: usize = 0x08;
    const GPIO_OUT_W1TC: usize = 0x0C;
    const GPIO_ENABLE_W1TS: usize = 0x24;
    const GPIO_PIN_N_REG: usize = 0x74; // + 4*pin
    const GPIO_FUNC_N_OUT_SEL_CFG_REG: usize = 0x0554;

// IO mux register block.
const IO_MUX_BASE: usize = 0x6000_9000;

// LED PWM controller register block.
const LED_PWM_BASE: usize = 0x6001_9000;
    const LEDC_TIMER0_CONF_REG: usize = 0xA0;
        const LEDC_TIMER0_DUTY_RES: usize = 0;
        const LEDC_CLK_DIV_TIMER0: usize = 4;
        const LEDC_TIMER0_PAUSE: usize = 22;
        const LEDC_TIMER0_RST: usize = 23;
        const LEDC_TIMER0_PARA_UP: usize = 25;
    const LEDC_CH0_CONF0_REG: usize = 0x00;
        const LEDC_TIMER_SEL_CH0: usize = 0;
        const LEDC_SIG_OUT_EN_CH0: usize = 2;
        const LEDC_PARA_UP_CH0: usize = 4;
    const LEDC_CH0_CONF1_REG: usize = 0x0C;
        const LEDC_DUTY_SCALE_CH0: usize = 0;
        const LEDC_DUTY_CYCLE_CH0: usize = 10;
        const LEDC_DUTY_NUM_CH0: usize = 20;
        const LEDC_DUTY_INC_CH0: usize = 30;
        const LEDC_DUTY_START_CH0: usize = 31;
    const LEDC_CONF_REG: usize = 0xD0;
        // const LEDC_APB_CLK_SEL: usize = 0;
    const LEDC_CH0_HPOINT_REG: usize = 0x04; 
    const LEDC_CH0_DUTY_REG: usize = 0x08; 
    const LEDC_INT_RAW_REG: usize = 0xC0;
        const LEDC_DUTY_CHNG_END_CH0_INT_RAW: usize = 4;
    const LEDC_INT_CLR_REG: usize = 0xCC; 

// System timer register block.
const SYSTIMER_BASE: usize = 0x6002_3000;

// System register block.
const SYSTEM_BASE: usize = 0x600C_0000;
    const SYSTEM_PERIP_CLK_EN0_REG: usize = 0x10;
        const SYSTEM_LEDC_CLK_EN: usize = 11; // Bit index.

    const SYSTEM_PERIP_RST_EN0_REG: usize = 0x18;
        const SYSTEM_LEDC_RST: usize = 11; // Bit index.

// Low-power management, like RTC and stuff.
const LPWR_BASE: usize = 0x6000_8000;
    const RTC_CNTL_WDTCONFIG0_REG: usize = 0x90;
        const RTC_CNTL_WDT_EN: usize = 31;
    const RTC_CNTL_WDTWPROTECT_REG: usize = 0xA8;
    const RTC_CNTL_SWD_CONF_REG: usize = 0xAC;
        const RTC_CNTL_SWD_AUTO_FEED_EN: usize = 31;
    const RTC_CNTL_SWD_WPROTECT_REG: usize = 0xB0;

// Timer groups.
const TIMG0_BASE: usize = 0x6001_F000;
const TIMG1_BASE: usize = 0x6002_0000;
    // These offsets are the same across both timer groups.
    const TIMG_WDTWPROTECT_REG: usize = 0x64;
    const TIMG_WDTCONFIG0_REG: usize = 0x48;
        const TIMG_WDT_EN: usize = 31;

pub unsafe fn fade(start: u32, inc: bool, cycle: u32, scale: u32, num: u32) {
    let reg = (LED_PWM_BASE + LEDC_CH0_DUTY_REG) as *mut u32;
    unsafe { reg.write_volatile(start << 4); }

    // Clear DUTY_CHNG_END for channel 0.
    let reg = (LED_PWM_BASE + LEDC_INT_CLR_REG) as *mut u32;
    unsafe { reg.write_volatile(1 << 4); }

    // Set up fade via CONF1.
    let reg = (LED_PWM_BASE + LEDC_CH0_CONF1_REG) as *mut u32;
    let val =
        1 << LEDC_DUTY_START_CH0
        | cycle << LEDC_DUTY_CYCLE_CH0
        | (inc as u32) << LEDC_DUTY_INC_CH0
        | scale << LEDC_DUTY_SCALE_CH0
        | num << LEDC_DUTY_NUM_CH0;
    unsafe { reg.write_volatile(val); }

    // Update via CONF0.
    let reg = (LED_PWM_BASE + LEDC_CH0_CONF0_REG) as *mut u32;
    let current = unsafe { reg.read_volatile() };
    unsafe { reg.write_volatile(current | (1 << LEDC_PARA_UP_CH0)); }
}

pub unsafe fn is_fading() -> bool {
    let reg = (LED_PWM_BASE + LEDC_INT_RAW_REG) as *mut u32;
    let r = unsafe { reg. read_volatile() };
    r & (1 << LEDC_DUTY_CHNG_END_CH0_INT_RAW) == 0
}

pub unsafe fn set_duty(duty: u32) {
    // Shift by 4 because the low 3 bits are for fractional values.
    let reg = (LED_PWM_BASE + LEDC_CH0_DUTY_REG) as *mut u32;
    unsafe { reg.write_volatile(duty << 4); }

    // Refer to "start_duty_without_fading" from HAL.
    let reg = (LED_PWM_BASE + LEDC_CH0_CONF1_REG) as *mut u32;
    let val =
        1 << LEDC_DUTY_START_CH0
        | 1 << LEDC_DUTY_INC_CH0
        | 1 << LEDC_DUTY_NUM_CH0
        | 1 << LEDC_DUTY_CYCLE_CH0
        | 0 << LEDC_DUTY_SCALE_CH0;
    unsafe { reg.write_volatile(val); }

    // Update.
    let reg = (LED_PWM_BASE + LEDC_CH0_CONF0_REG) as *mut u32;
    let val = unsafe { reg.read_volatile() };
    unsafe { reg.write_volatile(val | (1 << LEDC_PARA_UP_CH0)); }
}

// Uses Timer0 and PWM channel 0.
pub unsafe fn led_pwm_init(divisor: u32, duty_res: u32, duty: u32) {
    // "Before using any of these peripherals, it is mandatory to enable the clock for the given peripheral and
    // release the peripheral from reset state"
    //
    //   - 16.3.5, pg 430

    // Set enable.
    let reg = (SYSTEM_BASE + SYSTEM_PERIP_CLK_EN0_REG) as *mut u32;
    let current = unsafe { reg.read_volatile() };
    unsafe { reg.write_volatile(current | ( 1 << SYSTEM_LEDC_CLK_EN )); }

    // Reset: Turn bit on then off.
    let reg = (SYSTEM_BASE + SYSTEM_PERIP_RST_EN0_REG) as *mut u32;
    let current = unsafe { reg.read_volatile() };
    unsafe { reg.write_volatile(current | ( 1 << SYSTEM_LEDC_RST )); }
    unsafe { reg.write_volatile(current & !( 1 << SYSTEM_LEDC_RST )); }

    // Set clock source signal to APB_CLK.
    let reg = (LED_PWM_BASE + LEDC_CONF_REG) as *mut u32;
    unsafe { reg.write_volatile(1); }

    // Configure Timer0.
    let reg = (LED_PWM_BASE + LEDC_TIMER0_CONF_REG) as *mut u32;
    let val =
        1 << LEDC_TIMER0_PARA_UP
        | 0 << LEDC_TIMER0_RST
        | 0 << LEDC_TIMER0_PAUSE
        | divisor << LEDC_CLK_DIV_TIMER0
        | duty_res << LEDC_TIMER0_DUTY_RES;
    unsafe { reg.write_volatile(val); }

    // Configure PWM0.
    {
        // The HAL sets HPOINT to 0 I guess.
        let reg = (LED_PWM_BASE + LEDC_CH0_HPOINT_REG) as *mut u32;
        unsafe { reg.write_volatile(0); }
        
        // Shift by 4 because the low 3 bits are for fractional values.
        let reg = (LED_PWM_BASE + LEDC_CH0_DUTY_REG) as *mut u32;
        unsafe { reg.write_volatile(duty << 4); }

        // Refer to "start_duty_without_fading" from HAL.
        let reg = (LED_PWM_BASE + LEDC_CH0_CONF1_REG) as *mut u32;
        let val =
            1 << LEDC_DUTY_START_CH0
            | 1 << LEDC_DUTY_INC_CH0
            | 1 << LEDC_DUTY_NUM_CH0
            | 1 << LEDC_DUTY_CYCLE_CH0
            | 0 << LEDC_DUTY_SCALE_CH0;
        unsafe { reg.write_volatile(val); }

        let reg = (LED_PWM_BASE + LEDC_CH0_CONF0_REG) as *mut u32;
        let val =
            0 << LEDC_TIMER_SEL_CH0     // Set to Timer0.
            | 1 << LEDC_SIG_OUT_EN_CH0  // Enable output signal. Need to connect to GPIO...
            | 1 << LEDC_PARA_UP_CH0;    // This bit updates most things in HPOINT, CONF0/1, and DUTY.
        unsafe { reg.write_volatile(val); }
    }
}

// To the 0th channel, specifically.
pub unsafe fn connect_gpio_to_ledc(gpio: usize) {
    let reg = (GPIO_BASE + GPIO_FUNC_N_OUT_SEL_CFG_REG + 4 * gpio) as *mut u32;
    unsafe { reg.write_volatile(LEDC_LS_SIG_OUT0); }
}

// what is that init function doing
pub fn init() {
    // The HAL starts with a check for stack pointer bounds, which I don't think I've ever seen triggered,
    // but might as well.
    unsafe extern "C" {
        static _stack_end_cpu0: u32;
        static _stack_start_cpu0: u32;
    }
    let current_sp: usize;
    unsafe { core::arch::asm!("mv {0}, sp", out(reg) current_sp); }
    assert!(current_sp > &raw const _stack_end_cpu0 as usize
         && current_sp < &raw const _stack_start_cpu0 as usize);
    
    // Sets watchpoint (breakpoint?)
    // This is pretty much a wrapper around debugger::set_stack_watchpoint(),
    // but called with an argument __stack_chk_guard, which seems to be a stack canary bound to a variable from C:
    // https://gcc.gnu.org/onlinedocs/gcc-4.2.1//gccint/Stack-Smashing-Protection.html
    //
    // The watchpoint setting in the function is done with inline assembly through the csrw instruction, which writes
    // to the CSR (control and status) registers and I guess does stuff that makes it work.
    // https://www2.eecs.berkeley.edu/Pubs/TechRpts/2015/Archive/EECS-2015-49.pdf
    //
    // Also, the implementation talks about a "NAPOT_MATCH", where NAPOT seems to stand for naturally-aligned powers of two
    // (https://danielmangum.com/risc-v-tips/2021-11-18-pmp-naturally-aligned-powers-of-two/), but I think that's enough
    // of a rabbit hole for now.
    //
    // _Probably_ it's not needed for this wonky HAL which is super developer-unfriendly already.
    //
    // crate::soc::enable_main_stack_guard_monitoring();

    // Disables every peripheral by clearing their corresponding EN bits in the SYSTEM_PERIP_CLK_EN0_REG register,
    // _except_ for SYSTIMER, TIMG0, UART0, UART_MEM, and USB_DEVICE.
    // Said bits are SYSTEM_SYSTIMER_CLK_EN, SYSTEM_TIMERGROUP_CLK_EN, SYSTEM_UART_CLK_EN, SYSTEM_UART_MEM_CLK_EN, and SYSTEM_USB_DEVICE_CLK_EN.
    //
    // Normally, enabling/disabling of peripherals is done in accordance with a reference counter for each peripheral.
    // Some runtime checks are done to ensure that the reference count can't go below 0.
    // The bit-setting is normally unsafe because of potential race conditions, but this is guarded against by a
    // non-reentrant mutex that panics if it's called reentrantly.
    //
    // For some reason the disabling done in this function is done "forced", in that it does not update the reference counts
    // (other than with the five peripherals that are kept enabled, which get a +1).
    // I guess this is because at the beginning of the program when the ref counting values are set up, you don't know
    // whether or not the peripherals are enabled yet or not, and all the refcounts are 0. So you disable everything to make
    // sure that your refcounting is consistent with the actual state of the system.
    //
    // Anyway presumably the reason for all this disabling is dynamic power consumption (https://anysilicon.com/the-ultimate-guide-to-clock-gating/),
    // but I guess I don't have to worry about all that in this abomination, so it's probably ok to just leave
    // all peripherals receiving those clock signals between programs.
    //
    // system::disable_peripherals();

    // The take() function seems to just exist for safety reasons, to guarantee that
    // there is only one instance of the peripherals. Calling it involves a lock and
    // a static mut bool that makes sure it's not called twice.
    // 
    // Internally it calls steal() which is the same thing but just lets you have multiple
    // copies of things the good old fashioned way.
    //
    // Since we are choosing to live life dangerously it doesn't do anything useful.
    //
    // let mut peripherals = Peripherals::take();

    // The default config this is called with is a 160 MHz CPU clock, which is the maximum frequency
    // according to section 1.2. Apparently the default CPU clock is 20 MHz (section 6.2.4.1),
    // but this isn't even an option in the HAL.
    //
    // RTC is never defined in the manual, but it's probably this? https://en.wikipedia.org/wiki/Real-time_clock
    // The Wikipedia article mentions low power consumption, which is in line with the idea that the RTC stuff
    // is "slow speed clocks for low-power devices" (6.2.3).
    //
    // Low-power management (Section 9.3, pg 220) indicates RTC as a power domain containing the
    // power management unit (PMU).
    //
    // Somewhat confusingly RTC seems to sometimes be referred to as RC instead
    // (pg. 901 revision history: RTC_CLK renamed as RC_SLOW_CLK).
    //
    // TODO: In any case a huge number of register manipulations follow; need to figure out what they do.
    // Notably, this function eventually does stuff with "regi2c," which is apparently an undocumented feature of
    // the MCU that is used by ESP-IDF.
    //
    // Clocks::init(config.clock_config());

    // This line doesn't seem to do any computation; just makes a struct, with syntax being safety sugar I guess.
    //
    // let mut rtc = crate::rtc_cntl::Rtc::new(peripherals.LPWR.reborrow());

    // TODO: Huge number of register manipulations here; probably to enable RTC power domain.
    // This line also ends up using "regi2c," and also accesses a number of seemingly undocumented registers within the
    // RTC register block (timer3, timer5, etc.) so maybe ignore for now.
    //
    // crate::rtc_cntl::sleep::RtcSleepConfig::base_settings(&rtc);

    // Section 12.1:
    // "Watchdog timers are hardware timers used to detect and recover from malfunctions. They must be
    //  periodically fed (reset) to prevent a timeout. A system/software that is behaving unexpectedly (e.g. is stuck in
    //  a software loop or in overdue events) will fail to feed the watchdog thus trigger a watchdog timeout."
    //
    // The ESP32C3 has four of them:
    // - "Super watchdog": An analog watchdog timer.
    // - "RTC watchdog timer": A digital watchdog timer.
    // - "Main system watchdog timer 0" (MWDT0): A digital watchdog timer; is in timer group 0 (TIMG0).
    // - "Main system watchdog timer 1" (MWDT1): Is in timer group 1 (TIMG1).
    //
    // Also: https://www.digikey.com/en/articles/a-designers-guide-to-watchdog-timers
    // Amusingly the article also refers to resetting the timers as "tickling" them and "kick dog."
    //
    // Anyway, the HAL seems to disable all the watchdog timers in its init function. Presumably so that the programmer
    // doesn't have to worry about it, but is that a good idea? Don't know.
    //
    // 12.2.2.4: "During flash booting process, MWDT in timer group 0 (see Figure 11.1-1 Timer Units within Groups), as well as
    //            RWDT, are automatically enabled. Stage 0 for the enabled MWDT is automatically configured to reset the
    //            system upon expiry, known as core reset. Likewise, stage 0 for RWDT is configured to system reset, which
    //            resets the main system and RTC when it expires."
    //
    //            ^ This probably explains why the board repeatedly disconnects and displays weird behavior after flashing a program
    //              with the init function removed, even though an existing init function from a previous flash should have disabled
    //              those timers. Presumably the disconnections happen when the CPU/peripherals are reset.
    //
    // WDTs need to have a "key" written into a register before they can be manipulated, to prevent bugs from accidentally messing
    // with their behavior. The one analog watchdog timer has a write key of 0x8F1D312A, while the remaining (digital) timers have
    // a key of 0x50D83AA1.
    //
    // SWD.
    // Interestingly, the HAL implementation doesn't actually _disable_ SWD here, it
    // just sets it to "auto-feed" to make it feed itself on interrupts. Not entirely
    // sure what the practical differences are.
    let reg = (LPWR_BASE + RTC_CNTL_SWD_WPROTECT_REG) as *mut u32;
    unsafe { reg.write_volatile(0x8F1D312A);}
    {
        let reg = (LPWR_BASE + RTC_CNTL_SWD_CONF_REG) as *mut u32;
        unsafe { reg.write_volatile(reg.read_volatile() | (1 << RTC_CNTL_SWD_AUTO_FEED_EN)); }
    }
    unsafe { reg.write_volatile(0);}
    //
    // RWDT.
    // Technically the HAL sets the entire register to 0 and not just the enable bit, but this should
    // probably work fine.
    let reg = (LPWR_BASE + RTC_CNTL_WDTWPROTECT_REG) as *mut u32;
    unsafe { reg.write_volatile(0x50D83AA1);}
    {
        let reg = (LPWR_BASE + RTC_CNTL_WDTCONFIG0_REG) as *mut u32;
        unsafe { reg.write_volatile(reg.read_volatile() & (0 << RTC_CNTL_WDT_EN)); }
    }
    unsafe { reg.write_volatile(0);}
    //
    // MWDT0.
    let reg = (TIMG0_BASE + TIMG_WDTWPROTECT_REG) as *mut u32;
    unsafe { reg.write_volatile(0x50D83AA1);}
    {
        let reg = (TIMG0_BASE + TIMG_WDTCONFIG0_REG) as *mut u32;
        unsafe { reg.write_volatile(reg.read_volatile() & (0 << TIMG_WDT_EN)); }
    }
    unsafe { reg.write_volatile(0);}
    //
    // MWDT1.
    let reg = (TIMG1_BASE + TIMG_WDTWPROTECT_REG) as *mut u32;
    unsafe { reg.write_volatile(0x50D83AA1);}
    {
        let reg = (TIMG1_BASE + TIMG_WDTCONFIG0_REG) as *mut u32;
        unsafe { reg.write_volatile(reg.read_volatile() & (0 << TIMG_WDT_EN)); }
    }
    unsafe { reg.write_volatile(0);}

    // This call is just a wrapper around "init_timestamp_scaler," which initializes a static mut u32
    // which is subsequently used to convert between SYSTIMER ticks and microseconds.
    // The code for this repo just uses tick values directly so it's not needed.
    //
    // crate::time::implem::time_init();

    // TODO:
    // crate::gpio::interrupt::bind_default_interrupt_handler();

    // TODO: This assigns to a C static mut pointer called `syscall_table_ptr`.
    // Seems like it is maybe a feature of the IDF.
    //
    // unsafe {
    //     esp_rom_sys::init_syscall_table();
    // }
}


// This is slightly more hairy than expected because the
// counter might wrap around.
// Not sure if this is really as robust as the HAL version.
pub fn delay_ticks(ticks: u64) {
    let mut start = read_unit0_count();
    let mut end = start.wrapping_add(ticks);

    if start > end { core::mem::swap(&mut start, &mut end);}

    loop {
        let current = read_unit0_count();
        if current >= end {
            return;
        }
    }
}

// According to 10.3, average frequency is 16 MHz, so each tick is 1/16 us.
pub fn delay_us(us: u64) {
    delay_ticks(us * 16);
}

pub fn read_unit0_count() -> u64 {
    // man why is the clock stuff so arcane

    // Section 10.5
    const SYSTIMER_TIMER_UNIT0_OP_REG: usize = SYSTIMER_BASE + 0x04;
    let reg = SYSTIMER_TIMER_UNIT0_OP_REG as *mut u32;

    // SYSTIMER_TIMER_UNIT0_UPDATE: reads count into VALUE_HI and VALUE_LO
    // Safety: WT field.
    unsafe { reg.write_volatile(1 << 30); }

    // "valid": poll until true, meaning the update finished
    loop {
        // Safety: R/SS field; in other words, automatically set by hardware.
        let valid = unsafe { reg.read_volatile() & (1 << 29) != 0 };
        if valid { break; }
    }
    
    // Safety: Requires that the update is done before these reads, which happens
    //         earlier in this function.
    // NOTE: The HAL accomodates for an edge case here where an interrupt happens
    //       between reading hi and lo, but lazy.
    //       In some sense that means this is currently _actually_ kinda unsafe.
    let hi = (SYSTIMER_BASE + 0x40) as *mut u32;
    let lo = (SYSTIMER_BASE + 0x44) as *mut u32;

    let hi = unsafe { hi.read_volatile() };
    let lo = unsafe { lo.read_volatile() };

    (hi as u64) << 32 | lo as u64
}

pub unsafe fn set_enable(gpio: usize) {
    // Normally configuring GPIO_FUNCx_OUT_SEL_CFG_REG might be needed here,
    // but not for "simple GPIO output" (5.5.3).
    let reg = (GPIO_BASE + GPIO_ENABLE_W1TS) as *mut u32;
    unsafe { reg.write_volatile(1 << gpio); }
}

// Probably unnecessary, because it looks like 0 is the default?
pub unsafe fn pad_driver(gpio: usize) {
    let reg = (GPIO_BASE + GPIO_PIN_N_REG + 4 * gpio) as *mut u32;
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
    
    // Turn off FUN_WPD (bit 7), FUN_WPU (bit 8), and set drive strength (10-11);
    // also set MCU_SEL (12-14) to 1 (aka GPIO function)
    unsafe { reg.write_volatile(1 << 12 | 2 << 10); }
}

pub unsafe fn read_gpio(gpio: usize) -> bool {
    let out = (GPIO_BASE + GPIO_OUT_REG) as *mut u32;
    unsafe { out.read_volatile() & (1 << gpio) != 0 }
}

pub unsafe fn set_high(gpio: usize) {
    let w1ts = (GPIO_BASE + GPIO_OUT_W1TS) as *mut u32;
    unsafe { w1ts.write_volatile(1 << gpio); }
}

pub unsafe fn set_low(gpio: usize) {
    let w1tc = (GPIO_BASE + GPIO_OUT_W1TC) as *mut u32;
    unsafe { w1tc.write_volatile(1 << gpio); }
}