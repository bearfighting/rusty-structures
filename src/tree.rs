//! Ordered tree structures.

pub mod avl_tree;
pub mod b_tree;
pub mod binary_search_tree;
pub mod map;
pub mod red_black_tree;
pub mod trie;

pub use avl_tree::AvlTree;
pub use b_tree::{BTreeMap, BTreeSet};
pub use binary_search_tree::{BinarySearchTree, RecursiveBinarySearchTree};
pub use map::{
    AvlTreeMap, BinarySearchTreeMap, RecursiveBinarySearchTreeMap, RedBlackTreeMap, TreeMap,
};
pub use red_black_tree::RedBlackTree;
pub use trie::Trie;
