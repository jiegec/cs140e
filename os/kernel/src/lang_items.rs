use core::panic::PanicInfo;
use console::noblock_kprintln;
#[cfg(not(test))]
use std::Layout;

#[no_mangle]
#[cfg(not(test))]
#[panic_handler]
pub extern fn panic_fmt(panic_info: &PanicInfo) -> ! {
    // Avoid deadlock
    if let Some(location) = panic_info.location() {
        noblock_kprintln!("Kernel Panic at file {} line {}, column {}", 
            location.file(), location.line(), location.column());
    }
    if let Some(fmt) = panic_info.message() {
        use console::noblock_kprintln;
        noblock_kprintln!("\t message: {}", fmt);
    }

    loop {
        unsafe { asm!("wfe") }
    }
}

#[cfg(not(test))]
#[lang = "eh_personality"]
pub extern "C" fn eh_personality() {}

#[cfg(not(test))]
#[lang = "oom"]
pub extern "C" fn oom(layout: Layout) -> ! {
    // Avoid deadlock
    noblock_kprintln!("Out of memory when allocating {:?}", layout);

    loop {
        unsafe { asm!("wfe") }
    }
}