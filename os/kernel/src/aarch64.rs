/// Returns the current stack pointer.
#[inline(always)]
pub fn sp() -> *const u8 {
    let ptr: usize;
    unsafe {
        asm!("mov $0, sp" : "=r"(ptr));
    }

    ptr as *const u8
}

/// Returns the current exception level.
///
/// # Safety
/// This function should only be called when EL is >= 1.
#[inline(always)]
pub unsafe fn current_el() -> u8 {
    let el_reg: u64;
    asm!("mrs $0, CurrentEL" : "=r"(el_reg));
    ((el_reg & 0b1100) >> 2) as u8
}

/// Returns the SPSel value.
#[inline(always)]
pub fn sp_sel() -> u8 {
    let ptr: u32;
    unsafe {
        asm!("mrs $0, SPSel" : "=r"(ptr));
    }

    (ptr & 1) as u8
}

/// Returns the core currently executing.
///
/// # Safety
///
/// This function should only be called when EL is >= 1.
pub unsafe fn affinity() -> usize {
    let x: usize;
    asm!("mrs     $0, mpidr_el1
          and     $0, $0, #3"
          : "=r"(x));

    x
}

/// A NOOP that won't be optimized out.
pub fn nop() {
    unsafe {
        asm!("nop" :::: "volatile");
    }
}
