//! Fallible aggregate operations used when combining values can fail.

use super::Monoid;

/// A monoid-like aggregate whose combine operation can return an error.
///
/// This is an operational contract for data structures; unlike [`Monoid`],
/// its operation is not necessarily total over the value type. Implementors
/// should keep the same identity and associativity laws wherever combinations
/// succeed. Types implementing [`Monoid`] receive an infallible implementation
/// automatically.
pub trait TryMonoid: Clone {
    /// The error returned by a failed combination.
    type Error;

    /// Returns the identity value.
    fn identity() -> Self;

    /// Combines two values or returns an error.
    fn try_combine(left: &Self, right: &Self) -> Result<Self, Self::Error>;
}

impl<T: Monoid + Clone> TryMonoid for T {
    type Error = std::convert::Infallible;

    fn identity() -> Self {
        T::identity()
    }

    fn try_combine(left: &Self, right: &Self) -> Result<Self, Self::Error> {
        Ok(T::combine(left, right))
    }
}
