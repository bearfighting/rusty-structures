use crate::serialization::serialize;
use data_structures::AvlTree;
use serde::Serialize;
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;

#[derive(Clone)]
struct Node {
    value: i32,
    id: String,
    height: usize,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TreeView {
    kind: &'static str,
    root: Option<Box<NodeView>>,
    len: usize,
}
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct NodeView {
    id: String,
    value: i32,
    height: usize,
    balance_factor: i32,
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
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
enum EventView {
    Compared {
        node_id: String,
        node_value: i32,
        key: i32,
        ordering: &'static str,
    },
    Inserted {
        node_id: String,
        key: i32,
    },
    Duplicate {
        key: i32,
    },
    Rotated {
        direction: &'static str,
        pivot_id: String,
        root_id: String,
        case: &'static str,
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
pub struct AvlSession {
    tree: AvlTree<i32>,
    root: Option<Box<Node>>,
    ids: BTreeMap<i32, String>,
    next_id: u64,
}
#[wasm_bindgen]
impl AvlSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            tree: AvlTree::new(),
            root: None,
            ids: BTreeMap::new(),
            next_id: 0,
        }
    }
    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        serialize(&self.view())
    }
    pub fn reset(&mut self) -> Result<JsValue, JsValue> {
        self.tree.clear();
        self.root = None;
        self.ids.clear();
        serialize(&self.view())
    }
    pub fn load_example(&mut self) -> Result<JsValue, JsValue> {
        self.tree.clear();
        self.root = None;
        self.ids.clear();
        for value in [30, 20, 40, 10, 25, 35, 50] {
            let id = self.allocate(value);
            self.tree.insert(value);
            self.root = insert(self.root.take(), value, id, &mut Vec::new());
        }
        serialize(&self.view())
    }
    pub fn insert(&mut self, key: i32) -> Result<JsValue, JsValue> {
        let before = self.view();
        let mut events = Vec::new();
        let id = if self.ids.contains_key(&key) {
            self.ids[&key].clone()
        } else {
            self.allocate(key)
        };
        self.root = insert(self.root.take(), key, id, &mut events);
        let inserted = self.tree.insert(key);
        if inserted {
            events.push(EventView::Inserted {
                node_id: self.ids[&key].clone(),
                key,
            });
        } else {
            events.push(EventView::Duplicate { key });
        }
        if self.tree.iter().copied().collect::<Vec<_>>() != inorder(&self.root)
            || self.tree.height() != h(&self.root)
            || !valid_avl(&self.root, None, None)
        {
            return Err(JsValue::from_str(
                "AVL visual model diverged from Rust tree.",
            ));
        }
        self.trace(before, events)
    }
    pub fn search(&self, key: i32) -> Result<JsValue, JsValue> {
        let before = self.view();
        let mut events = Vec::new();
        let mut node = self.root.as_deref();
        while let Some(current) = node {
            let ordering = key.cmp(&current.value);
            events.push(EventView::Compared {
                node_id: current.id.clone(),
                node_value: current.value,
                key,
                ordering: match ordering {
                    std::cmp::Ordering::Less => "less",
                    std::cmp::Ordering::Equal => "equal",
                    std::cmp::Ordering::Greater => "greater",
                },
            });
            match ordering {
                std::cmp::Ordering::Less => node = current.left.as_deref(),
                std::cmp::Ordering::Greater => node = current.right.as_deref(),
                std::cmp::Ordering::Equal => {
                    events.push(EventView::Found {
                        node_id: current.id.clone(),
                        key,
                    });
                    return self.trace(before, events);
                }
            }
        }
        events.push(EventView::Missing { key });
        self.trace(before, events)
    }
    pub fn remove(&mut self, key: i32) -> Result<JsValue, JsValue> {
        let before = self.view();
        let mut events = Vec::new();
        let (root, removed) = remove_node(self.root.take(), key, &mut events);
        self.root = root;
        let actual = self.tree.remove(&key);
        if actual != removed {
            return Err(JsValue::from_str(
                "AVL removal model diverged from Rust tree.",
            ));
        }
        if removed.is_some() {
            if let Some(id) = self.ids.remove(&key) {
                events.push(EventView::Removed { node_id: id, key });
            }
        } else {
            events.push(EventView::Missing { key });
        }
        if self.tree.iter().copied().collect::<Vec<_>>() != inorder(&self.root)
            || self.tree.height() != h(&self.root)
            || !valid_avl(&self.root, None, None)
        {
            return Err(JsValue::from_str(
                "AVL visual model diverged from Rust tree.",
            ));
        }
        self.trace(before, events)
    }
}
impl Default for AvlSession {
    fn default() -> Self {
        Self::new()
    }
}
impl AvlSession {
    fn allocate(&mut self, key: i32) -> String {
        let id = format!("avl-node-{}", self.next_id);
        self.next_id += 1;
        self.ids.insert(key, id.clone());
        id
    }
    fn view(&self) -> TreeView {
        TreeView {
            kind: "binaryTree",
            root: view_node(&self.root),
            len: self.tree.len(),
        }
    }
    fn trace(&self, before: TreeView, events: Vec<EventView>) -> Result<JsValue, JsValue> {
        serialize(&TraceView {
            before,
            after: self.view(),
            events,
        })
    }
}
fn h(node: &Option<Box<Node>>) -> usize {
    node.as_ref().map_or(0, |n| n.height)
}
fn update(n: &mut Node) {
    n.height = 1 + h(&n.left).max(h(&n.right));
}
fn insert(
    root: Option<Box<Node>>,
    key: i32,
    id: String,
    events: &mut Vec<EventView>,
) -> Option<Box<Node>> {
    let Some(mut node) = root else {
        return Some(Box::new(Node {
            value: key,
            id,
            height: 1,
            left: None,
            right: None,
        }));
    };
    match key.cmp(&node.value) {
        std::cmp::Ordering::Less => {
            events.push(EventView::Compared {
                node_id: node.id.clone(),
                node_value: node.value,
                key,
                ordering: "less",
            });
            node.left = insert(node.left.take(), key, id, events);
        }
        std::cmp::Ordering::Greater => {
            events.push(EventView::Compared {
                node_id: node.id.clone(),
                node_value: node.value,
                key,
                ordering: "greater",
            });
            node.right = insert(node.right.take(), key, id, events);
        }
        std::cmp::Ordering::Equal => return Some(node),
    }
    rebalance(node, events)
}
fn rebalance(mut n: Box<Node>, events: &mut Vec<EventView>) -> Option<Box<Node>> {
    update(&mut n);
    let bf = h(&n.left) as i32 - h(&n.right) as i32;
    if bf > 1 {
        let left_right = n
            .left
            .as_ref()
            .is_some_and(|left| h(&left.right) > h(&left.left));
        if left_right {
            n.left = n.left.take().and_then(|x| rotate_left(x, events, "LR"));
        }
        rotate_right(n, events, if left_right { "LR" } else { "LL" })
    } else if bf < -1 {
        let right_left = n
            .right
            .as_ref()
            .is_some_and(|right| h(&right.left) > h(&right.right));
        if right_left {
            n.right = n.right.take().and_then(|x| rotate_right(x, events, "RL"));
        }
        rotate_left(n, events, if right_left { "RL" } else { "RR" })
    } else {
        Some(n)
    }
}
fn rotate_left(
    mut root: Box<Node>,
    events: &mut Vec<EventView>,
    case: &'static str,
) -> Option<Box<Node>> {
    let mut pivot = root.right.take()?;
    let root_id = root.id.clone();
    let pivot_id = pivot.id.clone();
    root.right = pivot.left.take();
    update(&mut root);
    pivot.left = Some(root);
    update(&mut pivot);
    events.push(EventView::Rotated {
        direction: "left",
        pivot_id,
        root_id,
        case,
    });
    Some(pivot)
}
fn rotate_right(
    mut root: Box<Node>,
    events: &mut Vec<EventView>,
    case: &'static str,
) -> Option<Box<Node>> {
    let mut pivot = root.left.take()?;
    let root_id = root.id.clone();
    let pivot_id = pivot.id.clone();
    root.left = pivot.right.take();
    update(&mut root);
    pivot.right = Some(root);
    update(&mut pivot);
    events.push(EventView::Rotated {
        direction: "right",
        pivot_id,
        root_id,
        case,
    });
    Some(pivot)
}
fn view_node(node: &Option<Box<Node>>) -> Option<Box<NodeView>> {
    node.as_ref().map(|n| {
        Box::new(NodeView {
            id: n.id.clone(),
            value: n.value,
            height: n.height,
            balance_factor: h(&n.left) as i32 - h(&n.right) as i32,
            left: view_node(&n.left),
            right: view_node(&n.right),
        })
    })
}
fn inorder(node: &Option<Box<Node>>) -> Vec<i32> {
    let mut out = Vec::new();
    fn walk(node: &Option<Box<Node>>, out: &mut Vec<i32>) {
        if let Some(n) = node {
            walk(&n.left, out);
            out.push(n.value);
            walk(&n.right, out);
        }
    }
    walk(node, &mut out);
    out
}
fn valid_avl(node: &Option<Box<Node>>, min: Option<i32>, max: Option<i32>) -> bool {
    let Some(node) = node else {
        return true;
    };
    let left_height = h(&node.left);
    let right_height = h(&node.right);
    node.height == 1 + left_height.max(right_height)
        && left_height.abs_diff(right_height) <= 1
        && min.is_none_or(|min| node.value > min)
        && max.is_none_or(|max| node.value < max)
        && valid_avl(&node.left, min, Some(node.value))
        && valid_avl(&node.right, Some(node.value), max)
}
fn remove_node(
    root: Option<Box<Node>>,
    key: i32,
    events: &mut Vec<EventView>,
) -> (Option<Box<Node>>, Option<i32>) {
    let Some(mut node) = root else {
        return (None, None);
    };
    match key.cmp(&node.value) {
        std::cmp::Ordering::Less => {
            events.push(EventView::Compared {
                node_id: node.id.clone(),
                node_value: node.value,
                key,
                ordering: "less",
            });
            let (left, removed) = remove_node(node.left.take(), key, events);
            node.left = left;
            if removed.is_some() {
                (rebalance(node, events), removed)
            } else {
                (Some(node), None)
            }
        }
        std::cmp::Ordering::Greater => {
            events.push(EventView::Compared {
                node_id: node.id.clone(),
                node_value: node.value,
                key,
                ordering: "greater",
            });
            let (right, removed) = remove_node(node.right.take(), key, events);
            node.right = right;
            if removed.is_some() {
                (rebalance(node, events), removed)
            } else {
                (Some(node), None)
            }
        }
        std::cmp::Ordering::Equal => {
            let removed = node.value;
            match (node.left.take(), node.right.take()) {
                (None, right) => (right, Some(removed)),
                (left, None) => (left, Some(removed)),
                (Some(left), Some(right)) => {
                    let (right, successor) = take_min(right, events);
                    events.push(EventView::SuccessorSelected {
                        removed_id: node.id,
                        successor_id: successor.id.clone(),
                        removed,
                        successor: successor.value,
                    });
                    (
                        rebalance(
                            Box::new(Node {
                                value: successor.value,
                                id: successor.id,
                                height: 1,
                                left: Some(left),
                                right,
                            }),
                            events,
                        ),
                        Some(removed),
                    )
                }
            }
        }
    }
}
fn take_min(mut node: Box<Node>, events: &mut Vec<EventView>) -> (Option<Box<Node>>, Box<Node>) {
    match node.left.take() {
        None => (node.right.take(), node),
        Some(left) => {
            let (left, min) = take_min(left, events);
            node.left = left;
            (rebalance(node, events), min)
        }
    }
}
