//! Error types exposed by `Queue`.

/// Error returned by [`Queue::try_enqueue`](super::Queue::try_enqueue).
///
/// The rejected value can be recovered with `into_value`.
pub type TryEnqueueError<T> = crate::errors::TryValueError<T>;
