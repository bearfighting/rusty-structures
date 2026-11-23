//! A left-leaning red-black tree.

mod iter;

pub use iter::Iter;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Color {
    Red,
    Black,
}

#[derive(Clone, Debug)]
struct Node<T> {
    value: T,
    color: Color,
    left: Option<Box<Node<T>>>,
    right: Option<Box<Node<T>>>,
}

/// A set-like left-leaning red-black tree with unique values.
///
/// Red links encode 3-nodes in a 2–3 tree and always lean left. Every path
/// from the root to an empty child has the same number of black links. These
/// invariants keep the height O(log n), so search, insertion, and removal take
/// O(log n) time. Equal values are rejected; iteration is in ascending order.
///
/// # Examples
///
/// ```
/// use data_structures::RedBlackTree;
///
/// let mut tree = RedBlackTree::new();
/// tree.insert(3);
/// tree.insert(1);
/// tree.insert(2);
/// assert_eq!(tree.iter().copied().collect::<Vec<_>>(), [1, 2, 3]);
/// assert_eq!(tree.remove(&2), Some(2));
/// ```
#[derive(Clone, Debug)]
pub struct RedBlackTree<T> {
    root: Option<Box<Node<T>>>,
    len: usize,
}

impl<T> RedBlackTree<T> {
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
        self.root = None;
        self.len = 0;
    }

    /// Returns an iterator over values in ascending order.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(self.root.as_deref(), self.len)
    }
}

impl<T: Ord> RedBlackTree<T> {
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
        let mut root = self.root.take()?;
        if !is_red(&root.left) && !is_red(&root.right) {
            root.color = Color::Red;
        }
        let (root, removed) = remove_at_by(root, query, &cmp);
        self.root = root;
        if let Some(root) = self.root.as_mut() {
            root.color = Color::Black;
        }
        if removed.is_some() {
            self.len -= 1;
        }
        removed
    }

    /// Inserts `value`; returns `false` if an equal value is already present.
    pub fn insert(&mut self, value: T) -> bool {
        let (root, inserted) = insert_at(self.root.take(), value);
        self.root = root;
        if let Some(root) = self.root.as_mut() {
            root.color = Color::Black;
        }
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
        let mut root = self.root.take()?;
        if !is_red(&root.left) && !is_red(&root.right) {
            root.color = Color::Red;
        }
        let (root, removed) = remove_at(root, value);
        self.root = root;
        if let Some(root) = self.root.as_mut() {
            root.color = Color::Black;
        }
        if removed.is_some() {
            self.len -= 1;
        }
        removed
    }
}

fn is_red<T>(link: &Option<Box<Node<T>>>) -> bool {
    link.as_ref().is_some_and(|node| node.color == Color::Red)
}

fn insert_at<T: Ord>(root: Option<Box<Node<T>>>, value: T) -> (Option<Box<Node<T>>>, bool) {
    let Some(mut node) = root else {
        return (
            Some(Box::new(Node {
                value,
                color: Color::Red,
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
    (Some(fix_up(node)), inserted)
}

fn remove_at<T: Ord>(mut node: Box<Node<T>>, value: &T) -> (Option<Box<Node<T>>>, Option<T>) {
    let mut removed = None;
    if value < &node.value {
        if node.left.is_some() {
            if !is_red(&node.left) && !left_is_red(&node.left) {
                node = move_red_left(node);
            }
            let left = node.left.take().expect("left branch exists");
            let (left, value) = remove_at(left, value);
            node.left = left;
            removed = value;
        }
    } else {
        if is_red(&node.left) {
            node = rotate_right(node);
        }
        match value.cmp(&node.value) {
            std::cmp::Ordering::Equal if node.right.is_none() => {
                return (None, Some(node.value));
            }
            _ => {}
        }

        if node.right.is_some() {
            if !is_red(&node.right) && !left_is_red(&node.right) {
                node = move_red_right(node);
            }
            if value == &node.value {
                let right = node.right.take().expect("right branch exists");
                let (right, successor) = take_min(right);
                node.right = right;
                removed = Some(std::mem::replace(&mut node.value, successor));
            } else {
                let right = node.right.take().expect("right branch exists");
                let (right, value) = remove_at(right, value);
                node.right = right;
                removed = value;
            }
        }
    }
    (Some(fix_up(node)), removed)
}

fn remove_at_by<T, Q: ?Sized>(
    mut node: Box<Node<T>>,
    query: &Q,
    cmp: &impl Fn(&Q, &T) -> std::cmp::Ordering,
) -> (Option<Box<Node<T>>>, Option<T>) {
    let mut removed = None;
    if cmp(query, &node.value) == std::cmp::Ordering::Less {
        if node.left.is_some() {
            if !is_red(&node.left) && !left_is_red(&node.left) {
                node = move_red_left(node);
            }
            let left = node.left.take().expect("left branch exists");
            let (left, value) = remove_at_by(left, query, cmp);
            node.left = left;
            removed = value;
        }
    } else {
        if is_red(&node.left) {
            node = rotate_right(node);
        }
        if cmp(query, &node.value) == std::cmp::Ordering::Equal && node.right.is_none() {
            return (None, Some(node.value));
        }
        if node.right.is_some() {
            if !is_red(&node.right) && !left_is_red(&node.right) {
                node = move_red_right(node);
            }
            if cmp(query, &node.value) == std::cmp::Ordering::Equal {
                let right = node.right.take().expect("right branch exists");
                let (right, successor) = take_min(right);
                node.right = right;
                removed = Some(std::mem::replace(&mut node.value, successor));
            } else {
                let right = node.right.take().expect("right branch exists");
                let (right, value) = remove_at_by(right, query, cmp);
                node.right = right;
                removed = value;
            }
        }
    }
    (Some(fix_up(node)), removed)
}

fn take_min<T>(mut node: Box<Node<T>>) -> (Option<Box<Node<T>>>, T) {
    if node.left.is_none() {
        return (node.right.take(), node.value);
    }
    if !is_red(&node.left) && !left_is_red(&node.left) {
        node = move_red_left(node);
    }
    let left = node.left.take().expect("left branch exists");
    let (left, value) = take_min(left);
    node.left = left;
    (Some(fix_up(node)), value)
}

fn left_is_red<T>(link: &Option<Box<Node<T>>>) -> bool {
    link.as_ref().is_some_and(|node| is_red(&node.left))
}

fn fix_up<T>(mut node: Box<Node<T>>) -> Box<Node<T>> {
    if is_red(&node.right) && !is_red(&node.left) {
        node = rotate_left(node);
    }
    if is_red(&node.left) && left_is_red(&node.left) {
        node = rotate_right(node);
    }
    if is_red(&node.left) && is_red(&node.right) {
        flip_colors(&mut node);
    }
    node
}

fn move_red_left<T>(mut node: Box<Node<T>>) -> Box<Node<T>> {
    flip_colors(&mut node);
    if left_is_red(&node.right) {
        let right = node.right.take().expect("right branch exists");
        node.right = Some(rotate_right(right));
        node = rotate_left(node);
        flip_colors(&mut node);
    }
    node
}

fn move_red_right<T>(mut node: Box<Node<T>>) -> Box<Node<T>> {
    flip_colors(&mut node);
    if left_is_red(&node.left) {
        node = rotate_right(node);
        flip_colors(&mut node);
    }
    node
}

fn rotate_left<T>(mut node: Box<Node<T>>) -> Box<Node<T>> {
    let mut pivot = node
        .right
        .take()
        .expect("left rotation needs a right child");
    node.right = pivot.left.take();
    pivot.color = node.color;
    node.color = Color::Red;
    pivot.left = Some(node);
    pivot
}

fn rotate_right<T>(mut node: Box<Node<T>>) -> Box<Node<T>> {
    let mut pivot = node.left.take().expect("right rotation needs a left child");
    node.left = pivot.right.take();
    pivot.color = node.color;
    node.color = Color::Red;
    pivot.right = Some(node);
    pivot
}

fn flip_colors<T>(node: &mut Node<T>) {
    node.color = opposite(node.color);
    if let Some(left) = node.left.as_mut() {
        left.color = opposite(left.color);
    }
    if let Some(right) = node.right.as_mut() {
        right.color = opposite(right.color);
    }
}

fn opposite(color: Color) -> Color {
    match color {
        Color::Red => Color::Black,
        Color::Black => Color::Red,
    }
}

impl<T> Default for RedBlackTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> IntoIterator for &'a RedBlackTree<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: Ord> Extend<T> for RedBlackTree<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.insert(value);
        }
    }
}

impl<T: Ord> FromIterator<T> for RedBlackTree<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut tree = Self::new();
        tree.extend(iter);
        tree
    }
}

#[cfg(test)]
mod tests {
    use super::{Color, Node, RedBlackTree, is_red};

    fn validate(node: &Option<Box<Node<i32>>>) -> usize {
        let Some(node) = node else {
            return 1;
        };
        assert!(!is_red(&node.right), "red links must lean left");
        if node.color == Color::Red {
            assert!(!is_red(&node.left), "red links cannot be consecutive");
        }
        if let Some(left) = node.left.as_ref() {
            assert!(left.value < node.value, "left keys must be smaller");
        }
        if let Some(right) = node.right.as_ref() {
            assert!(right.value > node.value, "right keys must be larger");
        }
        let left_black_height = validate(&node.left);
        let right_black_height = validate(&node.right);
        assert_eq!(
            left_black_height, right_black_height,
            "black heights differ"
        );
        left_black_height + usize::from(node.color == Color::Black)
    }

    fn check(tree: &RedBlackTree<i32>) {
        if let Some(root) = tree.root.as_ref() {
            assert_eq!(root.color, Color::Black, "root must be black");
        }
        validate(&tree.root);
        assert_eq!(tree.iter().count(), tree.len);
        let values: Vec<_> = tree.iter().copied().collect();
        assert!(values.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn insert_and_delete_preserve_red_black_invariants() {
        let mut tree = RedBlackTree::new();
        for value in (0..256).map(|value| (value * 73) % 256) {
            assert!(tree.insert(value));
            check(&tree);
        }
        for value in (0..256).step_by(3) {
            assert_eq!(tree.remove(&value), Some(value));
            check(&tree);
        }
        for value in 0..256 {
            if value % 3 != 0 {
                assert_eq!(tree.remove(&value), Some(value));
                check(&tree);
            }
        }
        assert!(tree.is_empty());
    }

    #[test]
    fn duplicate_insertions_and_missing_removals_keep_invariants() {
        let mut tree = RedBlackTree::new();
        let mut expected = std::collections::BTreeSet::new();
        for value in [8, 3, 10, 1, 6, 14, 4, 7, 13, 6, 3] {
            assert_eq!(tree.insert(value), expected.insert(value));
            check(&tree);
        }
        for value in [99, 3, 10, 99, 6, 8, 1, 14, 4, 7, 13, 0] {
            assert_eq!(tree.remove(&value), expected.take(&value));
            check(&tree);
        }
    }
}
