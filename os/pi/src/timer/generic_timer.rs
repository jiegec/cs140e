use volatile::prelude::*;
use volatile::Volatile;

/// The base address for the ARM generic timer registers.
const GEN_TIMER_REG_BASE: usize = 0x40000000;

/// Core interrupt sources (ref: QA7 4.10, page 16)
#[repr(u8)]
#[allow(dead_code)]
#[allow(non_snake_case)]
#[derive(Copy, Clone, PartialEq, Debug)]
enum CoreInterrupt {
    CNTPSIRQ = 0,
    CNTPNSIRQ = 1,
    CNTHPIRQ = 2,
    CNTVIRQ = 3,
    Mailbox0 = 4,
    Mailbox1 = 5,
    Mailbox2 = 6,
    Mailbox3 = 7,
    Gpu = 8,
    Pmu = 9,
    AxiOutstanding = 10,
    LocalTimer = 11,
}

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    CONTROL: Volatile<u32>,
    _unused1: [Volatile<u32>; 8],
    LOCAL_IRQ: Volatile<u32>,
    _unused2: [Volatile<u32>; 3],
    LOCAL_TIMER_CTL: Volatile<u32>,
    LOCAL_TIMER_FLAGS: Volatile<u32>,
    _unused3: [Volatile<u32>; 1],
    CORE_TIMER_IRQCNTL: [Volatile<u32>; 4],
    CORE_MAILBOX_IRQCNTL: [Volatile<u32>; 4],
    CORE_IRQ_SRC: [Volatile<u32>; 4],
}

/// The ARM generic timer.
pub struct Timer {
    registers: &'static mut Registers
}

pub fn set_cntp_ctl_el0(x: u64) {
    unsafe {
        asm!("msr cntp_ctl_el0, $0" :: "r"(x));
    }
}

pub fn set_cntk_ctl_el1(x: u64) {
    unsafe {
        asm!("msr cntkctl_el1, $0" :: "r"(x));
    }
}

pub fn set_cntp_tval_el0(x: u64) {
    unsafe {
        asm!("msr cntp_tval_el0, $0" :: "r"(x));
    }
}

pub fn get_cntfrq_el0() -> u64 {
    let x: u64;
    unsafe {
        asm!("mrs $0, cntfrq_el0"
            : "=r"(x));
    }
    x
}

pub fn get_cntpct_el0() -> u64 {
    let x: u64;
    unsafe {
        asm!("isb
              mrs $0, cntpct_el0"
            : "=r"(x) : : : "volatile");
    }
    x
}


impl Timer {
    /// Returns a new instance of `Timer`.
    pub fn new() -> Timer {
        Timer {
            registers: unsafe { &mut *(GEN_TIMER_REG_BASE as *mut Registers) },
        }
    }

    /// Reads the system timer's counter and returns the 64-bit counter value.
    /// The returned value is the number of elapsed microseconds.
    pub fn read(&self) -> u64 {
        let cntfrq = get_cntfrq_el0(); // 62500000
        (get_cntpct_el0() * 1000000 / (cntfrq as u64)) as u64
    }

    /// Sets up a match in timer 1 to occur `us` microseconds from now. If
    /// interrupts for timer 1 are enabled and IRQs are unmasked, then a timer
    /// interrupt will be issued in `us` microseconds.
    pub fn tick_in(&mut self, us: u32) {
        let cntfrq = get_cntfrq_el0(); // 62500000
        set_cntp_tval_el0(((cntfrq as f64) * (us as f64) / 1000000.0) as u64);
    }

    pub fn initialize() {
        let timer = Timer {
            registers: unsafe { &mut *(GEN_TIMER_REG_BASE as *mut Registers) },
        };
        timer.registers.CORE_TIMER_IRQCNTL[0].write(1 << (CoreInterrupt::CNTPNSIRQ as u8));
        set_cntp_ctl_el0(0x1); // enable timer interrupt and do not mask it
        set_cntk_ctl_el1(0x3); // allow EL0 to read timer counter
    }

    pub fn is_pending(&self) -> bool {
        self.registers.CORE_IRQ_SRC[0].read() & (1 << (CoreInterrupt::CNTPNSIRQ as u8)) != 0
    }
}

/// Returns the current time in microseconds.
pub fn current_time() -> u64 {
    Timer::new().read()
}

/// Spins until `us` microseconds have passed.
pub fn spin_sleep_us(us: u64) {
    let old = current_time();
    loop {
        let new = current_time();
        if old + us <= new {
            break;
        }
    }
}

/// Spins until `ms` milliseconds have passed.
pub fn spin_sleep_ms(ms: u64) {
    spin_sleep_us(ms * 1000);
}

/// Sets up a match in timer 1 to occur `us` microseconds from now. If
/// interrupts for timer 1 are enabled and IRQs are unmasked, then a timer
/// interrupt will be issued in `us` microseconds.
pub fn tick_in(us: u32) {
    Timer::new().tick_in(us)
}
