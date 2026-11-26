//! Segment tree for point assignment and associative range queries.

use crate::algebra::TryMonoid as Aggregate;

/// Error returned when adding two `i64` values overflows.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SumOverflow;

impl std::fmt::Display for SumOverflow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("i64 sum overflow")
    }
}

impl std::error::Error for SumOverflow {}

impl Aggregate for i64 {
    type Error = SumOverflow;

    fn identity() -> Self {
        0
    }

    fn try_combine(left: &Self, right: &Self) -> Result<Self, Self::Error> {
        left.checked_add(*right).ok_or(SumOverflow)
    }
}

/// Stores monoid values and answers half-open range queries.
///
/// The tree uses a flat array whose leaves occupy a power-of-two-sized region;
/// unused leaves contain the monoid identity. Point assignments and range
/// queries take O(log n) time, building from values takes O(n), and storage is
/// O(n). Errors produced by the monoid are returned without changing the tree;
/// for the built-in `i64` monoid, addition overflow returns [`SumOverflow`].
///
/// # Examples
///
/// ```
/// use data_structures::SegmentTree;
///
/// let mut sums = SegmentTree::from_values(&[2, 4, -1, 6])?;
/// assert_eq!(sums.range_sum(1, 4)?, Some(9));
/// sums.set(2, 5)?;
/// assert_eq!(sums.range_sum(1, 4)?, Some(15));
/// # Ok::<(), data_structures::SumOverflow>(())
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SegmentTree<T: Aggregate> {
    len: usize,
    leaf_count: usize,
    nodes: Vec<T>,
}

impl<T: Aggregate> SegmentTree<T> {
    /// Creates a tree for `len` identity-valued elements.
    #[must_use]
    pub fn new(len: usize) -> Self {
        let leaf_count = len.next_power_of_two().max(1);
        Self {
            len,
            leaf_count,
            nodes: vec![T::identity(); leaf_count * 2],
        }
    }

    /// Builds a tree from values in O(n) time.
    pub fn from_values(values: &[T]) -> Result<Self, T::Error> {
        let mut tree = Self::new(values.len());
        let leaves = tree.leaf_count;
        tree.nodes[leaves..leaves + values.len()].clone_from_slice(values);
        for node in (1..leaves).rev() {
            tree.nodes[node] = T::try_combine(&tree.nodes[node * 2], &tree.nodes[node * 2 + 1])?;
        }
        Ok(tree)
    }

    /// Returns the number of values represented by the tree.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the tree contains no values.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a reference to the value at `index`, or `None` if out of range.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        (index < self.len).then(|| &self.nodes[self.leaf_count + index])
    }

    /// Replaces the value at `index` and returns the previous value.
    ///
    /// Returns `Ok(None)` when `index` is outside `0..len`. If recomputing an
    /// aggregate fails, returns the monoid error and leaves the tree unchanged.
    pub fn set(&mut self, index: usize, value: T) -> Result<Option<T>, T::Error> {
        if index >= self.len {
            return Ok(None);
        }

        let leaf = self.leaf_count + index;
        let mut node = leaf;
        let mut aggregate = value.clone();
        let mut ancestors = Vec::new();
        while node > 1 {
            let parent = node / 2;
            let sibling = node ^ 1;
            aggregate = if node.is_multiple_of(2) {
                T::try_combine(&aggregate, &self.nodes[sibling])?
            } else {
                T::try_combine(&self.nodes[sibling], &aggregate)?
            };
            ancestors.push((parent, aggregate.clone()));
            node = parent;
        }

        let old = std::mem::replace(&mut self.nodes[leaf], value);
        for (parent, aggregate) in ancestors {
            self.nodes[parent] = aggregate;
        }
        Ok(Some(old))
    }

    /// Queries the monoid aggregate over the half-open range `start..end`.
    ///
    /// Returns `Ok(None)` unless `start <= end <= len`. An empty valid range
    /// returns the monoid identity. Errors from the monoid are propagated.
    pub fn query(&self, start: usize, end: usize) -> Result<Option<T>, T::Error> {
        if start > end || end > self.len {
            return Ok(None);
        }
        let mut left = start + self.leaf_count;
        let mut right = end + self.leaf_count;
        let mut left_result = T::identity();
        let mut right_result = T::identity();
        while left < right {
            if left % 2 == 1 {
                left_result = T::try_combine(&left_result, &self.nodes[left])?;
                left += 1;
            }
            if right % 2 == 1 {
                right -= 1;
                right_result = T::try_combine(&self.nodes[right], &right_result)?;
            }
            left /= 2;
            right /= 2;
        }
        Ok(Some(T::try_combine(&left_result, &right_result)?))
    }
}

impl SegmentTree<i64> {
    /// Returns the sum over the half-open range `start..end`.
    pub fn range_sum(&self, start: usize, end: usize) -> Result<Option<i64>, SumOverflow> {
        self.query(start, end)
    }
}

#[cfg(test)]
mod tests {
    use super::{SegmentTree, SumOverflow};
    use crate::algebra::Monoid;

    #[test]
    fn handles_boundaries_and_point_updates() {
        let mut tree = SegmentTree::from_values(&[2, 4, -1, 6, 3]).unwrap();
        assert_eq!(tree.len(), 5);
        assert_eq!(tree.range_sum(0, 0).unwrap(), Some(0));
        assert_eq!(tree.range_sum(0, 5).unwrap(), Some(14));
        assert_eq!(tree.range_sum(1, 4).unwrap(), Some(9));
        assert_eq!(tree.set(2, 5).unwrap(), Some(-1));
        assert_eq!(tree.get(2), Some(&5));
        assert_eq!(tree.range_sum(1, 4).unwrap(), Some(15));
        assert_eq!(tree.set(5, 9).unwrap(), None);
        assert_eq!(tree.range_sum(3, 2).unwrap(), None);
        assert_eq!(tree.range_sum(0, 6).unwrap(), None);
    }

    #[test]
    fn empty_tree_has_empty_identity_range() {
        let tree = SegmentTree::<i64>::new(0);
        assert!(tree.is_empty());
        assert_eq!(tree.range_sum(0, 0).unwrap(), Some(0));
        assert_eq!(tree.get(0), None);
    }

    #[test]
    fn preserves_order_for_non_commutative_monoid() {
        #[derive(Clone, Debug, Eq, PartialEq)]
        struct Text(String);

        impl Monoid for Text {
            fn identity() -> Self {
                Self(String::new())
            }

            fn combine(left: &Self, right: &Self) -> Self {
                Self(format!("{}{}", left.0, right.0))
            }
        }

        let tree = SegmentTree::from_values(&[
            Text("a".into()),
            Text("b".into()),
            Text("c".into()),
            Text("d".into()),
            Text("e".into()),
        ])
        .unwrap();
        assert_eq!(tree.query(1, 5).unwrap(), Some(Text("bcde".into())));
    }

    #[test]
    fn reports_overflow_and_keeps_failed_update_atomic() {
        assert_eq!(
            SegmentTree::from_values(&[i64::MAX, 1]).unwrap_err(),
            SumOverflow
        );

        let mut tree = SegmentTree::from_values(&[i64::MAX, 0]).unwrap();
        assert_eq!(tree.set(1, 1), Err(SumOverflow));
        assert_eq!(tree.get(1), Some(&0));
        assert_eq!(tree.get(0), Some(&i64::MAX));

        let tree = SegmentTree::from_values(&[i64::MAX, 0, 1, -1]).unwrap();
        assert_eq!(tree.range_sum(0, 3), Err(SumOverflow));
    }
}
