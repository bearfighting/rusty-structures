//! Lazy segment tree for range updates and range queries.

use std::marker::PhantomData;

/// Defines the aggregate and update rules for a [`LazySegmentTree`].
///
/// `combine` must be associative and `identity` must be its two-sided identity.
/// `apply` updates an aggregate covering `segment_len` values. `compose(old,
/// new)` combines updates in chronological order: apply `old` first, then
/// `new`. Implementations must ensure these operations are consistent and do
/// not panic for valid inputs. In particular, numeric implementations must
/// choose and document an overflow policy.
pub trait LazySegmentTreeSpec {
    /// The value stored at each leaf and returned by queries.
    type Value: Clone;
    /// An update applied to a range.
    type Update: Clone;

    /// Returns the aggregate identity.
    fn identity() -> Self::Value;
    /// Combines adjacent aggregates, preserving left-to-right order.
    fn combine(left: &Self::Value, right: &Self::Value) -> Self::Value;
    /// Applies an update to an aggregate spanning `segment_len` leaves.
    fn apply(update: &Self::Update, aggregate: &Self::Value, segment_len: usize) -> Self::Value;
    /// Composes two updates, applying `old` first and `new` second.
    fn compose(old: &Self::Update, new: &Self::Update) -> Self::Update;
}

/// A segment tree supporting range updates and range queries.
///
/// A specification defines the aggregate and update action. Updates and
/// queries take O(log n) time for the usual range decomposition; construction
/// takes O(n) time and storage is O(n). Queries take `&mut self` because they
/// may push pending updates down the tree while preserving its represented
/// values.
///
/// # Examples
///
/// ```
/// use data_structures::{LazySegmentTree, LazySegmentTreeSpec};
///
/// struct AddSum;
/// impl LazySegmentTreeSpec for AddSum {
///     type Value = i64;
///     type Update = i64;
///     fn identity() -> i64 { 0 }
///     fn combine(a: &i64, b: &i64) -> i64 { a + b }
///     fn apply(delta: &i64, sum: &i64, len: usize) -> i64 {
///         sum + delta * len as i64
///     }
///     fn compose(old: &i64, new: &i64) -> i64 { old + new }
/// }
///
/// let mut tree = LazySegmentTree::<AddSum>::from_values(&[1, 2, 3, 4]);
/// tree.update_range(1, 4, 5).unwrap();
/// assert_eq!(tree.query(0, 4), Some(25));
/// ```
#[derive(Clone, Debug)]
pub struct LazySegmentTree<S: LazySegmentTreeSpec> {
    len: usize,
    aggregates: Vec<S::Value>,
    pending: Vec<Option<S::Update>>,
    spec: PhantomData<S>,
}

impl<S: LazySegmentTreeSpec> LazySegmentTree<S> {
    /// Creates a tree containing `len` identity-valued elements.
    #[must_use]
    pub fn new(len: usize) -> Self {
        let capacity = len.saturating_mul(4).max(1);
        Self {
            len,
            aggregates: vec![S::identity(); capacity],
            pending: vec![None; capacity],
            spec: PhantomData,
        }
    }

    /// Builds a tree from values in O(n) time.
    #[must_use]
    pub fn from_values(values: &[S::Value]) -> Self {
        let mut tree = Self::new(values.len());
        if !values.is_empty() {
            tree.build(1, 0, values.len(), values);
        }
        tree
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

    /// Applies `update` to every value in the half-open range `start..end`.
    ///
    /// Returns `None` unless `start <= end <= len`. A valid empty range is a
    /// no-op and returns `Some(())`.
    pub fn update_range(&mut self, start: usize, end: usize, update: S::Update) -> Option<()> {
        if start > end || end > self.len {
            return None;
        }
        if start < end {
            self.update(1, 0, self.len, start, end, &update);
        }
        Some(())
    }

    /// Queries the aggregate over the half-open range `start..end`.
    ///
    /// Returns `None` unless `start <= end <= len`. A valid empty range returns
    /// the specification's identity value.
    pub fn query(&mut self, start: usize, end: usize) -> Option<S::Value> {
        if start > end || end > self.len {
            return None;
        }
        if start == end {
            return Some(S::identity());
        }
        Some(self.query_inner(1, 0, self.len, start, end))
    }

    fn build(&mut self, node: usize, left: usize, right: usize, values: &[S::Value]) {
        if right - left == 1 {
            self.aggregates[node] = values[left].clone();
            return;
        }
        let middle = left + (right - left) / 2;
        self.build(node * 2, left, middle, values);
        self.build(node * 2 + 1, middle, right, values);
        self.pull(node);
    }

    fn update(
        &mut self,
        node: usize,
        left: usize,
        right: usize,
        start: usize,
        end: usize,
        update: &S::Update,
    ) {
        if start <= left && right <= end {
            self.apply_to_node(node, right - left, update);
            return;
        }
        self.push(node, left, right);
        let middle = left + (right - left) / 2;
        if start < middle {
            self.update(node * 2, left, middle, start, end, update);
        }
        if end > middle {
            self.update(node * 2 + 1, middle, right, start, end, update);
        }
        self.pull(node);
    }

    fn query_inner(
        &mut self,
        node: usize,
        left: usize,
        right: usize,
        start: usize,
        end: usize,
    ) -> S::Value {
        if start <= left && right <= end {
            return self.aggregates[node].clone();
        }
        self.push(node, left, right);
        let middle = left + (right - left) / 2;
        match (start < middle, end > middle) {
            (true, true) => {
                let left_value = self.query_inner(node * 2, left, middle, start, end);
                let right_value = self.query_inner(node * 2 + 1, middle, right, start, end);
                S::combine(&left_value, &right_value)
            }
            (true, false) => self.query_inner(node * 2, left, middle, start, end),
            (false, true) => self.query_inner(node * 2 + 1, middle, right, start, end),
            (false, false) => S::identity(),
        }
    }

    fn apply_to_node(&mut self, node: usize, segment_len: usize, update: &S::Update) {
        self.aggregates[node] = S::apply(update, &self.aggregates[node], segment_len);
        self.pending[node] = Some(match self.pending[node].as_ref() {
            Some(old) => S::compose(old, update),
            None => update.clone(),
        });
    }

    fn push(&mut self, node: usize, left: usize, right: usize) {
        let Some(update) = self.pending[node].take() else {
            return;
        };
        if right - left > 1 {
            let middle = left + (right - left) / 2;
            self.apply_to_node(node * 2, middle - left, &update);
            self.apply_to_node(node * 2 + 1, right - middle, &update);
        }
    }

    fn pull(&mut self, node: usize) {
        self.aggregates[node] =
            S::combine(&self.aggregates[node * 2], &self.aggregates[node * 2 + 1]);
    }
}

#[cfg(test)]
mod tests {
    use super::{LazySegmentTree, LazySegmentTreeSpec};

    struct AddSum;

    impl LazySegmentTreeSpec for AddSum {
        type Value = i64;
        type Update = i64;

        fn identity() -> Self::Value {
            0
        }

        fn combine(left: &i64, right: &i64) -> i64 {
            left + right
        }

        fn apply(delta: &i64, sum: &i64, len: usize) -> i64 {
            sum + delta * len as i64
        }

        fn compose(old: &i64, new: &i64) -> i64 {
            old + new
        }
    }

    #[test]
    fn applies_overlapping_range_updates_and_queries() {
        let mut tree = LazySegmentTree::<AddSum>::from_values(&[1, 2, 3, 4, 5]);
        tree.update_range(1, 5, 3).unwrap();
        tree.update_range(2, 4, -2).unwrap();
        assert_eq!(tree.query(0, 5), Some(23));
        assert_eq!(tree.query(1, 4), Some(14));
        assert_eq!(tree.query(2, 2), Some(0));
        assert_eq!(tree.query(0, 6), None);
        assert_eq!(tree.update_range(4, 3, 1), None);
        assert_eq!(tree.update_range(5, 5, 9), Some(()));
    }

    #[test]
    fn supports_empty_tree_and_single_element() {
        let mut empty = LazySegmentTree::<AddSum>::new(0);
        assert_eq!(empty.query(0, 0), Some(0));
        assert_eq!(empty.update_range(0, 0, 7), Some(()));

        let mut single = LazySegmentTree::<AddSum>::from_values(&[4]);
        single.update_range(0, 1, 6).unwrap();
        assert_eq!(single.query(0, 1), Some(10));
    }
}
