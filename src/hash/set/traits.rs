//! Standard trait implementations for [`HashSet`](super::HashSet).

use super::HashSet;
use std::fmt;
use std::hash::{BuildHasher, Hash};

impl<T, S> Default for HashSet<T, S>
where
    S: BuildHasher + Default,
{
    fn default() -> Self {
        Self::with_hasher(S::default())
    }
}

impl<T, S: BuildHasher> fmt::Debug for HashSet<T, S>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<T, S> Clone for HashSet<T, S>
where
    T: Clone + Hash + Eq,
    S: BuildHasher + Clone,
{
    fn clone(&self) -> Self {
        let mut clone = Self::with_capacity_and_hasher(self.len(), self.hasher().clone());
        clone.extend(self.iter().cloned());
        clone
    }
}

impl<T, S1, S2> PartialEq<HashSet<T, S2>> for HashSet<T, S1>
where
    T: Hash + Eq,
    S1: BuildHasher,
    S2: BuildHasher,
{
    fn eq(&self, other: &HashSet<T, S2>) -> bool {
        self.len() == other.len() && self.is_subset(other)
    }
}

impl<T, S> Eq for HashSet<T, S>
where
    T: Hash + Eq,
    S: BuildHasher,
{
}

impl<T, S> Extend<T> for HashSet<T, S>
where
    T: Hash + Eq,
    S: BuildHasher,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.insert(value);
        }
    }
}

impl<T, S> FromIterator<T> for HashSet<T, S>
where
    T: Hash + Eq,
    S: BuildHasher + Default,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut set = Self::default();
        set.extend(iter);
        set
    }
}
