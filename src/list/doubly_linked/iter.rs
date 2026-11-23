//! Iterators over a doubly linked list.

use super::{DoublyLinkedList, Node};
use std::ptr::NonNull;

/// A shared double-ended iterator over a doubly linked list.
pub struct Iter<'a, T> {
    front: Option<&'a Node<T>>,
    back: Option<NonNull<Node<T>>>,
    remaining: usize,
}

impl<'a, T> Iter<'a, T> {
    pub(super) fn new(list: &'a DoublyLinkedList<T>) -> Self {
        Self {
            front: list.head.as_deref(),
            back: list.tail,
            remaining: list.len,
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let node = self.front.expect("remaining elements have a front node");
        self.front = node.next.as_deref();
        self.remaining -= 1;
        Some(&node.value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let tail = self.back.expect("remaining elements have a back node");
        // SAFETY: Iter borrows the list for 'a, so the tail node remains alive.
        let node = unsafe { tail.as_ref() };
        self.back = node.prev;
        self.remaining -= 1;
        Some(&node.value)
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {}
impl<T> std::iter::FusedIterator for Iter<'_, T> {}

/// A mutable iterator over a doubly linked list, from front to back.
pub struct IterMut<'a, T> {
    pub(super) next: Option<&'a mut Node<T>>,
    pub(super) remaining: usize,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.next.take()?;
        self.next = node.next.as_deref_mut();
        self.remaining -= 1;
        Some(&mut node.value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T> ExactSizeIterator for IterMut<'_, T> {}
impl<T> std::iter::FusedIterator for IterMut<'_, T> {}

/// An owning double-ended iterator over a doubly linked list.
pub struct IntoIter<T> {
    list: DoublyLinkedList<T>,
}

impl<T> IntoIter<T> {
    pub(super) fn new(list: DoublyLinkedList<T>) -> Self {
        Self { list }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.list.len, Some(self.list.len))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.list.pop_back()
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}
impl<T> std::iter::FusedIterator for IntoIter<T> {}
