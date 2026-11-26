//! Fenwick tree for prefix and range sums with point updates.

/// Stores `i64` values and supports point additions and prefix sums.
///
/// Public ranges are half-open: `prefix_sum(end)` sums `0..end`, and
/// `range_sum(start, end)` sums `start..end`. Internally, a one-based array
/// stores partial sums over ranges determined by each index's least
/// significant set bit. Updates and queries take O(log n) time; storage is
/// O(n). All intermediate and returned sums must fit in `i64`.
///
/// # Examples
///
/// ```
/// use data_structures::FenwickTree;
///
/// let mut sums = FenwickTree::from_values(&[3, -2, 5, 4]);
/// assert_eq!(sums.prefix_sum(3), Some(6));
/// assert_eq!(sums.range_sum(1, 4), Some(7));
/// sums.add(1, 6);
/// assert_eq!(sums.range_sum(1, 4), Some(13));
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FenwickTree {
    tree: Vec<i64>,
}

impl FenwickTree {
    /// Creates a tree for `len` zero-valued elements.
    #[must_use]
    pub fn new(len: usize) -> Self {
        Self {
            tree: vec![0; len.saturating_add(1)],
        }
    }

    /// Builds a tree from values in O(n) time.
    #[must_use]
    pub fn from_values(values: &[i64]) -> Self {
        let mut tree = vec![0; values.len().saturating_add(1)];
        for (index, &value) in values.iter().enumerate() {
            tree[index + 1] = value;
        }
        for index in 1..tree.len() {
            let parent = index + (index & index.wrapping_neg());
            if parent < tree.len() {
                tree[parent] += tree[index];
            }
        }
        Self { tree }
    }

    /// Returns the number of values represented by the tree.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tree.len() - 1
    }

    /// Returns whether the tree contains no values.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Adds `delta` to the value at `index`.
    ///
    /// Returns `None` if `index` is outside `0..len`.
    pub fn add(&mut self, index: usize, delta: i64) -> Option<()> {
        if index >= self.len() {
            return None;
        }
        let mut current = index + 1;
        while current < self.tree.len() {
            self.tree[current] += delta;
            current += current & current.wrapping_neg();
        }
        Some(())
    }

    /// Returns the sum of values in the half-open prefix `0..end`.
    ///
    /// Returns `None` if `end > len`; `end == len` is valid.
    #[must_use]
    pub fn prefix_sum(&self, end: usize) -> Option<i64> {
        if end > self.len() {
            return None;
        }
        let mut current = end;
        let mut sum = 0;
        while current > 0 {
            sum += self.tree[current];
            current -= current & current.wrapping_neg();
        }
        Some(sum)
    }

    /// Returns the sum of values in the half-open range `start..end`.
    ///
    /// Returns `None` unless `start <= end <= len`.
    #[must_use]
    pub fn range_sum(&self, start: usize, end: usize) -> Option<i64> {
        if start > end {
            return None;
        }
        Some(self.prefix_sum(end)? - self.prefix_sum(start)?)
    }
}
