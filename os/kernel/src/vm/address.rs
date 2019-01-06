use std::fmt;

/// A virtual address.
pub struct VirtualAddr(usize);

/// A physical address.
pub struct PhysicalAddr(usize);

macro_rules! impl_for {
    ($T:tt) => {
        impl<T: Sized> From<*mut T> for $T {
            fn from(raw_ptr: *mut T) -> $T {
                $T(raw_ptr as usize)
            }
        }

        impl $T {
            /// Returns the inner address of `self`.
            pub fn as_ptr(&self) -> *const u8 {
                self.0 as *const u8
            }

            /// Returns the inner address of `self`.
            ///
            /// # Safety
            ///
            /// This method is marked `unsafe` because it can be used to create
            /// multiple mutable aliases to the address represented by `self`. The
            /// caller must ensure that they do not alias.
            pub fn as_mut_ptr(&mut self) -> *mut u8 {
                self.0 as *mut u8
            }

            /// Returns the inner address of `self` as a `usize`.
            pub fn as_usize(&self) -> usize {
                self.0
            }

            /// Returns the inner address of `self` as a `u64`.
            #[cfg(target_pointer_width = "64")]
            pub fn as_u64(&self) -> u64 {
                self.0 as u64
            }
        }

        impl fmt::Debug for $T {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}({:#x})", stringify!($T), self.0)
            }
        }
    }
}

impl_for!(VirtualAddr);
impl_for!(PhysicalAddr);
