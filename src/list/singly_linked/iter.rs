//! Iterators for the singly linked list.

use super::{Node, SinglyLinkedList};
use std::iter::FusedIterator;

/// An iterator over shared references to a singly linked list.
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
    remaining: usize,
}

impl<'a, T> Iter<'a, T> {
    pub(super) fn new(list: &'a SinglyLinkedList<T>) -> Self {
        Self {
            next: list.head.as_deref(),
            remaining: list.len,
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.next?;
        self.next = node.next.as_deref();
        self.remaining -= 1;
        Some(&node.value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {}
impl<T> FusedIterator for Iter<'_, T> {}

/// A mutable iterator over a singly linked list.
pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
    remaining: usize,
}

impl<'a, T> IterMut<'a, T> {
    pub(super) fn new(list: &'a mut SinglyLinkedList<T>) -> Self {
        Self {
            next: list.head.as_deref_mut(),
            remaining: list.len,
        }
    }
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
impl<T> FusedIterator for IterMut<'_, T> {}

/// An owning iterator that removes list elements from the front.
pub struct IntoIter<T> {
    list: SinglyLinkedList<T>,
}

impl<T> IntoIter<T> {
    pub(super) fn new(list: SinglyLinkedList<T>) -> Self {
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

impl<T> ExactSizeIterator for IntoIter<T> {}
impl<T> FusedIterator for IntoIter<T> {}
