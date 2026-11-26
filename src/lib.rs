//! Readable, rigorous implementations of classic data structures for learning Rust.
//!
//! This crate is a teaching collection, not a replacement for Rust's standard
//! library. It favors implementations that make ownership, invariants, and
//! operation costs clear, while still validating important edge cases.
//!
//! Public modules are organized by domain, with related sequential containers
//! grouped under [`linear`] and disjoint-set/range-query structures under
//! [`range`]. Shared internals live in a private foundation layer, and composed
//! structures reuse concrete implementations beneath them. Node types and
//! storage layouts remain private.
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

pub mod algebra;
pub mod bit_set;
pub mod bloom_filter;
pub mod graph;
pub mod hash;
pub mod heap;
pub mod linear;
pub mod list;
pub mod lru_cache;
pub mod range;
pub mod skip_list;
pub mod tree;

mod foundation;

// Compatibility re-exports keep established module paths such as
// `data_structures::vector::Vector` available alongside the grouped domains.
pub use linear::{deque, fixed_capacity_queue, queue, stack, vector};
pub use range::{disjoint_set, fenwick, lazy_segment_tree, segment_tree, sparse_table};

pub use algebra::{Monoid, TryMonoid};
pub use bit_set::BitSet;
pub use bloom_filter::{BloomFilter, BloomFilterConfigError};
pub use deque::{Deque, TryDequePushError};
pub use disjoint_set::DisjointSet;
pub use fenwick::FenwickTree;
pub use fixed_capacity_queue::FixedCapacityQueue;
pub use graph::{
    AdjacencyMatrixGraph, Graph, NodeId, WeightedAdjacencyMatrixGraph, WeightedGraph, WeightedPath,
    connected_components, strongly_connected_components,
};
pub use hash::{HashMap, HashSet};
pub use heap::{BinaryHeap, DaryHeap, IndexPriorityQueue, PriorityQueue};
pub use lazy_segment_tree::{LazySegmentTree, LazySegmentTreeSpec};
pub use list::{DoublyLinkedList, SinglyLinkedList};
pub use lru_cache::LruCache;
pub use queue::{Queue, TryEnqueueError};
pub use segment_tree::{SegmentTree, SumOverflow};
pub use skip_list::{SkipListMap, SkipListSet};
pub use sparse_table::{IdempotentOp, Maximum, Minimum, SparseTable};
pub use stack::Stack;
pub use tree::{
    AvlTree, AvlTreeMap, BPlusTreeMap, BPlusTreeSet, BPlusTreeSnapshot, BPlusTreeSnapshotNode,
    BTreeMap, BTreeSet, BinarySearchTree, BinarySearchTreeMap, PersistentAvlTreeMap,
    PersistentAvlTreeMapIter, PersistentTrie, RadixTrie, RecursiveBinarySearchTree,
    RecursiveBinarySearchTreeMap, RedBlackTree, RedBlackTreeMap, TreeMap, Trie, TrieSnapshot,
    TrieSnapshotNode,
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
