//! An unbalanced binary search tree.

mod iter;
mod recursive;

pub use iter::{IntoIter, Iter};
pub use recursive::RecursiveBinarySearchTree;

#[derive(Clone, Debug)]
pub(super) struct Node<T> {
    value: T,
    left: Option<Box<Node<T>>>,
    right: Option<Box<Node<T>>>,
}

/// An iterative, set-like binary search tree with unique values.
///
/// Values compare less than a node are stored in its left subtree; greater
/// values are stored in its right subtree. Equal values are not inserted.
/// Search, insertion, and removal use loops and take O(h) time, where `h` is
/// the tree height; they can degrade to O(n) for an unbalanced tree. Iteration
/// yields values in ascending order. See [`RecursiveBinarySearchTree`] for a
/// recursive implementation of the same operations.
///
/// # Examples
///
/// ```
/// use data_structures::BinarySearchTree;
///
/// let mut tree = BinarySearchTree::new();
/// tree.insert(4);
/// tree.insert(2);
/// tree.insert(6);
/// assert_eq!(tree.iter().copied().collect::<Vec<_>>(), [2, 4, 6]);
/// assert_eq!(tree.remove(&4), Some(4));
/// ```
#[derive(Clone, Debug)]
pub struct BinarySearchTree<T> {
    root: Option<Box<Node<T>>>,
    len: usize,
}

impl<T> BinarySearchTree<T> {
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

fn drop_nodes<T>(mut root: Option<Box<Node<T>>>) {
    while let Some(mut node) = root {
        if let Some(mut left) = node.left.take() {
            node.left = left.right.take();
            left.right = Some(node);
            root = Some(left);
        } else {
            root = node.right.take();
        }
    }
}

impl<T: Ord> BinarySearchTree<T> {
    pub(crate) fn get_by<Q: ?Sized>(
        &self,
        query: &Q,
        cmp: impl Fn(&Q, &T) -> std::cmp::Ordering,
    ) -> Option<&T> {
        let mut node = self.root.as_deref();
        while let Some(current) = node {
            match cmp(query, &current.value) {
                std::cmp::Ordering::Less => node = current.left.as_deref(),
                std::cmp::Ordering::Greater => node = current.right.as_deref(),
                std::cmp::Ordering::Equal => return Some(&current.value),
            }
        }
        None
    }

    pub(crate) fn get_mut_by<Q: ?Sized>(
        &mut self,
        query: &Q,
        cmp: impl Fn(&Q, &T) -> std::cmp::Ordering,
    ) -> Option<&mut T> {
        let mut node = self.root.as_deref_mut();
        while let Some(current) = node {
            match cmp(query, &current.value) {
                std::cmp::Ordering::Less => node = current.left.as_deref_mut(),
                std::cmp::Ordering::Greater => node = current.right.as_deref_mut(),
                std::cmp::Ordering::Equal => return Some(&mut current.value),
            }
        }
        None
    }

    pub(crate) fn remove_by<Q: ?Sized>(
        &mut self,
        query: &Q,
        cmp: impl Fn(&Q, &T) -> std::cmp::Ordering,
    ) -> Option<T> {
        let mut link = &mut self.root;
        loop {
            let ordering = match link.as_ref() {
                Some(node) => cmp(query, &node.value),
                None => return None,
            };
            match ordering {
                std::cmp::Ordering::Less => link = &mut link.as_mut().expect("node exists").left,
                std::cmp::Ordering::Greater => {
                    link = &mut link.as_mut().expect("node exists").right
                }
                std::cmp::Ordering::Equal => {
                    let mut removed = link.take().expect("matched node exists");
                    *link = match (removed.left.take(), removed.right.take()) {
                        (None, right) => right,
                        (left, None) => left,
                        (Some(left), Some(right)) => {
                            let mut right = Some(right);
                            let mut successor = take_min_iterative(&mut right);
                            successor.left = Some(left);
                            successor.right = right;
                            Some(successor)
                        }
                    };
                    self.len -= 1;
                    return Some(removed.value);
                }
            }
        }
    }

    /// Inserts `value`; returns `false` if an equal value is already present.
    pub fn insert(&mut self, value: T) -> bool {
        let mut link = &mut self.root;
        loop {
            match link {
                Some(node) => match value.cmp(&node.value) {
                    std::cmp::Ordering::Less => link = &mut node.left,
                    std::cmp::Ordering::Greater => link = &mut node.right,
                    std::cmp::Ordering::Equal => return false,
                },
                None => {
                    *link = Some(Box::new(Node {
                        value,
                        left: None,
                        right: None,
                    }));
                    self.len += 1;
                    return true;
                }
            }
        }
    }

    /// Returns whether an equal value is present.
    pub fn contains(&self, value: &T) -> bool {
        self.get(value).is_some()
    }

    /// Returns the stored value equal to `value`, if present.
    pub fn get(&self, value: &T) -> Option<&T> {
        let mut node = self.root.as_deref();
        while let Some(current) = node {
            match value.cmp(&current.value) {
                std::cmp::Ordering::Less => node = current.left.as_deref(),
                std::cmp::Ordering::Greater => node = current.right.as_deref(),
                std::cmp::Ordering::Equal => return Some(&current.value),
            }
        }
        None
    }

    /// Removes and returns the value equal to `value`, if present.
    pub fn remove(&mut self, value: &T) -> Option<T> {
        let mut link = &mut self.root;
        loop {
            let ordering = match link.as_ref() {
                Some(node) => value.cmp(&node.value),
                None => return None,
            };
            match ordering {
                std::cmp::Ordering::Less => {
                    link = &mut link.as_mut().expect("the link contains a node").left;
                }
                std::cmp::Ordering::Greater => {
                    link = &mut link.as_mut().expect("the link contains a node").right;
                }
                std::cmp::Ordering::Equal => {
                    let mut removed = link.take().expect("the matched node exists");
                    let replacement = match (removed.left.take(), removed.right.take()) {
                        (None, right) => right,
                        (left, None) => left,
                        (Some(left), Some(right)) => {
                            let mut right = Some(right);
                            let mut successor = take_min_iterative(&mut right);
                            successor.left = Some(left);
                            successor.right = right;
                            Some(successor)
                        }
                    };
                    *link = replacement;
                    self.len -= 1;
                    return Some(removed.value);
                }
            }
        }
    }
}

fn take_min_iterative<T>(root: &mut Option<Box<Node<T>>>) -> Box<Node<T>> {
    let mut link = root;
    while link.as_ref().is_some_and(|node| node.left.is_some()) {
        link = &mut link.as_mut().expect("the link contains a node").left;
    }
    let mut min = link.take().expect("the subtree is non-empty");
    *link = min.right.take();
    min
}

fn remove_node<T: Ord>(root: Option<Box<Node<T>>>, value: &T) -> (Option<Box<Node<T>>>, Option<T>) {
    let Some(mut node) = root else {
        return (None, None);
    };

    match value.cmp(&node.value) {
        std::cmp::Ordering::Less => {
            let (left, removed) = remove_node(node.left.take(), value);
            node.left = left;
            (Some(node), removed)
        }
        std::cmp::Ordering::Greater => {
            let (right, removed) = remove_node(node.right.take(), value);
            node.right = right;
            (Some(node), removed)
        }
        std::cmp::Ordering::Equal => {
            let Node { value, left, right } = *node;
            match (left, right) {
                (None, right) => (right, Some(value)),
                (left, None) => (left, Some(value)),
                (Some(left), Some(right)) => {
                    let (right, successor) = extract_min(right);
                    (
                        Some(Box::new(Node {
                            value: successor,
                            left: Some(left),
                            right,
                        })),
                        Some(value),
                    )
                }
            }
        }
    }
}

pub(super) fn extract_min<T>(mut node: Box<Node<T>>) -> (Option<Box<Node<T>>>, T) {
    match node.left.take() {
        None => (node.right.take(), node.value),
        Some(left) => {
            let (left, value) = extract_min(left);
            node.left = left;
            (Some(node), value)
        }
    }
}

impl<T> Default for BinarySearchTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> IntoIterator for &'a BinarySearchTree<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> IntoIterator for BinarySearchTree<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(mut self) -> Self::IntoIter {
        let root = self.root.take();
        let len = self.len;
        self.len = 0;
        IntoIter::new(root, len)
    }
}

impl<T: Ord> Extend<T> for BinarySearchTree<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.insert(value);
        }
    }
}

impl<T: Ord> FromIterator<T> for BinarySearchTree<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut tree = Self::new();
        tree.extend(iter);
        tree
    }
}

impl<T> Drop for BinarySearchTree<T> {
    fn drop(&mut self) {
        self.clear();
    }
}
