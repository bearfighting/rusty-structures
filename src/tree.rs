//! Ordered tree structures.

pub mod avl_tree;
pub mod binary_search_tree;
pub mod trie;

pub use avl_tree::AvlTree;
pub use binary_search_tree::{BinarySearchTree, RecursiveBinarySearchTree};
pub use trie::Trie;
