#![cfg_attr(test, feature(inclusive_range_syntax))]
#![no_std]

#[cfg(test)]
mod tests;

use core::ops::{Deref, DerefMut};

/// A contiguous array type backed by a slice.
///
/// `StackVec`'s functionality is similar to that of `std::Vec`. You can `push`
/// and `pop` and iterate over the vector. Unlike `Vec`, however, `StackVec`
/// requires no memory allocation as it is backed by a user-supplied slice. As a
/// result, `StackVec`'s capacity is _bounded_ by the user-supplied slice. This
/// results in `push` being fallible: if `push` is called when the vector is
/// full, an `Err` is returned.
#[derive(Debug)]
pub struct StackVec<'a, T: 'a> {
    storage: &'a mut [T],
    len: usize,
}

impl<'a, T: 'a> StackVec<'a, T> {
    /// Constructs a new, empty `StackVec<T>` using `storage` as the backing
    /// store. The returned `StackVec` will be able to hold `storage.len()`
    /// values.
    pub fn new(storage: &'a mut [T]) -> StackVec<'a, T> {
        StackVec {
            len: 0,
            storage: storage,
        }
    }

    /// Constructs a new `StackVec<T>` using `storage` as the backing store. The
    /// first `len` elements of `storage` are treated as if they were `push`ed
    /// onto `self.` The returned `StackVec` will be able to hold a total of
    /// `storage.len()` values.
    ///
    /// # Panics
    ///
    /// Panics if `len > storage.len()`.
    pub fn with_len(storage: &'a mut [T], len: usize) -> StackVec<'a, T> {
        if len > storage.len() {
            panic!("len should not be greather than storage.len()")
        }
        StackVec {
            len: len,
            storage: storage,
        }
    }

    /// Returns the number of elements this vector can hold.
    pub fn capacity(&self) -> usize {
        self.storage.len()
    }

    /// Shortens the vector, keeping the first `len` elements. If `len` is
    /// greater than the vector's current length, this has no effect. Note that
    /// this method has no effect on the capacity of the vector.
    pub fn truncate(&mut self, len: usize) {
        if self.len > len {
            self.len = len
        }
    }

    /// Extracts a slice containing the entire vector, consuming `self`.
    ///
    /// Note that the returned slice's length will be the length of this vector,
    /// _not_ the length of the original backing storage.
    pub fn into_slice(self) -> &'a mut [T] {
        &mut self.storage[0..self.len]
    }

    /// Extracts a slice containing the entire vector.
    pub fn as_slice(&self) -> &[T] {
        &self.storage[0..self.len]
    }

    /// Extracts a mutable slice of the entire vector.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.storage[0..self.len]
    }

    /// Returns the number of elements in the vector, also referred to as its
    /// 'length'.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the vector contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns true if the vector is at capacity.
    pub fn is_full(&self) -> bool {
        self.len == self.storage.len()
    }

    /// Appends `value` to the back of this vector if the vector is not full.
    ///
    /// # Error
    ///
    /// If this vector is full, an `Err` is returned. Otherwise, `Ok` is
    /// returned.
    pub fn push(&mut self, value: T) -> Result<(), ()> {
        if self.len == self.storage.len() {
            Err(())
        } else {
            self.storage[self.len] = value;
            self.len += 1;
            Ok(())
        }
    }
}

impl<'a, T: Clone + 'a> StackVec<'a, T> {
    /// If this vector is not empty, removes the last element from this vector
    /// by cloning it and returns it. Otherwise returns `None`.
    pub fn pop(&mut self) -> Option<T> {
        if self.len > 0 {
            self.len -= 1;
            Some(self.storage[self.len].clone())
        } else {
            None
        }
    }
}

// FIXME: Implement `Deref`, `DerefMut`, and `IntoIterator` for `StackVec`.

impl<'a, T: 'a> Deref for StackVec<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.storage[0..self.len]
    }
}

impl<'a, T: 'a> DerefMut for StackVec<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.storage[0..self.len]
    }
}

impl<'a, T: 'a> IntoIterator for StackVec<'a, T> {
    type Item = &'a T;
    type IntoIter = ::core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.storage[0..self.len].into_iter()
    }
}

/* Alternative solution:
pub struct StackVecIntoIterator<'a, T: 'a> {
    index: usize,
    vec: StackVec<'a, T>,
}

impl<'a, T: 'a> StackVecIntoIterator<'a, T> {
    fn new(sv: StackVec<'a, T>) -> StackVecIntoIterator<'a, T> {
        StackVecIntoIterator { index: 0, vec: sv }
    }
}

pub struct RefWrapper<T> {
    element: T,
}

impl<T> Deref for RefWrapper<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.element
    }
}

impl<'a, T: 'a + Clone> Iterator for StackVecIntoIterator<'a, T> {
    type Item = RefWrapper<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vec.len {
            self.index += 1;
            Some(RefWrapper {
                element: self.vec[self.index - 1].clone(),
            })
        } else {
            None
        }
    }
}

impl<'a, T: 'a + Clone> IntoIterator for StackVec<'a, T> {
    type Item = RefWrapper<T>;
    type IntoIter = StackVecIntoIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        StackVecIntoIterator::new(self)
    }
}
*/

// FIXME: Implement IntoIterator` for `&StackVec`.

impl<'a, T: 'a> IntoIterator for &'a StackVec<'a, T> {
    type Item = &'a T;
    type IntoIter = ::core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.storage[0..self.len].into_iter()
    }
}

/* Alternative implementation:
pub struct StackVecIterator<'a, T: 'a> {
    index: usize,
    vec: &'a StackVec<'a, T>,
}

impl<'a, T: 'a> StackVecIterator<'a, T> {
    fn new(sv: &'a StackVec<'a, T>) -> StackVecIterator<'a, T> {
        StackVecIterator { index: 0, vec: sv }
    }
}

impl<'a, T: 'a> Iterator for StackVecIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vec.len {
            self.index += 1;
            Some(&self.vec[self.index - 1])
        } else {
            None
        }
    }
}

impl<'a, T: 'a> IntoIterator for &'a StackVec<'a, T> {
    type Item = &'a T;
    type IntoIter = StackVecIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(&self)
    }
}
*/

// My helper functions
impl<'a, T: Clone> StackVec<'a, T> {
    pub fn remove(&mut self, index: usize) -> T {
        if index >= self.len {
            panic!("index out of bound")
        }

        let item = self.storage[index].clone();
        self.len -= 1;

        for i in index..self.len {
            self.storage[i] = self.storage[i+1].clone();
        }

        item
    }

    pub fn insert(&mut self, index: usize, element: T) -> Result<(), ()> {
        if self.len == self.storage.len() || index > self.len() {
            Err(())
        } else {
            for i in (index..self.len).rev() {
                self.storage[i+1] = self.storage[i].clone()
            }
            self.storage[index] = element;
            self.len += 1;
            Ok(())
        }
    }
}
