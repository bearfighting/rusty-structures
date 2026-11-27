use super::Node;
use std::collections::btree_map;
use std::iter::FusedIterator;

struct Frame<'a> {
    node: &'a Node,
    children: btree_map::Iter<'a, char, Node>,
    yielded_word: bool,
}

impl<'a> Frame<'a> {
    fn new(node: &'a Node) -> Self {
        Self {
            node,
            children: node.children.iter(),
            yielded_word: false,
        }
    }
}

/// An iterator over stored strings in lexicographic order.
pub struct Iter<'a> {
    stack: Vec<Frame<'a>>,
    current: String,
    remaining: usize,
}

/// An iterator over words below a selected Trie prefix.
pub struct PrefixIter<'a> {
    stack: Vec<Frame<'a>>,
    current: String,
}

impl<'a> Iter<'a> {
    pub(super) fn new(root: &'a Node, remaining: usize) -> Self {
        Self {
            stack: vec![Frame::new(root)],
            current: String::new(),
            remaining,
        }
    }
}

impl<'a> PrefixIter<'a> {
    pub(super) fn new(node: &'a Node, prefix: &str) -> Self {
        Self {
            stack: vec![Frame::new(node)],
            current: prefix.to_owned(),
        }
    }

    pub(super) fn empty() -> Self {
        Self {
            stack: Vec::new(),
            current: String::new(),
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

            if let Some((&character, child)) = frame.children.next() {
                self.current.push(character);
                self.stack.push(Frame::new(child));
            } else {
                self.stack.pop();
                if !self.stack.is_empty() {
                    self.current.pop();
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl ExactSizeIterator for Iter<'_> {}
impl FusedIterator for Iter<'_> {}

impl Iterator for PrefixIter<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let frame = self.stack.last_mut()?;
            if !frame.yielded_word {
                frame.yielded_word = true;
                if frame.node.is_word {
                    return Some(self.current.clone());
                }
            }

            if let Some((&character, child)) = frame.children.next() {
                self.current.push(character);
                self.stack.push(Frame::new(child));
            } else {
                self.stack.pop();
                if !self.stack.is_empty() {
                    self.current.pop();
                }
            }
        }
    }
}

impl FusedIterator for PrefixIter<'_> {}
