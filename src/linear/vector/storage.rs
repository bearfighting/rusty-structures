//! Raw allocation and deallocation for `Vector`.
//!
//! This module owns the allocation. `Vector` tracks which slots are initialized;
//! `Storage` only tracks the pointer and the number of slots available.

use super::TryReserveError;
use std::alloc::{Layout, alloc, dealloc};
use std::mem::{self, MaybeUninit};
use std::ptr::{self, NonNull};

pub(super) struct Storage<T> {
    ptr: NonNull<T>,
    capacity: usize,
}

impl<T> Storage<T> {
    pub(super) const fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            capacity: if mem::size_of::<T>() == 0 {
                usize::MAX
            } else {
                0
            },
        }
    }

    pub(super) fn try_with_capacity(capacity: usize) -> Result<Self, TryReserveError> {
        let mut storage = Self::new();
        if mem::size_of::<T>() != 0 && capacity > 0 {
            storage.grow_to(capacity, 0)?;
        }
        Ok(storage)
    }

    pub(super) const fn capacity(&self) -> usize {
        self.capacity
    }

    pub(super) const fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    pub(super) fn grow_to(
        &mut self,
        new_capacity: usize,
        initialized_len: usize,
    ) -> Result<(), TryReserveError> {
        debug_assert!(new_capacity > self.capacity);
        let layout = Layout::array::<MaybeUninit<T>>(new_capacity)
            .map_err(|_| TryReserveError::CapacityOverflow)?;
        // SAFETY: layout is non-zero-sized and valid for an array of T.
        let raw = unsafe { alloc(layout) }.cast::<T>();
        let new_ptr = NonNull::new(raw).ok_or(TryReserveError::AllocError)?;

        // SAFETY: initialized_len elements fit in both allocations. Copying
        // transfers them to the new allocation; they are not dropped here.
        unsafe {
            ptr::copy_nonoverlapping(self.ptr.as_ptr(), new_ptr.as_ptr(), initialized_len);
            self.deallocate();
        }

        self.ptr = new_ptr;
        self.capacity = new_capacity;
        Ok(())
    }

    fn deallocate(&mut self) {
        if mem::size_of::<T>() != 0 && self.capacity > 0 {
            let layout = Layout::array::<MaybeUninit<T>>(self.capacity)
                .expect("an existing storage capacity has a valid layout");
            // SAFETY: ptr was allocated with this exact layout and is not used
            // after deallocation. Its initialized elements have already moved.
            unsafe { dealloc(self.ptr.as_ptr().cast(), layout) };
        }
    }
}

impl<T> Drop for Storage<T> {
    fn drop(&mut self) {
        self.deallocate();
    }
}
