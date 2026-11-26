//! A fixed-capacity FIFO queue backed by the shared ring buffer.

use crate::TryReserveError;
use crate::foundation::ring_buffer::RingBuffer;

pub use crate::foundation::ring_buffer::RingIter as Iter;

/// A FIFO queue that never grows after construction.
///
/// Enqueue and dequeue take O(1) time. Enqueueing into a full queue returns the
/// value to the caller. Storage is allocated once when the queue is created;
/// a zero-capacity queue rejects every enqueue.
///
/// # Examples
///
/// ```
/// use data_structures::FixedCapacityQueue;
///
/// let mut queue = FixedCapacityQueue::with_capacity(2);
/// queue.try_enqueue("first").unwrap();
/// queue.try_enqueue("second").unwrap();
/// assert_eq!(queue.try_enqueue("third"), Err("third"));
/// assert_eq!(queue.dequeue(), Some("first"));
/// queue.try_enqueue("third").unwrap();
/// assert_eq!(queue.iter().copied().collect::<Vec<_>>(), ["second", "third"]);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixedCapacityQueue<T> {
    buffer: RingBuffer<T>,
}

impl<T> FixedCapacityQueue<T> {
    /// Creates an empty fixed-capacity queue.
    ///
    /// Panics if the requested storage cannot be allocated. Use
    /// [`try_with_capacity`](Self::try_with_capacity) to handle allocation errors.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::try_with_capacity(capacity)
            .unwrap_or_else(|error| panic!("could not allocate fixed queue: {error}"))
    }

    /// Creates an empty fixed-capacity queue, returning allocation errors.
    pub fn try_with_capacity(capacity: usize) -> Result<Self, TryReserveError> {
        Ok(Self {
            buffer: RingBuffer::try_with_capacity(capacity)?,
        })
    }

    /// Returns the number of queued values.
    #[must_use]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns the maximum number of values the queue can hold.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Returns whether the queue has no values.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Returns whether the queue has reached capacity.
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    /// Adds a value to the back, returning it if the queue is full.
    pub fn try_enqueue(&mut self, value: T) -> Result<(), T> {
        self.buffer.try_push_back_fixed(value)
    }

    /// Removes and returns the value at the front, or `None` if empty.
    pub fn dequeue(&mut self) -> Option<T> {
        self.buffer.pop_front()
    }

    /// Returns a shared reference to the front value.
    #[must_use]
    pub fn front(&self) -> Option<&T> {
        self.buffer.front()
    }

    /// Returns a mutable reference to the front value.
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.buffer.front_mut()
    }

    /// Returns a shared reference to the back value.
    #[must_use]
    pub fn back(&self) -> Option<&T> {
        self.buffer.back()
    }

    /// Removes all values while retaining the fixed storage.
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

impl<'a, T> IntoIterator for &'a FixedCapacityQueue<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::FixedCapacityQueue;

    #[test]
    fn wraps_and_reuses_slots_without_growing() {
        let mut queue = FixedCapacityQueue::with_capacity(3);
        assert_eq!(queue.capacity(), 3);
        queue.try_enqueue(1).unwrap();
        queue.try_enqueue(2).unwrap();
        queue.try_enqueue(3).unwrap();
        assert!(queue.is_full());
        assert_eq!(queue.try_enqueue(4), Err(4));
        assert_eq!(queue.dequeue(), Some(1));
        queue.try_enqueue(4).unwrap();
        assert_eq!(queue.iter().copied().collect::<Vec<_>>(), [2, 3, 4]);
        assert_eq!(queue.capacity(), 3);
        assert_eq!(queue.dequeue(), Some(2));
        assert_eq!(queue.dequeue(), Some(3));
        assert_eq!(queue.dequeue(), Some(4));
        assert!(queue.is_empty());
    }

    #[test]
    fn handles_zero_capacity_and_clear() {
        let mut empty = FixedCapacityQueue::<u8>::with_capacity(0);
        assert!(empty.is_full());
        assert_eq!(empty.try_enqueue(7), Err(7));

        let mut queue = FixedCapacityQueue::with_capacity(2);
        queue.try_enqueue(1).unwrap();
        queue.try_enqueue(2).unwrap();
        queue.clear();
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.capacity(), 2);
        queue.try_enqueue(3).unwrap();
        assert_eq!(queue.front(), Some(&3));
        assert_eq!(queue.back(), Some(&3));
    }
}
