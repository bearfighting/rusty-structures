use std::fmt;

/// Configuration errors returned when creating a [`super::BloomFilter`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BloomFilterConfigError {
    /// Expected item count must be greater than zero.
    EmptyCapacity,
    /// False-positive probability must be finite and strictly between zero and one.
    InvalidFalsePositiveRate,
    /// The requested filter size cannot be represented by `usize`.
    CapacityOverflow,
    /// The requested bit array could not be allocated.
    AllocationFailed,
}

impl fmt::Display for BloomFilterConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCapacity => formatter.write_str("expected item count must be nonzero"),
            Self::InvalidFalsePositiveRate => {
                formatter.write_str("false-positive probability must be between zero and one")
            }
            Self::CapacityOverflow => formatter.write_str("requested filter capacity overflows"),
            Self::AllocationFailed => formatter.write_str("could not allocate Bloom filter bits"),
        }
    }
}

impl std::error::Error for BloomFilterConfigError {}
