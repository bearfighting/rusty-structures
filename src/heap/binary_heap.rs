//! Binary max-heap implementation.
//!
//! The heap stores its complete binary tree in a `Vector`. Children of index
//! `i` are at `2 * i + 1` and `2 * i + 2`; every parent is at least as large
//! as either child.

use crate::Vector;

/// A binary max-heap.
///
/// The greatest element is at the root. `push` and `pop` take O(log n) time;
/// `peek` takes O(1). Iteration visits the internal heap layout, not sorted
/// order.
///
/// # Examples
///
/// ```
/// use data_structures::BinaryHeap;
///
/// let mut heap = BinaryHeap::new();
/// heap.push(4);
/// heap.push(9);
/// heap.push(2);
/// assert_eq!(heap.peek(), Some(&9));
/// assert_eq!(heap.pop(), Some(9));
/// ```
#[derive(Clone, Debug)]
pub struct BinaryHeap<T> {
    items: Vector<T>,
}

impl<T> BinaryHeap<T> {
    /// Creates an empty heap.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            items: Vector::new(),
        }
    }

    /// Creates an empty heap with room for at least `capacity` elements.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vector::with_capacity(capacity),
        }
    }

    /// Returns the number of elements in the heap.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns the number of elements the heap can hold without reallocating.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.items.capacity()
    }

    /// Returns whether the heap is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Removes all elements from the heap.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Returns the greatest element without removing it.
    #[must_use]
    pub fn peek(&self) -> Option<&T> {
        self.items.first()
    }

    /// Returns an iterator over the internal heap layout.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.items.iter()
    }
}

impl<T: Ord> BinaryHeap<T> {
    /// Adds an element and restores the heap invariant.
    pub fn push(&mut self, value: T) {
        self.items.push(value);
        self.sift_up(self.len() - 1);
    }

    /// Removes and returns the greatest element, or `None` if empty.
    pub fn pop(&mut self) -> Option<T> {
        let last = self.items.pop()?;
        if self.is_empty() {
            return Some(last);
        }

        let greatest = std::mem::replace(&mut self.items[0], last);
        self.sift_down(0);
        Some(greatest)
    }

    /// Builds a heap from values in O(n) time.
    pub fn from_vec(values: Vec<T>) -> Self {
        Self::from_items(values.into_iter().collect())
    }

    fn from_items(items: Vector<T>) -> Self {
        let mut heap = Self { items };
        for parent in (0..heap.len() / 2).rev() {
            heap.sift_down(parent);
        }
        heap
    }

    fn sift_up(&mut self, mut child: usize) {
        while child > 0 {
            let parent = (child - 1) / 2;
            if self.items[parent] >= self.items[child] {
                break;
            }
            self.items.as_mut_slice().swap(parent, child);
            child = parent;
        }
    }

    fn sift_down(&mut self, mut parent: usize) {
        let len = self.len();
        loop {
            if len < 2 || parent > (len - 2) / 2 {
                break;
            }
            let left = parent * 2 + 1;
            let right = left + 1;
            let greater = if right < len && self.items[right] > self.items[left] {
                right
            } else {
                left
            };
            if self.items[parent] >= self.items[greater] {
                break;
            }
            self.items.as_mut_slice().swap(parent, greater);
            parent = greater;
        }
    }
}

impl<T> Default for BinaryHeap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Extend<T> for BinaryHeap<T>
where
    T: Ord,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.push(value);
        }
    }
}

impl<T: Ord> FromIterator<T> for BinaryHeap<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_items(iter.into_iter().collect())
    }
}

impl<T: Ord> From<Vec<T>> for BinaryHeap<T> {
    fn from(values: Vec<T>) -> Self {
        Self::from_vec(values)
    }
}

impl<'a, T> IntoIterator for &'a BinaryHeap<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> IntoIterator for BinaryHeap<T> {
    type Item = T;
    type IntoIter = crate::vector::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}
