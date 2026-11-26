//! D-ary max-heap backed by the project's `Vector`.

use crate::Vector;

/// A d-ary max-heap with a compile-time branching factor `D`.
///
/// `D` must be at least two. The greatest value is at the root. `push` takes
/// O(log_D n) comparisons; `pop` takes O(D log_D n), since each level scans up
/// to `D` children. `peek` is O(1), building from a vector is O(n), and storage
/// is O(n). Iteration visits the internal heap layout, not sorted order.
///
/// # Examples
///
/// ```
/// use data_structures::DaryHeap;
///
/// let mut heap = DaryHeap::<_, 4>::new();
/// for value in [4, 9, 2, 7, 1] {
///     heap.push(value);
/// }
/// assert_eq!(heap.peek(), Some(&9));
/// assert_eq!(heap.pop(), Some(9));
/// ```
#[derive(Clone, Debug)]
pub struct DaryHeap<T, const D: usize = 4> {
    items: Vector<T>,
}

impl<T, const D: usize> DaryHeap<T, D> {
    /// Creates an empty d-ary heap.
    ///
    /// # Panics
    ///
    /// Panics if `D < 2`.
    #[must_use]
    pub fn new() -> Self {
        assert!(D >= 2, "DaryHeap branching factor must be at least 2");
        Self {
            items: Vector::new(),
        }
    }

    /// Creates an empty heap with room for at least `capacity` values.
    ///
    /// # Panics
    ///
    /// Panics if `D < 2`.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        assert!(D >= 2, "DaryHeap branching factor must be at least 2");
        Self {
            items: Vector::with_capacity(capacity),
        }
    }

    /// Returns the number of values in the heap.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns the number of values the heap can hold without reallocating.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.items.capacity()
    }

    /// Returns whether the heap contains no values.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Removes all values from the heap.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Returns the greatest value without removing it.
    #[must_use]
    pub fn peek(&self) -> Option<&T> {
        self.items.first()
    }

    /// Returns an iterator over the internal heap layout.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.items.iter()
    }
}

impl<T: Ord, const D: usize> DaryHeap<T, D> {
    /// Adds a value and restores the heap invariant.
    pub fn push(&mut self, value: T) {
        self.items.push(value);
        self.sift_up(self.len() - 1);
    }

    /// Removes and returns the greatest value, or `None` if empty.
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
    ///
    /// # Panics
    ///
    /// Panics if `D < 2`.
    pub fn from_vec(values: Vec<T>) -> Self {
        assert!(D >= 2, "DaryHeap branching factor must be at least 2");
        let mut heap = Self {
            items: values.into_iter().collect(),
        };
        for parent in (0..heap.len()).rev() {
            heap.sift_down(parent);
        }
        heap
    }

    fn sift_up(&mut self, mut child: usize) {
        while child > 0 {
            let parent = (child - 1) / D;
            if self.items[parent] >= self.items[child] {
                break;
            }
            self.items.as_mut_slice().swap(parent, child);
            child = parent;
        }
    }

    fn sift_down(&mut self, mut parent: usize) {
        let len = self.len();
        while let Some(first_child) = parent.checked_mul(D).and_then(|value| value.checked_add(1)) {
            if first_child >= len {
                break;
            }
            let end_child = first_child.saturating_add(D).min(len);
            let mut greatest = first_child;
            for child in first_child + 1..end_child {
                if self.items[child] > self.items[greatest] {
                    greatest = child;
                }
            }
            if self.items[parent] >= self.items[greatest] {
                break;
            }
            self.items.as_mut_slice().swap(parent, greatest);
            parent = greatest;
        }
    }
}

impl<T, const D: usize> Default for DaryHeap<T, D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord, const D: usize> Extend<T> for DaryHeap<T, D> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, values: I) {
        for value in values {
            self.push(value);
        }
    }
}

impl<T: Ord, const D: usize> FromIterator<T> for DaryHeap<T, D> {
    fn from_iter<I: IntoIterator<Item = T>>(values: I) -> Self {
        Self::from_vec(values.into_iter().collect())
    }
}

impl<T: Ord, const D: usize> From<Vec<T>> for DaryHeap<T, D> {
    fn from(values: Vec<T>) -> Self {
        Self::from_vec(values)
    }
}

impl<'a, T, const D: usize> IntoIterator for &'a DaryHeap<T, D> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, const D: usize> IntoIterator for DaryHeap<T, D> {
    type Item = T;
    type IntoIter = crate::vector::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::DaryHeap;
    use std::collections::BinaryHeap;

    fn matches_binary_heap<const D: usize>() {
        let mut heap = DaryHeap::<_, D>::new();
        let mut expected = BinaryHeap::new();
        let mut state = 17u64;
        for step in 0..800 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            if step % 3 != 0 {
                let value = (state >> 20) as i64;
                heap.push(value);
                expected.push(value);
            } else {
                assert_eq!(heap.pop(), expected.pop());
            }
            assert_eq!(heap.peek(), expected.peek());
            assert_eq!(heap.len(), expected.len());
        }
        while !expected.is_empty() {
            assert_eq!(heap.pop(), expected.pop());
        }
        assert!(heap.is_empty());
    }

    #[test]
    fn preserves_order_for_multiple_branching_factors() {
        matches_binary_heap::<2>();
        matches_binary_heap::<3>();
        matches_binary_heap::<4>();
        matches_binary_heap::<8>();
    }

    #[test]
    #[should_panic(expected = "branching factor must be at least 2")]
    fn rejects_branching_factor_below_two() {
        let _ = DaryHeap::<i32, 1>::new();
    }

    #[test]
    fn builds_from_values() {
        let mut heap = DaryHeap::<_, 5>::from_vec(vec![2, 9, 1, 7, 4]);
        assert_eq!(heap.pop(), Some(9));
        assert_eq!(heap.pop(), Some(7));
        assert_eq!(heap.pop(), Some(4));
    }
}
