//! A B-tree map and set using a small node capacity for clarity.

mod iter;
mod set;

pub use iter::{Iter, Keys};
pub use set::BTreeSet;

use std::borrow::Borrow;
use std::iter::FusedIterator;

const MIN_DEGREE: usize = 2;
const MAX_KEYS: usize = 2 * MIN_DEGREE - 1;

#[derive(Clone, Debug)]
struct Node<K, V> {
    entries: Vec<(K, V)>,
    children: Vec<Box<Node<K, V>>>,
}

impl<K, V> Node<K, V> {
    fn new_leaf() -> Self {
        Self {
            entries: Vec::new(),
            children: Vec::new(),
        }
    }

    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

/// An ordered map implemented as a 2–3–4 B-tree.
///
/// Each node stores up to three sorted key-value pairs and up to four child
/// links. Splitting full nodes on insertion and borrowing or merging on
/// deletion keeps all leaves at the same depth. Lookups and updates take
/// O(log n) tree levels; searching within a node takes O(log t) comparisons,
/// where the minimum degree `t` is 2. The small node size favors readability
/// over production cache tuning.
///
/// # Examples
///
/// ```
/// use data_structures::BTreeMap;
///
/// let mut ages = BTreeMap::new();
/// ages.insert("Ada", 36);
/// ages.insert("Grace", 85);
/// assert_eq!(ages.get("Ada"), Some(&36));
/// assert_eq!(ages.remove("Grace"), Some(85));
/// ```
#[derive(Clone, Debug)]
pub struct BTreeMap<K, V> {
    root: Option<Box<Node<K, V>>>,
    len: usize,
}

impl<K, V> BTreeMap<K, V> {
    /// Creates an empty B-tree map.
    #[must_use]
    pub const fn new() -> Self {
        Self { root: None, len: 0 }
    }

    /// Returns the number of key-value pairs.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the map contains no entries.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of levels, or zero for an empty map.
    #[must_use]
    pub fn height(&self) -> usize {
        let mut height = 0;
        let mut node = self.root.as_deref();
        while let Some(current) = node {
            height += 1;
            node = current.children.first().map(Box::as_ref);
        }
        height
    }

    /// Removes all entries.
    pub fn clear(&mut self) {
        self.root = None;
        self.len = 0;
    }

    /// Returns an iterator over key-value pairs in key order.
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter::new(self.root.as_deref(), self.len)
    }

    /// Returns an iterator over keys in key order.
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys::new(self.iter())
    }

    /// Returns an iterator over values in key order.
    pub fn values(&self) -> impl Iterator<Item = &V> + ExactSizeIterator + FusedIterator {
        self.iter().map(|(_, value)| value)
    }
}

impl<K: Ord, V> BTreeMap<K, V> {
    /// Returns a reference to the value for `key`, if present.
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let mut node = self.root.as_deref()?;
        loop {
            match node
                .entries
                .binary_search_by(|(entry, _)| entry.borrow().cmp(key))
            {
                Ok(index) => return Some(&node.entries[index].1),
                Err(_) if node.is_leaf() => return None,
                Err(index) => node = node.children[index].as_ref(),
            }
        }
    }

    /// Returns a mutable reference to the value for `key`, if present.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let mut node = self.root.as_deref_mut()?;
        loop {
            match node
                .entries
                .binary_search_by(|(entry, _)| entry.borrow().cmp(key))
            {
                Ok(index) => return Some(&mut node.entries[index].1),
                Err(_) if node.is_leaf() => return None,
                Err(index) => node = node.children[index].as_mut(),
            }
        }
    }

    /// Returns whether `key` exists in the map.
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get(key).is_some()
    }

    /// Inserts a key-value pair, returning the old value if the key existed.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if let Some(old) = self.get_mut(&key) {
            return Some(std::mem::replace(old, value));
        }

        if self.root.is_none() {
            let mut root = Node::new_leaf();
            root.entries.push((key, value));
            self.root = Some(Box::new(root));
            self.len = 1;
            return None;
        }

        if self
            .root
            .as_ref()
            .is_some_and(|root| root.entries.len() == MAX_KEYS)
        {
            let old_root = self.root.take().expect("root exists");
            let mut root = Node::new_leaf();
            root.children.push(old_root);
            split_child(&mut root, 0);
            self.root = Some(Box::new(root));
        }

        insert_nonfull(self.root.as_deref_mut().expect("root exists"), key, value);
        self.len += 1;
        None
    }

    /// Removes a key and returns its value, if present.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let root = self.root.as_deref_mut()?;
        let (_, value) = remove_from(root, key)?;
        self.len -= 1;
        if self
            .root
            .as_ref()
            .is_some_and(|root| root.entries.is_empty())
        {
            let mut root = self.root.take().expect("root exists");
            self.root = root.children.pop();
        }
        Some(value)
    }
}

fn split_child<K, V>(parent: &mut Node<K, V>, index: usize) {
    let child = &mut parent.children[index];
    let mut right = Box::new(Node {
        entries: child.entries.split_off(MIN_DEGREE),
        children: Vec::new(),
    });
    let median = child.entries.pop().expect("full child has a median entry");
    if !child.is_leaf() {
        right.children = child.children.split_off(MIN_DEGREE);
    }
    parent.entries.insert(index, median);
    parent.children.insert(index + 1, right);
}

fn insert_nonfull<K: Ord, V>(node: &mut Node<K, V>, key: K, value: V) {
    let index = match node.entries.binary_search_by(|(entry, _)| entry.cmp(&key)) {
        Ok(_) => unreachable!("duplicate keys are handled before insertion"),
        Err(index) => index,
    };
    if node.is_leaf() {
        node.entries.insert(index, (key, value));
        return;
    }

    if node.children[index].entries.len() == MAX_KEYS {
        split_child(node, index);
        match key.cmp(&node.entries[index].0) {
            std::cmp::Ordering::Greater => {
                insert_nonfull(&mut node.children[index + 1], key, value)
            }
            std::cmp::Ordering::Less => insert_nonfull(&mut node.children[index], key, value),
            std::cmp::Ordering::Equal => {
                unreachable!("duplicate keys are handled before insertion")
            }
        }
    } else {
        insert_nonfull(&mut node.children[index], key, value);
    }
}

fn remove_from<K, V, Q>(node: &mut Node<K, V>, key: &Q) -> Option<(K, V)>
where
    K: Borrow<Q>,
    Q: Ord + ?Sized,
{
    let search = node
        .entries
        .binary_search_by(|(entry, _)| entry.borrow().cmp(key));
    if let Ok(index) = search {
        if node.is_leaf() {
            return Some(node.entries.remove(index));
        }

        if node.children[index].entries.len() >= MIN_DEGREE {
            let predecessor = remove_max(&mut node.children[index]);
            return Some(std::mem::replace(&mut node.entries[index], predecessor));
        }
        if node.children[index + 1].entries.len() >= MIN_DEGREE {
            let successor = remove_min(&mut node.children[index + 1]);
            return Some(std::mem::replace(&mut node.entries[index], successor));
        }
        merge_children(node, index);
        return remove_from(&mut node.children[index], key);
    }

    let mut child_index = search.expect_err("search did not find the key");
    if node.is_leaf() {
        return None;
    }
    if node.children[child_index].entries.len() == 1 {
        if child_index > 0 && node.children[child_index - 1].entries.len() >= MIN_DEGREE {
            borrow_from_previous(node, child_index);
        } else if child_index < node.entries.len()
            && node.children[child_index + 1].entries.len() >= MIN_DEGREE
        {
            borrow_from_next(node, child_index);
        } else if child_index < node.entries.len() {
            merge_children(node, child_index);
        } else {
            merge_children(node, child_index - 1);
            child_index -= 1;
        }
    }
    remove_from(&mut node.children[child_index], key)
}

fn remove_min<K, V>(node: &mut Node<K, V>) -> (K, V) {
    if node.is_leaf() {
        return node.entries.remove(0);
    }
    if node.children[0].entries.len() == 1 {
        if node.children[1].entries.len() >= MIN_DEGREE {
            borrow_from_next(node, 0);
        } else {
            merge_children(node, 0);
        }
    }
    remove_min(&mut node.children[0])
}

fn remove_max<K, V>(node: &mut Node<K, V>) -> (K, V) {
    let last = node.entries.len() - 1;
    if node.is_leaf() {
        return node.entries.remove(last);
    }
    if node.children[last + 1].entries.len() == 1 {
        if node.children[last].entries.len() >= MIN_DEGREE {
            borrow_from_previous(node, last + 1);
        } else {
            merge_children(node, last);
        }
    }
    let last_child = node.children.len() - 1;
    remove_max(&mut node.children[last_child])
}

fn borrow_from_previous<K, V>(parent: &mut Node<K, V>, child_index: usize) {
    let separator = parent.entries.remove(child_index - 1);
    let (siblings, child_slice) = parent.children.split_at_mut(child_index);
    let sibling = siblings.last_mut().expect("left sibling exists");
    let child = &mut child_slice[0];
    child.entries.insert(0, separator);
    parent.entries.insert(
        child_index - 1,
        sibling.entries.pop().expect("sibling has a key"),
    );
    if !sibling.is_leaf() {
        child
            .children
            .insert(0, sibling.children.pop().expect("sibling has a child"));
    }
}

fn borrow_from_next<K, V>(parent: &mut Node<K, V>, child_index: usize) {
    let separator = parent.entries.remove(child_index);
    let (left, siblings) = parent.children.split_at_mut(child_index + 1);
    let child = left.last_mut().expect("child exists");
    let sibling = &mut siblings[0];
    child.entries.push(separator);
    parent
        .entries
        .insert(child_index, sibling.entries.remove(0));
    if !sibling.is_leaf() {
        child.children.push(sibling.children.remove(0));
    }
}

fn merge_children<K, V>(parent: &mut Node<K, V>, index: usize) {
    let separator = parent.entries.remove(index);
    let right = parent.children.remove(index + 1);
    let left = &mut parent.children[index];
    left.entries.push(separator);
    left.entries.extend(right.entries);
    left.children.extend(right.children);
}

impl<K, V> Default for BTreeMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord, V> Extend<(K, V)> for BTreeMap<K, V> {
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, entries: I) {
        for (key, value) in entries {
            self.insert(key, value);
        }
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for BTreeMap<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(entries: I) -> Self {
        let mut map = Self::new();
        map.extend(entries);
        map
    }
}

impl<'a, K: Ord, V> IntoIterator for &'a BTreeMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::{BTreeMap, Node};

    fn validate(
        node: &Node<i32, i32>,
        lower: Option<i32>,
        upper: Option<i32>,
        depth: usize,
        leaf_depth: &mut Option<usize>,
    ) {
        assert!(!node.entries.is_empty() && node.entries.len() <= 3);
        assert!(node.entries.windows(2).all(|pair| pair[0].0 < pair[1].0));
        for (key, _) in &node.entries {
            assert!(lower.is_none_or(|bound| bound < *key));
            assert!(upper.is_none_or(|bound| *key < bound));
        }

        if node.is_leaf() {
            if let Some(expected) = leaf_depth {
                assert_eq!(*expected, depth, "leaves must have the same depth");
            } else {
                *leaf_depth = Some(depth);
            }
            return;
        }

        assert_eq!(node.children.len(), node.entries.len() + 1);
        for index in 0..node.children.len() {
            let child_lower = if index == 0 {
                lower
            } else {
                Some(node.entries[index - 1].0)
            };
            let child_upper = if index == node.entries.len() {
                upper
            } else {
                Some(node.entries[index].0)
            };
            validate(
                &node.children[index],
                child_lower,
                child_upper,
                depth + 1,
                leaf_depth,
            );
        }
    }

    fn check(map: &BTreeMap<i32, i32>, expected: &std::collections::BTreeMap<i32, i32>) {
        assert_eq!(map.len(), expected.len());
        assert_eq!(
            map.iter().map(|(&k, &v)| (k, v)).collect::<Vec<_>>(),
            expected.iter().map(|(&k, &v)| (k, v)).collect::<Vec<_>>()
        );
        if let Some(root) = map.root.as_deref() {
            let mut leaf_depth = None;
            validate(root, None, None, 0, &mut leaf_depth);
        }
    }

    #[test]
    fn splits_borrows_merges_and_matches_standard_map() {
        let mut map = BTreeMap::new();
        let mut expected = std::collections::BTreeMap::new();
        for value in (0..300).map(|value| (value * 97) % 300) {
            assert_eq!(
                map.insert(value, value * 10),
                expected.insert(value, value * 10)
            );
            check(&map, &expected);
        }
        assert_eq!(map.insert(150, -1), Some(1500));
        expected.insert(150, -1);
        check(&map, &expected);

        for value in (0..300).step_by(2).chain([301, 303]) {
            assert_eq!(map.remove(&value), expected.remove(&value));
            check(&map, &expected);
        }
        for value in (1..300).step_by(2) {
            assert_eq!(map.remove(&value), expected.remove(&value));
            check(&map, &expected);
        }
        assert!(map.is_empty());
        assert_eq!(map.height(), 0);
    }
}
