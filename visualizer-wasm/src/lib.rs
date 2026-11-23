use data_structures::BinarySearchTree;
use data_structures::tree::binary_search_tree::{BstEvent, BstTrace, BstTreeSnapshotNode};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TreeView {
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
        key: i32,
    },
    Missing {
        key: i32,
    },
    SuccessorSelected {
        removed: i32,
        successor: i32,
    },
    Removed {
        key: i32,
    },
}

#[wasm_bindgen]
pub struct BstSession {
    tree: BinarySearchTree<i32>,
}

#[wasm_bindgen]
impl BstSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            tree: BinarySearchTree::new(),
        }
    }

    pub fn load_example(&mut self) -> Result<JsValue, JsValue> {
        self.tree.clear();
        for key in [8, 4, 12, 2, 6, 10, 14] {
            self.tree.insert(key);
        }
        serialize(&tree_view(self.tree.snapshot_for_visualization()))
    }

    pub fn reset(&mut self) -> Result<JsValue, JsValue> {
        self.tree.clear();
        serialize(&tree_view(self.tree.snapshot_for_visualization()))
    }

    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        serialize(&tree_view(self.tree.snapshot_for_visualization()))
    }

    pub fn insert(&mut self, key: i32) -> Result<JsValue, JsValue> {
        let before = tree_view(self.tree.snapshot_for_visualization());
        let trace = self.tree.insert_traced(key);
        trace_view(before, trace)
    }

    pub fn search(&self, key: i32) -> Result<JsValue, JsValue> {
        let before = tree_view(self.tree.snapshot_for_visualization());
        let trace = self.tree.search_traced(&key);
        trace_view(before, trace)
    }

    pub fn remove(&mut self, key: i32) -> Result<JsValue, JsValue> {
        let before = tree_view(self.tree.snapshot_for_visualization());
        let trace = self.tree.remove_traced(&key);
        trace_view(before, trace)
    }
}

impl Default for BstSession {
    fn default() -> Self {
        Self::new()
    }
}

fn trace_view(before: TreeView, trace: BstTrace<i32>) -> Result<JsValue, JsValue> {
    let events = trace.events.into_iter().map(event_view).collect();
    serialize(&TraceView {
        before,
        after: tree_view(trace.snapshot),
        events,
    })
}

fn event_view(event: BstEvent<i32>) -> EventView {
    match event {
        BstEvent::Compare {
            node,
            key,
            ordering,
        } => EventView::Compare {
            node_id: node.to_string(),
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
            from_id: from.to_string(),
            direction: "left".into(),
        },
        BstEvent::DescendRight { from } => EventView::Descend {
            from_id: from.to_string(),
            direction: "right".into(),
        },
        BstEvent::Inserted { key } => EventView::Inserted {
            node_id: key.to_string(),
            key,
        },
        BstEvent::Duplicate { key } => EventView::Duplicate { key },
        BstEvent::Found { key } => EventView::Found { key },
        BstEvent::Missing { key } => EventView::Missing { key },
        BstEvent::SuccessorSelected { removed, successor } => {
            EventView::SuccessorSelected { removed, successor }
        }
        BstEvent::Removed { key } => EventView::Removed { key },
    }
}

fn tree_view(
    snapshot: data_structures::tree::binary_search_tree::BstTreeSnapshot<i32>,
) -> TreeView {
    TreeView {
        root: snapshot.root.map(node_view),
        len: snapshot.len,
    }
}

fn node_view(node: Box<BstTreeSnapshotNode<i32>>) -> NodeView {
    NodeView {
        id: node.value.to_string(),
        value: node.value,
        left: node.left.map(|child| Box::new(node_view(child))),
        right: node.right.map(|child| Box::new(node_view(child))),
    }
}

fn serialize<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(|error| JsValue::from_str(&error.to_string()))
}
