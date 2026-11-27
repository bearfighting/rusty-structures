use data_structures::graph::breadth_first_search;
use data_structures::{Graph, NodeId, WeightedGraph};

pub fn run() {
    let mut graph = Graph::undirected();
    let nodes: Vec<NodeId> = (0..6).map(|_| graph.add_node()).collect();
    for (from, to) in [(0, 1), (0, 2), (1, 3), (2, 4), (4, 5)] {
        graph.add_edge(nodes[from], nodes[to]);
    }

    // A graph exposes local neighbor iterators. BFS combines a queue and a
    // visited set internally; the returned order can then feed other adapters.
    let neighbors: Vec<_> = graph.neighbors(nodes[0]).unwrap().collect();
    println!("neighbors of node 0: {neighbors:?}");

    let order = breadth_first_search(&graph, nodes[0]).unwrap();
    let labeled: Vec<_> = order
        .iter()
        .enumerate()
        .map(|(depth, &node)| (depth, node))
        .collect();
    println!("BFS order with visit positions: {labeled:?}");

    let mut dependencies = Graph::directed();
    let tasks: Vec<_> = (0..4).map(|_| dependencies.add_node()).collect();
    dependencies.add_edge(tasks[0], tasks[1]);
    dependencies.add_edge(tasks[1], tasks[2]);
    dependencies.add_edge(tasks[2], tasks[3]);
    let order = dependencies.topological_sort().unwrap();
    println!("valid build order: {order:?}");

    let mut roads = WeightedGraph::directed();
    let places: Vec<_> = (0..4).map(|_| roads.add_node()).collect();
    roads.add_edge(places[0], places[1], 10);
    roads.add_edge(places[0], places[2], 4);
    roads.add_edge(places[2], places[1], 2);
    roads.add_edge(places[1], places[3], 3);
    roads.add_edge(places[2], places[3], 15);
    let route = roads.dijkstra(places[0], places[3]).unwrap();
    println!("shortest route: {:?}, cost {}", route.nodes, route.distance);
}
