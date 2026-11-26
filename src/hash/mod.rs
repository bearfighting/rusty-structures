//! Hash-based collections.
//!
//! [`HashMap`] stores key-value pairs. [`HashSet`] reuses it to store unique
//! values without duplicating hashing or collision handling.

pub mod map;
pub mod set;

pub use map::HashMap;
pub use set::HashSet;
