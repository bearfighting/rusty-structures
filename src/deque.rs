//! A double-ended queue backed by the shared ring buffer.
//!
//! A deque supports insertion and removal at both ends. The storage and
//! circular index calculations live in the crate-private ring buffer module.

mod errors;
mod iter;
mod traits;

use crate::ring_buffer::RingBuffer;
pub use errors::TryDequePushError;
pub use iter::{IntoIter, Iter};

/// A double-ended queue.
///
/// Pushing, popping, and inspecting either end take O(1) time, except when a
/// push grows the buffer. Growth takes O(n) time; pushes are amortized O(1).
///
/// # Examples
///
/// ```
/// use data_structures::Deque;
///
/// let mut values = Deque::new();
/// values.push_back(2);
/// values.push_front(1);
/// values.push_back(3);
///
/// assert_eq!(values.pop_front(), Some(1));
/// assert_eq!(values.pop_back(), Some(3));
/// assert_eq!(values.pop_front(), Some(2));
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Deque<T> {
    buffer: RingBuffer<T>,
}

impl<T> Deque<T> {
    /// Creates an empty deque without allocating memory.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buffer: RingBuffer::new(),
        }
    }

    /// Creates an empty deque with room for at least `capacity` values.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::try_with_capacity(capacity).unwrap_or_else(|error| panic!("{error}"))
    }

    /// Fallibly creates an empty deque with room for at least `capacity` values.
    pub fn try_with_capacity(capacity: usize) -> Result<Self, crate::TryReserveError> {
        Ok(Self {
            buffer: RingBuffer::try_with_capacity(capacity)?,
        })
    }

    /// Returns the number of values in the deque.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns whether the deque contains no values.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Returns the number of values the deque can hold without growing.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Adds a value to the front of the deque.
    pub fn push_front(&mut self, value: T) {
        self.try_push_front(value)
            .unwrap_or_else(|error| panic!("{error}"));
    }

    /// Fallibly adds a value to the front, returning it if allocation fails.
    pub fn try_push_front(&mut self, value: T) -> Result<(), TryDequePushError<T>> {
        self.buffer
            .try_push_front(value)
            .map_err(|(value, error)| TryDequePushError::new(value, error))
    }

    /// Adds a value to the back of the deque.
    pub fn push_back(&mut self, value: T) {
        self.try_push_back(value)
            .unwrap_or_else(|error| panic!("{error}"));
    }

    /// Fallibly adds a value to the back, returning it if allocation fails.
    pub fn try_push_back(&mut self, value: T) -> Result<(), TryDequePushError<T>> {
        self.buffer
            .try_push_back(value)
            .map_err(|(value, error)| TryDequePushError::new(value, error))
    }

    /// Removes and returns the front value, or `None` if empty.
    pub fn pop_front(&mut self) -> Option<T> {
        self.buffer.pop_front()
    }

    /// Removes and returns the back value, or `None` if empty.
    pub fn pop_back(&mut self) -> Option<T> {
        self.buffer.pop_back()
    }

    /// Returns a shared reference to the front value, or `None` if empty.
    #[must_use]
    pub fn front(&self) -> Option<&T> {
        self.buffer.front()
    }

    /// Returns a mutable reference to the front value, or `None` if empty.
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.buffer.front_mut()
    }

    /// Returns a shared reference to the back value, or `None` if empty.
    #[must_use]
    pub fn back(&self) -> Option<&T> {
        self.buffer.back()
    }

    /// Returns a mutable reference to the back value, or `None` if empty.
    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.buffer.back_mut()
    }

    /// Removes all values while keeping the allocated buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Returns an iterator from the front of the deque to the back.
    pub fn iter(&self) -> Iter<'_, T> {
        self.buffer.iter()
    }

    /// Returns a mutable iterator from the front of the deque to the back.
    pub fn iter_mut(
        &mut self,
    ) -> impl DoubleEndedIterator<Item = &mut T> + std::iter::FusedIterator {
        self.buffer.iter_mut()
    }
}
