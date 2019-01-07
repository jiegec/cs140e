use console::kprintln;
use dwarf;

/// Returns the current stack pointer.
#[inline(always)]
pub fn sp() -> *const u8 {
    let ptr: usize;
    unsafe {
        asm!("mov $0, sp" : "=r"(ptr));
    }

    ptr as *const u8
}

/// Returns the current frame pointer.
#[inline(always)]
pub fn fp() -> usize {
    let ptr: usize;
    unsafe {
        asm!("mov $0, x29" : "=r"(ptr));
    }

    ptr
}

/// Returns the current link register.
#[inline(always)]
pub fn lr() -> usize {
    let ptr: usize;
    unsafe {
        asm!("mov $0, x30" : "=r"(ptr));
    }

    ptr
}

/// Returns the current program counter.
#[inline(always)]
pub fn pc() -> usize {
    let ptr: usize;
    unsafe {
        asm!("adr $0, ." : "=r"(ptr));
    }

    ptr
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

extern "C" {
    fn __text_start();
    fn __text_end();
}

pub fn bt() {
    #[cfg(not(test))]
    unsafe {
        let mut current_pc = lr();
        let mut current_fp = fp();
        let mut stack_num = 0;
        while current_pc >= __text_start as usize && current_pc <= __text_end as usize && current_fp as usize != 0 {
            kprintln!("#{} {:#018X} {}", stack_num, current_pc, 
                dwarf::get_function_from_pc(current_pc).unwrap_or_else(|| "unknown".to_string()));
            stack_num = stack_num + 1;
            current_fp = *(current_fp as *const usize);
            if current_fp as usize != 0 {
                current_pc = *(current_fp as *const usize).offset(1);
            }
        }
    }
}