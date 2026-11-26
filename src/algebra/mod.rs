//! Reusable algebraic contracts used by data structures.
//!
//! Keep this module independent of concrete containers. Add further algebraic
//! structures when a real implementation needs them.

mod monoid;
mod try_monoid;

pub use monoid::Monoid;
pub use try_monoid::TryMonoid;
