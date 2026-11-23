//! A self-balancing binary search tree using AVL rotations.

mod iter;

pub use iter::Iter;

#[derive(Clone, Debug)]
struct Node<T> {
    value: T,
    height: usize,
    left: Option<Box<Node<T>>>,
    right: Option<Box<Node<T>>>,
}

/// A recursive AVL tree that stores unique values in sorted order.
///
/// Equal values are rejected by [`insert`](Self::insert). After each insertion
/// or removal, rotations restore the invariant that each node's child subtree
/// heights differ by at most one. Search, insertion, and removal take O(log n)
/// time; the height of an empty tree is zero.
///
/// # Examples
///
/// ```
/// use data_structures::AvlTree;
///
/// let mut tree = AvlTree::new();
/// tree.insert(3);
/// tree.insert(1);
/// tree.insert(2); // a left-right rotation
/// assert_eq!(tree.iter().copied().collect::<Vec<_>>(), [1, 2, 3]);
/// assert_eq!(tree.height(), 2);
/// ```
#[derive(Clone, Debug)]
pub struct AvlTree<T> {
    root: Option<Box<Node<T>>>,
    len: usize,
}

impl<T> AvlTree<T> {
    /// Creates an empty AVL tree.
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

    /// Returns the height of the tree, with an empty tree having height zero.
    #[must_use]
    pub fn height(&self) -> usize {
        height(&self.root)
    }

    /// Removes all values from the tree.
    pub fn clear(&mut self) {
        self.root = None;
        self.len = 0;
    }

    /// Returns an iterator over values in ascending order.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(self.root.as_deref(), self.len)
    }
}

impl<T: Ord> AvlTree<T> {
    /// Inserts `value`; returns `false` if an equal value is already present.
    pub fn insert(&mut self, value: T) -> bool {
        let (root, inserted) = insert_at(self.root.take(), value);
        self.root = root;
        if inserted {
            self.len += 1;
        }
        inserted
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
        let (root, removed) = remove_at(self.root.take(), value);
        self.root = root;
        if removed.is_some() {
            self.len -= 1;
        }
        removed
    }
}

fn height<T>(node: &Option<Box<Node<T>>>) -> usize {
    node.as_ref().map_or(0, |node| node.height)
}

fn update_height<T>(node: &mut Node<T>) {
    node.height = 1 + height(&node.left).max(height(&node.right));
}

fn insert_at<T: Ord>(root: Option<Box<Node<T>>>, value: T) -> (Option<Box<Node<T>>>, bool) {
    let Some(mut node) = root else {
        return (
            Some(Box::new(Node {
                value,
                height: 1,
                left: None,
                right: None,
            })),
            true,
        );
    };

    let inserted = match value.cmp(&node.value) {
        std::cmp::Ordering::Less => {
            let (left, inserted) = insert_at(node.left.take(), value);
            node.left = left;
            inserted
        }
        std::cmp::Ordering::Greater => {
            let (right, inserted) = insert_at(node.right.take(), value);
            node.right = right;
            inserted
        }
        std::cmp::Ordering::Equal => return (Some(node), false),
    };

    (Some(rebalance(node)), inserted)
}

fn remove_at<T: Ord>(root: Option<Box<Node<T>>>, value: &T) -> (Option<Box<Node<T>>>, Option<T>) {
    let Some(mut node) = root else {
        return (None, None);
    };

    let removed = match value.cmp(&node.value) {
        std::cmp::Ordering::Less => {
            let (left, removed) = remove_at(node.left.take(), value);
            node.left = left;
            removed
        }
        std::cmp::Ordering::Greater => {
            let (right, removed) = remove_at(node.right.take(), value);
            node.right = right;
            removed
        }
        std::cmp::Ordering::Equal => {
            let Node {
                value, left, right, ..
            } = *node;
            return match (left, right) {
                (None, right) => (right, Some(value)),
                (left, None) => (left, Some(value)),
                (Some(left), Some(right)) => {
                    let (right, successor) = take_min(right);
                    let root = Box::new(Node {
                        value: successor,
                        height: 1,
                        left: Some(left),
                        right,
                    });
                    (Some(rebalance(root)), Some(value))
                }
            };
        }
    };

    if removed.is_some() {
        (Some(rebalance(node)), removed)
    } else {
        (Some(node), None)
    }
}

fn take_min<T>(mut node: Box<Node<T>>) -> (Option<Box<Node<T>>>, T) {
    match node.left.take() {
        None => (node.right.take(), node.value),
        Some(left) => {
            let (left, value) = take_min(left);
            node.left = left;
            (Some(rebalance(node)), value)
        }
    }
}

fn rebalance<T>(mut node: Box<Node<T>>) -> Box<Node<T>> {
    update_height(&mut node);
    let left_height = height(&node.left);
    let right_height = height(&node.right);

    if left_height > right_height.saturating_add(1) {
        let left = node
            .left
            .as_ref()
            .expect("left-heavy node has a left child");
        if height(&left.left) < height(&left.right) {
            node.left = node.left.take().map(rotate_left);
        }
        rotate_right(node)
    } else if right_height > left_height.saturating_add(1) {
        let right = node
            .right
            .as_ref()
            .expect("right-heavy node has a right child");
        if height(&right.right) < height(&right.left) {
            node.right = node.right.take().map(rotate_right);
        }
        rotate_left(node)
    } else {
        node
    }
}

fn rotate_left<T>(mut root: Box<Node<T>>) -> Box<Node<T>> {
    let mut pivot = root
        .right
        .take()
        .expect("left rotation needs a right child");
    root.right = pivot.left.take();
    update_height(&mut root);
    pivot.left = Some(root);
    update_height(&mut pivot);
    pivot
}

fn rotate_right<T>(mut root: Box<Node<T>>) -> Box<Node<T>> {
    let mut pivot = root.left.take().expect("right rotation needs a left child");
    root.left = pivot.right.take();
    update_height(&mut root);
    pivot.right = Some(root);
    update_height(&mut pivot);
    pivot
}

impl<T> Default for AvlTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> IntoIterator for &'a AvlTree<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: Ord> Extend<T> for AvlTree<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.insert(value);
        }
    }
}

impl<T: Ord> FromIterator<T> for AvlTree<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut tree = Self::new();
        tree.extend(iter);
        tree
    }
}

#[cfg(test)]
mod tests {
    use super::{AvlTree, Node};

    fn check(node: &Option<Box<Node<i32>>>) -> usize {
        let Some(node) = node else {
            return 0;
        };
        let left = check(&node.left);
        let right = check(&node.right);
        assert!(left.abs_diff(right) <= 1, "AVL balance invariant violated");
        assert_eq!(
            node.height,
            1 + left.max(right),
            "stored height is incorrect"
        );
        left.max(right) + 1
    }

    #[test]
    fn rotations_restore_balance_after_insertions_and_deletions() {
        for values in [[3, 2, 1], [1, 2, 3], [3, 1, 2], [1, 3, 2]] {
            let mut tree = AvlTree::new();
            for value in values {
                tree.insert(value);
                check(&tree.root);
            }
        }

        let mut tree: AvlTree<_> = (0..128).collect();
        for value in (0..128).step_by(2) {
            tree.remove(&value);
            check(&tree.root);
        }
    }
}
