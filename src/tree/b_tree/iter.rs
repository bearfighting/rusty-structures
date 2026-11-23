use super::Node;
use std::iter::FusedIterator;

struct Frame<'a, K, V> {
    node: &'a Node<K, V>,
    next_slot: usize,
}

impl<'a, K, V> Frame<'a, K, V> {
    fn new(node: &'a Node<K, V>) -> Self {
        Self { node, next_slot: 0 }
    }
}

/// An in-order iterator over B-tree entries.
pub struct Iter<'a, K, V> {
    stack: Vec<Frame<'a, K, V>>,
    remaining: usize,
}

impl<'a, K, V> Iter<'a, K, V> {
    pub(super) fn new(root: Option<&'a Node<K, V>>, remaining: usize) -> Self {
        Self {
            stack: root.into_iter().map(Frame::new).collect(),
            remaining,
        }
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let frame = self.stack.last_mut()?;
            let slot_count = frame.node.entries.len() * 2 + 1;
            if frame.next_slot >= slot_count {
                self.stack.pop();
                continue;
            }
            let slot = frame.next_slot;
            frame.next_slot += 1;
            if slot % 2 == 0 {
                let child_index = slot / 2;
                if let Some(child) = frame.node.children.get(child_index) {
                    self.stack.push(Frame::new(child));
                }
            } else {
                let entry = &frame.node.entries[slot / 2];
                self.remaining -= 1;
                return Some((&entry.0, &entry.1));
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V> {}
impl<K, V> FusedIterator for Iter<'_, K, V> {}

/// An iterator over B-tree keys.
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
