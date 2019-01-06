#[repr(C)]
#[derive(Default, Debug, Copy, Clone)]
pub struct TrapFrame {
    // FIXME: Fill me in.
    pub elr: u64,
    pub spsr: u64,
    pub sp: u64,
    pub tpidr: u64,
    pub q0to31: [u128; 32],
    pub x1to29: [u64; 29],
    pub __r1: u64, // may be used to store lr temporaily
    pub x30: u64,
    pub x0: u64,
}
