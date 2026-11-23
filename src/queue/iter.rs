//! Iterators exposed by the queue API.

use super::Queue;
use std::iter::FusedIterator;

/// A shared iterator over queue values, from front to back.
pub use crate::foundation::ring_buffer::RingIter as Iter;

/// An owning iterator that removes values from the front of a queue.
pub struct IntoIter<T> {
    queue: Queue<T>,
}

impl<T> IntoIter<T> {
    pub(super) fn new(queue: Queue<T>) -> Self {
        Self { queue }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.queue.dequeue()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.queue.len();
        (len, Some(len))
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}
impl<T> FusedIterator for IntoIter<T> {}
