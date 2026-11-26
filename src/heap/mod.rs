//! Heap-based priority structures.
//!
//! `BinaryHeap` and `DaryHeap` expose max-heap semantics. `PriorityQueue`
//! provides an item/priority API, while `IndexPriorityQueue` supports updates
//! by stable integer index.

mod binary_heap;
mod dary_heap;
mod index_priority_queue;
mod priority_queue;

pub use binary_heap::BinaryHeap;
pub use dary_heap::DaryHeap;
pub use index_priority_queue::IndexPriorityQueue;
pub use priority_queue::PriorityQueue;
