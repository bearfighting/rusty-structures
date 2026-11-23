//! Error types exposed by `Vector`.

pub use crate::errors::TryReserveError;

/// Error returned by [`Vector::try_push`](super::Vector::try_push).
///
/// The rejected value can be recovered with `into_value`.
pub type TryPushError<T> = crate::errors::TryValueError<T>;
