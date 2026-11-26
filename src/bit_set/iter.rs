/// Iterates over set indices in ascending order.
pub struct Iter<'a> {
    words: &'a [u64],
    word_index: usize,
    pending: u64,
    remaining: usize,
}

impl<'a> Iter<'a> {
    pub(super) fn new(words: &'a [u64], remaining: usize) -> Self {
        Self {
            words,
            word_index: 0,
            pending: 0,
            remaining,
        }
    }
}

impl Iterator for Iter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.pending != 0 {
                let bit = self.pending.trailing_zeros() as usize;
                self.pending &= self.pending - 1;
                self.remaining -= 1;
                return Some((self.word_index - 1) * u64::BITS as usize + bit);
            }
            let word = *self.words.get(self.word_index)?;
            self.word_index += 1;
            self.pending = word;
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl ExactSizeIterator for Iter<'_> {}
impl std::iter::FusedIterator for Iter<'_> {}
