use super::{GraphView, NodeId, WeightedGraphView, algorithms};

/// A nonnegative weighted graph stored in an adjacency matrix.
///
/// A `None` cell means there is no edge; `Some(0)` is a valid zero-cost edge.
/// The matrix uses O(V²) space, edge and weight queries take O(1), and adding
/// a node takes O(V) time. Neighbors are visited by ascending node ID. When a
/// duplicate edge is added, its original weight is retained.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeightedAdjacencyMatrixGraph {
    directed: bool,
    matrix: Vec<Vec<Option<u64>>>,
}

impl WeightedAdjacencyMatrixGraph {
    /// Creates an empty directed weighted graph.
    #[must_use]
    pub const fn directed() -> Self {
        Self {
            directed: true,
            matrix: Vec::new(),
        }
    }

    /// Creates an empty undirected weighted graph.
    #[must_use]
    pub const fn undirected() -> Self {
        Self {
            directed: false,
            matrix: Vec::new(),
        }
    }

    /// Returns whether the graph is directed.
    #[must_use]
    pub const fn is_directed(&self) -> bool {
        self.directed
    }

    /// Returns the number of nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.matrix.len()
    }

    /// Adds a node and returns its identifier.
    pub fn add_node(&mut self) -> NodeId {
        let id = self.node_count();
        for row in &mut self.matrix {
            row.push(None);
        }
        self.matrix.push(vec![None; id + 1]);
        id
    }

    /// Adds a weighted edge if both node IDs exist. Returns `false` for an
    /// invalid ID. If an edge already exists, its original weight is kept.
    pub fn add_edge(&mut self, from: NodeId, to: NodeId, weight: u64) -> bool {
        if from >= self.node_count() || to >= self.node_count() {
            return false;
        }
        if self.matrix[from][to].is_none() {
            self.matrix[from][to] = Some(weight);
            if !self.directed {
                self.matrix[to][from] = Some(weight);
            }
        }
        true
    }

    /// Returns an edge's weight, or `None` if the edge is absent or an ID is
    /// invalid.
    #[must_use]
    pub fn edge_weight(&self, from: NodeId, to: NodeId) -> Option<u64> {
        self.matrix.get(from)?.get(to).copied().flatten()
    }

    /// Returns whether an edge exists. Invalid IDs return `false`.
    #[must_use]
    pub fn has_edge(&self, from: NodeId, to: NodeId) -> bool {
        self.edge_weight(from, to).is_some()
    }

    /// Returns outgoing `(neighbor, weight)` pairs in ascending node ID order,
    /// or `None` for an invalid node ID.
    pub fn neighbors(&self, node: NodeId) -> Option<impl Iterator<Item = (NodeId, u64)> + '_> {
        self.matrix.get(node).map(|row| {
            row.iter()
                .enumerate()
                .filter_map(|(id, weight)| weight.map(|weight| (id, weight)))
        })
    }

    /// Finds a minimum-total-weight path using Dijkstra's algorithm.
    pub fn dijkstra(&self, start: NodeId, target: NodeId) -> Option<algorithms::WeightedPath> {
        algorithms::dijkstra(self, start, target)
    }
}

impl Default for WeightedAdjacencyMatrixGraph {
    fn default() -> Self {
        Self::directed()
    }
}

impl GraphView for WeightedAdjacencyMatrixGraph {
    fn node_count(&self) -> usize {
        self.matrix.len()
    }

    fn is_directed(&self) -> bool {
        self.directed
    }

    fn for_each_neighbor(&self, node: NodeId, visit: &mut dyn FnMut(NodeId)) -> bool {
        let Some(row) = self.matrix.get(node) else {
            return false;
        };
        for (neighbor, weight) in row.iter().enumerate() {
            if weight.is_some() {
                visit(neighbor);
            }
        }
        true
    }
}

impl WeightedGraphView for WeightedAdjacencyMatrixGraph {
    fn for_each_weighted_neighbor(&self, node: NodeId, visit: &mut dyn FnMut(NodeId, u64)) -> bool {
        let Some(row) = self.matrix.get(node) else {
            return false;
        };
        for (neighbor, weight) in row.iter().enumerate() {
            if let Some(weight) = weight {
                visit(neighbor, *weight);
            }
        }
        true
    }
}
