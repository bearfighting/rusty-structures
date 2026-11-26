//! Linear containers and their shared sequence storage.
//!
//! `Vector` is the concrete contiguous collection. `Stack`, `Queue`, and
//! `Deque` provide different access semantics over vector or ring-buffer
//! storage. Root-level module paths remain re-exported by the crate facade.

pub mod deque;
pub mod fixed_capacity_queue;
pub mod queue;
pub mod stack;
pub mod vector;

pub use fixed_capacity_queue::FixedCapacityQueue;
