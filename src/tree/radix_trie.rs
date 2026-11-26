//! A path-compressed prefix tree over Unicode scalar values.

use std::collections::BTreeMap;

mod iter;

pub use iter::Iter;

#[derive(Clone, Debug, Default)]
pub(super) struct Node {
    pub(super) is_word: bool,
    pub(super) children: BTreeMap<char, Edge>,
}

#[derive(Clone, Debug)]
pub(super) struct Edge {
    pub(super) label: Vec<char>,
    pub(super) child: Node,
}

impl Node {
    fn is_empty(&self) -> bool {
        !self.is_word && self.children.is_empty()
    }
}

/// A set of unique strings stored in a path-compressed trie.
///
/// Each edge stores a non-empty run of Unicode scalar values. Insertion splits
/// an edge at the first differing character; removal merges non-word nodes
/// that have only one child. Lookup and updates take O(k log d) time, where `k`
/// is the number of characters and `d` is the maximum branching factor.
/// Iteration visits words in lexicographic order. Empty strings are supported.
///
/// # Examples
///
/// ```
/// use data_structures::RadixTrie;
///
/// let mut words = RadixTrie::new();
/// words.insert("compression");
/// words.insert("compressor");
/// assert!(words.contains("compression"));
/// assert!(words.starts_with("compres"));
/// assert_eq!(words.iter().collect::<Vec<_>>(), ["compression", "compressor"]);
/// ```
#[derive(Clone, Debug, Default)]
pub struct RadixTrie {
    root: Node,
    len: usize,
}

impl RadixTrie {
    /// Creates an empty radix trie.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of stored strings.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the trie contains no strings.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of stored edges, useful for inspecting compression.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        fn count(node: &Node) -> usize {
            node.children
                .values()
                .map(|edge| 1 + count(&edge.child))
                .sum()
        }
        count(&self.root)
    }

    /// Removes all stored strings.
    pub fn clear(&mut self) {
        self.root = Node::default();
        self.len = 0;
    }

    /// Inserts `word`, returning `false` if it was already present.
    pub fn insert(&mut self, word: &str) -> bool {
        let characters: Vec<_> = word.chars().collect();
        if insert_at(&mut self.root, &characters) {
            self.len += 1;
            true
        } else {
            false
        }
    }

    /// Returns whether the exact string is stored.
    #[must_use]
    pub fn contains(&self, word: &str) -> bool {
        let characters: Vec<_> = word.chars().collect();
        contains_at(&self.root, &characters)
    }

    /// Returns whether at least one stored string starts with `prefix`.
    ///
    /// A prefix may end in the middle of a compressed edge. The empty prefix
    /// matches if the trie contains at least one word.
    #[must_use]
    pub fn starts_with(&self, prefix: &str) -> bool {
        if self.is_empty() {
            return false;
        }
        let characters: Vec<_> = prefix.chars().collect();
        starts_with_at(&self.root, &characters)
    }

    /// Removes `word` and recompresses any newly redundant paths.
    pub fn remove(&mut self, word: &str) -> bool {
        let characters: Vec<_> = word.chars().collect();
        if remove_at(&mut self.root, &characters) {
            self.len -= 1;
            true
        } else {
            false
        }
    }

    /// Returns stored strings in lexicographic order.
    pub fn iter(&self) -> Iter<'_> {
        Iter::new(&self.root, self.len)
    }
}

fn insert_at(node: &mut Node, remaining: &[char]) -> bool {
    if remaining.is_empty() {
        if node.is_word {
            return false;
        }
        node.is_word = true;
        return true;
    }

    let first = remaining[0];
    let Some(edge) = node.children.get_mut(&first) else {
        let leaf = Node {
            is_word: true,
            ..Node::default()
        };
        node.children.insert(
            first,
            Edge {
                label: remaining.to_vec(),
                child: leaf,
            },
        );
        return true;
    };

    let shared = common_prefix_len(&edge.label, remaining);
    if shared == edge.label.len() {
        return insert_at(&mut edge.child, &remaining[shared..]);
    }

    let old_suffix = edge.label[shared..].to_vec();
    let old_child = std::mem::take(&mut edge.child);
    let mut branch = Node::default();
    branch.children.insert(
        old_suffix[0],
        Edge {
            label: old_suffix,
            child: old_child,
        },
    );
    if shared == remaining.len() {
        branch.is_word = true;
    } else {
        let new_suffix = remaining[shared..].to_vec();
        let new_leaf = Node {
            is_word: true,
            ..Node::default()
        };
        branch.children.insert(
            new_suffix[0],
            Edge {
                label: new_suffix,
                child: new_leaf,
            },
        );
    }
    edge.label.truncate(shared);
    edge.child = branch;
    true
}

fn contains_at(node: &Node, remaining: &[char]) -> bool {
    if remaining.is_empty() {
        return node.is_word;
    }
    let Some(edge) = node.children.get(&remaining[0]) else {
        return false;
    };
    if remaining.len() < edge.label.len() || remaining[..edge.label.len()] != edge.label {
        return false;
    }
    contains_at(&edge.child, &remaining[edge.label.len()..])
}

fn starts_with_at(node: &Node, remaining: &[char]) -> bool {
    if remaining.is_empty() {
        return true;
    }
    let Some(edge) = node.children.get(&remaining[0]) else {
        return false;
    };
    let shared = common_prefix_len(&edge.label, remaining);
    if shared == remaining.len() {
        true
    } else if shared == edge.label.len() {
        starts_with_at(&edge.child, &remaining[shared..])
    } else {
        false
    }
}

fn remove_at(node: &mut Node, remaining: &[char]) -> bool {
    if remaining.is_empty() {
        if !node.is_word {
            return false;
        }
        node.is_word = false;
        return true;
    }

    let first = remaining[0];
    let Some(edge) = node.children.get_mut(&first) else {
        return false;
    };
    if remaining.len() < edge.label.len() || remaining[..edge.label.len()] != edge.label {
        return false;
    }
    if !remove_at(&mut edge.child, &remaining[edge.label.len()..]) {
        return false;
    }

    if edge.child.is_empty() {
        node.children.remove(&first);
    } else if !edge.child.is_word && edge.child.children.len() == 1 {
        let only_child = edge.child.children.pop_first().expect("one child exists").1;
        edge.label.extend(only_child.label);
        edge.child = only_child.child;
    }
    true
}

fn common_prefix_len(left: &[char], right: &[char]) -> usize {
    left.iter()
        .zip(right)
        .take_while(|(left, right)| left == right)
        .count()
}

impl<'a> IntoIterator for &'a RadixTrie {
    type Item = String;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Extend<String> for RadixTrie {
    fn extend<I: IntoIterator<Item = String>>(&mut self, words: I) {
        for word in words {
            self.insert(&word);
        }
    }
}

impl FromIterator<String> for RadixTrie {
    fn from_iter<I: IntoIterator<Item = String>>(words: I) -> Self {
        let mut trie = Self::new();
        trie.extend(words);
        trie
    }
}

#[cfg(test)]
mod tests {
    use super::RadixTrie;
    use crate::Trie;

    #[test]
    fn splits_edges_and_matches_prefixes_inside_edges() {
        let mut trie = RadixTrie::new();
        assert!(trie.insert("compression"));
        assert!(trie.insert("compressor"));
        assert!(trie.insert("comp"));
        assert!(!trie.insert("compression"));
        assert!(trie.contains("compression"));
        assert!(!trie.contains("compress"));
        assert!(trie.starts_with("compress"));
        assert_eq!(
            trie.iter().collect::<Vec<_>>(),
            ["comp", "compression", "compressor"]
        );
        assert!(trie.edge_count() < "compression".chars().count() + "compressor".chars().count());
    }

    #[test]
    fn removes_and_recompresses_without_affecting_prefix_words() {
        let mut trie = RadixTrie::new();
        trie.insert("catalog");
        trie.insert("cat");
        trie.insert("cater");
        assert!(trie.remove("cat"));
        assert!(!trie.contains("cat"));
        assert!(trie.starts_with("cat"));
        assert!(trie.remove("cater"));
        assert!(trie.contains("catalog"));
        assert_eq!(trie.iter().collect::<Vec<_>>(), ["catalog"]);
        assert!(!trie.remove("missing"));
    }

    #[test]
    fn handles_empty_unicode_and_agrees_with_uncompressed_trie() {
        let mut compressed = RadixTrie::new();
        let mut ordinary = Trie::new();
        for word in ["", "猫", "猫咪", "🐈", "🐈‍⬛", "猫科"] {
            assert_eq!(compressed.insert(word), ordinary.insert(word));
        }
        for prefix in ["", "猫", "猫咪", "🐈", "🐈‍", "犬"] {
            assert_eq!(compressed.starts_with(prefix), ordinary.starts_with(prefix));
        }
        for word in ["", "猫咪", "🐈‍⬛", "犬"] {
            assert_eq!(compressed.remove(word), ordinary.remove(word));
            assert_eq!(
                compressed.iter().collect::<Vec<_>>(),
                ordinary.iter().collect::<Vec<_>>()
            );
        }
    }
}
