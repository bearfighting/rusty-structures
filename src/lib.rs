//! Readable, rigorous implementations of classic data structures for learning Rust.
//!
//! This crate is a teaching collection, not a replacement for Rust's standard
//! library. It favors implementations that make ownership, invariants, and
//! operation costs clear, while still validating important edge cases.
//!
//! Public modules are organized by data-structure domain. Shared internals live
//! in a private foundation layer, and composed structures reuse the concrete
//! implementations beneath them. Node types and storage layouts remain private.
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
//! The crate is a single library organized by data-structure domain and layered
//! from public APIs through adapters to concrete storage and shared foundation.
//! Algorithms may be separated from structures when a real API or dependency
//! boundary emerges.
//!
//! # Roadmap
//!
//! See the repository's `ROADMAP.md` for planned stages and `PROJECT_PLAN.md`
//! for project-wide design and quality conventions.

pub mod deque;
pub mod hash;
pub mod heap;
pub mod list;
pub mod queue;
pub mod stack;
pub mod tree;
pub mod vector;

mod foundation;

pub use deque::{Deque, TryDequePushError};
pub use hash::{HashMap, HashSet};
pub use heap::{BinaryHeap, PriorityQueue};
pub use list::{DoublyLinkedList, SinglyLinkedList};
pub use queue::{Queue, TryEnqueueError};
pub use stack::Stack;
pub use tree::{
    AvlTree, AvlTreeMap, BPlusTreeMap, BPlusTreeSet, BTreeMap, BTreeSet, BinarySearchTree,
    BinarySearchTreeMap, RecursiveBinarySearchTree, RecursiveBinarySearchTreeMap, RedBlackTree,
    RedBlackTreeMap, TreeMap, Trie,
};
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
