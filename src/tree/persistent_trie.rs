//! Immutable, structurally shared prefix tree.

use std::collections::BTreeMap;
use std::sync::Arc;

mod iter;

pub use iter::Iter;

#[derive(Clone, Debug, Default)]
struct Node {
    is_word: bool,
    children: BTreeMap<char, Arc<Node>>,
}

impl Node {
    fn is_empty(&self) -> bool {
        !self.is_word && self.children.is_empty()
    }
}

/// An immutable set of strings stored in a structurally shared trie.
///
/// Insertion and removal return a new version and a flag indicating whether
/// the set changed. Nodes on the modified path are copied; untouched subtrees
/// are shared with `Arc`. Strings are indexed by Unicode scalar values, and
/// iteration visits words in lexicographic order.
///
/// # Examples
///
/// ```
/// use data_structures::PersistentTrie;
///
/// let empty = PersistentTrie::new();
/// let (cats, inserted) = empty.insert("猫");
/// let (more, _) = cats.insert("猫咪");
/// assert!(inserted);
/// assert!(cats.contains("猫"));
/// assert!(!cats.contains("猫咪"));
/// assert!(more.contains("猫咪"));
/// ```
#[derive(Clone, Debug, Default)]
pub struct PersistentTrie {
    root: Arc<Node>,
    len: usize,
}

impl PersistentTrie {
    /// Creates an empty trie version.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of strings in this version.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns whether this version contains no strings.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns whether the exact string is present in this version.
    #[must_use]
    pub fn contains(&self, word: &str) -> bool {
        let mut node = self.root.as_ref();
        for character in word.chars() {
            let Some(child) = node.children.get(&character) else {
                return false;
            };
            node = child;
        }
        node.is_word
    }

    /// Returns whether at least one word starts with `prefix`.
    ///
    /// The empty prefix matches if this version contains at least one word.
    #[must_use]
    pub fn starts_with(&self, prefix: &str) -> bool {
        if self.is_empty() {
            return false;
        }
        let mut node = self.root.as_ref();
        for character in prefix.chars() {
            let Some(child) = node.children.get(&character) else {
                return false;
            };
            node = child;
        }
        true
    }

    /// Inserts `word` into a new version.
    ///
    /// Returns the new trie and `true` if the word was newly inserted. If the
    /// word already exists, returns a version sharing the same root and `false`.
    pub fn insert(&self, word: &str) -> (Self, bool) {
        let characters: Vec<_> = word.chars().collect();
        let (root, inserted) = insert_at(&self.root, &characters);
        let len = self.len + usize::from(inserted);
        (Self { root, len }, inserted)
    }

    /// Removes `word` from a new version.
    ///
    /// Returns the new trie and `true` if the word was present. If it was
    /// absent, the returned version shares the same root and `false`.
    pub fn remove(&self, word: &str) -> (Self, bool) {
        let characters: Vec<_> = word.chars().collect();
        let (root, removed) = remove_at(&self.root, &characters);
        let len = self.len - usize::from(removed);
        (Self { root, len }, removed)
    }

    /// Returns words in this version in lexicographic order.
    pub fn iter(&self) -> Iter<'_> {
        Iter::new(&self.root, self.len)
    }
}

fn insert_at(node: &Arc<Node>, remaining: &[char]) -> (Arc<Node>, bool) {
    if remaining.is_empty() {
        if node.is_word {
            return (Arc::clone(node), false);
        }
        let mut updated = (**node).clone();
        updated.is_word = true;
        return (Arc::new(updated), true);
    }

    let character = remaining[0];
    let child = node.children.get(&character).cloned().unwrap_or_default();
    let (updated_child, inserted) = insert_at(&child, &remaining[1..]);
    if !inserted {
        return (Arc::clone(node), false);
    }
    let mut updated = (**node).clone();
    updated.children.insert(character, updated_child);
    (Arc::new(updated), true)
}

fn remove_at(node: &Arc<Node>, remaining: &[char]) -> (Arc<Node>, bool) {
    if remaining.is_empty() {
        if !node.is_word {
            return (Arc::clone(node), false);
        }
        let mut updated = (**node).clone();
        updated.is_word = false;
        return (Arc::new(updated), true);
    }

    let character = remaining[0];
    let Some(child) = node.children.get(&character) else {
        return (Arc::clone(node), false);
    };
    let (updated_child, removed) = remove_at(child, &remaining[1..]);
    if !removed {
        return (Arc::clone(node), false);
    }

    let mut updated = (**node).clone();
    if updated_child.is_empty() {
        updated.children.remove(&character);
    } else {
        updated.children.insert(character, updated_child);
    }
    (Arc::new(updated), true)
}

impl<'a> IntoIterator for &'a PersistentTrie {
    type Item = String;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl FromIterator<String> for PersistentTrie {
    fn from_iter<I: IntoIterator<Item = String>>(words: I) -> Self {
        words
            .into_iter()
            .fold(Self::new(), |trie, word| trie.insert(&word).0)
    }
}

#[cfg(test)]
mod tests {
    use super::PersistentTrie;
    use std::sync::Arc;

    #[test]
    fn versions_remain_independent_after_insert_and_remove() {
        let empty = PersistentTrie::new();
        let (one_word, inserted) = empty.insert("catalog");
        assert!(inserted);
        let (two_words, inserted) = one_word.insert("cat");
        assert!(inserted);
        let (three_words, inserted) = two_words.insert("cater");
        assert!(inserted);

        let (removed, did_remove) = three_words.remove("cat");
        assert!(did_remove);
        assert!(empty.is_empty());
        assert_eq!(one_word.iter().collect::<Vec<_>>(), ["catalog"]);
        assert_eq!(two_words.iter().collect::<Vec<_>>(), ["cat", "catalog"]);
        assert_eq!(
            three_words.iter().collect::<Vec<_>>(),
            ["cat", "catalog", "cater"]
        );
        assert_eq!(removed.iter().collect::<Vec<_>>(), ["catalog", "cater"]);
    }

    #[test]
    fn shares_untouched_branches_and_reuses_root_for_no_op() {
        let (with_cat, _) = PersistentTrie::new().insert("cat");
        let (base, _) = with_cat.insert("dog");
        let (updated, _) = base.insert("car");
        assert!(Arc::ptr_eq(
            base.root.children.get(&'d').unwrap(),
            updated.root.children.get(&'d').unwrap()
        ));

        let (same, inserted) = base.insert("cat");
        assert!(!inserted);
        assert!(Arc::ptr_eq(&base.root, &same.root));
        let (same, removed) = base.remove("fox");
        assert!(!removed);
        assert!(Arc::ptr_eq(&base.root, &same.root));
    }

    #[test]
    fn supports_empty_unicode_prefix_queries_and_ordered_iteration() {
        let words = ["", "猫", "猫咪", "🐈", "🐈‍⬛"];
        let trie = words
            .into_iter()
            .fold(PersistentTrie::new(), |trie, word| trie.insert(word).0);
        assert!(trie.contains(""));
        assert!(trie.starts_with("🐈‍"));
        assert!(!trie.starts_with("犬"));
        assert_eq!(
            trie.iter().collect::<Vec<_>>(),
            ["", "猫", "猫咪", "🐈", "🐈‍⬛"]
        );
        let (without_empty, removed) = trie.remove("");
        assert!(removed);
        assert!(!without_empty.contains(""));
        assert!(trie.contains(""));
    }
}
