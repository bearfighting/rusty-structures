//! A disjoint-set union structure for tracking connected components.
//!
//! `DisjointSet` starts with each element in its own set. [`find`](DisjointSet::find)
//! uses path compression, while [`union`](DisjointSet::union) uses union by size.
//! Both operations take amortized O(α(n)) time, where α is the inverse Ackermann
//! function, and the structure uses O(n) space.

/// Tracks a partition of the integer range `0..len` into disjoint sets.
///
/// The root of each element is stored in `parent`; only roots have meaningful
/// sizes in `size`. `components` equals the number of roots. Elements are
/// identified by their index, so operations with an index outside `0..len`
/// return `None`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisjointSet {
    parent: Vec<usize>,
    size: Vec<usize>,
    components: usize,
}

impl DisjointSet {
    /// Creates `len` singleton sets, one for each index in `0..len`.
    pub fn new(len: usize) -> Self {
        Self {
            parent: (0..len).collect(),
            size: vec![1; len],
            components: len,
        }
    }

    /// Returns the number of elements in the structure.
    pub fn len(&self) -> usize {
        self.parent.len()
    }

    /// Returns whether the structure contains no elements.
    pub fn is_empty(&self) -> bool {
        self.parent.is_empty()
    }

    /// Returns the number of disjoint sets currently represented.
    pub fn component_count(&self) -> usize {
        self.components
    }

    /// Returns the representative for `element`, compressing its path.
    ///
    /// Returns `None` when `element` is outside `0..len`. The representative is
    /// an implementation detail and may change after a later union.
    pub fn find(&mut self, element: usize) -> Option<usize> {
        if element >= self.len() {
            return None;
        }
        let mut root = element;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        let mut current = element;
        while self.parent[current] != current {
            let next = self.parent[current];
            self.parent[current] = root;
            current = next;
        }
        Some(root)
    }

    /// Merges the sets containing `first` and `second`.
    ///
    /// Returns `Some(true)` if two sets were merged, `Some(false)` if they were
    /// already in the same set, or `None` if either index is out of range.
    pub fn union(&mut self, first: usize, second: usize) -> Option<bool> {
        let first_root = self.find(first)?;
        let second_root = self.find(second)?;
        if first_root == second_root {
            return Some(false);
        }

        let (large, small) = if self.size[first_root] >= self.size[second_root] {
            (first_root, second_root)
        } else {
            (second_root, first_root)
        };
        self.parent[small] = large;
        self.size[large] += self.size[small];
        self.components -= 1;
        Some(true)
    }

    /// Returns whether `first` and `second` belong to the same set.
    ///
    /// Returns `None` if either index is outside `0..len`.
    pub fn connected(&mut self, first: usize, second: usize) -> Option<bool> {
        let first_root = self.find(first)?;
        let second_root = self.find(second)?;
        Some(first_root == second_root)
    }
}

impl From<usize> for DisjointSet {
    fn from(len: usize) -> Self {
        Self::new(len)
    }
}
