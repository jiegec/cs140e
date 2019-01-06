// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Useful synchronization primitives.
//!
//! This module contains useful safe and unsafe synchronization primitives.
//! Most of the primitives in this module do not provide any sort of locking
//! and/or blocking at all, but rather provide the necessary tools to build
//! other types of concurrent primitives.

#![stable(feature = "rust1", since = "1.0.0")]

#[stable(feature = "rust1", since = "1.0.0")]
pub use alloc::sync::{Arc, Weak};
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::sync::atomic;

//- #[stable(feature = "rust1", since = "1.0.0")]
//- pub use self::barrier::{Barrier, BarrierWaitResult};
//- #[stable(feature = "rust1", since = "1.0.0")]
//- pub use self::condvar::{Condvar, WaitTimeoutResult};
//- #[stable(feature = "rust1", since = "1.0.0")]
//- pub use self::mutex::{Mutex, MutexGuard};
//- #[stable(feature = "rust1", since = "1.0.0")]
//- pub use self::once::{Once, OnceState, ONCE_INIT};
//- #[stable(feature = "rust1", since = "1.0.0")]
//- pub use sys_common::poison::{PoisonError, TryLockError, TryLockResult, LockResult};
//- #[stable(feature = "rust1", since = "1.0.0")]
//- pub use self::rwlock::{RwLock, RwLockReadGuard, RwLockWriteGuard};

//- pub mod mpsc;

//- mod barrier;
//- mod condvar;
//- mod mutex;
//- mod once;
//- mod rwlock;

//- EVERYTHING BELOW HERE WAS ADDED
use sync::atomic::{AtomicBool, Ordering};
use cell::UnsafeCell;
use ops::{DerefMut, Deref, Drop};
use fmt;

#[repr(align(32))]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Mutex<T> {
    data: UnsafeCell<T>,
    lock: AtomicBool,
}

#[stable(feature = "rust1", since = "1.0.0")]
unsafe impl<T: Send> Send for Mutex<T> { }

#[stable(feature = "rust1", since = "1.0.0")]
unsafe impl<T: Send> Sync for Mutex<T> { }

#[stable(feature = "rust1", since = "1.0.0")]
pub struct MutexGuard<'a, T: 'a> {
    lock: &'a Mutex<T>
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T> !Send for MutexGuard<'a, T> { }

#[stable(feature = "rust1", since = "1.0.0")]
unsafe impl<'a, T: Sync> Sync for MutexGuard<'a, T> { }

impl<T> Mutex<T> {
    #[stable(feature = "rust1", since = "1.0.0")]
    pub const fn new(val: T) -> Mutex<T> {
        Mutex {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(val)
        }
    }
}

impl<T> Mutex<T> {
    // Once MMU/cache is enabled, do the right thing here. For now, we don't
    // need any real synchronization.
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        if !self.lock.load(Ordering::Relaxed) {
            self.lock.store(true, Ordering::Relaxed);
            Some(MutexGuard { lock: &self })
        } else {
            None
        }
    }

    // Once MMU/cache is enabled, do the right thing here. For now, we don't
    // need any real synchronization.
    #[inline(never)]
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn lock(&self) -> Result<MutexGuard<T>, !> {
        // Wait until we can "aquire" the lock, then "acquire" it.
        loop {
            match self.try_lock() {
                Some(guard) => return Ok(guard),
                None => continue
            }
        }
    }

    fn unlock(&self) {
        self.lock.store(false, Ordering::Relaxed);
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T: 'a> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { & *self.lock.data.get() }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T: 'a> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T: 'a> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.unlock()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: fmt::Debug> fmt::Debug for Mutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.try_lock() {
            Some(guard) => f.debug_struct("Mutex").field("data", &&*guard).finish(),
            None => f.debug_struct("Mutex").field("data", &"<locked>").finish()
        }
    }
}
