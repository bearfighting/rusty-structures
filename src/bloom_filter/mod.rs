//! Probabilistic set-membership filter.

mod errors;
mod filter;

pub use errors::BloomFilterConfigError;
pub use filter::BloomFilter;
