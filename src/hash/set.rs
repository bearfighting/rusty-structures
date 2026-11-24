//! A hash set implemented by storing values as keys in [`HashMap`].

mod iter;
mod traits;

pub use iter::{IntoIter, Iter};

use super::map::HashMap;
use std::borrow::Borrow;
use std::hash::{BuildHasher, Hash, RandomState};

/// A set of unique values.
///
/// `S` creates the hashers used for values. It defaults to [`RandomState`] and
/// can be replaced when constructing the set. The set reuses [`HashMap`]'s
/// hashing, collision handling, and growth logic.
///
/// # Examples
///
/// ```
/// use data_structures::HashSet;
///
/// let mut names = HashSet::new();
/// assert!(names.insert("Ada"));
/// assert!(!names.insert("Ada"));
/// assert!(names.contains("Ada"));
/// assert!(names.remove("Ada"));
/// ```
pub struct HashSet<T, S = RandomState> {
    map: HashMap<T, (), S>,
}

impl<T> HashSet<T, RandomState> {
    /// Creates an empty set using the default randomized hash builder.
    #[must_use]
    pub fn new() -> Self {
        Self::with_hasher(RandomState::new())
    }

    /// Creates an empty set with room for at least `capacity` values.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, RandomState::new())
    }
}

impl<T, S> HashSet<T, S>
where
    S: BuildHasher,
{
    /// Creates an empty set using `hash_builder`.
    #[must_use]
    pub fn with_hasher(hash_builder: S) -> Self {
        Self {
            map: HashMap::with_hasher(hash_builder),
        }
    }

    /// Creates an empty set with room for at least `capacity` values and uses
    /// `hash_builder` to hash them.
    #[must_use]
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        Self {
            map: HashMap::with_capacity_and_hasher(capacity, hash_builder),
        }
    }

    /// Returns the number of values in the set.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns whether the set contains no values.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns the number of values the set can hold before its next growth.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }

    /// Returns the number of buckets currently used by the backing map.
    #[must_use]
    pub fn bucket_count(&self) -> usize {
        self.map.bucket_count()
    }

    /// Returns the set's hash builder.
    #[must_use]
    pub const fn hasher(&self) -> &S {
        self.map.hasher()
    }

    /// Removes all values while retaining the allocated buckets.
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Returns an iterator over values in arbitrary order.
    pub fn iter(&self) -> Iter<'_, T, S> {
        Iter::new(self.map.iter())
    }
}

impl<T, S> HashSet<T, S>
where
    T: Hash + Eq,
    S: BuildHasher,
{
    /// Adds `value` if an equal value is not already present.
    ///
    /// Returns `true` when the set changed.
    pub fn insert(&mut self, value: T) -> bool {
        self.map.insert(value, ()).is_none()
    }

    /// Returns a reference to the stored value equal to `value`.
    #[must_use]
    pub fn get<Q>(&self, value: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.get_key_value(value).map(|(stored, ())| stored)
    }

    /// Returns whether the set contains a value equal to `value`.
    #[must_use]
    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.contains_key(value)
    }

    /// Removes a value equal to `value` and reports whether one was present.
    pub fn remove<Q>(&mut self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.remove(value).is_some()
    }

    /// Removes and returns the stored value equal to `value`.
    pub fn take<Q>(&mut self, value: &Q) -> Option<T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.remove_entry(value).map(|(stored, ())| stored)
    }

    /// Returns whether this set and `other` have no values in common.
    #[must_use]
    pub fn is_disjoint<S2>(&self, other: &HashSet<T, S2>) -> bool
    where
        S2: BuildHasher,
    {
        self.iter().all(|value| !other.contains(value))
    }

    /// Returns whether every value in this set is also in `other`.
    #[must_use]
    pub fn is_subset<S2>(&self, other: &HashSet<T, S2>) -> bool
    where
        S2: BuildHasher,
    {
        self.iter().all(|value| other.contains(value))
    }
}
