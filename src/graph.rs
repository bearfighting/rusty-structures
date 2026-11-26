//! Graph representations and storage-independent algorithms.
//!
//! `Graph` and `WeightedGraph` are adjacency-list implementations. Algorithms
//! operate on [`GraphView`], so additional storage representations can reuse
//! them without changing their public behavior.

mod adjacency_list;
mod adjacency_matrix;
pub mod algorithms;
mod traits;
mod weighted_adjacency_matrix;

/// Stable identifier for a node within a graph.
pub type NodeId = usize;

pub use adjacency_list::{Graph, WeightedGraph};
pub use adjacency_matrix::AdjacencyMatrixGraph;
pub use algorithms::{
    WeightedPath, breadth_first_search, connected_components, depth_first_search, dijkstra,
    shortest_path, strongly_connected_components, topological_sort,
};
pub use traits::{GraphView, WeightedGraphView};
pub use weighted_adjacency_matrix::WeightedAdjacencyMatrixGraph;
