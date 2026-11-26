use super::{GraphView, NodeId, WeightedGraphView, algorithms};

/// A graph stored as a list of outgoing neighbors for each node.
///
/// Nodes are added in order and identified by their index. Edges can be
/// directed or undirected, as selected at construction. Duplicate edges are
/// ignored. In an undirected graph each edge appears in both endpoint lists,
/// except a self-loop, which appears once. Adding or querying an edge takes
/// O(degree) time because duplicate detection scans the endpoint's neighbors.
/// Adding a node takes amortized O(1); neighbor iteration takes O(1) to start
/// and O(degree) to consume.
///
/// # Examples
///
/// ```
/// use data_structures::Graph;
///
/// let mut graph = Graph::undirected();
/// let a = graph.add_node();
/// let b = graph.add_node();
/// graph.add_edge(a, b);
/// assert_eq!(graph.neighbors(a).unwrap().collect::<Vec<_>>(), vec![b]);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Graph {
    directed: bool,
    adjacency: Vec<Vec<NodeId>>,
}

/// A graph whose edges have nonnegative integer costs.
///
/// `WeightedGraph` follows the same node and direction conventions as
/// [`Graph`]. Duplicate edges are ignored, including a later edge with a
/// different weight. Undirected edges are stored in both endpoint lists.
/// Nonnegative `u64` weights make the representation suitable for Dijkstra's
/// algorithm without imposing a numeric abstraction on the unweighted graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeightedGraph {
    directed: bool,
    adjacency: Vec<Vec<(NodeId, u64)>>,
}

impl WeightedGraph {
    /// Creates an empty directed weighted graph.
    #[must_use]
    pub const fn directed() -> Self {
        Self {
            directed: true,
            adjacency: Vec::new(),
        }
    }

    /// Creates an empty undirected weighted graph.
    #[must_use]
    pub const fn undirected() -> Self {
        Self {
            directed: false,
            adjacency: Vec::new(),
        }
    }

    /// Returns whether this graph is directed.
    #[must_use]
    pub const fn is_directed(&self) -> bool {
        self.directed
    }

    /// Returns the number of nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.adjacency.len()
    }

    /// Adds a node and returns its identifier.
    pub fn add_node(&mut self) -> NodeId {
        let id = self.adjacency.len();
        self.adjacency.push(Vec::new());
        id
    }

    /// Adds a weighted edge if both endpoints exist. Returns `false` when an
    /// endpoint is invalid. If the edge already exists its original weight is
    /// retained and the method returns `true`.
    pub fn add_edge(&mut self, from: NodeId, to: NodeId, weight: u64) -> bool {
        if from >= self.node_count() || to >= self.node_count() {
            return false;
        }
        if !self.adjacency[from]
            .iter()
            .any(|&(neighbor, _)| neighbor == to)
        {
            self.adjacency[from].push((to, weight));
        }
        if !self.directed
            && from != to
            && !self.adjacency[to]
                .iter()
                .any(|&(neighbor, _)| neighbor == from)
        {
            self.adjacency[to].push((from, weight));
        }
        true
    }

    /// Returns the stored weight of an edge, or `None` if absent or invalid.
    #[must_use]
    pub fn edge_weight(&self, from: NodeId, to: NodeId) -> Option<u64> {
        self.adjacency
            .get(from)?
            .iter()
            .find_map(|&(neighbor, weight)| (neighbor == to).then_some(weight))
    }

    /// Returns outgoing `(neighbor, weight)` pairs, or `None` for an invalid ID.
    pub fn neighbors(&self, node: NodeId) -> Option<impl Iterator<Item = (NodeId, u64)> + '_> {
        self.adjacency
            .get(node)
            .map(|neighbors| neighbors.iter().copied())
    }

    /// Finds a minimum-total-weight path using Dijkstra's algorithm.
    ///
    /// Returns `None` for invalid node IDs, an unreachable target, or when no
    /// path has a total weight representable as `u64`. Edge weights are
    /// nonnegative by construction. Runs in O((V + E) log V) time and uses
    /// O(V) auxiliary space, excluding the returned path.
    pub fn dijkstra(&self, start: NodeId, target: NodeId) -> Option<algorithms::WeightedPath> {
        algorithms::dijkstra(self, start, target)
    }
}

impl Default for WeightedGraph {
    fn default() -> Self {
        Self::directed()
    }
}

impl Graph {
    /// Creates an empty directed graph.
    #[must_use]
    pub const fn directed() -> Self {
        Self {
            directed: true,
            adjacency: Vec::new(),
        }
    }

    /// Creates an empty undirected graph.
    #[must_use]
    pub const fn undirected() -> Self {
        Self {
            directed: false,
            adjacency: Vec::new(),
        }
    }

    /// Returns whether this graph is directed.
    #[must_use]
    pub const fn is_directed(&self) -> bool {
        self.directed
    }

    /// Returns the number of nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.adjacency.len()
    }

    /// Returns the number of directed adjacency entries (an undirected edge
    /// normally contributes two entries).
    #[must_use]
    pub fn adjacency_count(&self) -> usize {
        self.adjacency.iter().map(Vec::len).sum()
    }

    /// Adds a node and returns its identifier.
    pub fn add_node(&mut self) -> NodeId {
        let id = self.adjacency.len();
        self.adjacency.push(Vec::new());
        id
    }

    /// Adds an edge if both node identifiers exist. Returns `false` for an
    /// invalid endpoint and `true` if the edge was accepted; an existing edge
    /// is considered accepted and does not create a duplicate.
    pub fn add_edge(&mut self, from: NodeId, to: NodeId) -> bool {
        if from >= self.node_count() || to >= self.node_count() {
            return false;
        }
        if !self.adjacency[from].contains(&to) {
            self.adjacency[from].push(to);
        }
        if !self.directed && from != to && !self.adjacency[to].contains(&from) {
            self.adjacency[to].push(from);
        }
        true
    }

    /// Returns whether an edge exists. Invalid node identifiers return false.
    #[must_use]
    pub fn has_edge(&self, from: NodeId, to: NodeId) -> bool {
        self.adjacency
            .get(from)
            .is_some_and(|neighbors| neighbors.contains(&to))
    }

    /// Returns the outgoing neighbors of a node, or `None` for an invalid ID.
    pub fn neighbors(&self, node: NodeId) -> Option<impl Iterator<Item = NodeId> + '_> {
        self.adjacency
            .get(node)
            .map(|neighbors| neighbors.iter().copied())
    }

    /// Visits nodes in depth-first preorder from `start`.
    ///
    /// Neighbors are considered in the order their edges were added. Returns
    /// `None` when `start` is not a valid node. Only nodes reachable from the
    /// start are returned. Runs in O(V + E) time and uses O(V) auxiliary space
    /// in the worst case.
    pub fn depth_first_search(&self, start: NodeId) -> Option<Vec<NodeId>> {
        algorithms::depth_first_search(self, start)
    }

    /// Visits nodes in breadth-first order from `start`.
    ///
    /// Neighbors are considered in the order their edges were added. Returns
    /// `None` when `start` is not a valid node. Only nodes reachable from the
    /// start are returned. Runs in O(V + E) time and uses O(V) auxiliary space
    /// in the worst case.
    pub fn breadth_first_search(&self, start: NodeId) -> Option<Vec<NodeId>> {
        algorithms::breadth_first_search(self, start)
    }

    /// Returns a topological ordering of this directed acyclic graph.
    ///
    /// Nodes with zero in-degree are selected by ascending node ID, making the
    /// result deterministic when multiple valid orderings exist. Returns
    /// `None` for an undirected graph or when the directed graph contains a
    /// cycle. Runs in O(V + E) time and uses O(V) auxiliary space.
    pub fn topological_sort(&self) -> Option<Vec<NodeId>> {
        algorithms::topological_sort(self)
    }

    /// Returns a shortest path from `start` to `target` in number of edges.
    ///
    /// Breadth-first search finds a shortest path in an unweighted graph.
    /// When multiple shortest paths exist, neighbors are explored in edge
    /// insertion order, so the selected path is deterministic. Returns `None`
    /// if either identifier is invalid or `target` is unreachable. Runs in
    /// O(V + E) time and uses O(V) auxiliary space.
    pub fn shortest_path(&self, start: NodeId, target: NodeId) -> Option<Vec<NodeId>> {
        algorithms::shortest_path(self, start, target)
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::directed()
    }
}

impl GraphView for Graph {
    fn node_count(&self) -> usize {
        self.adjacency.len()
    }

    fn is_directed(&self) -> bool {
        self.directed
    }

    fn for_each_neighbor(&self, node: NodeId, visit: &mut dyn FnMut(NodeId)) -> bool {
        let Some(neighbors) = self.adjacency.get(node) else {
            return false;
        };
        neighbors.iter().copied().for_each(visit);
        true
    }
}

impl GraphView for WeightedGraph {
    fn node_count(&self) -> usize {
        self.adjacency.len()
    }

    fn is_directed(&self) -> bool {
        self.directed
    }

    fn for_each_neighbor(&self, node: NodeId, visit: &mut dyn FnMut(NodeId)) -> bool {
        let Some(neighbors) = self.adjacency.get(node) else {
            return false;
        };
        neighbors
            .iter()
            .map(|&(neighbor, _)| neighbor)
            .for_each(visit);
        true
    }
}

impl WeightedGraphView for WeightedGraph {
    fn for_each_weighted_neighbor(&self, node: NodeId, visit: &mut dyn FnMut(NodeId, u64)) -> bool {
        let Some(neighbors) = self.adjacency.get(node) else {
            return false;
        };
        neighbors
            .iter()
            .copied()
            .for_each(|(neighbor, weight)| visit(neighbor, weight));
        true
    }
}
