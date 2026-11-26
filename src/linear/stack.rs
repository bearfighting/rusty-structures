//! A last-in, first-out collection backed by [`Vector`].
//!
//! The most recently pushed value is at the top. `pop` and `peek` operate on
//! that value, and iterators visit values from top to bottom.

use crate::{TryPushError, Vector};
use std::iter::Rev;
use std::slice;

mod traits;

/// A last-in, first-out collection.
///
/// `Stack<T>` uses the project's [`Vector<T>`] for storage, so pushing is
/// amortized O(1), popping is O(1), and inspecting the top is O(1).
///
/// # Examples
///
/// ```
/// use data_structures::Stack;
///
/// let mut stack = Stack::new();
/// stack.push("first");
/// stack.push("last");
///
/// assert_eq!(stack.peek(), Some(&"last"));
/// assert_eq!(stack.pop(), Some("last"));
/// assert_eq!(stack.pop(), Some("first"));
/// assert_eq!(stack.pop(), None);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stack<T> {
    items: Vector<T>,
}

impl<T> Stack<T> {
    /// Creates an empty stack.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            items: Vector::new(),
        }
    }

    /// Creates an empty stack with room for at least `capacity` values.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vector::with_capacity(capacity),
        }
    }

    /// Creates an empty stack with room for at least `capacity` values.
    pub fn try_with_capacity(capacity: usize) -> Result<Self, crate::TryReserveError> {
        Ok(Self {
            items: Vector::try_with_capacity(capacity)?,
        })
    }

    /// Returns the number of values in the stack.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns whether the stack contains no values.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Removes all values from the stack.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Adds a value to the top of the stack.
    pub fn push(&mut self, value: T) {
        self.items.push(value);
    }

    /// Fallibly adds a value to the top, returning it if allocation fails.
    pub fn try_push(&mut self, value: T) -> Result<(), TryPushError<T>> {
        self.items.try_push(value)
    }

    /// Removes and returns the top value, or `None` if the stack is empty.
    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    /// Returns a shared reference to the top value, or `None` if empty.
    #[must_use]
    pub fn peek(&self) -> Option<&T> {
        self.items.last()
    }

    /// Returns a mutable reference to the top value, or `None` if empty.
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.items.last_mut()
    }

    /// Returns an iterator from the top value down to the bottom.
    pub fn iter(&self) -> Rev<slice::Iter<'_, T>> {
        self.items.iter().rev()
    }

    /// Returns a mutable iterator from the top value down to the bottom.
    pub fn iter_mut(&mut self) -> Rev<slice::IterMut<'_, T>> {
        self.items.iter_mut().rev()
    }
}
