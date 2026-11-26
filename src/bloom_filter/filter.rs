//! Bloom filter storage and operations.

use super::BloomFilterConfigError;
use std::borrow::Borrow;
use std::hash::{BuildHasher, Hash, RandomState};
use std::marker::PhantomData;

/// A space-efficient probabilistic set-membership filter.
///
/// A query can return a false positive, but an inserted value is never reported
/// absent. The filter does not store values and does not support removal. The
/// bit count is approximately `-n ln(p) / (ln 2)^2`, and the hash count is
/// approximately `(m / n) ln 2`, where `n` is the expected item count, `p` is
/// the requested false-positive probability, and `m` is the bit count.
///
/// # Examples
///
/// ```
/// use data_structures::BloomFilter;
///
/// let mut seen = BloomFilter::<String>::new(1_000, 0.01)?;
/// seen.insert("rust".to_owned());
/// assert!(seen.might_contain("rust"));
/// assert!(!seen.might_contain("go"));
/// # Ok::<(), data_structures::BloomFilterConfigError>(())
/// ```
#[derive(Clone, Debug)]
pub struct BloomFilter<T, S = RandomState> {
    bits: Vec<u64>,
    bit_count: usize,
    hash_count: usize,
    inserted_count: usize,
    hash_builder: S,
    marker: PhantomData<fn(&T)>,
}

impl<T> BloomFilter<T, RandomState> {
    /// Creates a filter for `expected_items` with the requested false-positive probability.
    pub fn new(
        expected_items: usize,
        false_positive_probability: f64,
    ) -> Result<Self, BloomFilterConfigError> {
        Self::with_hasher(
            expected_items,
            false_positive_probability,
            RandomState::new(),
        )
    }
}

impl<T, S: BuildHasher> BloomFilter<T, S> {
    /// Creates a filter using a caller-provided hash builder.
    pub fn with_hasher(
        expected_items: usize,
        false_positive_probability: f64,
        hash_builder: S,
    ) -> Result<Self, BloomFilterConfigError> {
        if expected_items == 0 {
            return Err(BloomFilterConfigError::EmptyCapacity);
        }
        if !false_positive_probability.is_finite()
            || false_positive_probability <= 0.0
            || false_positive_probability >= 1.0
        {
            return Err(BloomFilterConfigError::InvalidFalsePositiveRate);
        }

        let n = expected_items as f64;
        let ln_2 = std::f64::consts::LN_2;
        let requested_bits = (-(n * false_positive_probability.ln()) / (ln_2 * ln_2)).ceil();
        if !requested_bits.is_finite() || requested_bits > usize::MAX as f64 {
            return Err(BloomFilterConfigError::CapacityOverflow);
        }
        let bit_count = (requested_bits as usize).max(1);
        let words = bit_count
            .checked_add(u64::BITS as usize - 1)
            .ok_or(BloomFilterConfigError::CapacityOverflow)?
            / u64::BITS as usize;
        let hash_count = ((bit_count as f64 / n) * ln_2).round().max(1.0) as usize;

        let mut bits = Vec::new();
        bits.try_reserve_exact(words)
            .map_err(|_| BloomFilterConfigError::AllocationFailed)?;
        bits.resize(words, 0);
        Ok(Self {
            bits,
            bit_count,
            hash_count,
            inserted_count: 0,
            hash_builder,
            marker: PhantomData,
        })
    }

    /// Returns the number of bits allocated for the filter.
    #[must_use]
    pub fn bit_count(&self) -> usize {
        self.bit_count
    }

    /// Returns the number of bit positions checked per operation.
    #[must_use]
    pub fn hash_count(&self) -> usize {
        self.hash_count
    }

    /// Returns the number of calls to [`insert`](Self::insert), including duplicates.
    #[must_use]
    pub fn inserted_count(&self) -> usize {
        self.inserted_count
    }

    /// Estimates the current false-positive probability from the occupancy model.
    #[must_use]
    pub fn estimated_false_positive_probability(&self) -> f64 {
        let set_probability = 1.0
            - (-(self.hash_count as f64 * self.inserted_count as f64) / self.bit_count as f64)
                .exp();
        set_probability.powf(self.hash_count as f64)
    }

    /// Clears all bits and resets the insertion count.
    pub fn clear(&mut self) {
        self.bits.fill(0);
        self.inserted_count = 0;
    }

    fn hash_pair<Q: Hash + ?Sized>(&self, value: &Q) -> (u64, u64)
    where
        T: Borrow<Q>,
    {
        let first = self.hash_builder.hash_one(value);
        (first, mix64(first ^ 0x9e37_79b9_7f4a_7c15) | 1)
    }

    fn indexes(
        bit_count: usize,
        hash_count: usize,
        first: u64,
        second: u64,
    ) -> impl Iterator<Item = usize> {
        let modulus = bit_count as u64;
        (0..hash_count).map(move |index| {
            first
                .wrapping_add((index as u64).wrapping_mul(second))
                .wrapping_rem(modulus) as usize
        })
    }
}

impl<T: Hash, S: BuildHasher> BloomFilter<T, S> {
    /// Inserts `value` into the filter.
    pub fn insert(&mut self, value: T) {
        let (first, second) = self.hash_pair(&value);
        for index in Self::indexes(self.bit_count, self.hash_count, first, second) {
            self.bits[index / u64::BITS as usize] |= 1 << (index % u64::BITS as usize);
        }
        self.inserted_count = self.inserted_count.saturating_add(1);
    }

    /// Returns whether `value` may have been inserted.
    ///
    /// `false` means the value is definitely absent. `true` means it may be
    /// present and can be a false positive.
    #[must_use]
    pub fn might_contain<Q>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + ?Sized,
    {
        let (first, second) = self.hash_pair(value);
        Self::indexes(self.bit_count, self.hash_count, first, second).all(|index| {
            self.bits[index / u64::BITS as usize] & (1 << (index % u64::BITS as usize)) != 0
        })
    }
}

fn mix64(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::{BloomFilter, BloomFilterConfigError};

    #[test]
    fn inserted_values_are_always_reported_as_possible_members() {
        let mut filter = BloomFilter::<String>::new(100, 0.01).unwrap();
        let values = (0..100)
            .map(|index| format!("value-{index}"))
            .collect::<Vec<_>>();
        for value in &values {
            filter.insert(value.clone());
        }
        for value in &values {
            assert!(filter.might_contain(value));
        }
        assert_eq!(filter.inserted_count(), 100);
        assert!(filter.estimated_false_positive_probability() < 0.02);
    }

    #[test]
    fn validates_configuration_and_clear_resets_state() {
        assert_eq!(
            BloomFilter::<u8>::new(0, 0.01).unwrap_err(),
            BloomFilterConfigError::EmptyCapacity
        );
        assert_eq!(
            BloomFilter::<u8>::new(10, 1.0).unwrap_err(),
            BloomFilterConfigError::InvalidFalsePositiveRate
        );

        let mut filter = BloomFilter::<u8>::new(5, 0.05).unwrap();
        filter.insert(7);
        assert!(filter.might_contain(&7));
        filter.clear();
        assert_eq!(filter.inserted_count(), 0);
        assert!(!filter.might_contain(&7));
    }
}
