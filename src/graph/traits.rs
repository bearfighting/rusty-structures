use super::NodeId;

/// Read-only graph operations required by the storage-independent algorithms.
///
/// Implement this trait to make another representation, such as an adjacency
/// matrix, usable by the algorithms in this module. Implementations visit each
/// outgoing neighbor once, return only valid node IDs, and return `false` when
/// the requested node ID is invalid.
pub trait GraphView {
    /// Returns the number of nodes.
    fn node_count(&self) -> usize;

    /// Returns whether edges are directed.
    fn is_directed(&self) -> bool;

    /// Calls `visit` for each outgoing neighbor, returning `false` for an
    /// invalid node ID.
    fn for_each_neighbor(&self, node: NodeId, visit: &mut dyn FnMut(NodeId)) -> bool;
}

/// Read-only access to edge weights, required by weighted shortest-path
/// algorithms.
pub trait WeightedGraphView: GraphView {
    /// Calls `visit` for each outgoing `(neighbor, weight)` pair, returning
    /// `false` when the requested node ID is invalid. Weights must be
    /// nonnegative; this interface uses `u64` to make that requirement explicit.
    fn for_each_weighted_neighbor(&self, node: NodeId, visit: &mut dyn FnMut(NodeId, u64)) -> bool;
}
