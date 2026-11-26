//! Least-recently-used cache combining a hash index and recency list.

mod cache;
mod iter;

pub use cache::LruCache;
pub use iter::Iter;
