//! A first-in, first-out collection.
//!
//! `Queue<T>` presents the familiar FIFO operations. The ring-buffer details
//! are kept in a private helper so the public API can be read separately from
//! the indexing and slot-management code.
//!
//! # Code map
//!
//! - Start here for the public queue operations.
//! - `foundation/ring_buffer.rs` contains the shared circular storage.
//! - `linear/queue/iter.rs` contains the queue iterators.
//! - `linear/queue/errors.rs` contains fallible enqueue errors.
//! - `linear/queue/traits.rs` contains collection and `IntoIterator` support.

mod errors;
mod iter;
mod traits;

use crate::foundation::ring_buffer::RingBuffer;
pub use errors::TryEnqueueError;
pub use iter::{IntoIter, Iter};

/// A first-in, first-out collection.
///
/// Values leave the queue in the order they were inserted. Enqueue, dequeue,
/// and front access take O(1) time except when enqueue grows the buffer, which
/// takes O(n) time. Growth is geometric, so enqueue is amortized O(1).
///
/// # Examples
///
/// ```
/// use data_structures::Queue;
///
/// let mut queue = Queue::new();
/// queue.enqueue("first");
/// queue.enqueue("second");
///
/// assert_eq!(queue.front(), Some(&"first"));
/// assert_eq!(queue.dequeue(), Some("first"));
/// assert_eq!(queue.dequeue(), Some("second"));
/// assert_eq!(queue.dequeue(), None);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Queue<T> {
    buffer: RingBuffer<T>,
}

impl<T> Queue<T> {
    /// Creates an empty queue without allocating memory.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buffer: RingBuffer::new(),
        }
    }

    /// Creates an empty queue with room for at least `capacity` values.
    ///
    /// Panics if the buffer cannot be allocated. Use
    /// [`try_with_capacity`](Self::try_with_capacity) to handle the error.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::try_with_capacity(capacity).unwrap_or_else(|error| panic!("{error}"))
    }

    /// Fallibly creates an empty queue with room for at least `capacity` values.
    pub fn try_with_capacity(capacity: usize) -> Result<Self, crate::TryReserveError> {
        Ok(Self {
            buffer: RingBuffer::try_with_capacity(capacity)?,
        })
    }

    /// Returns the number of values in the queue.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns whether the queue contains no values.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Returns the number of values the queue can hold without growing.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Adds a value to the back of the queue.
    ///
    /// Panics if the buffer cannot grow. Use [`try_enqueue`](Self::try_enqueue)
    /// to handle capacity or allocation errors.
    pub fn enqueue(&mut self, value: T) {
        self.try_enqueue(value)
            .unwrap_or_else(|error| panic!("{error}"));
    }

    /// Fallibly adds a value to the back of the queue.
    ///
    /// If the queue cannot grow, the returned error contains the unqueued value
    /// and the queue remains unchanged.
    pub fn try_enqueue(&mut self, value: T) -> Result<(), TryEnqueueError<T>> {
        self.buffer
            .try_push_back(value)
            .map_err(|(value, error)| TryEnqueueError::new(value, error))
    }

    /// Removes and returns the value at the front, or `None` if empty.
    pub fn dequeue(&mut self) -> Option<T> {
        self.buffer.pop_front()
    }

    /// Returns a shared reference to the value at the front, or `None` if empty.
    #[must_use]
    pub fn front(&self) -> Option<&T> {
        self.buffer.front()
    }

    /// Returns a mutable reference to the value at the front, or `None` if empty.
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.buffer.front_mut()
    }

    /// Returns a shared reference to the value at the back, or `None` if empty.
    #[must_use]
    pub fn back(&self) -> Option<&T> {
        self.buffer.back()
    }

    /// Removes all values while keeping the allocated buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Returns an iterator from the front of the queue to the back.
    pub fn iter(&self) -> Iter<'_, T> {
        self.buffer.iter()
    }

    /// Returns a mutable iterator from the front of the queue to the back.
    pub fn iter_mut(
        &mut self,
    ) -> impl DoubleEndedIterator<Item = &mut T> + std::iter::FusedIterator {
        self.buffer.iter_mut()
    }
}
