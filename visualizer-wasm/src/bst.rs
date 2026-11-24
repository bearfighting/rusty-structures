use crate::serialization::serialize;
use data_structures::BinarySearchTree;
use data_structures::tree::binary_search_tree::{BstEvent, BstTrace, BstTreeSnapshotNode};
use serde::Serialize;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TreeView {
    kind: &'static str,
    root: Option<NodeView>,
    len: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NodeView {
    id: String,
    value: i32,
    left: Option<Box<NodeView>>,
    right: Option<Box<NodeView>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TraceView {
    before: TreeView,
    after: TreeView,
    events: Vec<EventView>,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum EventView {
    Compare {
        node_id: String,
        node_value: i32,
        key: i32,
        ordering: String,
    },
    Descend {
        from_id: String,
        direction: String,
    },
    Inserted {
        node_id: String,
        key: i32,
    },
    Duplicate {
        key: i32,
    },
    Found {
        node_id: String,
        key: i32,
    },
    Missing {
        key: i32,
    },
    SuccessorSelected {
        removed_id: String,
        successor_id: String,
        removed: i32,
        successor: i32,
    },
    Removed {
        node_id: String,
        key: i32,
    },
}

#[wasm_bindgen]
pub struct BstSession {
    tree: BinarySearchTree<i32>,
    node_ids: HashMap<i32, String>,
    next_node_id: u64,
}

#[wasm_bindgen]
impl BstSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            tree: BinarySearchTree::new(),
            node_ids: HashMap::new(),
            next_node_id: 0,
        }
    }

    pub fn load_example(&mut self) -> Result<JsValue, JsValue> {
        self.tree.clear();
        self.node_ids.clear();
        for key in [8, 4, 12, 2, 6, 10, 14] {
            self.tree.insert(key);
            self.allocate_node_id(key);
        }
        serialize(&self.tree_view())
    }

    pub fn reset(&mut self) -> Result<JsValue, JsValue> {
        self.tree.clear();
        self.node_ids.clear();
        serialize(&self.tree_view())
    }

    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        serialize(&self.tree_view())
    }

    pub fn insert(&mut self, key: i32) -> Result<JsValue, JsValue> {
        let before = self.tree_view();
        let trace = self.tree.insert_traced(key);
        self.trace_view(before, trace)
    }

    pub fn search(&mut self, key: i32) -> Result<JsValue, JsValue> {
        let before = self.tree_view();
        let trace = self.tree.search_traced(&key);
        self.trace_view(before, trace)
    }

    pub fn remove(&mut self, key: i32) -> Result<JsValue, JsValue> {
        let before = self.tree_view();
        let trace = self.tree.remove_traced(&key);
        self.trace_view(before, trace)
    }
}

impl Default for BstSession {
    fn default() -> Self {
        Self::new()
    }
}

impl BstSession {
    fn allocate_node_id(&mut self, value: i32) -> String {
        let id = format!("bst-node-{}", self.next_node_id);
        self.next_node_id += 1;
        self.node_ids.insert(value, id.clone());
        id
    }

    fn node_id(&mut self, value: i32) -> String {
        self.node_ids
            .get(&value)
            .cloned()
            .unwrap_or_else(|| self.allocate_node_id(value))
    }

    fn tree_view(&self) -> TreeView {
        tree_view(self.tree.snapshot_for_visualization(), &self.node_ids)
    }

    fn trace_view(&mut self, before: TreeView, trace: BstTrace<i32>) -> Result<JsValue, JsValue> {
        let events = trace
            .events
            .into_iter()
            .map(|event| self.event_view(event))
            .collect();
        let after = tree_view(trace.snapshot, &self.node_ids);
        serialize(&TraceView {
            before,
            after,
            events,
        })
    }

    fn event_view(&mut self, event: BstEvent<i32>) -> EventView {
        match event {
            BstEvent::Compare {
                node,
                key,
                ordering,
            } => EventView::Compare {
                node_id: self.node_id(node),
                node_value: node,
                key,
                ordering: match ordering {
                    std::cmp::Ordering::Less => "less",
                    std::cmp::Ordering::Equal => "equal",
                    std::cmp::Ordering::Greater => "greater",
                }
                .into(),
            },
            BstEvent::DescendLeft { from } => EventView::Descend {
                from_id: self.node_id(from),
                direction: "left".into(),
            },
            BstEvent::DescendRight { from } => EventView::Descend {
                from_id: self.node_id(from),
                direction: "right".into(),
            },
            BstEvent::Inserted { key } => {
                let node_id = self.node_id(key);
                EventView::Inserted { node_id, key }
            }
            BstEvent::Duplicate { key } => EventView::Duplicate { key },
            BstEvent::Found { key } => EventView::Found {
                node_id: self.node_id(key),
                key,
            },
            BstEvent::Missing { key } => EventView::Missing { key },
            BstEvent::SuccessorSelected { removed, successor } => EventView::SuccessorSelected {
                removed_id: self.node_id(removed),
                successor_id: self.node_id(successor),
                removed,
                successor,
            },
            BstEvent::Removed { key } => {
                let node_id = self.node_id(key);
                self.node_ids.remove(&key);
                EventView::Removed { node_id, key }
            }
        }
    }
}

fn tree_view(
    snapshot: data_structures::tree::binary_search_tree::BstTreeSnapshot<i32>,
    node_ids: &HashMap<i32, String>,
) -> TreeView {
    TreeView {
        kind: "binaryTree",
        root: snapshot.root.map(|node| node_view(node, node_ids)),
        len: snapshot.len,
    }
}

fn node_view(node: Box<BstTreeSnapshotNode<i32>>, node_ids: &HashMap<i32, String>) -> NodeView {
    NodeView {
        id: node_ids
            .get(&node.value)
            .expect("every live BST node has a visual ID")
            .clone(),
        value: node.value,
        left: node.left.map(|child| Box::new(node_view(child, node_ids))),
        right: node.right.map(|child| Box::new(node_view(child, node_ids))),
    }
}
