//! Disjoint-set and range-query structures.
//!
//! These structures share a problem domain, but their storage and algorithms
//! are independent: `FenwickTree` and `SegmentTree` handle range aggregates,
//! `LazySegmentTree` adds deferred range updates, and `DisjointSet` tracks
//! partitions of an indexed set.

pub mod disjoint_set;
pub mod fenwick;
pub mod lazy_segment_tree;
pub mod segment_tree;
pub mod sparse_table;

pub use disjoint_set::DisjointSet;
pub use fenwick::FenwickTree;
pub use lazy_segment_tree::{LazySegmentTree, LazySegmentTreeSpec};
pub use segment_tree::{SegmentTree, SumOverflow};
pub use sparse_table::{IdempotentOp, Maximum, Minimum, SparseTable};
