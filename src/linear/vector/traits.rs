//! Standard trait implementations for [`Vector`](super::Vector).

use super::Vector;
use std::fmt;
use std::ops::{Deref, DerefMut, Index, IndexMut};

impl<T> Default for Vector<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Clone for Vector<T> {
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }
}

impl<T: fmt::Debug> fmt::Debug for Vector<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T: PartialEq> PartialEq for Vector<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Eq> Eq for Vector<T> {}

impl<T> Deref for Vector<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> DerefMut for Vector<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T> Index<usize> for Vector<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl<T> IndexMut<usize> for Vector<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.as_mut_slice()[index]
    }
}

impl<T> Extend<T> for Vector<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iterator = iter.into_iter();
        self.reserve(iterator.size_hint().0);
        for value in iterator {
            self.push(value);
        }
    }
}

impl<T> FromIterator<T> for Vector<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut vector = Self::new();
        vector.extend(iter);
        vector
    }
}

impl<T, const N: usize> From<[T; N]> for Vector<T> {
    fn from(array: [T; N]) -> Self {
        let mut vector = Self::with_capacity(N);
        vector.extend(array);
        vector
    }
}
