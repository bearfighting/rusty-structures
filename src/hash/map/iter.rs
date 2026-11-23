//! Iteration over hash map entries.

use super::HashMap;
use std::hash::BuildHasher;

/// An iterator over the entries in a [`HashMap`].
pub struct Iter<'a, K, V, S> {
    map: &'a HashMap<K, V, S>,
    bucket_index: usize,
    entry_index: usize,
    remaining: usize,
}

impl<'a, K, V, S: BuildHasher> Iter<'a, K, V, S> {
    pub(super) fn new(map: &'a HashMap<K, V, S>) -> Self {
        Self {
            map,
            bucket_index: 0,
            entry_index: 0,
            remaining: map.len(),
        }
    }
}

impl<'a, K, V, S: BuildHasher> Iterator for Iter<'a, K, V, S> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while self.bucket_index < self.map.buckets.len() {
            let bucket = &self.map.buckets[self.bucket_index];
            if let Some((key, value)) = bucket.get(self.entry_index) {
                self.entry_index += 1;
                self.remaining -= 1;
                return Some((key, value));
            }
            self.bucket_index += 1;
            self.entry_index = 0;
        }
        None
    }
}

impl<K, V, S: BuildHasher> ExactSizeIterator for Iter<'_, K, V, S> {}
impl<K, V, S: BuildHasher> std::iter::FusedIterator for Iter<'_, K, V, S> {}

impl<'a, K, V, S: BuildHasher> IntoIterator for &'a HashMap<K, V, S> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V, S>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An owning iterator over key-value pairs in arbitrary order.
pub struct IntoIter<K, V> {
    buckets: crate::vector::IntoIter<crate::Vector<(K, V)>>,
    current_bucket: Option<crate::vector::IntoIter<(K, V)>>,
    remaining: usize,
}

impl<K, V> IntoIter<K, V> {
    pub(super) fn new<S>(map: HashMap<K, V, S>) -> Self {
        Self {
            buckets: map.buckets.into_iter(),
            current_bucket: None,
            remaining: map.len,
        }
    }
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(bucket) = self.current_bucket.as_mut()
                && let Some(entry) = bucket.next()
            {
                self.remaining -= 1;
                return Some(entry);
            }

            self.current_bucket = Some(self.buckets.next()?.into_iter());
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {}
impl<K, V> std::iter::FusedIterator for IntoIter<K, V> {}

impl<K, V, S> IntoIterator for HashMap<K, V, S> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}
