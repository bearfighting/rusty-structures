//! Readable, idiomatic implementations of classic data structures.
//!
//! This crate is organized by data-structure domain. Public modules expose
//! stable, intentional APIs; implementation details such as node types and
//! storage layouts remain private unless users need them to work with a type.
//!
//! # Design principles
//!
//! - Express ownership directly: borrowed access returns references, while
//!   removal and consuming iteration return owned values.
//! - Use [`Option`] for absent values and [`Result`] for recoverable errors.
//! - Implement standard traits when their semantics fit the data structure.
//! - Prefer safe Rust and small, well-justified abstractions.
//! - Document invariants, operation complexity, iteration order, and relevant
//!   limitations for each public data structure.
//!
//! The crate starts as a single library. The [`mod@vector`] module is the first
//! implementation; further modules will be added as roadmap stages are
//! completed. Algorithms may be separated from data structure implementations
//! when a real API or dependency boundary emerges.
//!
//! # Roadmap
//!
//! See the repository's `ROADMAP.md` for planned stages and `PROJECT_PLAN.md`
//! for project-wide design and quality conventions.

pub mod deque;
pub mod hash;
pub mod list;
pub mod queue;
pub mod stack;
pub mod vector;

mod errors;
mod ring_buffer;

pub use deque::{Deque, TryDequePushError};
pub use hash::{HashMap, HashSet};
pub use list::{DoublyLinkedList, SinglyLinkedList};
pub use queue::{Queue, TryEnqueueError};
pub use stack::Stack;
pub use vector::{TryPushError, TryReserveError, Vector};

/// Creates a [`Vector`] containing the given elements or repeated value.
///
/// The element-list form accepts zero or more expressions. The repeat form
/// clones the value `count` times, like `std::vec!`.
///
/// # Examples
///
/// ```
/// use data_structures::{vector, Vector};
///
/// let values: Vector<_> = vector![1, 2, 3];
/// let repeated = vector!["rust"; 3];
/// assert_eq!(&*values, &[1, 2, 3]);
/// assert_eq!(&*repeated, &["rust"; 3]);
/// ```
#[macro_export]
macro_rules! vector {
    ($value:expr; $count:expr $(,)?) => {
        $crate::Vector::from_elem($value, $count)
    };
    ($($element:expr),* $(,)?) => {
        $crate::Vector::from([$($element),*])
    };
}
