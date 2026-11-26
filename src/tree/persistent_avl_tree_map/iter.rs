use super::Node;
use std::iter::FusedIterator;

/// An in-order iterator over a persistent AVL map version.
pub struct Iter<'a, K, V> {
    stack: Vec<&'a Node<K, V>>,
    remaining: usize,
}

impl<'a, K, V> Iter<'a, K, V> {
    pub(super) fn new(root: Option<&'a Node<K, V>>, remaining: usize) -> Self {
        let mut iter = Self {
            stack: Vec::new(),
            remaining,
        };
        iter.push_left(root);
        iter
    }

    fn push_left(&mut self, mut node: Option<&'a Node<K, V>>) {
        while let Some(current) = node {
            self.stack.push(current);
            node = current.left.as_deref();
        }
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        self.push_left(node.right.as_deref());
        self.remaining -= 1;
        Some((&node.key, &node.value))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V> {}
impl<K, V> FusedIterator for Iter<'_, K, V> {}
