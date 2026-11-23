//! Common ordered-map contract and concrete tree-backed adapters.
//!
//! The public trait lives here. Concrete wrappers and their private backend
//! plumbing live in the nested backend module.

mod backend;

pub use backend::{
    AvlTreeMap, BinarySearchTreeMap, MapIter, RecursiveBinarySearchTreeMap, RedBlackTreeMap,
};
use std::iter::FusedIterator;

/// Common ordered key-value operations implemented by the tree map variants.
pub trait TreeMap<K: Ord, V> {
    /// An in-order iterator over key-value pairs.
    type Iter<'a>: Iterator<Item = (&'a K, &'a V)> + ExactSizeIterator + FusedIterator
    where
        Self: 'a,
        K: 'a,
        V: 'a;

    /// Returns the number of entries.
    fn len(&self) -> usize;
    /// Returns whether the map is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Inserts a pair and returns the previous value for the key, if any.
    fn insert(&mut self, key: K, value: V) -> Option<V>;
    /// Returns the value associated with `key`.
    fn get(&self, key: &K) -> Option<&V>;
    /// Returns mutable access to the value associated with `key`.
    fn get_mut(&mut self, key: &K) -> Option<&mut V>;
    /// Returns whether `key` is present.
    fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }
    /// Removes a pair and returns its value.
    fn remove(&mut self, key: &K) -> Option<V>;
    /// Iterates over pairs in ascending key order.
    fn iter(&self) -> Self::Iter<'_>;
}
