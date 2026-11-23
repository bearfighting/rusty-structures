use super::BTreeMap;
use std::iter::FusedIterator;

/// An ordered set implemented using a B-tree.
#[derive(Clone, Debug, Default)]
pub struct BTreeSet<T> {
    map: BTreeMap<T, ()>,
}

impl<T> BTreeSet<T> {
    /// Creates an empty B-tree set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    /// Returns the number of values in the set.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns whether the set is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Removes all values from the set.
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Returns values in ascending order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &T> + FusedIterator {
        self.map.keys()
    }
}

impl<T: Ord> BTreeSet<T> {
    /// Inserts a value; returns `false` if it was already present.
    pub fn insert(&mut self, value: T) -> bool {
        self.map.insert(value, ()).is_none()
    }

    /// Returns whether an equal value is present.
    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        T: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.map.contains_key(value)
    }

    /// Removes an equal value; returns `true` when the set changed.
    pub fn remove<Q>(&mut self, value: &Q) -> bool
    where
        T: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.map.remove(value).is_some()
    }
}
