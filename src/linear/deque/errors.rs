//! Error types exposed by `Deque`.

/// Error returned when a fallible deque push cannot reserve memory.
///
/// The rejected value can be recovered with `into_value`.
pub type TryDequePushError<T> = crate::foundation::errors::TryValueError<T>;
