//! Immutable AVL map with structural sharing.

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::sync::Arc;

mod iter;

pub use iter::Iter;

type Link<K, V> = Option<Arc<Node<K, V>>>;

#[derive(Clone, Debug)]
pub(super) struct Node<K, V> {
    pub(super) key: K,
    pub(super) value: V,
    pub(super) height: usize,
    pub(super) left: Link<K, V>,
    pub(super) right: Link<K, V>,
}

/// An immutable AVL map whose versions share unchanged subtrees.
///
/// Insertion and removal return a new map version and the displaced value, if
/// any. Nodes on the search path are copied and rebalanced; untouched
/// subtrees are shared with `Arc`. Search and iteration borrow values without
/// cloning. Mutation requires `K: Clone` and `V: Clone` because copied path
/// nodes retain their key-value pairs.
///
/// # Examples
///
/// ```
/// use data_structures::PersistentAvlTreeMap;
///
/// let empty = PersistentAvlTreeMap::new();
/// let (first, _) = empty.insert(3, "three");
/// let (second, _) = first.insert(1, "one");
/// assert_eq!(first.get(&1), None);
/// assert_eq!(second.get(&1), Some(&"one"));
/// ```
#[derive(Clone, Debug)]
pub struct PersistentAvlTreeMap<K, V> {
    root: Link<K, V>,
    len: usize,
}

impl<K, V> PersistentAvlTreeMap<K, V> {
    /// Creates an empty persistent AVL map.
    #[must_use]
    pub const fn new() -> Self {
        Self { root: None, len: 0 }
    }

    /// Returns the number of key-value pairs in this version.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns whether this version contains no pairs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the height of this version, with an empty map having height zero.
    #[must_use]
    pub fn height(&self) -> usize {
        height(&self.root)
    }

    /// Iterates over key-value pairs in ascending key order.
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter::new(self.root.as_deref(), self.len)
    }
}

impl<K: Ord, V> PersistentAvlTreeMap<K, V> {
    /// Returns the value associated with `key`.
    #[must_use]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let mut node = self.root.as_deref();
        while let Some(current) = node {
            match current.key.borrow().cmp(key) {
                Ordering::Less => node = current.right.as_deref(),
                Ordering::Greater => node = current.left.as_deref(),
                Ordering::Equal => return Some(&current.value),
            }
        }
        None
    }

    /// Returns whether `key` is present.
    #[must_use]
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get(key).is_some()
    }

    /// Inserts a pair into a new version and returns its previous value, if any.
    pub fn insert(&self, key: K, value: V) -> (Self, Option<V>)
    where
        K: Clone,
        V: Clone,
    {
        let (root, previous) = insert_at(self.root.as_ref(), key, value);
        let len = self.len + usize::from(previous.is_none());
        (
            Self {
                root: Some(root),
                len,
            },
            previous,
        )
    }

    /// Removes `key` from a new version and returns its value, if present.
    pub fn remove<Q>(&self, key: &Q) -> (Self, Option<V>)
    where
        K: Borrow<Q> + Clone,
        V: Clone,
        Q: Ord + ?Sized,
    {
        let Some(root) = self.root.as_ref() else {
            return (self.clone_version(), None);
        };
        let (root, removed) = remove_at(root, key);
        let len = self.len - usize::from(removed.is_some());
        (Self { root, len }, removed)
    }

    /// Returns all entries as an owned standard-library map.
    pub fn to_btree_map(&self) -> std::collections::BTreeMap<K, V>
    where
        K: Clone,
        V: Clone,
    {
        self.iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    }

    fn clone_version(&self) -> Self {
        Self {
            root: self.root.clone(),
            len: self.len,
        }
    }
}

impl<K, V> Default for PersistentAvlTreeMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, K, V> IntoIterator for &'a PersistentAvlTreeMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

fn height<K, V>(node: &Link<K, V>) -> usize {
    node.as_ref().map_or(0, |node| node.height)
}

fn make_node<K, V>(key: K, value: V, left: Link<K, V>, right: Link<K, V>) -> Arc<Node<K, V>> {
    Arc::new(Node {
        key,
        value,
        height: 1 + height(&left).max(height(&right)),
        left,
        right,
    })
}

fn balance_factor<K, V>(node: &Node<K, V>) -> isize {
    height(&node.left) as isize - height(&node.right) as isize
}

fn rotate_right<K: Clone, V: Clone>(root: &Arc<Node<K, V>>) -> Arc<Node<K, V>> {
    let pivot = root
        .left
        .as_ref()
        .expect("right rotation requires left child");
    let new_right = make_node(
        root.key.clone(),
        root.value.clone(),
        pivot.right.clone(),
        root.right.clone(),
    );
    make_node(
        pivot.key.clone(),
        pivot.value.clone(),
        pivot.left.clone(),
        Some(new_right),
    )
}

fn rotate_left<K: Clone, V: Clone>(root: &Arc<Node<K, V>>) -> Arc<Node<K, V>> {
    let pivot = root
        .right
        .as_ref()
        .expect("left rotation requires right child");
    let new_left = make_node(
        root.key.clone(),
        root.value.clone(),
        root.left.clone(),
        pivot.left.clone(),
    );
    make_node(
        pivot.key.clone(),
        pivot.value.clone(),
        Some(new_left),
        pivot.right.clone(),
    )
}

fn rebalance<K: Clone, V: Clone>(
    key: K,
    value: V,
    left: Link<K, V>,
    right: Link<K, V>,
) -> Arc<Node<K, V>> {
    let root = make_node(key, value, left, right);
    let factor = balance_factor(&root);
    if factor > 1 {
        let left = root.left.as_ref().expect("left-heavy node has left child");
        if balance_factor(left) < 0 {
            let rotated_left = rotate_left(left);
            let adjusted = make_node(
                root.key.clone(),
                root.value.clone(),
                Some(rotated_left),
                root.right.clone(),
            );
            rotate_right(&adjusted)
        } else {
            rotate_right(&root)
        }
    } else if factor < -1 {
        let right = root
            .right
            .as_ref()
            .expect("right-heavy node has right child");
        if balance_factor(right) > 0 {
            let rotated_right = rotate_right(right);
            let adjusted = make_node(
                root.key.clone(),
                root.value.clone(),
                root.left.clone(),
                Some(rotated_right),
            );
            rotate_left(&adjusted)
        } else {
            rotate_left(&root)
        }
    } else {
        root
    }
}

fn insert_at<K: Ord + Clone, V: Clone>(
    node: Option<&Arc<Node<K, V>>>,
    key: K,
    value: V,
) -> (Arc<Node<K, V>>, Option<V>) {
    let Some(node) = node else {
        return (make_node(key, value, None, None), None);
    };
    match key.cmp(&node.key) {
        Ordering::Less => {
            let (left, previous) = insert_at(node.left.as_ref(), key, value);
            (
                rebalance(
                    node.key.clone(),
                    node.value.clone(),
                    Some(left),
                    node.right.clone(),
                ),
                previous,
            )
        }
        Ordering::Greater => {
            let (right, previous) = insert_at(node.right.as_ref(), key, value);
            (
                rebalance(
                    node.key.clone(),
                    node.value.clone(),
                    node.left.clone(),
                    Some(right),
                ),
                previous,
            )
        }
        Ordering::Equal => (
            make_node(
                node.key.clone(),
                value,
                node.left.clone(),
                node.right.clone(),
            ),
            Some(node.value.clone()),
        ),
    }
}

fn remove_at<K: Borrow<Q> + Clone, V: Clone, Q: Ord + ?Sized>(
    node: &Arc<Node<K, V>>,
    key: &Q,
) -> (Link<K, V>, Option<V>) {
    match node.key.borrow().cmp(key) {
        Ordering::Less => {
            let Some(right) = node.right.as_ref() else {
                return (Some(Arc::clone(node)), None);
            };
            let (new_right, removed) = remove_at(right, key);
            if removed.is_none() {
                return (Some(Arc::clone(node)), None);
            }
            (
                Some(rebalance(
                    node.key.clone(),
                    node.value.clone(),
                    node.left.clone(),
                    new_right,
                )),
                removed,
            )
        }
        Ordering::Greater => {
            let Some(left) = node.left.as_ref() else {
                return (Some(Arc::clone(node)), None);
            };
            let (new_left, removed) = remove_at(left, key);
            if removed.is_none() {
                return (Some(Arc::clone(node)), None);
            }
            (
                Some(rebalance(
                    node.key.clone(),
                    node.value.clone(),
                    new_left,
                    node.right.clone(),
                )),
                removed,
            )
        }
        Ordering::Equal => {
            let removed = node.value.clone();
            match (&node.left, &node.right) {
                (None, _) => (node.right.clone(), Some(removed)),
                (_, None) => (node.left.clone(), Some(removed)),
                (Some(_), Some(right)) => {
                    let (new_right, successor_key, successor_value) = extract_min(right);
                    (
                        Some(rebalance(
                            successor_key,
                            successor_value,
                            node.left.clone(),
                            new_right,
                        )),
                        Some(removed),
                    )
                }
            }
        }
    }
}

fn extract_min<K: Clone, V: Clone>(node: &Arc<Node<K, V>>) -> (Link<K, V>, K, V) {
    let Some(left) = node.left.as_ref() else {
        return (node.right.clone(), node.key.clone(), node.value.clone());
    };
    let (new_left, key, value) = extract_min(left);
    (
        Some(rebalance(
            node.key.clone(),
            node.value.clone(),
            new_left,
            node.right.clone(),
        )),
        key,
        value,
    )
}

#[cfg(test)]
mod tests {
    use super::PersistentAvlTreeMap;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    fn validate<K: Ord, V>(node: &Option<Arc<super::Node<K, V>>>) -> (usize, usize) {
        let Some(node) = node else {
            return (0, 0);
        };
        let (left_height, left_count) = validate(&node.left);
        let (right_height, right_count) = validate(&node.right);
        assert!(left_height.abs_diff(right_height) <= 1);
        let height = 1 + left_height.max(right_height);
        assert_eq!(node.height, height);
        (height, left_count + right_count + 1)
    }

    #[test]
    fn versions_are_independent_and_match_btree_map() {
        let mut versions = vec![PersistentAvlTreeMap::new()];
        let mut expected_versions = vec![BTreeMap::new()];
        for key in [8, 3, 10, 1, 6, 14, 4, 7, 13, 2, 5, 9, 12, 11, 15] {
            let (next, previous) = versions.last().unwrap().insert(key, key * 10);
            assert_eq!(previous, None);
            let mut expected = expected_versions.last().unwrap().clone();
            expected.insert(key, key * 10);
            versions.push(next);
            expected_versions.push(expected);
        }
        for (tree, expected) in versions.iter().zip(&expected_versions) {
            assert_eq!(
                tree.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>(),
                expected.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>()
            );
            let (_, count) = validate(&tree.root);
            assert_eq!(count, tree.len());
        }

        let (removed, value) = versions.last().unwrap().remove(&8);
        assert_eq!(value, Some(80));
        assert!(!removed.contains_key(&8));
        assert!(versions.last().unwrap().contains_key(&8));
        validate(&removed.root);
    }

    #[test]
    fn shares_unmodified_subtrees_and_returns_replaced_values() {
        let mut map = PersistentAvlTreeMap::new();
        for key in [4, 2, 6, 1, 3, 5, 7] {
            map = map.insert(key, key).0;
        }
        let (updated, previous) = map.insert(1, 100);
        assert_eq!(previous, Some(1));
        assert_eq!(map.get(&1), Some(&1));
        assert_eq!(updated.get(&1), Some(&100));
        assert!(Arc::ptr_eq(
            map.root.as_ref().unwrap().right.as_ref().unwrap(),
            updated.root.as_ref().unwrap().right.as_ref().unwrap()
        ));
        let (same, removed) = map.remove(&100);
        assert_eq!(removed, None);
        assert!(Arc::ptr_eq(
            map.root.as_ref().unwrap(),
            same.root.as_ref().unwrap()
        ));
    }

    #[test]
    fn preserves_avl_invariants_through_seeded_insertions_and_removals() {
        let mut tree = PersistentAvlTreeMap::new();
        let mut expected = BTreeMap::new();
        let mut state = 73u64;
        for _ in 0..600 {
            state = state
                .wrapping_mul(2862933555777941757)
                .wrapping_add(3037000493);
            let key = (state % 127) as i32;
            if state & 2 == 0 {
                let value = (state >> 16) as i64;
                let (next, previous) = tree.insert(key, value);
                assert_eq!(previous, expected.insert(key, value));
                tree = next;
            } else {
                let (next, removed) = tree.remove(&key);
                assert_eq!(removed, expected.remove(&key));
                tree = next;
            }
            let (_, count) = validate(&tree.root);
            assert_eq!(count, tree.len());
            assert_eq!(tree.to_btree_map(), expected);
        }
    }
}
