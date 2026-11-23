//! Linked data structures.
//!
//! Each list variant has its own type and API. Start with
//! [`SinglyLinkedList`], which owns its nodes and supports efficient insertion
//! at both ends.

pub mod doubly_linked;
pub mod singly_linked;

pub use doubly_linked::DoublyLinkedList;
pub use singly_linked::SinglyLinkedList;
