//! Recursive binary search tree operations for teaching and comparison.

use super::{IntoIter, Iter, Node, drop_nodes, remove_node};

/// A recursive implementation of a set-like binary search tree.
///
/// Duplicate values are rejected. Search, insertion, and removal use recursive
/// calls and take O(h) time and call-stack space, where `h` is the tree height.
/// A degenerate tree can therefore use O(n) call-stack space. Iteration yields
/// values in ascending order. Clearing and dropping use the same iterative
/// cleanup as [`super::BinarySearchTree`] to avoid recursive destruction.
///
/// # Examples
///
/// ```
/// use data_structures::RecursiveBinarySearchTree;
///
/// let mut tree = RecursiveBinarySearchTree::new();
/// tree.insert(4);
/// tree.insert(2);
/// tree.insert(6);
/// assert_eq!(tree.iter().copied().collect::<Vec<_>>(), [2, 4, 6]);
/// assert_eq!(tree.remove(&4), Some(4));
/// ```
#[derive(Clone, Debug)]
pub struct RecursiveBinarySearchTree<T> {
    root: Option<Box<Node<T>>>,
    len: usize,
}

impl<T> RecursiveBinarySearchTree<T> {
    /// Creates an empty tree.
    #[must_use]
    pub const fn new() -> Self {
        Self { root: None, len: 0 }
    }

    /// Returns the number of values in the tree.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the tree is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Removes all values from the tree.
    pub fn clear(&mut self) {
        drop_nodes(self.root.take());
        self.len = 0;
    }

    /// Returns an iterator over values in ascending order.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(self.root.as_deref(), self.len)
    }
}

impl<T: Ord> RecursiveBinarySearchTree<T> {
    /// Inserts `value`; returns `false` if an equal value is already present.
    pub fn insert(&mut self, value: T) -> bool {
        if insert_node(&mut self.root, value) {
            self.len += 1;
            true
        } else {
            false
        }
    }

    /// Returns whether an equal value is present.
    pub fn contains(&self, value: &T) -> bool {
        self.get(value).is_some()
    }

    /// Returns the stored value equal to `value`, if present.
    pub fn get(&self, value: &T) -> Option<&T> {
        get_node(self.root.as_deref(), value)
    }

    /// Removes and returns the value equal to `value`, if present.
    pub fn remove(&mut self, value: &T) -> Option<T> {
        let (root, removed) = remove_node(self.root.take(), value);
        self.root = root;
        if removed.is_some() {
            self.len -= 1;
        }
        removed
    }
}

fn insert_node<T: Ord>(link: &mut Option<Box<Node<T>>>, value: T) -> bool {
    match link {
        None => {
            *link = Some(Box::new(Node {
                value,
                left: None,
                right: None,
            }));
            true
        }
        Some(node) => match value.cmp(&node.value) {
            std::cmp::Ordering::Less => insert_node(&mut node.left, value),
            std::cmp::Ordering::Greater => insert_node(&mut node.right, value),
            std::cmp::Ordering::Equal => false,
        },
    }
}

fn get_node<'a, T: Ord>(node: Option<&'a Node<T>>, value: &T) -> Option<&'a T> {
    let node = node?;
    match value.cmp(&node.value) {
        std::cmp::Ordering::Less => get_node(node.left.as_deref(), value),
        std::cmp::Ordering::Greater => get_node(node.right.as_deref(), value),
        std::cmp::Ordering::Equal => Some(&node.value),
    }
}

impl<T> Default for RecursiveBinarySearchTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> IntoIterator for &'a RecursiveBinarySearchTree<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> IntoIterator for RecursiveBinarySearchTree<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(mut self) -> Self::IntoIter {
        let root = self.root.take();
        let len = self.len;
        self.len = 0;
        IntoIter::new(root, len)
    }
}

impl<T: Ord> Extend<T> for RecursiveBinarySearchTree<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.insert(value);
        }
    }
}

impl<T: Ord> FromIterator<T> for RecursiveBinarySearchTree<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut tree = Self::new();
        tree.extend(iter);
        tree
    }
}

impl<T> Drop for RecursiveBinarySearchTree<T> {
    fn drop(&mut self) {
        self.clear();
    }
}
