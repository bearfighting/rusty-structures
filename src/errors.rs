//! Shared allocation errors for growable collections.

use std::error;
use std::fmt;

/// The reason a fallible allocation or reservation could not be completed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TryReserveError {
    /// The requested element count or memory layout exceeded the limit.
    CapacityOverflow,
    /// The allocator could not provide the requested memory.
    AllocError,
}

impl fmt::Display for TryReserveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapacityOverflow => f.write_str("collection capacity overflow"),
            Self::AllocError => f.write_str("memory allocation failed"),
        }
    }
}

impl error::Error for TryReserveError {}

/// A shared error wrapper that preserves a value rejected by a fallible insert.
pub struct TryValueError<T> {
    value: T,
    error: TryReserveError,
}

impl<T> TryValueError<T> {
    pub(crate) const fn new(value: T, error: TryReserveError) -> Self {
        Self { value, error }
    }

    /// Returns the reason insertion failed.
    #[must_use]
    pub const fn error(&self) -> TryReserveError {
        self.error
    }

    /// Returns the value that was not inserted.
    #[must_use]
    pub fn into_value(self) -> T {
        self.value
    }

    /// Returns both the uninserted value and the allocation error.
    #[must_use]
    pub fn into_parts(self) -> (T, TryReserveError) {
        (self.value, self.error)
    }
}

impl<T: fmt::Debug> fmt::Debug for TryValueError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TryValueError")
            .field("value", &self.value)
            .field("error", &self.error)
            .finish()
    }
}

impl<T> fmt::Display for TryValueError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.error, f)
    }
}

impl<T: fmt::Debug> error::Error for TryValueError<T> {}
