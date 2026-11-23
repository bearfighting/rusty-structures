//! A B+ tree map and set with a small branching factor for learning.

mod iter;
mod set;

pub use iter::{Iter, Keys};
pub use set::BPlusTreeSet;

use std::borrow::Borrow;
use std::iter::FusedIterator;
use std::rc::Rc;

type NodeId = usize;
const MAX_LEAF_ENTRIES: usize = 3;
const MIN_LEAF_ENTRIES: usize = 2;
const MAX_CHILDREN: usize = 4;
const MIN_CHILDREN: usize = 2;

#[derive(Clone, Debug)]
pub(super) enum Node<K, V> {
    Leaf {
        entries: Vec<(Rc<K>, V)>,
        next: Option<NodeId>,
    },
    Internal {
        separators: Vec<Rc<K>>,
        children: Vec<NodeId>,
    },
}

impl<K, V> Node<K, V> {
    fn occupancy(&self) -> usize {
        match self {
            Self::Leaf { entries, .. } => entries.len(),
            Self::Internal { children, .. } => children.len(),
        }
    }
    fn minimum_occupancy(&self) -> usize {
        match self {
            Self::Leaf { .. } => MIN_LEAF_ENTRIES,
            Self::Internal { .. } => MIN_CHILDREN,
        }
    }
    fn is_underfull(&self) -> bool {
        self.occupancy() < self.minimum_occupancy()
    }
}

/// An ordered map implemented as a B+ tree.
///
/// Internal nodes contain separator keys; entries live only in leaves. The
/// implementation uses an arena of nodes addressed by `NodeId`. A linked leaf
/// chain supports sequential scans without returning to the internal index.
/// Leaves hold up to three entries and internal nodes up to four children.
///
/// # Examples
///
/// ```
/// use data_structures::BPlusTreeMap;
///
/// let mut map = BPlusTreeMap::new();
/// map.insert("Ada", 36);
/// map.insert("Grace", 85);
/// assert_eq!(map.get("Ada"), Some(&36));
/// assert_eq!(map.iter().map(|(key, _)| *key).collect::<Vec<_>>(), ["Ada", "Grace"]);
/// ```
#[derive(Clone, Debug)]
pub struct BPlusTreeMap<K, V> {
    nodes: Vec<Option<Node<K, V>>>,
    free_nodes: Vec<NodeId>,
    root: Option<NodeId>,
    first_leaf: Option<NodeId>,
    len: usize,
}

impl<K, V> BPlusTreeMap<K, V> {
    /// Creates an empty B+ tree map.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            nodes: Vec::new(),
            free_nodes: Vec::new(),
            root: None,
            first_leaf: None,
            len: 0,
        }
    }
    /// Returns the number of entries.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }
    /// Returns whether the map is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// Returns the number of levels, or zero for an empty tree.
    pub fn height(&self) -> usize {
        let mut height = 0;
        let mut id = self.root;
        while let Some(node_id) = id {
            height += 1;
            id = match self.node(node_id) {
                Node::Leaf { .. } => None,
                Node::Internal { children, .. } => children.first().copied(),
            };
        }
        height
    }
    /// Removes all entries and releases the node arena.
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.free_nodes.clear();
        self.root = None;
        self.first_leaf = None;
        self.len = 0;
    }
    /// Iterates over entries in ascending key order by following the leaf chain.
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter::new(&self.nodes, self.first_leaf, self.len)
    }
    /// Iterates over keys in ascending order.
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys::new(self.iter())
    }
    /// Iterates over values in key order.
    pub fn values(&self) -> impl ExactSizeIterator<Item = &V> + FusedIterator {
        self.iter().map(|(_, value)| value)
    }

    fn node(&self, id: NodeId) -> &Node<K, V> {
        self.nodes[id]
            .as_ref()
            .expect("live node ID points to a node")
    }
    fn node_mut(&mut self, id: NodeId) -> &mut Node<K, V> {
        self.nodes[id]
            .as_mut()
            .expect("live node ID points to a node")
    }
    fn allocate(&mut self, node: Node<K, V>) -> NodeId {
        if let Some(id) = self.free_nodes.pop() {
            debug_assert!(self.nodes[id].is_none());
            self.nodes[id] = Some(node);
            return id;
        }
        let id = self.nodes.len();
        self.nodes.push(Some(node));
        id
    }
    fn first_key(&self, mut id: NodeId) -> &Rc<K> {
        loop {
            match self.node(id) {
                Node::Leaf { entries, .. } => {
                    return &entries.first().expect("non-empty subtree").0;
                }
                Node::Internal { children, .. } => {
                    id = *children.first().expect("internal node has a child")
                }
            }
        }
    }
    fn refresh(&mut self, id: NodeId) {
        let children = match self.node(id) {
            Node::Leaf { .. } => return,
            Node::Internal { children, .. } => children.clone(),
        };
        let separators = children
            .iter()
            .skip(1)
            .map(|child| Rc::clone(self.first_key(*child)))
            .collect();
        if let Node::Internal {
            separators: target, ..
        } = self.node_mut(id)
        {
            *target = separators;
        }
    }
    fn child_index<Q>(&self, id: NodeId, key: &Q) -> usize
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        match self.node(id) {
            Node::Internal { separators, .. } => {
                separators.partition_point(|sep| sep.as_ref().borrow() <= key)
            }
            Node::Leaf { .. } => unreachable!("leaf has no child index"),
        }
    }
}

impl<K: Ord, V> BPlusTreeMap<K, V> {
    /// Returns the value associated with `key`.
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let mut id = self.root?;
        loop {
            match self.node(id) {
                Node::Leaf { entries, .. } => {
                    let index = entries
                        .binary_search_by(|(stored, _)| stored.as_ref().borrow().cmp(key))
                        .ok()?;
                    return Some(&entries[index].1);
                }
                Node::Internal { children, .. } => id = children[self.child_index(id, key)],
            }
        }
    }
    /// Returns mutable access to the value associated with `key`.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let mut id = self.root?;
        loop {
            if matches!(self.node(id), Node::Leaf { .. }) {
                let Node::Leaf { entries, .. } = self.node_mut(id) else {
                    unreachable!()
                };
                let index = entries
                    .binary_search_by(|(stored, _)| stored.as_ref().borrow().cmp(key))
                    .ok()?;
                return Some(&mut entries[index].1);
            }
            let index = self.child_index(id, key);
            id = match self.node(id) {
                Node::Internal { children, .. } => children[index],
                _ => unreachable!(),
            };
        }
    }
    /// Returns whether `key` exists.
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get(key).is_some()
    }

    /// Inserts an entry, returning the previous value when replacing a key.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if let Some(old) = self.get_mut(&key) {
            return Some(std::mem::replace(old, value));
        }
        let key = Rc::new(key);
        let Some(root) = self.root else {
            let leaf = self.allocate(Node::Leaf {
                entries: vec![(key, value)],
                next: None,
            });
            self.root = Some(leaf);
            self.first_leaf = Some(leaf);
            self.len = 1;
            return None;
        };
        if let Some(right) = self.insert_at(root, key, value) {
            self.root = Some(self.allocate(Node::Internal {
                separators: Vec::new(),
                children: vec![root, right],
            }));
            self.refresh(self.root.expect("root was just allocated"));
        }
        self.len += 1;
        None
    }
    fn insert_at(&mut self, id: NodeId, key: Rc<K>, value: V) -> Option<NodeId> {
        match self.node(id) {
            Node::Leaf { .. } => {
                let (index, old_next) = match self.node(id) {
                    Node::Leaf { entries, next } => (
                        entries.partition_point(|(stored, _)| stored.as_ref() < key.as_ref()),
                        *next,
                    ),
                    _ => unreachable!(),
                };
                let Node::Leaf { entries, .. } = self.node_mut(id) else {
                    unreachable!()
                };
                entries.insert(index, (key, value));
                if entries.len() <= MAX_LEAF_ENTRIES {
                    return None;
                }
                let right_entries = entries.split_off(MIN_LEAF_ENTRIES);
                let right = self.allocate(Node::Leaf {
                    entries: right_entries,
                    next: old_next,
                });
                if let Node::Leaf { next, .. } = self.node_mut(id) {
                    *next = Some(right);
                }
                Some(right)
            }
            Node::Internal { .. } => {
                let child_index = self.child_index(id, key.as_ref());
                let child = match self.node(id) {
                    Node::Internal { children, .. } => children[child_index],
                    _ => unreachable!(),
                };
                let split = self.insert_at(child, key, value);
                if let Some(right) = split {
                    let Node::Internal { children, .. } = self.node_mut(id) else {
                        unreachable!()
                    };
                    children.insert(child_index + 1, right);
                }
                self.refresh(id);
                let children = match self.node(id) {
                    Node::Internal { children, .. } => children.len(),
                    _ => unreachable!(),
                };
                if children <= MAX_CHILDREN {
                    return None;
                }
                let right_children = match self.node_mut(id) {
                    Node::Internal { children, .. } => children.split_off(children.len() / 2),
                    _ => unreachable!(),
                };
                self.refresh(id);
                let right = self.allocate(Node::Internal {
                    separators: Vec::new(),
                    children: right_children,
                });
                self.refresh(right);
                Some(right)
            }
        }
    }

    /// Removes an entry and returns its value, if present.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let root = self.root?;
        let removed = self.remove_at(root, key)?;
        self.len -= 1;
        while let Some(root) = self.root {
            let only_child = match self.node(root) {
                Node::Internal { children, .. } if children.len() == 1 => Some(children[0]),
                _ => None,
            };
            let Some(child) = only_child else { break };
            self.nodes[root] = None;
            self.free_nodes.push(root);
            self.root = Some(child);
        }
        if self.len == 0 {
            if let Some(root) = self.root.take() {
                self.nodes[root] = None;
                self.free_nodes.push(root);
            }
            self.first_leaf = None;
        }
        Some(removed)
    }
    fn remove_at<Q>(&mut self, id: NodeId, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        match self.node(id) {
            Node::Leaf { .. } => {
                let Node::Leaf { entries, .. } = self.node_mut(id) else {
                    unreachable!()
                };
                let index = entries
                    .binary_search_by(|(stored, _)| stored.as_ref().borrow().cmp(key))
                    .ok()?;
                Some(entries.remove(index).1)
            }
            Node::Internal { .. } => {
                let child_index = self.child_index(id, key);
                let child = match self.node(id) {
                    Node::Internal { children, .. } => children[child_index],
                    _ => unreachable!(),
                };
                let value = self.remove_at(child, key)?;
                if self.node(child).is_underfull() {
                    self.rebalance_child(id, child_index);
                }
                self.refresh(id);
                Some(value)
            }
        }
    }
    fn rebalance_child(&mut self, parent: NodeId, index: usize) {
        let children = match self.node(parent) {
            Node::Internal { children, .. } => children.clone(),
            _ => unreachable!(),
        };
        if index > 0
            && self.node(children[index - 1]).occupancy()
                > self.node(children[index]).minimum_occupancy()
        {
            self.borrow_from_left(children[index - 1], children[index]);
        } else if index + 1 < children.len()
            && self.node(children[index + 1]).occupancy()
                > self.node(children[index]).minimum_occupancy()
        {
            self.borrow_from_right(children[index], children[index + 1]);
        } else if index > 0 {
            self.merge_nodes(parent, index - 1);
        } else {
            self.merge_nodes(parent, index);
        }
    }
    fn borrow_from_left(&mut self, left: NodeId, current: NodeId) {
        let (source, target) = two_nodes_mut(&mut self.nodes, left, current);
        match (source, target) {
            (
                Node::Leaf {
                    entries: source, ..
                },
                Node::Leaf {
                    entries: target, ..
                },
            ) => target.insert(0, source.pop().expect("left leaf has spare entry")),
            (
                Node::Internal {
                    children: source, ..
                },
                Node::Internal {
                    children: target, ..
                },
            ) => target.insert(0, source.pop().expect("left internal node has spare child")),
            _ => unreachable!("siblings at one level have matching node types"),
        }
        self.refresh(left);
        self.refresh(current);
    }
    fn borrow_from_right(&mut self, current: NodeId, right: NodeId) {
        let (target, source) = two_nodes_mut(&mut self.nodes, current, right);
        match (target, source) {
            (
                Node::Leaf {
                    entries: target, ..
                },
                Node::Leaf {
                    entries: source, ..
                },
            ) => target.push(source.remove(0)),
            (
                Node::Internal {
                    children: target, ..
                },
                Node::Internal {
                    children: source, ..
                },
            ) => target.push(source.remove(0)),
            _ => unreachable!("siblings at one level have matching node types"),
        }
        self.refresh(current);
        self.refresh(right);
    }
    fn merge_nodes(&mut self, parent: NodeId, left_index: usize) {
        let (left, right) = match self.node(parent) {
            Node::Internal { children, .. } => (children[left_index], children[left_index + 1]),
            _ => unreachable!(),
        };
        let right_node = self.nodes[right].take().expect("right sibling is live");
        match (self.node_mut(left), right_node) {
            (
                Node::Leaf {
                    entries: left_entries,
                    next: left_next,
                },
                Node::Leaf {
                    entries: mut right_entries,
                    next,
                },
            ) => {
                left_entries.append(&mut right_entries);
                *left_next = next;
            }
            (
                Node::Internal {
                    children: left_children,
                    ..
                },
                Node::Internal {
                    children: mut right_children,
                    ..
                },
            ) => left_children.append(&mut right_children),
            _ => unreachable!("siblings at one level have matching node types"),
        }
        if let Node::Internal { children, .. } = self.node_mut(parent) {
            children.remove(left_index + 1);
        }
        self.free_nodes.push(right);
        self.refresh(left);
        self.refresh(parent);
    }
}

fn two_nodes_mut<K, V>(
    nodes: &mut [Option<Node<K, V>>],
    a: NodeId,
    b: NodeId,
) -> (&mut Node<K, V>, &mut Node<K, V>) {
    assert_ne!(a, b, "siblings have distinct arena IDs");
    if a < b {
        let (left, right) = nodes.split_at_mut(b);
        (
            left[a].as_mut().expect("live node"),
            right[0].as_mut().expect("live node"),
        )
    } else {
        let (left, right) = nodes.split_at_mut(a);
        (
            right[0].as_mut().expect("live node"),
            left[b].as_mut().expect("live node"),
        )
    }
}

impl<K, V> Default for BPlusTreeMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
impl<K: Ord, V> Extend<(K, V)> for BPlusTreeMap<K, V> {
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        for (key, value) in iter {
            self.insert(key, value);
        }
    }
}
impl<K: Ord, V> FromIterator<(K, V)> for BPlusTreeMap<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut map = Self::new();
        map.extend(iter);
        map
    }
}
impl<K: Ord, V> crate::tree::map::TreeMap<K, V> for BPlusTreeMap<K, V> {
    type Iter<'a>
        = Iter<'a, K, V>
    where
        Self: 'a,
        K: 'a,
        V: 'a;
    fn len(&self) -> usize {
        self.len()
    }
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        BPlusTreeMap::insert(self, key, value)
    }
    fn get(&self, key: &K) -> Option<&V> {
        BPlusTreeMap::get(self, key)
    }
    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        BPlusTreeMap::get_mut(self, key)
    }
    fn remove(&mut self, key: &K) -> Option<V> {
        BPlusTreeMap::remove(self, key)
    }
    fn iter(&self) -> Self::Iter<'_> {
        BPlusTreeMap::iter(self)
    }
}
impl<'a, K, V> IntoIterator for &'a BPlusTreeMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BPlusTreeMap, MAX_CHILDREN, MAX_LEAF_ENTRIES, MIN_CHILDREN, MIN_LEAF_ENTRIES, Node, NodeId,
    };
    use std::collections::BTreeMap as StdBTreeMap;

    #[derive(Default)]
    struct Invariants {
        leaf_depth: Option<usize>,
        leaves: Vec<NodeId>,
        reachable: Vec<NodeId>,
        entries: usize,
    }

    fn check<'a, K: Ord + std::fmt::Debug, V>(
        map: &'a BPlusTreeMap<K, V>,
        id: NodeId,
        is_root: bool,
        depth: usize,
        state: &mut Invariants,
    ) -> (Option<&'a K>, Option<&'a K>) {
        state.reachable.push(id);
        let node = map.node(id);
        match node {
            Node::Leaf {
                entries: leaf_entries,
                ..
            } => {
                assert!(!leaf_entries.is_empty() || is_root);
                assert!(leaf_entries.len() <= MAX_LEAF_ENTRIES);
                if !is_root {
                    assert!(leaf_entries.len() >= MIN_LEAF_ENTRIES);
                }
                for pair in leaf_entries.windows(2) {
                    assert!(pair[0].0 < pair[1].0);
                }
                if let Some(expected) = state.leaf_depth {
                    assert_eq!(expected, depth);
                } else {
                    state.leaf_depth = Some(depth);
                }
                state.leaves.push(id);
                state.entries += leaf_entries.len();
                (
                    leaf_entries.first().map(|entry| entry.0.as_ref()),
                    leaf_entries.last().map(|entry| entry.0.as_ref()),
                )
            }
            Node::Internal {
                separators,
                children,
            } => {
                assert!(children.len() <= MAX_CHILDREN);
                assert!(children.len() >= if is_root { 2 } else { MIN_CHILDREN });
                assert_eq!(separators.len() + 1, children.len());
                let ranges: Vec<_> = children
                    .iter()
                    .map(|child| check(map, *child, false, depth + 1, state))
                    .collect();
                for index in 0..separators.len() {
                    assert_eq!(separators[index].as_ref(), ranges[index + 1].0.unwrap());
                    assert!(ranges[index].1.unwrap() < ranges[index + 1].0.unwrap());
                }
                (ranges.first().unwrap().0, ranges.last().unwrap().1)
            }
        }
    }

    fn validate(map: &BPlusTreeMap<i32, i32>, expected: &StdBTreeMap<i32, i32>) {
        let mut state = Invariants::default();
        if let Some(root) = map.root {
            check(map, root, true, 1, &mut state);
        }
        assert_eq!(state.entries, map.len);
        state.reachable.sort_unstable();
        state.reachable.dedup();
        assert_eq!(
            state.reachable.len(),
            map.nodes.iter().filter(|node| node.is_some()).count(),
            "arena has no unreachable live nodes"
        );
        let mut free = map.free_nodes.clone();
        free.sort_unstable();
        free.dedup();
        assert_eq!(
            free,
            map.nodes
                .iter()
                .enumerate()
                .filter_map(|(id, node)| node.is_none().then_some(id))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            map.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>(),
            expected.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>()
        );
        assert_eq!(map.first_leaf, state.leaves.first().copied());
        let mut linked = Vec::new();
        let mut next = map.first_leaf;
        while let Some(id) = next {
            assert!(!linked.contains(&id), "leaf chain has no cycle");
            linked.push(id);
            next = match map.node(id) {
                Node::Leaf { next, .. } => *next,
                _ => panic!("leaf link targets a leaf"),
            };
        }
        assert_eq!(linked, state.leaves, "leaf links follow tree order");
    }

    #[test]
    fn arena_splits_borrows_merges_and_matches_standard_map() {
        let mut map = BPlusTreeMap::new();
        let mut expected = StdBTreeMap::new();
        for key in (0..180).map(|key| (key * 73) % 180) {
            assert_eq!(map.insert(key, key * 10), expected.insert(key, key * 10));
            validate(&map, &expected);
        }
        for key in (0..180).map(|key| (key * 47) % 180) {
            assert_eq!(map.remove(&key), expected.remove(&key));
            validate(&map, &expected);
        }
        assert!(map.is_empty());
        assert_eq!(map.height(), 0);
        assert!(map.first_leaf.is_none());
    }

    #[test]
    fn arena_reuses_nodes_after_repeated_clear_to_empty_cycles() {
        let mut map = BPlusTreeMap::new();
        let mut expected = StdBTreeMap::new();
        let mut high_water = None;
        for _ in 0..8 {
            for key in (0..48).rev() {
                map.insert(key, key);
                expected.insert(key, key);
                validate(&map, &expected);
            }
            for key in 0..48 {
                map.remove(&key);
                expected.remove(&key);
                validate(&map, &expected);
            }
            if let Some(capacity) = high_water {
                assert_eq!(map.nodes.len(), capacity);
            } else {
                high_water = Some(map.nodes.len());
            }
        }
    }
}
