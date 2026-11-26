use super::{GraphView, NodeId, algorithms};

/// A graph stored in an adjacency matrix.
///
/// The matrix uses O(V²) space and supports edge queries in O(1). Adding a
/// node takes O(V) time to extend each row. Neighbors are returned
/// in ascending node ID order. Duplicate edges are ignored.
///
/// # Examples
///
/// ```
/// use data_structures::AdjacencyMatrixGraph;
///
/// let mut graph = AdjacencyMatrixGraph::undirected();
/// let a = graph.add_node();
/// let b = graph.add_node();
/// assert!(graph.add_edge(a, b));
/// assert!(graph.has_edge(b, a));
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdjacencyMatrixGraph {
    directed: bool,
    matrix: Vec<Vec<bool>>,
}

impl AdjacencyMatrixGraph {
    /// Creates an empty directed graph.
    #[must_use]
    pub const fn directed() -> Self {
        Self {
            directed: true,
            matrix: Vec::new(),
        }
    }

    /// Creates an empty undirected graph.
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
            row.push(false);
        }
        self.matrix.push(vec![false; id + 1]);
        id
    }

    /// Adds an edge if both node IDs exist. Returns `false` for an invalid ID.
    pub fn add_edge(&mut self, from: NodeId, to: NodeId) -> bool {
        let count = self.node_count();
        if from >= count || to >= count {
            return false;
        }
        self.matrix[from][to] = true;
        if !self.directed {
            self.matrix[to][from] = true;
        }
        true
    }

    /// Returns whether an edge exists. Invalid IDs return `false`.
    #[must_use]
    pub fn has_edge(&self, from: NodeId, to: NodeId) -> bool {
        self.matrix
            .get(from)
            .and_then(|row| row.get(to))
            .copied()
            .unwrap_or(false)
    }

    /// Returns outgoing neighbors in ascending node ID order, or `None` for
    /// an invalid node ID.
    pub fn neighbors(&self, node: NodeId) -> Option<impl Iterator<Item = NodeId> + '_> {
        self.matrix.get(node).map(|row| {
            row.iter()
                .enumerate()
                .filter_map(|(id, &edge)| edge.then_some(id))
        })
    }

    /// Runs depth-first preorder from a node.
    pub fn depth_first_search(&self, start: NodeId) -> Option<Vec<NodeId>> {
        algorithms::depth_first_search(self, start)
    }

    /// Runs breadth-first traversal from a node.
    pub fn breadth_first_search(&self, start: NodeId) -> Option<Vec<NodeId>> {
        algorithms::breadth_first_search(self, start)
    }

    /// Returns a topological ordering, or `None` for undirected or cyclic graphs.
    pub fn topological_sort(&self) -> Option<Vec<NodeId>> {
        algorithms::topological_sort(self)
    }

    /// Returns a shortest path measured in number of edges.
    pub fn shortest_path(&self, start: NodeId, target: NodeId) -> Option<Vec<NodeId>> {
        algorithms::shortest_path(self, start, target)
    }
}

impl Default for AdjacencyMatrixGraph {
    fn default() -> Self {
        Self::directed()
    }
}

impl GraphView for AdjacencyMatrixGraph {
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
        for (neighbor, &is_edge) in row.iter().enumerate() {
            if is_edge {
                visit(neighbor);
            }
        }
        true
    }
}
