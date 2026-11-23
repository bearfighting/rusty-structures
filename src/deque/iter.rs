//! Iterators exposed by the deque API.

use super::Deque;
use crate::foundation::ring_buffer::RingBuffer;
use std::iter::FusedIterator;

/// A shared iterator over deque values, from front to back.
pub use crate::foundation::ring_buffer::RingIter as Iter;

/// An owning iterator that removes values from either end of the deque.
pub struct IntoIter<T> {
    buffer: RingBuffer<T>,
}

impl<T> IntoIter<T> {
    pub(super) fn new(deque: Deque<T>) -> Self {
        Self {
            buffer: deque.buffer,
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.buffer.len();
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.buffer.pop_back()
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}
impl<T> FusedIterator for IntoIter<T> {}
