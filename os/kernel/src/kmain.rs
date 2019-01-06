#![feature(alloc, allocator_api)]
#![feature(asm)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(exclusive_range_pattern)]
#![feature(lang_items)]
#![feature(naked_functions)]
#![feature(never_type)]
#![feature(optin_builtin_traits)]
#![feature(ptr_internals)]
#![feature(panic_info_message)]
#![feature(raw_vec_internals)]

#[macro_use]
#[allow(unused_imports)]
extern crate alloc;
extern crate core;
extern crate fat32;
extern crate pi;
extern crate stack_vec;

pub mod allocator;
#[cfg(not(test))]
use pi::timer::spin_sleep_ms;

pub mod lang_items;
pub mod mutex;
pub mod console;
pub mod user;
pub mod fs;
pub mod traps;
pub mod aarch64;
pub mod process;
pub mod vm;

#[cfg(not(test))]
use allocator::Allocator;
use fs::FileSystem;
use process::GlobalScheduler;
#[cfg(feature = "qemu")]
use pi::timer::Timer;

#[cfg(not(test))]
#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();

#[cfg(test)]
use std::alloc::System;
#[cfg(test)]
#[global_allocator]
pub static ALLOCATOR: System = System;

pub static FILE_SYSTEM: FileSystem = FileSystem::uninitialized();

pub static SCHEDULER: GlobalScheduler = GlobalScheduler::uninitialized();

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn kmain() {
    ALLOCATOR.initialize();
    #[cfg(feature = "qemu")]
    Timer::initialize();
    FILE_SYSTEM.initialize();
    spin_sleep_ms(200);
    SCHEDULER.start();
}

pub extern "C" fn shell_thread() {
    loop {
        user::shell("$ ");
    }
}

pub extern "C" fn shell_thread_2() {
    loop {
        user::timer();
    }
}
