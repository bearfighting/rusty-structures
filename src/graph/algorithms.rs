//! Traversal and ordering algorithms that work with any [`GraphView`].
//!
//! Implementations provide only node count, direction, and neighbor access;
//! these algorithms do not depend on adjacency-list storage.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

use super::{GraphView, NodeId, WeightedGraphView};

/// A minimum-total-weight path found by Dijkstra's algorithm.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeightedPath {
    /// Sum of the edge weights along the path.
    pub distance: u64,
    /// Node IDs from the start through the target, inclusive.
    pub nodes: Vec<NodeId>,
}

/// Runs depth-first preorder using any [`GraphView`] representation.
pub fn depth_first_search<G: GraphView + ?Sized>(graph: &G, start: NodeId) -> Option<Vec<NodeId>> {
    let count = graph.node_count();
    if start >= count {
        return None;
    }
    let mut visited = vec![false; count];
    let mut order = Vec::new();
    let mut stack = vec![start];
    while let Some(node) = stack.pop() {
        if visited[node] {
            continue;
        }
        visited[node] = true;
        order.push(node);
        let mut neighbors = Vec::new();
        graph.for_each_neighbor(node, &mut |neighbor| neighbors.push(neighbor));
        for neighbor in neighbors.into_iter().rev() {
            if !visited[neighbor] {
                stack.push(neighbor);
            }
        }
    }
    Some(order)
}

/// Runs breadth-first traversal using any [`GraphView`] representation.
pub fn breadth_first_search<G: GraphView + ?Sized>(
    graph: &G,
    start: NodeId,
) -> Option<Vec<NodeId>> {
    let count = graph.node_count();
    if start >= count {
        return None;
    }
    let mut visited = vec![false; count];
    let mut order = Vec::new();
    let mut queue = VecDeque::from([start]);
    visited[start] = true;
    while let Some(node) = queue.pop_front() {
        order.push(node);
        graph.for_each_neighbor(node, &mut |neighbor| {
            if !visited[neighbor] {
                visited[neighbor] = true;
                queue.push_back(neighbor);
            }
        });
    }
    Some(order)
}

/// Returns a deterministic topological ordering using any [`GraphView`].
pub fn topological_sort<G: GraphView + ?Sized>(graph: &G) -> Option<Vec<NodeId>> {
    if !graph.is_directed() {
        return None;
    }
    let count = graph.node_count();
    let mut in_degree = vec![0usize; count];
    for node in 0..count {
        graph.for_each_neighbor(node, &mut |neighbor| in_degree[neighbor] += 1);
    }
    let mut ready = VecDeque::new();
    for (node, &degree) in in_degree.iter().enumerate() {
        if degree == 0 {
            ready.push_back(node);
        }
    }
    let mut order = Vec::with_capacity(count);
    while let Some(node) = ready.pop_front() {
        order.push(node);
        graph.for_each_neighbor(node, &mut |neighbor| {
            in_degree[neighbor] -= 1;
            if in_degree[neighbor] == 0 {
                ready.push_back(neighbor);
            }
        });
    }
    (order.len() == count).then_some(order)
}

/// Finds a shortest path measured in number of edges, representation-independently.
pub fn shortest_path<G: GraphView + ?Sized>(
    graph: &G,
    start: NodeId,
    target: NodeId,
) -> Option<Vec<NodeId>> {
    let count = graph.node_count();
    if start >= count || target >= count {
        return None;
    }
    let mut predecessor = vec![None; count];
    let mut visited = vec![false; count];
    let mut queue = VecDeque::from([start]);
    visited[start] = true;
    while let Some(node) = queue.pop_front() {
        if node == target {
            break;
        }
        graph.for_each_neighbor(node, &mut |neighbor| {
            if !visited[neighbor] {
                visited[neighbor] = true;
                predecessor[neighbor] = Some(node);
                queue.push_back(neighbor);
            }
        });
    }
    if !visited[target] {
        return None;
    }
    let mut path = vec![target];
    let mut node = target;
    while node != start {
        node = predecessor[node]?;
        path.push(node);
    }
    path.reverse();
    Some(path)
}

/// Finds a minimum-total-weight path using Dijkstra's algorithm.
///
/// This works with any [`WeightedGraphView`] whose edge weights are
/// nonnegative. Equal-cost alternatives are resolved by the order in which
/// neighbors are visited. Candidates whose distance overflows `u64` are
/// ignored; `None` is returned if no representable path reaches the target.
/// Runs in O((V + E) log V) time and uses O(V) auxiliary space, excluding the
/// returned path.
pub fn dijkstra<G: WeightedGraphView + ?Sized>(
    graph: &G,
    start: NodeId,
    target: NodeId,
) -> Option<WeightedPath> {
    let count = graph.node_count();
    if start >= count || target >= count {
        return None;
    }

    let mut distance = vec![None; count];
    let mut predecessor = vec![None; count];
    let mut pending = BinaryHeap::new();
    distance[start] = Some(0);
    pending.push((Reverse(0u64), start));

    while let Some((Reverse(cost), node)) = pending.pop() {
        if Some(cost) != distance[node] {
            continue;
        }
        if node == target {
            break;
        }
        graph.for_each_weighted_neighbor(node, &mut |neighbor, weight| {
            if let Some(candidate) = cost.checked_add(weight)
                && distance[neighbor].is_none_or(|current| candidate < current)
            {
                distance[neighbor] = Some(candidate);
                predecessor[neighbor] = Some(node);
                pending.push((Reverse(candidate), neighbor));
            }
        });
    }

    let total = distance[target]?;
    let mut nodes = vec![target];
    let mut node = target;
    while node != start {
        node = predecessor[node]?;
        nodes.push(node);
    }
    nodes.reverse();
    Some(WeightedPath {
        distance: total,
        nodes,
    })
}

/// Finds the connected components of an undirected graph.
///
/// Each component contains its node IDs in depth-first traversal order, and
/// components are started in ascending node ID order. Returns `None` for a
/// directed graph. Runs in O(V + E) time and uses O(V) auxiliary space.
pub fn connected_components<G: GraphView + ?Sized>(graph: &G) -> Option<Vec<Vec<NodeId>>> {
    if graph.is_directed() {
        return None;
    }
    let count = graph.node_count();
    let mut visited = vec![false; count];
    let mut components = Vec::new();
    for start in 0..count {
        if visited[start] {
            continue;
        }
        let mut component = Vec::new();
        let mut stack = vec![start];
        visited[start] = true;
        while let Some(node) = stack.pop() {
            component.push(node);
            let mut neighbors = Vec::new();
            graph.for_each_neighbor(node, &mut |neighbor| neighbors.push(neighbor));
            for neighbor in neighbors.into_iter().rev() {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    stack.push(neighbor);
                }
            }
        }
        components.push(component);
    }
    Some(components)
}

/// Finds the strongly connected components of a directed graph using
/// Kosaraju's two-pass algorithm.
///
/// Nodes within each component are in ascending node ID order; component
/// ordering follows the reverse-graph traversal. Returns `None` for an
/// undirected graph. Runs in O(V + E) time and uses O(V + E) auxiliary space
/// to build the reverse adjacency lists.
pub fn strongly_connected_components<G: GraphView + ?Sized>(graph: &G) -> Option<Vec<Vec<NodeId>>> {
    if !graph.is_directed() {
        return None;
    }
    let count = graph.node_count();
    let mut adjacency = vec![Vec::new(); count];
    let mut reverse = vec![Vec::new(); count];
    for (node, outgoing) in adjacency.iter_mut().enumerate() {
        graph.for_each_neighbor(node, &mut |neighbor| {
            outgoing.push(neighbor);
            reverse[neighbor].push(node);
        });
    }

    // Iterative postorder DFS records finish times without using the call stack.
    let mut visited = vec![false; count];
    let mut finish_order = Vec::with_capacity(count);
    for start in 0..count {
        if visited[start] {
            continue;
        }
        visited[start] = true;
        let mut stack = vec![(start, 0usize)];
        while let Some(&(node, next_neighbor)) = stack.last() {
            if next_neighbor == adjacency[node].len() {
                finish_order.push(node);
                stack.pop();
                continue;
            }
            stack.last_mut().expect("stack is not empty").1 += 1;
            let neighbor = adjacency[node][next_neighbor];
            if !visited[neighbor] {
                visited[neighbor] = true;
                stack.push((neighbor, 0));
            }
        }
    }

    visited.fill(false);
    let mut components = Vec::new();
    for start in finish_order.into_iter().rev() {
        if visited[start] {
            continue;
        }
        let mut component = Vec::new();
        let mut stack = vec![start];
        visited[start] = true;
        while let Some(node) = stack.pop() {
            component.push(node);
            for &neighbor in &reverse[node] {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    stack.push(neighbor);
                }
            }
        }
        component.sort_unstable();
        components.push(component);
    }
    Some(components)
}
