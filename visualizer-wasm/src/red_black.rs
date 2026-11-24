use crate::serialization::serialize;
use data_structures::RedBlackTree;
use serde::Serialize;
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum Color {
    Red,
    Black,
}
#[derive(Clone)]
struct Node {
    value: i32,
    id: String,
    color: Color,
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
    color: &'static str,
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
    Rotated {
        direction: &'static str,
        root_id: String,
        pivot_id: String,
    },
    ColorsFlipped {
        node_id: String,
        child_ids: Vec<String>,
    },
    RedLinkMoved {
        direction: &'static str,
        node_id: String,
    },
    Inserted {
        node_id: String,
        key: i32,
    },
    Duplicate {
        key: i32,
    },
    SuccessorSelected {
        removed_id: String,
        successor_id: String,
        removed: i32,
        successor: i32,
    },
    Found {
        node_id: String,
        key: i32,
    },
    Missing {
        key: i32,
    },
    Removed {
        node_id: String,
        key: i32,
    },
}

#[wasm_bindgen]
pub struct RedBlackSession {
    tree: RedBlackTree<i32>,
    root: Option<Box<Node>>,
    ids: BTreeMap<i32, String>,
    next_id: u64,
}
#[wasm_bindgen]
impl RedBlackSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            tree: RedBlackTree::new(),
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
        for v in [8, 3, 10, 1, 6, 14, 4, 7, 13] {
            let id = self.alloc(v);
            self.tree.insert(v);
            self.root = insert(self.root.take(), v, id, &mut Vec::new());
            if let Some(r) = self.root.as_mut() {
                r.color = Color::Black;
            }
        }
        serialize(&self.view())
    }
    pub fn insert(&mut self, key: i32) -> Result<JsValue, JsValue> {
        let before = self.view();
        let mut events = Vec::new();
        let id = if let Some(id) = self.ids.get(&key) {
            id.clone()
        } else {
            self.alloc(key)
        };
        self.root = insert(self.root.take(), key, id, &mut events);
        let inserted = self.tree.insert(key);
        if let Some(r) = self.root.as_mut() {
            if r.color == Color::Red {
                r.color = Color::Black;
                events.push(EventView::ColorsFlipped {
                    node_id: r.id.clone(),
                    child_ids: vec![],
                });
            }
        }
        if inserted {
            events.push(EventView::Inserted {
                node_id: self.ids[&key].clone(),
                key,
            });
        } else {
            events.push(EventView::Duplicate { key });
        }
        self.verify()?;
        self.trace(before, events)
    }
    pub fn search(&self, key: i32) -> Result<JsValue, JsValue> {
        let before = self.view();
        let mut events = Vec::new();
        let mut n = self.root.as_deref();
        while let Some(cur) = n {
            let o = key.cmp(&cur.value);
            events.push(EventView::Compared {
                node_id: cur.id.clone(),
                node_value: cur.value,
                key,
                ordering: ord(o),
            });
            match o {
                std::cmp::Ordering::Less => n = cur.left.as_deref(),
                std::cmp::Ordering::Greater => n = cur.right.as_deref(),
                std::cmp::Ordering::Equal => {
                    events.push(EventView::Found {
                        node_id: cur.id.clone(),
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
        let mut root = self.root.take();
        if let Some(r) = root.as_mut() {
            if !red(&r.left) && !red(&r.right) {
                set_color(r, Color::Red, &mut events);
            }
        }
        let (mut root, removed) = match root.take() {
            Some(r) => remove_at(r, key, &mut events),
            None => (None, None),
        };
        if let Some(r) = root.as_mut() {
            if r.color == Color::Red {
                set_color(r, Color::Black, &mut events);
            }
        }
        self.root = root;
        let actual = self.tree.remove(&key);
        if actual != removed {
            return Err(JsValue::from_str(
                "Red-black visual model diverged from Rust tree.",
            ));
        }
        if removed.is_some() {
            if let Some(id) = self.ids.remove(&key) {
                events.push(EventView::Removed { node_id: id, key });
            }
        } else {
            events.push(EventView::Missing { key });
        }
        self.verify()?;
        self.trace(before, events)
    }
}
impl Default for RedBlackSession {
    fn default() -> Self {
        Self::new()
    }
}
impl RedBlackSession {
    fn alloc(&mut self, v: i32) -> String {
        let id = format!("rb-node-{}", self.next_id);
        self.next_id += 1;
        self.ids.insert(v, id.clone());
        id
    }
    fn view(&self) -> TreeView {
        TreeView {
            kind: "binaryTree",
            root: view(&self.root),
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
    fn verify(&self) -> Result<(), JsValue> {
        if self.tree.iter().copied().collect::<Vec<_>>() != inorder(&self.root)
            || !valid(&self.root, true).0
        {
            return Err(JsValue::from_str(
                "Red-black visual model diverged from Rust tree.",
            ));
        }
        Ok(())
    }
}
fn ord(o: std::cmp::Ordering) -> &'static str {
    match o {
        std::cmp::Ordering::Less => "less",
        std::cmp::Ordering::Equal => "equal",
        std::cmp::Ordering::Greater => "greater",
    }
}
fn red(n: &Option<Box<Node>>) -> bool {
    n.as_ref().is_some_and(|x| x.color == Color::Red)
}
fn left_red(n: &Option<Box<Node>>) -> bool {
    n.as_ref().is_some_and(|x| red(&x.left))
}
fn opposite(c: Color) -> Color {
    if c == Color::Red {
        Color::Black
    } else {
        Color::Red
    }
}
fn set_color(n: &mut Node, c: Color, ev: &mut Vec<EventView>) {
    if n.color == c {
        return;
    }
    n.color = c;
    ev.push(EventView::ColorsFlipped {
        node_id: n.id.clone(),
        child_ids: vec![],
    });
}
fn flip(n: &mut Node, ev: &mut Vec<EventView>) {
    n.color = opposite(n.color);
    let mut ids = Vec::new();
    if let Some(l) = n.left.as_mut() {
        l.color = opposite(l.color);
        ids.push(l.id.clone());
    }
    if let Some(r) = n.right.as_mut() {
        r.color = opposite(r.color);
        ids.push(r.id.clone());
    }
    ev.push(EventView::ColorsFlipped {
        node_id: n.id.clone(),
        child_ids: ids,
    });
}
fn rotate_left(mut n: Box<Node>, ev: &mut Vec<EventView>) -> Box<Node> {
    let mut p = n.right.take().expect("left rotation child");
    let root = n.id.clone();
    let pivot = p.id.clone();
    n.right = p.left.take();
    p.color = n.color;
    n.color = Color::Red;
    p.left = Some(n);
    ev.push(EventView::Rotated {
        direction: "left",
        root_id: root,
        pivot_id: pivot,
    });
    p
}
fn rotate_right(mut n: Box<Node>, ev: &mut Vec<EventView>) -> Box<Node> {
    let mut p = n.left.take().expect("right rotation child");
    let root = n.id.clone();
    let pivot = p.id.clone();
    n.left = p.right.take();
    p.color = n.color;
    n.color = Color::Red;
    p.right = Some(n);
    ev.push(EventView::Rotated {
        direction: "right",
        root_id: root,
        pivot_id: pivot,
    });
    p
}
fn fix(mut n: Box<Node>, ev: &mut Vec<EventView>) -> Box<Node> {
    if red(&n.right) && !red(&n.left) {
        n = rotate_left(n, ev);
    }
    if red(&n.left) && left_red(&n.left) {
        n = rotate_right(n, ev);
    }
    if red(&n.left) && red(&n.right) {
        flip(&mut n, ev);
    }
    n
}
fn insert(
    root: Option<Box<Node>>,
    v: i32,
    id: String,
    ev: &mut Vec<EventView>,
) -> Option<Box<Node>> {
    let Some(mut n) = root else {
        return Some(Box::new(Node {
            value: v,
            id,
            color: Color::Red,
            left: None,
            right: None,
        }));
    };
    match v.cmp(&n.value) {
        std::cmp::Ordering::Less => {
            ev.push(EventView::Compared {
                node_id: n.id.clone(),
                node_value: n.value,
                key: v,
                ordering: "less",
            });
            n.left = insert(n.left.take(), v, id, ev);
        }
        std::cmp::Ordering::Greater => {
            ev.push(EventView::Compared {
                node_id: n.id.clone(),
                node_value: n.value,
                key: v,
                ordering: "greater",
            });
            n.right = insert(n.right.take(), v, id, ev);
        }
        std::cmp::Ordering::Equal => return Some(n),
    }
    Some(fix(n, ev))
}
fn move_left(mut n: Box<Node>, ev: &mut Vec<EventView>) -> Box<Node> {
    ev.push(EventView::RedLinkMoved {
        direction: "left",
        node_id: n.id.clone(),
    });
    flip(&mut n, ev);
    if left_red(&n.right) {
        let r = n.right.take().expect("right exists");
        n.right = Some(rotate_right(r, ev));
        n = rotate_left(n, ev);
        flip(&mut n, ev);
    }
    n
}
fn move_right(mut n: Box<Node>, ev: &mut Vec<EventView>) -> Box<Node> {
    ev.push(EventView::RedLinkMoved {
        direction: "right",
        node_id: n.id.clone(),
    });
    flip(&mut n, ev);
    if left_red(&n.left) {
        n = rotate_right(n, ev);
        flip(&mut n, ev);
    }
    n
}
fn take_min(mut n: Box<Node>, ev: &mut Vec<EventView>) -> (Option<Box<Node>>, Box<Node>) {
    if n.left.is_none() {
        return (n.right.take(), n);
    }
    if !red(&n.left) && !left_red(&n.left) {
        n = move_left(n, ev);
    }
    let l = n.left.take().expect("left branch");
    let (l, min) = take_min(l, ev);
    n.left = l;
    (Some(fix(n, ev)), min)
}
fn remove_at(
    mut n: Box<Node>,
    key: i32,
    ev: &mut Vec<EventView>,
) -> (Option<Box<Node>>, Option<i32>) {
    let mut removed = None;
    if key < n.value {
        ev.push(EventView::Compared {
            node_id: n.id.clone(),
            node_value: n.value,
            key,
            ordering: "less",
        });
        if n.left.is_some() {
            if !red(&n.left) && !left_red(&n.left) {
                n = move_left(n, ev);
            }
            let l = n.left.take().expect("left branch");
            let (l, v) = remove_at(l, key, ev);
            n.left = l;
            removed = v;
        }
    } else {
        ev.push(EventView::Compared {
            node_id: n.id.clone(),
            node_value: n.value,
            key,
            ordering: ord(key.cmp(&n.value)),
        });
        if red(&n.left) {
            n = rotate_right(n, ev);
        }
        if key == n.value && n.right.is_none() {
            return (None, Some(n.value));
        }
        if n.right.is_some() {
            if !red(&n.right) && !left_red(&n.right) {
                n = move_right(n, ev);
            }
            if key == n.value {
                let r = n.right.take().expect("right branch");
                let (r, s) = take_min(r, ev);
                n.right = r;
                let old = std::mem::replace(&mut n.value, s.value);
                let old_id = std::mem::replace(&mut n.id, s.id);
                ev.push(EventView::SuccessorSelected {
                    removed_id: old_id,
                    successor_id: n.id.clone(),
                    removed: old,
                    successor: n.value,
                });
                removed = Some(old);
            } else {
                let r = n.right.take().expect("right branch");
                let (r, v) = remove_at(r, key, ev);
                n.right = r;
                removed = v;
            }
        }
    }
    (Some(fix(n, ev)), removed)
}
fn view(n: &Option<Box<Node>>) -> Option<Box<NodeView>> {
    n.as_ref().map(|x| {
        Box::new(NodeView {
            id: x.id.clone(),
            value: x.value,
            color: if x.color == Color::Red {
                "red"
            } else {
                "black"
            },
            left: view(&x.left),
            right: view(&x.right),
        })
    })
}
fn inorder(n: &Option<Box<Node>>) -> Vec<i32> {
    fn go(n: &Option<Box<Node>>, v: &mut Vec<i32>) {
        if let Some(x) = n {
            go(&x.left, v);
            v.push(x.value);
            go(&x.right, v);
        }
    }
    let mut v = Vec::new();
    go(n, &mut v);
    v
}
fn valid(n: &Option<Box<Node>>, root: bool) -> (bool, usize) {
    let Some(x) = n else { return (true, 1) };
    let (l, lb) = valid(&x.left, false);
    let (r, rb) = valid(&x.right, false);
    let ok = l
        && r
        && lb == rb
        && !(x.color == Color::Red && root)
        && !red(&x.right)
        && !(x.color == Color::Red && red(&x.left));
    (ok, lb + usize::from(x.color == Color::Black))
}
