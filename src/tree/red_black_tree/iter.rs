use super::Node;
use std::iter::FusedIterator;

/// An in-order iterator over a red-black tree.
pub struct Iter<'a, T> {
    stack: Vec<&'a Node<T>>,
    remaining: usize,
}

impl<'a, T> Iter<'a, T> {
    pub(super) fn new(root: Option<&'a Node<T>>, remaining: usize) -> Self {
        let mut iter = Self {
            stack: Vec::new(),
            remaining,
        };
        iter.push_left(root);
        iter
    }

    fn push_left(&mut self, mut node: Option<&'a Node<T>>) {
        while let Some(current) = node {
            self.stack.push(current);
            node = current.left.as_deref();
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        self.push_left(node.right.as_deref());
        self.remaining -= 1;
        Some(&node.value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {}
impl<T> FusedIterator for Iter<'_, T> {}
