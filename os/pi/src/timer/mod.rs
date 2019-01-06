#[cfg(feature = "qemu")]
mod generic_timer;
#[cfg(not(feature = "qemu"))]
mod system_timer;

#[cfg(feature = "qemu")]
pub use self::generic_timer::*;
#[cfg(not(feature = "qemu"))]
pub use self::system_timer::*;