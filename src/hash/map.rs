//! A separate-chaining hash map with a replaceable hash builder.
//!
//! # Code map
//!
//! - This file contains construction, lookup, insertion, removal, and resizing.
//! - `map/iter.rs` walks the buckets in arbitrary order.
//! - `map/traits.rs` implements collection and formatting traits.

use crate::Vector;
use std::borrow::Borrow;
use std::hash::{BuildHasher, Hash, Hasher, RandomState};
use std::mem;

mod iter;
mod traits;

pub use iter::{IntoIter, Iter};

const MIN_BUCKETS: usize = 8;

/// A hash map that stores key-value pairs in buckets.
///
/// `S` creates the hashers used to map keys to buckets. It defaults to
/// [`RandomState`], and can be replaced when constructing the map. Collisions
/// are stored in a small vector in the selected bucket. Lookup and insertion
/// are average O(1) while the load factor stays bounded; a bucket with many
/// collisions takes O(n) time to scan.
///
/// # Examples
///
/// ```
/// use data_structures::HashMap;
///
/// let mut scores = HashMap::new();
/// scores.insert("Ada", 10);
/// scores.insert("Linus", 20);
///
/// assert_eq!(scores.get("Ada"), Some(&10));
/// assert_eq!(scores.remove("Linus"), Some(20));
/// ```
pub struct HashMap<K, V, S = RandomState> {
    buckets: Vector<Vector<(K, V)>>,
    len: usize,
    hash_builder: S,
}

impl<K, V> HashMap<K, V, RandomState> {
    /// Creates an empty map using the default randomized hash builder.
    #[must_use]
    pub fn new() -> Self {
        Self::with_hasher(RandomState::new())
    }

    /// Creates an empty map with room for at least `capacity` entries.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, RandomState::new())
    }
}

impl<K, V, S> HashMap<K, V, S>
where
    S: BuildHasher,
{
    /// Creates an empty map using `hash_builder`.
    ///
    /// The builder remains fixed while entries are in the map. Replacing it
    /// later would require moving every entry to its bucket under the new hashes.
    /// A custom builder is useful for specialized workloads, but weak hashers
    /// can make collision attacks easier. The default [`RandomState`] is a
    /// safer choice for maps that process untrusted keys.
    #[must_use]
    pub fn with_hasher(hash_builder: S) -> Self {
        Self {
            buckets: Self::empty_buckets(MIN_BUCKETS),
            len: 0,
            hash_builder,
        }
    }

    /// Creates an empty map with room for at least `capacity` entries and uses
    /// `hash_builder` to hash keys.
    #[must_use]
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        let bucket_count = Self::buckets_for_entries(capacity);
        Self {
            buckets: Self::empty_buckets(bucket_count),
            len: 0,
            hash_builder,
        }
    }

    /// Returns the number of entries in the map.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the map contains no entries.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of entries the map can hold before its next growth.
    #[must_use]
    pub fn capacity(&self) -> usize {
        Self::max_entries_for_buckets(self.buckets.len())
    }

    /// Returns the number of buckets currently used by the map.
    #[must_use]
    pub fn bucket_count(&self) -> usize {
        self.buckets.len()
    }

    /// Returns the map's hash builder.
    #[must_use]
    pub const fn hasher(&self) -> &S {
        &self.hash_builder
    }

    /// Returns an iterator over key-value pairs in arbitrary order.
    pub fn iter(&self) -> Iter<'_, K, V, S> {
        Iter::new(self)
    }

    /// Removes all entries while retaining the bucket allocation.
    pub fn clear(&mut self) {
        for bucket in self.buckets.iter_mut() {
            bucket.clear();
        }
        self.len = 0;
    }

    fn empty_buckets(count: usize) -> Vector<Vector<(K, V)>> {
        let mut buckets = Vector::with_capacity(count);
        for _ in 0..count {
            buckets.push(Vector::new());
        }
        buckets
    }

    fn buckets_for_entries(entries: usize) -> usize {
        let needed = entries
            .checked_mul(4)
            .and_then(|value| value.checked_add(2))
            .map(|value| value / 3)
            .expect("requested hash map capacity is too large");
        needed.max(MIN_BUCKETS)
    }

    fn max_entries_for_buckets(bucket_count: usize) -> usize {
        let remainder = usize::from(bucket_count % 4 != 0);
        bucket_count - bucket_count / 4 - remainder
    }

    fn hash<Q: Hash + ?Sized>(&self, key: &Q) -> u64 {
        let mut hasher = self.hash_builder.build_hasher();
        key.hash(&mut hasher);
        hasher.finish()
    }

    fn bucket_index<Q: Hash + ?Sized>(&self, key: &Q) -> usize {
        self.hash(key) as usize % self.buckets.len()
    }

    fn grow(&mut self)
    where
        K: Hash,
    {
        let new_count = self
            .buckets
            .len()
            .checked_mul(2)
            .expect("hash map capacity overflow");

        // Count each destination bucket first so the new bucket vectors can
        // reserve space before any entries are moved out of the old table.
        let mut entry_counts = Vector::with_capacity(new_count);
        let mut destination_buckets = Vector::with_capacity(self.len);
        for _ in 0..new_count {
            entry_counts.push(0_usize);
        }
        for (key, _) in self.iter() {
            let index = self.hash(key) as usize % new_count;
            entry_counts[index] += 1;
            destination_buckets.push(index);
        }

        let mut new_buckets = Vector::with_capacity(new_count);
        for count in entry_counts {
            new_buckets.push(Vector::with_capacity(count));
        }

        // Once storage is ready, move entries into their new buckets.
        let old_buckets = mem::replace(&mut self.buckets, new_buckets);
        let mut destination_buckets = destination_buckets.into_iter();
        for bucket in old_buckets {
            for (key, value) in bucket {
                let index = destination_buckets
                    .next()
                    .expect("each entry has a destination bucket");
                self.buckets[index].push((key, value));
            }
        }
    }
}

impl<K, V, S> HashMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    /// Inserts a key-value pair.
    ///
    /// If the key already exists, its value is replaced and the old value is
    /// returned. The key already in the map is retained.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let index = self.bucket_index(&key);
        if let Some((_, existing_value)) = self.buckets[index]
            .iter_mut()
            .find(|(existing_key, _)| existing_key == &key)
        {
            return Some(mem::replace(existing_value, value));
        }

        let next_len = self.len.checked_add(1).expect("hash map length overflow");
        if next_len > self.capacity() {
            self.grow();
        }

        let index = self.bucket_index(&key);
        self.buckets[index].push((key, value));
        self.len += 1;
        None
    }

    /// Returns a shared reference to the value for `key`.
    #[must_use]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.get_key_value(key).map(|(_, value)| value)
    }

    /// Returns the stored key and value matching `key`.
    #[must_use]
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let index = self.bucket_index(key);
        self.buckets[index]
            .iter()
            .find(|(existing_key, _)| existing_key.borrow() == key)
            .map(|(stored_key, value)| (stored_key, value))
    }

    /// Returns a mutable reference to the value for `key`.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let index = self.bucket_index(key);
        self.buckets[index]
            .iter_mut()
            .find(|(existing_key, _)| existing_key.borrow() == key)
            .map(|(_, value)| value)
    }

    /// Returns whether the map contains `key`.
    #[must_use]
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.get(key).is_some()
    }

    /// Removes `key` and returns its value, if present.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.remove_entry(key).map(|(_, value)| value)
    }

    /// Removes `key` and returns the stored key together with its value.
    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let index = self.bucket_index(key);
        let entry_index = self.buckets[index]
            .iter()
            .position(|(existing_key, _)| existing_key.borrow() == key)?;
        let entry = self.buckets[index].remove(entry_index)?;
        self.len -= 1;
        Some(entry)
    }
}
