//! A growable, contiguous array type implemented with owned raw storage.
//!
//! `Vector<T>` provides amortized O(1) `push` and `pop`, O(1) indexed access,
//! and O(n) insertion or removal away from the end. Its iteration order is
//! from the first element to the last. The allocation is private; elements
//! are initialized exactly in `0..len`.
//!
//! # Code map
//!
//! - This module contains the main type and its basic operations.
//! - `errors.rs` defines errors returned by fallible operations.
//! - `iter.rs` contains borrowed and owning iterator implementations.
//! - `storage.rs` owns the allocation and grows it when more room is needed.
//! - `traits.rs` contains standard trait implementations such as `FromIterator`.

use std::mem;
use std::ptr;
use std::slice;

mod errors;
mod iter;
mod storage;
mod traits;

pub use errors::{TryPushError, TryReserveError};
pub use iter::IntoIter;
use storage::Storage;

/// A growable array with contiguous storage.
///
/// This is a learning implementation of a vector. It owns its allocation and
/// stores initialized elements contiguously, but it does not aim to match every
/// optimization or API detail of [`std::vec::Vec`].
///
/// # Examples
///
/// ```
/// use data_structures::Vector;
///
/// let mut values = Vector::new();
/// values.push(10);
/// values.push(20);
/// assert_eq!(values.as_slice(), &[10, 20]);
/// assert_eq!(values.pop(), Some(20));
/// ```
///
/// Callers that need to handle allocation errors can use the fallible methods:
///
/// ```
/// use data_structures::Vector;
///
/// let mut values = Vector::try_with_capacity(8)?;
/// values.try_push("ready")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct Vector<T> {
    // `storage` owns the allocation; `len` is the count of its initialized slots.
    storage: Storage<T>,
    len: usize,
}

impl<T> Vector<T> {
    /// Creates an empty vector without allocating memory.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            storage: Storage::new(),
            len: 0,
        }
    }

    /// Creates an empty vector with room for at least `capacity` elements.
    ///
    /// Panics if the requested capacity overflows or allocation fails. Use
    /// [`try_with_capacity`](Self::try_with_capacity) to handle these errors.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::try_with_capacity(capacity).unwrap_or_else(|error| panic!("{error}"))
    }

    /// Creates an empty vector with room for at least `capacity` elements.
    pub fn try_with_capacity(capacity: usize) -> Result<Self, TryReserveError> {
        Ok(Self {
            storage: Storage::try_with_capacity(capacity)?,
            len: 0,
        })
    }

    /// Creates a vector containing `count` clones of `value`.
    ///
    /// Panics if the requested allocation cannot be made. Use
    /// [`try_with_capacity`](Self::try_with_capacity) when allocation failure
    /// must be handled explicitly.
    #[must_use]
    pub fn from_elem(value: T, count: usize) -> Self
    where
        T: Clone,
    {
        let mut vector = Self::with_capacity(count);
        for _ in 0..count {
            vector.push(value.clone());
        }
        vector
    }

    /// Returns the number of initialized elements.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the vector contains no elements.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of elements the vector can hold without reallocating.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.storage.capacity()
    }

    /// Ensures that the vector can hold at least `additional` more elements.
    ///
    /// Amortized growth is geometric. Panics if capacity overflows or memory
    /// allocation fails. Use [`try_reserve`](Self::try_reserve) to handle errors.
    pub fn reserve(&mut self, additional: usize) {
        self.try_reserve(additional)
            .unwrap_or_else(|error| panic!("{error}"));
    }

    /// Fallibly ensures room for at least `additional` more elements.
    ///
    /// On error, the vector and its elements are unchanged. Successful growth
    /// is geometric, so repeated pushes have amortized O(1) cost.
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let required = self
            .len
            .checked_add(additional)
            .ok_or(TryReserveError::CapacityOverflow)?;
        if required <= self.capacity() {
            return Ok(());
        }

        if mem::size_of::<T>() == 0 {
            // ZST vectors use a logical capacity of usize::MAX.
            return Ok(());
        }

        let doubled = self.capacity().saturating_mul(2).max(4);
        self.storage.grow_to(required.max(doubled), self.len)
    }

    /// Removes all elements. The allocated capacity is retained.
    pub fn clear(&mut self) {
        self.truncate(0);
    }

    /// Shortens the vector to `new_len`, dropping removed elements.
    pub fn truncate(&mut self, new_len: usize) {
        while self.len > new_len {
            self.len -= 1;
            // SAFETY: the old final element is initialized and len is reduced
            // before dropping, so unwinding cannot drop it a second time.
            unsafe { ptr::drop_in_place(self.storage.as_ptr().add(self.len)) };
        }
    }

    /// Appends an element to the end of the vector.
    pub fn push(&mut self, value: T) {
        self.reserve(1);
        self.write_reserved(value);
    }

    /// Fallibly appends an element, returning it if capacity cannot be reserved.
    pub fn try_push(&mut self, value: T) -> Result<(), TryPushError<T>> {
        if let Err(error) = self.try_reserve(1) {
            return Err(TryPushError::new(value, error));
        }
        self.write_reserved(value);
        Ok(())
    }

    fn write_reserved(&mut self, value: T) {
        // SAFETY: reserve guarantees one spare slot; the slot is within the
        // allocation (or is a valid aligned ZST pointer), and is not initialized.
        unsafe { self.storage.as_ptr().add(self.len).write(value) };
        self.len += 1;
    }

    /// Removes and returns the last element, or `None` if the vector is empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        // SAFETY: the previous final slot was initialized and is no longer
        // included in the vector's initialized range.
        Some(unsafe { self.storage.as_ptr().add(self.len).read() })
    }

    /// Returns a reference to the element at `index`, or `None` if out of bounds.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            // SAFETY: index is in the initialized range and self keeps it alive.
            Some(unsafe { &*self.storage.as_ptr().add(index) })
        } else {
            None
        }
    }

    /// Returns a mutable reference to the element at `index`, or `None` if out of bounds.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            // SAFETY: index is in the initialized range; the exclusive borrow
            // of self guarantees unique access to the element.
            Some(unsafe { &mut *self.storage.as_ptr().add(index) })
        } else {
            None
        }
    }

    /// Returns a shared slice of all elements.
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: exactly len elements are initialized and ptr is aligned.
        unsafe { slice::from_raw_parts(self.storage.as_ptr(), self.len) }
    }

    /// Returns a mutable slice of all elements.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: exactly len elements are initialized; &mut self guarantees
        // exclusive access to the allocation.
        unsafe { slice::from_raw_parts_mut(self.storage.as_ptr(), self.len) }
    }

    /// Returns a raw pointer to the first element, or a dangling aligned pointer when empty.
    #[must_use]
    pub const fn as_ptr(&self) -> *const T {
        self.storage.as_ptr()
    }

    /// Returns a mutable raw pointer to the first element, or a dangling aligned pointer when empty.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.storage.as_ptr()
    }

    /// Returns an iterator over shared references to the elements.
    pub fn iter(&self) -> slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    /// Returns an iterator over mutable references to the elements.
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, T> {
        self.as_mut_slice().iter_mut()
    }

    /// Inserts `value` at `index`, shifting later elements to the right.
    ///
    /// Panics if `index > len`.
    pub fn insert(&mut self, index: usize, value: T) {
        assert!(index <= self.len, "insertion index out of bounds");
        self.reserve(1);
        // SAFETY: reserve leaves one spare slot. ptr::copy supports overlap and
        // shifts initialized elements to make index available.
        unsafe {
            let at = self.storage.as_ptr().add(index);
            ptr::copy(at, at.add(1), self.len - index);
            at.write(value);
        }
        self.len += 1;
    }

    /// Removes and returns the element at `index`, shifting later elements left.
    ///
    /// Returns `None` if `index` is out of bounds.
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len {
            return None;
        }
        // SAFETY: index is initialized. Reading it transfers ownership; the
        // following overlapping copy closes the gap before reducing len.
        unsafe {
            let at = self.storage.as_ptr().add(index);
            let value = at.read();
            ptr::copy(at.add(1), at, self.len - index - 1);
            self.len -= 1;
            Some(value)
        }
    }

    /// Removes and returns an element by replacing it with the last element.
    ///
    /// Returns `None` if `index` is out of bounds. The order of elements may change.
    pub fn swap_remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len {
            return None;
        }
        let last = self.len - 1;
        // SAFETY: both locations are initialized. If distinct, reading the last
        // element then writing it into index leaves exactly len - 1 live values.
        unsafe {
            let at = self.storage.as_ptr().add(index);
            let value = at.read();
            if index != last {
                at.write(self.storage.as_ptr().add(last).read());
            }
            self.len = last;
            Some(value)
        }
    }
}

impl<T> Drop for Vector<T> {
    fn drop(&mut self) {
        // SAFETY: the initialized range is exactly 0..len. `Storage` owns and
        // releases the allocation after the elements have been dropped.
        unsafe {
            ptr::drop_in_place(ptr::slice_from_raw_parts_mut(
                self.storage.as_ptr(),
                self.len,
            ));
        }
    }
}
