#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(decl_macro)]
#![feature(never_type)]

extern crate core;
extern crate volatile;

pub mod timer;
pub mod uart;
pub mod gpio;
pub mod common;
pub mod atags;
pub mod interrupt;
