//! Ordered map implemented as a probabilistic skip list.

use super::Iter;
use std::borrow::Borrow;

const MAX_LEVEL: usize = 32;
const DEFAULT_SEED: u64 = 0x9e37_79b9_7f4a_7c15;

#[derive(Clone, Debug)]
pub(super) struct Node<K, V> {
    pub(super) key: K,
    pub(super) value: V,
    pub(super) forward: Vec<Option<usize>>,
}

/// An ordered map backed by a skip list.
///
/// Each node receives a pseudorandom height. Lookup, insertion, and removal
/// take expected O(log n) time and worst-case O(n) time. Iteration visits keys
/// in ascending order. The index-based node arena is safe Rust; removed slots
/// are reused. Expected space is O(n), with a worst case of O(n log n).
///
/// `new` seeds the level generator from the current clock. Use [`with_seed`](Self::with_seed)
/// to reproduce a particular skip-list shape.
///
/// # Examples
///
/// ```
/// use data_structures::SkipListMap;
///
/// let mut scores = SkipListMap::with_seed(7);
/// scores.insert("Ada", 10);
/// scores.insert("Grace", 12);
/// assert_eq!(scores.get("Ada"), Some(&10));
/// assert_eq!(scores.iter().next(), Some((&"Ada", &10)));
/// ```
#[derive(Clone, Debug)]
pub struct SkipListMap<K, V> {
    head: Vec<Option<usize>>,
    pub(super) nodes: Vec<Option<Node<K, V>>>,
    free_slots: Vec<usize>,
    len: usize,
    random_state: u64,
}

impl<K, V> SkipListMap<K, V> {
    /// Creates an empty map with a clock-derived pseudorandom seed.
    #[must_use]
    pub fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(DEFAULT_SEED, |duration| duration.as_nanos() as u64);
        Self::with_seed(seed)
    }

    /// Creates an empty map with a deterministic level-generator seed.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        Self {
            head: vec![None; MAX_LEVEL],
            nodes: Vec::new(),
            free_slots: Vec::new(),
            len: 0,
            random_state: if seed == 0 { DEFAULT_SEED } else { seed },
        }
    }

    /// Returns the number of key-value pairs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the map contains no pairs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns an iterator over key-value pairs in ascending key order.
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            map: self,
            next: self.head[0],
            remaining: self.len,
        }
    }

    /// Removes all pairs while retaining the level-generator state.
    pub fn clear(&mut self) {
        self.head.fill(None);
        self.nodes.clear();
        self.free_slots.clear();
        self.len = 0;
    }

    fn next_at(&self, from: Option<usize>, level: usize) -> Option<usize> {
        match from {
            Some(index) => self.nodes[index]
                .as_ref()
                .and_then(|node| node.forward.get(level).copied().flatten()),
            None => self.head[level],
        }
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

    fn next_random(&mut self) -> u64 {
        let mut value = self.random_state;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.random_state = value;
        value
    }

    fn random_level(&mut self) -> usize {
        let mut level = 1;
        while level < MAX_LEVEL && self.next_random() & 1 == 1 {
            level += 1;
        }
        level
    }
}

impl<K: Ord, V> SkipListMap<K, V> {
    /// Returns the value associated with `key`.
    #[must_use]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let index = self.find(key)?;
        Some(&self.nodes[index].as_ref()?.value)
    }

    /// Returns a mutable reference to the value associated with `key`.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let index = self.find(key)?;
        Some(&mut self.nodes[index].as_mut()?.value)
    }

    /// Returns whether `key` is present.
    #[must_use]
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.find(key).is_some()
    }

    /// Inserts a key-value pair, returning the previous value for that key.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let (predecessors, candidate) = self.predecessors(&key);
        if let Some(index) = candidate
            && self.nodes[index]
                .as_ref()
                .is_some_and(|node| node.key.borrow().cmp(&key) == std::cmp::Ordering::Equal)
        {
            return Some(std::mem::replace(
                &mut self.nodes[index]
                    .as_mut()
                    .expect("candidate node exists")
                    .value,
                value,
            ));
        }

        let level = self.random_level();
        let mut node = Node {
            key,
            value,
            forward: Vec::with_capacity(level),
        };
        for (index, predecessor) in predecessors.iter().copied().enumerate().take(level) {
            node.forward.push(self.next_at(predecessor, index));
        }
        let node_index = self.allocate(node);
        for (level, predecessor) in predecessors.iter().copied().enumerate().take(level) {
            match predecessor {
                Some(previous) => {
                    self.nodes[previous]
                        .as_mut()
                        .expect("predecessor node exists")
                        .forward[level] = Some(node_index);
                }
                None => self.head[level] = Some(node_index),
            }
        }
        self.len += 1;
        None
    }

    /// Removes `key` and returns its value, if present.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let (predecessors, candidate) = self.predecessors(key);
        let index = candidate?;
        if self.nodes[index].as_ref()?.key.borrow().cmp(key) != std::cmp::Ordering::Equal {
            return None;
        }
        let forward = self.nodes[index].as_ref()?.forward.clone();
        for (level, successor) in forward.into_iter().enumerate() {
            match predecessors[level] {
                Some(previous) => {
                    self.nodes[previous]
                        .as_mut()
                        .expect("predecessor node exists")
                        .forward[level] = successor;
                }
                None => self.head[level] = successor,
            }
        }
        let node = self.nodes[index].take().expect("candidate node exists");
        self.free_slots.push(index);
        self.len -= 1;
        Some(node.value)
    }

    fn find<Q>(&self, key: &Q) -> Option<usize>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let (_, candidate) = self.predecessors(key);
        candidate.filter(|&index| {
            self.nodes[index]
                .as_ref()
                .is_some_and(|node| node.key.borrow().cmp(key) == std::cmp::Ordering::Equal)
        })
    }

    fn predecessors<Q>(&self, key: &Q) -> ([Option<usize>; MAX_LEVEL], Option<usize>)
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let mut predecessors = [None; MAX_LEVEL];
        let mut current = None;
        for level in (0..MAX_LEVEL).rev() {
            while let Some(next) = self.next_at(current, level) {
                let node = self.nodes[next].as_ref().expect("linked node exists");
                if node.key.borrow().cmp(key) == std::cmp::Ordering::Less {
                    current = Some(next);
                } else {
                    break;
                }
            }
            predecessors[level] = current;
        }
        (predecessors, self.next_at(current, 0))
    }
}

impl<K, V> Default for SkipListMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, K, V> IntoIterator for &'a SkipListMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::SkipListMap;
    use std::collections::BTreeMap;

    #[test]
    fn keeps_keys_ordered_and_reuses_removed_slots() {
        let mut map = SkipListMap::with_seed(42);
        for key in [7, 1, 5, 2, 9, 3] {
            assert_eq!(map.insert(key, key * 10), None);
        }
        assert_eq!(map.insert(5, 55), Some(50));
        assert_eq!(map.get(&5), Some(&55));
        *map.get_mut(&5).unwrap() += 1;
        assert_eq!(map.get(&5), Some(&56));
        assert_eq!(map.remove(&7), Some(70));
        assert_eq!(map.remove(&100), None);
        assert_eq!(map.insert(8, 80), None);
        assert_eq!(
            map.iter().map(|(key, _)| *key).collect::<Vec<_>>(),
            vec![1, 2, 3, 5, 8, 9]
        );
        assert_eq!(map.len(), 6);
    }

    #[test]
    fn agrees_with_btree_map_for_seeded_operations() {
        let mut actual = SkipListMap::with_seed(1234);
        let mut expected = BTreeMap::new();
        let mut state = 91u64;
        for _ in 0..500 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let key = (state % 73) as i32;
            if state & 1 == 0 {
                let value = (state >> 16) as i64;
                assert_eq!(actual.insert(key, value), expected.insert(key, value));
            } else {
                assert_eq!(actual.remove(&key), expected.remove(&key));
            }
            assert_eq!(
                actual.iter().collect::<Vec<_>>(),
                expected.iter().collect::<Vec<_>>()
            );
        }
    }
}
