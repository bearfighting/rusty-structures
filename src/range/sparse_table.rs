//! Static O(1) range queries for associative, idempotent operations.

/// An associative, idempotent operation usable by a [`SparseTable`].
///
/// `combine` must be associative and idempotent (`combine(x, x) == x`). The
/// sparse-table query combines two overlapping power-of-two ranges, so both
/// laws are required. The type system cannot verify them.
pub trait IdempotentOp<T> {
    /// Combines two values.
    fn combine(&self, left: &T, right: &T) -> T;
}

/// Operation that selects the smaller value.
#[derive(Clone, Copy, Debug, Default)]
pub struct Minimum;

impl<T: Ord + Clone> IdempotentOp<T> for Minimum {
    fn combine(&self, left: &T, right: &T) -> T {
        if left <= right {
            left.clone()
        } else {
            right.clone()
        }
    }
}

/// Operation that selects the larger value.
#[derive(Clone, Copy, Debug, Default)]
pub struct Maximum;

impl<T: Ord + Clone> IdempotentOp<T> for Maximum {
    fn combine(&self, left: &T, right: &T) -> T {
        if left >= right {
            left.clone()
        } else {
            right.clone()
        }
    }
}

/// Precomputes static range aggregates for an associative, idempotent operation.
///
/// Construction takes O(n log n) time and space. A non-empty half-open range
/// query takes O(1) time. Values cannot be updated after construction; build a
/// new table when the underlying data changes.
///
/// # Examples
///
/// ```
/// use data_structures::range::{Maximum, Minimum, SparseTable};
///
/// let values = [7, 2, 9, 4, 6];
/// let minima = SparseTable::new(&values, Minimum);
/// let maxima = SparseTable::new(&values, Maximum);
/// assert_eq!(minima.query(1, 5), Some(2));
/// assert_eq!(maxima.query(1, 4), Some(9));
/// ```
#[derive(Clone, Debug)]
pub struct SparseTable<T, O> {
    len: usize,
    levels: Vec<Vec<T>>,
    operation: O,
}

impl<T: Clone, O: IdempotentOp<T>> SparseTable<T, O> {
    /// Builds a table over `values` using `operation`.
    #[must_use]
    pub fn new(values: &[T], operation: O) -> Self {
        let len = values.len();
        let mut levels = Vec::new();
        if len > 0 {
            levels.push(values.to_vec());
            let mut width = 2;
            while width <= len {
                let half = width / 2;
                let previous = levels.last().expect("base level exists");
                let mut current = Vec::with_capacity(len - width + 1);
                for start in 0..=len - width {
                    current.push(operation.combine(&previous[start], &previous[start + half]));
                }
                levels.push(current);
                width *= 2;
            }
        }
        Self {
            len,
            levels,
            operation,
        }
    }

    /// Returns the number of values indexed by the table.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the table contains no values.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Queries a non-empty half-open range `start..end` in O(1) time.
    ///
    /// Returns `None` unless `start < end <= len`.
    #[must_use]
    pub fn query(&self, start: usize, end: usize) -> Option<T> {
        if start >= end || end > self.len {
            return None;
        }
        let range_len = end - start;
        let level = usize::BITS as usize - 1 - range_len.leading_zeros() as usize;
        let width = 1usize << level;
        let left = &self.levels[level][start];
        let right = &self.levels[level][end - width];
        Some(self.operation.combine(left, right))
    }
}

#[cfg(test)]
mod tests {
    use super::{Maximum, Minimum, SparseTable};

    #[test]
    fn answers_minimum_and_maximum_queries() {
        let values = [7, 2, 9, 4, 6, 1, 5];
        let minima = SparseTable::new(&values, Minimum);
        let maxima = SparseTable::new(&values, Maximum);
        assert_eq!(minima.query(1, 5), Some(2));
        assert_eq!(minima.query(5, 6), Some(1));
        assert_eq!(maxima.query(1, 5), Some(9));
        assert_eq!(maxima.query(0, 7), Some(9));
    }

    #[test]
    fn handles_empty_data_and_rejects_empty_or_invalid_ranges() {
        let empty = SparseTable::new(&[] as &[i32], Minimum);
        assert!(empty.is_empty());
        assert_eq!(empty.query(0, 0), None);

        let table = SparseTable::new(&[3, 1], Minimum);
        assert_eq!(table.query(0, 0), None);
        assert_eq!(table.query(2, 1), None);
        assert_eq!(table.query(0, 3), None);
    }
}
