//! Iterators over hash set values.

use super::HashSet;
use crate::hash::map;
use std::hash::BuildHasher;

/// An iterator over values in arbitrary order.
pub struct Iter<'a, T, S> {
    inner: map::Iter<'a, T, (), S>,
}

impl<'a, T, S> Iter<'a, T, S> {
    pub(super) fn new(inner: map::Iter<'a, T, (), S>) -> Self {
        Self { inner }
    }
}

impl<'a, T, S: BuildHasher> Iterator for Iter<'a, T, S> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(value, ())| value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<T, S: BuildHasher> ExactSizeIterator for Iter<'_, T, S> {}
impl<T, S: BuildHasher> std::iter::FusedIterator for Iter<'_, T, S> {}

/// An owning iterator over values in arbitrary order.
pub struct IntoIter<T> {
    inner: map::IntoIter<T, ()>,
}

impl<T> IntoIter<T> {
    pub(super) fn new<S>(set: HashSet<T, S>) -> Self {
        Self {
            inner: set.map.into_iter(),
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(value, ())| value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}
impl<T> std::iter::FusedIterator for IntoIter<T> {}

impl<'a, T, S: BuildHasher> IntoIterator for &'a HashSet<T, S> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T, S>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, S> IntoIterator for HashSet<T, S> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}
