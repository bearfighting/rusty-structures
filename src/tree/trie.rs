//! A prefix tree for strings.

use std::collections::BTreeMap;

mod iter;

pub use iter::{Iter, PrefixIter};

#[derive(Clone, Debug, Default)]
struct Node {
    is_word: bool,
    children: BTreeMap<char, Node>,
}

/// An owned read-only copy of a trie, intended for diagnostics and visualizers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrieSnapshot {
    /// The root node, whose prefix is the empty string.
    pub root: TrieSnapshotNode,
    /// Number of complete words stored in the trie.
    pub len: usize,
}

/// A node in an owned [`TrieSnapshot`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrieSnapshotNode {
    /// The complete prefix represented by this node.
    pub prefix: String,
    /// Whether this prefix is itself a stored word.
    pub is_word: bool,
    /// Child edges in character order.
    pub children: Vec<(char, TrieSnapshotNode)>,
}

impl Node {
    fn is_empty(&self) -> bool {
        !self.is_word && self.children.is_empty()
    }
}

/// A set of unique strings stored by their Unicode scalar values.
///
/// Each character in a string follows one edge. Child edges use a `BTreeMap`,
/// so iteration is in lexicographic order. Looking up a string of `k` Unicode
/// scalar values takes O(k log d) time, where `d` is the maximum number of
/// children at a node. Empty strings are supported. Removing a string prunes
/// nodes that are no longer needed by another string.
///
/// # Examples
///
/// ```
/// use data_structures::Trie;
///
/// let mut words = Trie::new();
/// words.insert("猫");
/// words.insert("猫咪");
/// assert!(words.starts_with("猫"));
/// assert!(!words.contains("猫科"));
/// assert_eq!(words.iter().collect::<Vec<_>>(), ["猫", "猫咪"]);
/// ```
#[derive(Clone, Debug, Default)]
pub struct Trie {
    root: Node,
    len: usize,
}

impl Trie {
    /// Creates an empty trie.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            root: Node {
                is_word: false,
                children: BTreeMap::new(),
            },
            len: 0,
        }
    }

    /// Returns the number of stored strings.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the trie contains no strings.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Removes all stored strings.
    pub fn clear(&mut self) {
        self.root = Node::default();
        self.len = 0;
    }

    /// Inserts a string; returns `false` if it was already present.
    pub fn insert(&mut self, word: &str) -> bool {
        let mut node = &mut self.root;
        for character in word.chars() {
            node = node.children.entry(character).or_default();
        }
        if node.is_word {
            return false;
        }
        node.is_word = true;
        self.len += 1;
        true
    }

    /// Returns whether the exact string is stored.
    #[must_use]
    pub fn contains(&self, word: &str) -> bool {
        self.node_at(word).is_some_and(|node| node.is_word)
    }

    /// Returns whether at least one stored string starts with `prefix`.
    ///
    /// The empty prefix matches when the trie contains at least one string.
    #[must_use]
    pub fn starts_with(&self, prefix: &str) -> bool {
        !self.is_empty() && self.node_at(prefix).is_some()
    }

    /// Removes a string and prunes any nodes no longer needed by other strings.
    ///
    /// The operation takes O(k log d) time and O(k) temporary space, where `k`
    /// is the number of characters and `d` is the maximum node degree.
    pub fn remove(&mut self, word: &str) -> bool {
        let characters: Vec<_> = word.chars().collect();
        let (removed, _) = remove_at(&mut self.root, &characters, 0);
        if removed {
            self.len -= 1;
        }
        removed
    }

    /// Returns the node reached by following all characters in `prefix`.
    fn node_at(&self, prefix: &str) -> Option<&Node> {
        prefix
            .chars()
            .try_fold(&self.root, |node, character| node.children.get(&character))
    }

    /// Returns stored strings in lexicographic order.
    pub fn iter(&self) -> Iter<'_> {
        Iter::new(&self.root, self.len)
    }

    /// Iterates over stored words that start with `prefix`, in lexicographic order.
    ///
    /// The iterator starts at the prefix node instead of scanning unrelated
    /// branches. Locating the prefix takes O(k log d), where `k` is its number
    /// of Unicode scalar values and `d` is the maximum node degree. Traversal
    /// then visits only the matching subtree. The empty prefix iterates all words.
    pub fn words_with_prefix(&self, prefix: &str) -> PrefixIter<'_> {
        self.node_at(prefix)
            .map_or_else(PrefixIter::empty, |node| PrefixIter::new(node, prefix))
    }

    /// Copies the node layout for diagnostics and visualizers.
    #[must_use]
    pub fn visualization_snapshot(&self) -> TrieSnapshot {
        fn copy(node: &Node, prefix: String) -> TrieSnapshotNode {
            let children = node
                .children
                .iter()
                .map(|(character, child)| {
                    let mut child_prefix = prefix.clone();
                    child_prefix.push(*character);
                    (*character, copy(child, child_prefix))
                })
                .collect();
            TrieSnapshotNode {
                prefix,
                is_word: node.is_word,
                children,
            }
        }
        TrieSnapshot {
            root: copy(&self.root, String::new()),
            len: self.len,
        }
    }
}

fn remove_at(node: &mut Node, characters: &[char], index: usize) -> (bool, bool) {
    let removed = if index == characters.len() {
        if !node.is_word {
            false
        } else {
            node.is_word = false;
            true
        }
    } else {
        let character = characters[index];
        let Some(child) = node.children.get_mut(&character) else {
            return (false, false);
        };
        let (removed, prune_child) = remove_at(child, characters, index + 1);
        if prune_child {
            node.children.remove(&character);
        }
        removed
    };

    (removed, node.is_empty())
}

impl<'a> IntoIterator for &'a Trie {
    type Item = String;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Extend<String> for Trie {
    fn extend<I: IntoIterator<Item = String>>(&mut self, words: I) {
        for word in words {
            self.insert(&word);
        }
    }
}

impl FromIterator<String> for Trie {
    fn from_iter<I: IntoIterator<Item = String>>(words: I) -> Self {
        let mut trie = Self::new();
        trie.extend(words);
        trie
    }
}
