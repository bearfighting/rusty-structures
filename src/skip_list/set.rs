use super::SkipListMap;
use std::borrow::Borrow;

/// An ordered set backed by a skip list.
///
/// Operations have expected O(log n) and worst-case O(n) time. Iteration visits
/// values in ascending order.
#[derive(Clone, Debug)]
pub struct SkipListSet<T> {
    entries: SkipListMap<T, ()>,
}

impl<T> SkipListSet<T> {
    /// Creates an empty set with a clock-derived pseudorandom seed.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: SkipListMap::new(),
        }
    }

    /// Creates an empty set with a deterministic level-generator seed.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        Self {
            entries: SkipListMap::with_seed(seed),
        }
    }

    /// Returns the number of values in the set.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the set contains no values.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Removes all values.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl<T: Ord> SkipListSet<T> {
    /// Adds `value`, returning `true` if it was not already present.
    pub fn insert(&mut self, value: T) -> bool {
        self.entries.insert(value, ()).is_none()
    }

    /// Removes `value`, returning whether it was present.
    pub fn remove<Q>(&mut self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.entries.remove(value).is_some()
    }

    /// Returns whether `value` is present.
    #[must_use]
    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.entries.contains_key(value)
    }

    /// Returns values in ascending order.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.entries.iter().map(|(key, _)| key)
    }
}

impl<T> Default for SkipListSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::SkipListSet;

    #[test]
    fn keeps_unique_values_in_order() {
        let mut set = SkipListSet::with_seed(9);
        assert!(set.insert(4));
        assert!(set.insert(1));
        assert!(set.insert(3));
        assert!(!set.insert(3));
        assert!(set.contains(&1));
        assert!(set.remove(&3));
        assert!(!set.remove(&3));
        assert_eq!(set.iter().copied().collect::<Vec<_>>(), vec![1, 4]);
    }
}
