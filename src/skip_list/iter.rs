use super::map::SkipListMap;

/// An iterator over skip-list key-value pairs in ascending key order.
pub struct Iter<'a, K, V> {
    pub(super) map: &'a SkipListMap<K, V>,
    pub(super) next: Option<usize>,
    pub(super) remaining: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.next?;
        let node = self.map.nodes[index]
            .as_ref()
            .expect("iterator points to a live node");
        self.next = node.forward[0];
        self.remaining -= 1;
        Some((&node.key, &node.value))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V> {}
