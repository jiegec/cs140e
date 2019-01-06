#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(unique)]
#![feature(ptr_internals)]
#![no_std]

//! Wrapper types that guarantee read-only, write-only, or read/write volatile
//! access to a raw pointer.

use core::ptr::Unique;

/// Trait implemented by **readable** volatile wrappers.
pub trait Readable<T> {
    /// Returns the inner pointer.
    fn inner(&self) -> *const T;

    /// Reads and returns the value pointed to by `self`. The read is always
    /// done using volatile semantics.
    #[inline(always)]
    fn read(&self) -> T {
        unsafe { ::core::ptr::read_volatile(self.inner()) }
    }

    /// Returns `true` if the value pointed to by `self` has the mask `mask`.
    /// This is equivalent to `(self.read() & mask) == mask`.
    #[inline(always)]
    fn has_mask(&self, mask: T) -> bool
        where T: ::core::ops::BitAnd<Output = T>,
              T: PartialEq + Copy
    {
        (self.read() & mask) == mask
    }
}

/// Trait implemented by **writeable** volatile wrappers.
pub trait Writeable<T> {
    /// Returns the inner pointer. This function is unsafe because this returns
    /// a _copy_ of the mutable pointer, resulting in a mutable alias.
    unsafe fn inner(&mut self) -> *mut T;

    /// Writes the value `val` to the inner address of `self`. The write is
    /// always done using volatile semantics.
    #[inline(always)]
    fn write(&mut self, val: T) {
        unsafe { ::core::ptr::write_volatile(self.inner(), val) }
    }
}

/// Trait implemented by **readable _and_ writeable** volatile wrappers.
pub trait ReadableWriteable<T>: Readable<T> + Writeable<T>
    where T: ::core::ops::BitAnd<Output = T>,
          T: ::core::ops::BitOr<Output = T>
{
    /// Applies the mask `mask` using `&` to the value referred to by `self`.
    /// This is equivalent to `self.write(self.read() & mask)`.
    fn and_mask(&mut self, mask: T) {
        let init_val = self.read();
        self.write(init_val & mask);
    }

    /// Applies the mask `mask` using `|` to the value referred to by `self`.
    /// This is equivalent to `self.write(self.read() | mask)`.
    fn or_mask(&mut self, mask: T) {
        let init_val = self.read();
        self.write(init_val | mask);
    }
}

#[doc(hidden)]
macro readable_writeable($type:ident, $self:ident.$($impl:tt)+) {
    impl<T> ReadableWriteable<T> for $type<T>
        where T: ::core::ops::BitAnd<Output = T>, T: ::core::ops::BitOr<Output = T> { }
}

#[doc(hidden)]
macro readable($type:ident, $self:ident.$($impl:tt)+) {
    impl<T> Readable<T> for $type<T> {
        fn inner(&$self) -> *const T { $self.$($impl)+ }
    }
}

#[doc(hidden)]
macro writeable($type:ident, $self:ident.$($impl:tt)+) {
    impl<T> Writeable<T> for $type<T> {
        unsafe fn inner(&mut $self) -> *mut T { $self.$($impl)+ }
    }
}

/// A wrapper type that enforces **read-only** _volatile_ accesses to a raw
/// pointer.
pub struct ReadVolatile<T>(*const T);
readable!(ReadVolatile, self.0);
unsafe impl<T: Send> Send for ReadVolatile<T> {  }

impl<T> ReadVolatile<T> {
    /// Returns a new `ReadVolatile` that allows volatile read-only access to
    /// `ptr`.
    ///
    /// # Safety
    ///
    /// The caller _must_ guarantee that `ptr` points to a value of type `T`
    /// that is valid for the `'static` lifetime. This is equivalent to casting
    /// the `*const T` to `&'static T`.
    pub const unsafe fn new(ptr: *const T) -> ReadVolatile<T> {
        ReadVolatile(ptr)
    }
}

/// A wrapper type that enforces **write-only** _volatile_ accesses to a raw
/// pointer.
pub struct WriteVolatile<T>(*mut T);
writeable!(WriteVolatile, self.0);
unsafe impl<T: Send> Send for WriteVolatile<T> {  }

impl<T> WriteVolatile<T> {
    /// Returns a new `WriteVolatile` that allows volatile write-only access to
    /// `ptr`.
    ///
    /// # Safety
    ///
    /// The caller _must_ guarantee that `ptr` points to a value of type `T`
    /// that is valid for the `'static` lifetime. Furthermore, `ptr` must be the
    /// only accessible mutable reference to the value. This is equivalent to
    /// casting the `*mut T` to `&'static mut T`.
    pub const unsafe fn new(ptr: *mut T) -> WriteVolatile<T> {
        WriteVolatile(ptr)
    }
}

/// A wrapper type that enforces _volatile_ (read **or** write) accesses to a
/// raw pointer.
pub struct Volatile<T>(*mut T);
readable!(Volatile, self.0);
writeable!(Volatile, self.0);
readable_writeable!(Volatile, self.0);
unsafe impl<T: Send> Send for Volatile<T> {  }

impl<T> Volatile<T> {
    /// Returns a new `Volatile` that allows volatile read/write access to
    /// `ptr`.
    ///
    /// # Safety
    ///
    /// The caller _must_ guarantee that `ptr` points to a value of type `T`
    /// that is valid for the `'static` lifetime. This type allows aliasing the
    /// value pointed to by `ptr` and thus cannot implement `Send + Send`. If
    /// `ptr` is guaranteed to be the only pointer to the value, use
    /// `UniqueVolatile` instead.
    pub const unsafe fn new(ptr: *mut T) -> Volatile<T> {
        Volatile(ptr)
    }
}

/// A wrapper type that enforces _volatile_ (read **or** write) accesses to a
/// raw pointer. Implements `Sync + Send`.
///
/// `Sync` is implemented if `T: Sync`. Likewise, `Send` is implemented if `T:
/// Send`.
pub struct UniqueVolatile<T>(Unique<T>);
readable!(UniqueVolatile, self.0.as_ptr());
writeable!(UniqueVolatile, self.0.as_ptr());
readable_writeable!(UniqueVolatile, self.as_ptr());

impl<T> UniqueVolatile<T> {
    /// Returns a new `UniqueVolatile` that allows volatile read/write access to
    /// `ptr`.
    ///
    /// # Safety
    ///
    /// The caller _must_ guarantee that `ptr` points to a value of type `T`
    /// that is valid for the `'static` lifetime. Furthermore, unlike
    /// `Volatile`, `ptr` must be the only accessible mutable reference to the
    /// value. This is equivalent to casting the `*mut T` to `&'static mut T`.
    pub const unsafe fn new(ptr: *mut T) -> UniqueVolatile<T> {
        UniqueVolatile(Unique::new_unchecked(ptr))
    }
}
