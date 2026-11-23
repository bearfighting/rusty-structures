//! Iterators for `Vector`.
//!
//! Borrowed iterators delegate to slice iterators. The owning iterator keeps
//! the allocation and transfers each element out exactly once.

use super::Vector;
use std::iter::FusedIterator;
use std::ptr;
use std::slice;

impl<'a, T> IntoIterator for &'a Vector<T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Vector<T> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// An owning iterator over a [`Vector`].
pub struct IntoIter<T> {
    vector: Vector<T>,
    front: usize,
    back: usize,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            return None;
        }
        let index = self.front;
        self.front += 1;
        // SAFETY: index is in the remaining initialized range and is advanced
        // before returning ownership of the value.
        Some(unsafe { self.vector.storage.as_ptr().add(index).read() })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.back - self.front;
        (remaining, Some(remaining))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let remaining = self.back - self.front;
        if n >= remaining {
            self.front = self.back;
            return None;
        }
        self.front += n;
        self.next()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            return None;
        }
        self.back -= 1;
        // SAFETY: back was in the remaining initialized range and is reduced
        // before returning ownership of the value.
        Some(unsafe { self.vector.storage.as_ptr().add(self.back).read() })
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}
impl<T> FusedIterator for IntoIter<T> {}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        // Drop only values that have not been yielded. Set the vector length to
        // zero first so its own destructor only releases the allocation.
        let remaining = self.back - self.front;
        self.vector.len = 0;
        // SAFETY: the remaining interval contains exactly the not-yet-yielded
        // initialized values; yielded values are outside this range.
        unsafe {
            ptr::drop_in_place(ptr::slice_from_raw_parts_mut(
                self.vector.storage.as_ptr().add(self.front),
                remaining,
            ));
        }
    }
}

impl<T> IntoIterator for Vector<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(mut self) -> Self::IntoIter {
        let back = self.len;
        self.len = 0;
        IntoIter {
            vector: self,
            front: 0,
            back,
        }
    }
}
