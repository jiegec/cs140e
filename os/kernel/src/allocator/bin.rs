use std::cmp::max;
use std::cmp::min;
use std::fmt;
use alloc::alloc::{AllocErr, Layout};

use allocator::util::*;
use allocator::linked_list::LinkedList;
use std::mem::size_of;

const SIZEOF_USIZE: usize = size_of::<usize>();

/// A simple allocator that allocates based on size classes.
pub struct Allocator {
    // FIXME: Add the necessary fields.
    // minimum=usize=8B, 2^3B
    free_list: [LinkedList; 32],
    allocated: usize,
    total: usize,
}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Allocator {
        let mut free_list = [LinkedList::new(); 32];
        let mut current_start = start;

        let mut total = 0;
        while current_start + SIZEOF_USIZE <= end {
            let lowbit = current_start & (!current_start + 1);
            let size = min(lowbit, prev_power_of_two(end - current_start));
            total += size;
            unsafe {
                free_list[size.trailing_zeros() as usize].push(current_start as *mut usize);
            }
            current_start += size;
        }

        Allocator {
            free_list: free_list,
            allocated: 0,
            total,
        }
    }

    /// Allocates memory. Returns a pointer meeting the size and alignment
    /// properties of `layout.size()` and `layout.align()`.
    ///
    /// If this method returns an `Ok(addr)`, `addr` will be non-null address
    /// pointing to a block of storage suitable for holding an instance of
    /// `layout`. In particular, the block will be at least `layout.size()`
    /// bytes large and will be aligned to `layout.align()`. The returned block
    /// of storage may or may not have its contents initialized or zeroed.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure that `layout.size() > 0` and that
    /// `layout.align()` is a power of two. Parameters not meeting these
    /// conditions may result in undefined behavior.
    ///
    /// # Errors
    ///
    /// Returning `Err` indicates that either memory is exhausted
    /// (`AllocError::Exhausted`) or `layout` does not meet this allocator's
    /// size or alignment constraints (`AllocError::Unsupported`).
    pub fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), SIZEOF_USIZE),
        );
        let class = size.trailing_zeros() as usize;
        for i in class..self.free_list.len() {
            if !self.free_list[i].is_empty() {
                for j in (class + 1..i + 1).rev() {
                    let block = self.free_list[j]
                        .pop()
                        .expect("bigger block should have free space");
                    unsafe {
                        self.free_list[j - 1].push((block as usize + (1 << (j - 1))) as *mut usize);
                        self.free_list[j - 1].push(block);
                    }
                }

                let result = Ok(self.free_list[class]
                    .pop()
                    .expect("current block should have free space now")
                    as *mut u8);
                self.allocated += size;
                return result;
            }
        }
        Err(AllocErr {})
    }

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure the following:
    ///
    ///   * `ptr` must denote a block of memory currently allocated via this
    ///     allocator
    ///   * `layout` must properly represent the original layout used in the
    ///     allocation call that returned `ptr`
    ///
    /// Parameters not meeting these conditions may result in undefined
    /// behavior.
    pub fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), SIZEOF_USIZE),
        );
        let class = size.trailing_zeros() as usize;

        unsafe {
            self.free_list[class].push(ptr as *mut usize);
            let mut current_ptr = ptr as usize;
            let mut current_class = class;
            loop {
                let buddy = current_ptr ^ (1 << current_class);
                let mut flag = false;
                for block in self.free_list[current_class].iter_mut() {
                    if block.value() as usize == buddy {
                        block.pop();
                        flag = true;
                        break;
                    }
                }
                if flag {
                    self.free_list[current_class].pop();
                    current_ptr = min(current_ptr, buddy);
                    current_class += 1;
                    self.free_list[current_class].push(current_ptr as *mut usize);
                } else {
                    break;
                }
            }
        }

        self.allocated -= size;
    }
}

// FIXME: Implement `Debug` for `Allocator`.
impl fmt::Debug for Allocator {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("BinAllocator")
            .field("allocated", &self.allocated)
            .field("total", &self.total)
            .finish()
    }
}
