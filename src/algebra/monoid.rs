//! Total associative operations with an identity element.

/// A type with an associative binary operation and a two-sided identity.
///
/// Implementors must ensure `combine` is associative and `identity` is a
/// two-sided identity. These algebraic laws cannot be checked by the type
/// system.
pub trait Monoid {
    /// Returns the identity element for `combine`.
    fn identity() -> Self;

    /// Combines two values. This operation must be associative and total.
    fn combine(left: &Self, right: &Self) -> Self;
}
