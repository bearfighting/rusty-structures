use super::{Node, NodeId};
use std::iter::FusedIterator;
use std::rc::Rc;

/// An in-order iterator that follows the B+ tree's linked leaves.
pub struct Iter<'a, K, V> {
    nodes: &'a [Option<Node<K, V>>],
    leaf: Option<NodeId>,
    next_entry: usize,
    remaining: usize,
}

impl<'a, K, V> Iter<'a, K, V> {
    pub(super) fn new(
        nodes: &'a [Option<Node<K, V>>],
        first_leaf: Option<NodeId>,
        remaining: usize,
    ) -> Self {
        Self {
            nodes,
            leaf: first_leaf,
            next_entry: 0,
            remaining,
        }
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        while self.remaining > 0 {
            let id = self.leaf?;
            let Node::Leaf { entries, next } = self.nodes[id].as_ref().expect("leaf ID is live")
            else {
                unreachable!("leaf chain points to a leaf node");
            };
            if let Some((key, value)) = entries.get(self.next_entry) {
                self.next_entry += 1;
                self.remaining -= 1;
                return Some((Rc::as_ref(key), value));
            }
            self.leaf = *next;
            self.next_entry = 0;
        }
        None
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}
impl<K, V> ExactSizeIterator for Iter<'_, K, V> {}
impl<K, V> FusedIterator for Iter<'_, K, V> {}

/// An iterator over keys in ascending order.
pub struct Keys<'a, K, V>(Iter<'a, K, V>);
impl<'a, K, V> Keys<'a, K, V> {
    pub(super) fn new(iter: Iter<'a, K, V>) -> Self {
        Self(iter)
    }
}
impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(key, _)| key)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
impl<K, V> ExactSizeIterator for Keys<'_, K, V> {}
impl<K, V> FusedIterator for Keys<'_, K, V> {}
