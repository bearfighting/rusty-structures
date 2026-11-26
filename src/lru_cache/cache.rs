//! LRU cache implementation.

use super::Iter;
use crate::HashMap;
use std::borrow::Borrow;
use std::hash::{BuildHasher, Hash, RandomState};

#[derive(Clone, Debug)]
pub(super) struct Node<K, V> {
    pub(super) key: K,
    pub(super) value: V,
    pub(super) previous: Option<usize>,
    pub(super) next: Option<usize>,
}

/// A fixed-capacity cache that evicts the least recently used entry.
///
/// Successful `get` and `get_mut` calls make the entry most recently used.
/// Iteration visits entries from most recently used to least recently used.
/// Lookup and update are average O(1), with O(1) recency changes and eviction.
/// A zero-capacity cache rejects insertions and returns the pair to the caller.
///
/// # Examples
///
/// ```
/// use data_structures::LruCache;
///
/// let mut cache = LruCache::new(2);
/// cache.put("a", 1).unwrap();
/// cache.put("b", 2).unwrap();
/// cache.get(&"a");
/// assert_eq!(cache.put("c", 3).unwrap(), Some(("b", 2)));
/// assert_eq!(cache.get(&"a"), Some(&1));
/// assert_eq!(cache.get(&"b"), None);
/// ```
pub struct LruCache<K, V, S = RandomState> {
    index: HashMap<K, usize, S>,
    pub(super) nodes: Vec<Option<Node<K, V>>>,
    free_slots: Vec<usize>,
    most_recent: Option<usize>,
    least_recent: Option<usize>,
    capacity: usize,
}

impl<K, V> LruCache<K, V, RandomState> {
    /// Creates an empty cache with the given capacity and randomized hashing.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self::with_hasher(capacity, RandomState::new())
    }
}

impl<K, V, S: BuildHasher> LruCache<K, V, S> {
    /// Creates an empty cache with a custom hash builder.
    #[must_use]
    pub fn with_hasher(capacity: usize, hash_builder: S) -> Self {
        Self {
            index: HashMap::with_capacity_and_hasher(capacity, hash_builder),
            nodes: Vec::with_capacity(capacity),
            free_slots: Vec::new(),
            most_recent: None,
            least_recent: None,
            capacity,
        }
    }

    /// Returns the number of entries currently stored.
    #[must_use]
    pub fn len(&self) -> usize {
        self.index.len()
    }

    /// Returns whether the cache is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// Returns the fixed entry capacity.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns entries from most recently used to least recently used.
    pub fn iter(&self) -> Iter<'_, K, V, S> {
        Iter {
            cache: self,
            next: self.most_recent,
            remaining: self.len(),
        }
    }

    /// Removes all entries while retaining allocated storage.
    pub fn clear(&mut self) {
        self.index.clear();
        self.nodes.clear();
        self.free_slots.clear();
        self.most_recent = None;
        self.least_recent = None;
    }

    fn allocate(&mut self, node: Node<K, V>) -> usize {
        if let Some(index) = self.free_slots.pop() {
            self.nodes[index] = Some(node);
            index
        } else {
            let index = self.nodes.len();
            self.nodes.push(Some(node));
            index
        }
    }

    fn detach(&mut self, index: usize) {
        let (previous, next) = {
            let node = self.nodes[index].as_ref().expect("linked node exists");
            (node.previous, node.next)
        };
        if let Some(previous) = previous {
            self.nodes[previous]
                .as_mut()
                .expect("previous node exists")
                .next = next;
        } else {
            self.most_recent = next;
        }
        if let Some(next) = next {
            self.nodes[next]
                .as_mut()
                .expect("next node exists")
                .previous = previous;
        } else {
            self.least_recent = previous;
        }
        let node = self.nodes[index].as_mut().expect("linked node exists");
        node.previous = None;
        node.next = None;
    }

    fn attach_most_recent(&mut self, index: usize) {
        let old_head = self.most_recent;
        {
            let node = self.nodes[index].as_mut().expect("node exists");
            node.previous = None;
            node.next = old_head;
        }
        if let Some(old_head) = old_head {
            self.nodes[old_head]
                .as_mut()
                .expect("head node exists")
                .previous = Some(index);
        } else {
            self.least_recent = Some(index);
        }
        self.most_recent = Some(index);
    }

    fn touch(&mut self, index: usize) {
        if self.most_recent != Some(index) {
            self.detach(index);
            self.attach_most_recent(index);
        }
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> LruCache<K, V, S> {
    fn evict_least_recent(&mut self) -> Option<(K, V)> {
        let index = self.least_recent?;
        self.detach(index);
        let node = self.nodes[index].take().expect("tail node exists");
        self.index.remove::<K>(&node.key);
        self.free_slots.push(index);
        Some((node.key, node.value))
    }

    /// Returns a value and marks its entry most recently used.
    pub fn get<Q>(&mut self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let index = *self.index.get(key)?;
        self.touch(index);
        Some(
            &self.nodes[index]
                .as_ref()
                .expect("indexed node exists")
                .value,
        )
    }

    /// Returns a mutable value and marks its entry most recently used.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let index = *self.index.get(key)?;
        self.touch(index);
        Some(
            &mut self.nodes[index]
                .as_mut()
                .expect("indexed node exists")
                .value,
        )
    }

    /// Returns a value without changing its recency.
    pub fn peek<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let index = *self.index.get(key)?;
        Some(
            &self.nodes[index]
                .as_ref()
                .expect("indexed node exists")
                .value,
        )
    }

    /// Returns whether `key` is present without changing recency.
    #[must_use]
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.index.contains_key(key)
    }

    /// Inserts or updates an entry.
    ///
    /// On a full cache, returns the least recently used entry that was evicted.
    /// Updating an existing key replaces its value and marks it most recent;
    /// the previous value is dropped. If capacity is zero, returns the rejected
    /// `(key, value)` pair in `Err`.
    pub fn put(&mut self, key: K, value: V) -> Result<Option<(K, V)>, (K, V)>
    where
        K: Clone,
    {
        if self.capacity == 0 {
            return Err((key, value));
        }

        if let Some(&index) = self.index.get(&key) {
            let node = self.nodes[index].as_mut().expect("indexed node exists");
            node.value = value;
            self.touch(index);
            return Ok(None);
        }

        let evicted = if self.len() == self.capacity {
            self.evict_least_recent()
        } else {
            None
        };
        let index = self.allocate(Node {
            key,
            value,
            previous: None,
            next: None,
        });
        let node = self.nodes[index].as_ref().expect("allocated node exists");
        self.index.insert(node.key.clone(), index);
        self.attach_most_recent(index);
        Ok(evicted)
    }

    /// Removes an entry and returns its value without treating it as recently used.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let index = *self.index.get(key)?;
        self.detach(index);
        let node = self.nodes[index].take().expect("indexed node exists");
        self.index.remove::<K>(&node.key);
        self.free_slots.push(index);
        Some(node.value)
    }
}

impl<'a, K, V, S: BuildHasher> IntoIterator for &'a LruCache<K, V, S> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V, S>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::LruCache;

    #[test]
    fn evicts_least_recent_and_updates_recency_on_get() {
        let mut cache = LruCache::new(2);
        assert_eq!(cache.put("a", 1), Ok(None));
        assert_eq!(cache.put("b", 2), Ok(None));
        assert_eq!(cache.get(&"a"), Some(&1));
        assert_eq!(cache.put("c", 3), Ok(Some(("b", 2))));
        assert_eq!(cache.peek(&"b"), None);
        assert_eq!(
            cache.iter().map(|(key, _)| *key).collect::<Vec<_>>(),
            ["c", "a"]
        );
    }

    #[test]
    fn updates_and_borrowed_lookup_work() {
        let mut cache = LruCache::new(2);
        cache.put(String::from("cedar"), 4).unwrap();
        assert_eq!(cache.get("cedar"), Some(&4));
        assert_eq!(cache.put(String::from("cedar"), 8), Ok(None));
        *cache.get_mut("cedar").unwrap() = 9;
        assert_eq!(cache.get("cedar"), Some(&9));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn zero_capacity_returns_unstored_entry() {
        let mut cache = LruCache::new(0);
        assert_eq!(cache.put("key", 9), Err(("key", 9)));
        assert!(cache.is_empty());
    }
}
