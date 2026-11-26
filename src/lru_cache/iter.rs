use super::cache::LruCache;

/// Iterates over cache entries from most to least recently used.
pub struct Iter<'a, K, V, S = std::collections::hash_map::RandomState> {
    pub(super) cache: &'a LruCache<K, V, S>,
    pub(super) next: Option<usize>,
    pub(super) remaining: usize,
}

impl<'a, K, V, S> Iterator for Iter<'a, K, V, S> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.next?;
        let node = self.cache.nodes[index]
            .as_ref()
            .expect("iterator points to a live node");
        self.next = node.next;
        self.remaining -= 1;
        Some((&node.key, &node.value))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<K, V, S> ExactSizeIterator for Iter<'_, K, V, S> {}
