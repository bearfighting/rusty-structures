//! Growable bitmap-backed set.

use super::Iter;

/// A growable set of non-negative integer indices stored as packed bits.
///
/// Membership, insertion, and removal take O(1) time, except insertion beyond
/// the current allocation, which grows the bit vector. Set operations take
/// O(w) time for `w` machine words. Iteration is ascending and takes O(w + n)
/// time for `n` present values. A `BitSet` uses one bit per index up to its
/// highest set index, so it is intended for dense, bounded integer domains.
///
/// # Examples
///
/// ```
/// use data_structures::BitSet;
///
/// let mut set = BitSet::new();
/// set.insert(2);
/// set.insert(65);
/// assert!(set.contains(65));
/// assert_eq!(set.iter().collect::<Vec<_>>(), [2, 65]);
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BitSet {
    words: Vec<u64>,
    len: usize,
}

impl BitSet {
    /// Creates an empty set without allocating.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            words: Vec::new(),
            len: 0,
        }
    }

    /// Creates an empty set with storage reserved for at least `bit_capacity` indices.
    #[must_use]
    pub fn with_capacity(bit_capacity: usize) -> Self {
        let word_capacity = bit_capacity.div_ceil(u64::BITS as usize);
        Self {
            words: Vec::with_capacity(word_capacity),
            len: 0,
        }
    }

    /// Returns the number of indices present in the set.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the set is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of indices that fit in the currently allocated words.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.words.capacity().saturating_mul(u64::BITS as usize)
    }

    /// Reserves storage for at least `bit_capacity` indices without changing membership.
    pub fn reserve(&mut self, bit_capacity: usize) {
        let needed_words = bit_capacity.div_ceil(u64::BITS as usize);
        self.words
            .reserve(needed_words.saturating_sub(self.words.len()));
    }

    /// Returns whether `index` is present.
    #[must_use]
    pub fn contains(&self, index: usize) -> bool {
        let word = index / u64::BITS as usize;
        let mask = 1u64 << (index % u64::BITS as usize);
        self.words.get(word).is_some_and(|bits| bits & mask != 0)
    }

    /// Inserts `index`, returning whether it was newly added.
    pub fn insert(&mut self, index: usize) -> bool {
        let word_index = index / u64::BITS as usize;
        if word_index >= self.words.len() {
            self.words.resize(word_index + 1, 0);
        }
        let mask = 1u64 << (index % u64::BITS as usize);
        let word = &mut self.words[word_index];
        let was_absent = *word & mask == 0;
        if was_absent {
            *word |= mask;
            self.len += 1;
        }
        was_absent
    }

    /// Removes `index`, returning whether it was present.
    pub fn remove(&mut self, index: usize) -> bool {
        let word_index = index / u64::BITS as usize;
        let Some(word) = self.words.get_mut(word_index) else {
            return false;
        };
        let mask = 1u64 << (index % u64::BITS as usize);
        let was_present = *word & mask != 0;
        if was_present {
            *word &= !mask;
            self.len -= 1;
        }
        was_present
    }

    /// Removes every index while retaining the allocated words.
    pub fn clear(&mut self) {
        self.words.fill(0);
        self.len = 0;
    }

    /// Adds every index present in `other` to this set.
    pub fn union_with(&mut self, other: &Self) {
        if self.words.len() < other.words.len() {
            self.words.resize(other.words.len(), 0);
        }
        for (word, &other_word) in self.words.iter_mut().zip(&other.words) {
            let added = other_word & !*word;
            self.len += added.count_ones() as usize;
            *word |= other_word;
        }
    }

    /// Retains only indices that are present in both sets.
    pub fn intersect_with(&mut self, other: &Self) {
        for (index, word) in self.words.iter_mut().enumerate() {
            *word &= other.words.get(index).copied().unwrap_or(0);
        }
        self.len = self
            .words
            .iter()
            .map(|word| word.count_ones() as usize)
            .sum();
    }

    /// Removes every index that is present in `other`.
    pub fn difference_with(&mut self, other: &Self) {
        for (index, word) in self.words.iter_mut().enumerate() {
            *word &= !other.words.get(index).copied().unwrap_or(0);
        }
        self.len = self
            .words
            .iter()
            .map(|word| word.count_ones() as usize)
            .sum();
    }

    /// Returns whether every index in this set is also present in `other`.
    #[must_use]
    pub fn is_subset(&self, other: &Self) -> bool {
        self.words
            .iter()
            .enumerate()
            .all(|(index, word)| word & !other.words.get(index).copied().unwrap_or(0) == 0)
    }

    /// Returns whether the sets have no indices in common.
    #[must_use]
    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.words
            .iter()
            .zip(&other.words)
            .all(|(left, right)| left & right == 0)
    }

    /// Iterates over present indices in ascending order.
    pub fn iter(&self) -> Iter<'_> {
        Iter::new(&self.words, self.len)
    }
}

impl Extend<usize> for BitSet {
    fn extend<I: IntoIterator<Item = usize>>(&mut self, indices: I) {
        for index in indices {
            self.insert(index);
        }
    }
}

impl FromIterator<usize> for BitSet {
    fn from_iter<I: IntoIterator<Item = usize>>(indices: I) -> Self {
        let mut set = Self::new();
        set.extend(indices);
        set
    }
}

impl<'a> IntoIterator for &'a BitSet {
    type Item = usize;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::BitSet;
    use std::collections::BTreeSet;

    #[test]
    fn handles_word_boundaries_duplicates_and_sorted_iteration() {
        let mut set = BitSet::new();
        for value in [65, 0, 63, 64, 127, 64] {
            set.insert(value);
        }
        assert_eq!(set.len(), 5);
        assert!(!set.insert(64));
        assert!(set.contains(127));
        assert!(!set.contains(128));
        assert_eq!(set.iter().collect::<Vec<_>>(), [0, 63, 64, 65, 127]);
        assert!(set.remove(64));
        assert!(!set.remove(64));
        assert_eq!(set.iter().collect::<Vec<_>>(), [0, 63, 65, 127]);
    }

    #[test]
    fn supports_set_algebra_and_capacity() {
        let left: BitSet = [1, 2, 64, 100].into_iter().collect();
        let right: BitSet = [2, 3, 64].into_iter().collect();
        let mut union = left.clone();
        union.union_with(&right);
        assert_eq!(union.iter().collect::<Vec<_>>(), [1, 2, 3, 64, 100]);
        let mut intersection = left.clone();
        intersection.intersect_with(&right);
        assert_eq!(intersection.iter().collect::<Vec<_>>(), [2, 64]);
        let mut difference = left.clone();
        difference.difference_with(&right);
        assert_eq!(difference.iter().collect::<Vec<_>>(), [1, 100]);
        assert!(intersection.is_subset(&left));
        assert!(difference.is_disjoint(&right));

        let reserved = BitSet::with_capacity(130);
        assert!(reserved.capacity() >= 130);
    }

    #[test]
    fn agrees_with_ordered_set_for_varied_indices() {
        let mut bits = BitSet::new();
        let mut ordered = BTreeSet::new();
        let mut state = 5u64;
        for _ in 0..300 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let value = (state % 512) as usize;
            if state & 1 == 0 {
                assert_eq!(bits.insert(value), ordered.insert(value));
            } else {
                assert_eq!(bits.remove(value), ordered.remove(&value));
            }
            assert_eq!(
                bits.iter().collect::<Vec<_>>(),
                ordered.iter().copied().collect::<Vec<_>>()
            );
        }
    }
}
