use super::Node;
use std::collections::btree_map;
use std::iter::FusedIterator;

struct Frame<'a> {
    node: &'a Node,
    children: btree_map::Iter<'a, char, super::Edge>,
    yielded_word: bool,
    restore_len: usize,
}

impl<'a> Frame<'a> {
    fn new(node: &'a Node, restore_len: usize) -> Self {
        Self {
            node,
            children: node.children.iter(),
            yielded_word: false,
            restore_len,
        }
    }
}

/// An iterator over stored strings in lexicographic order.
pub struct Iter<'a> {
    stack: Vec<Frame<'a>>,
    current: String,
    remaining: usize,
}

impl<'a> Iter<'a> {
    pub(super) fn new(root: &'a Node, remaining: usize) -> Self {
        Self {
            stack: vec![Frame::new(root, 0)],
            current: String::new(),
            remaining,
        }
    }
}

impl Iterator for Iter<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let frame = self.stack.last_mut()?;
            if !frame.yielded_word {
                frame.yielded_word = true;
                if frame.node.is_word {
                    self.remaining -= 1;
                    return Some(self.current.clone());
                }
            }
            if let Some((_, edge)) = frame.children.next() {
                let restore_len = self.current.len();
                self.current.extend(edge.label.iter());
                self.stack.push(Frame::new(&edge.child, restore_len));
            } else {
                let finished = self.stack.pop().expect("frame exists");
                self.current.truncate(finished.restore_len);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl ExactSizeIterator for Iter<'_> {}
impl FusedIterator for Iter<'_> {}
