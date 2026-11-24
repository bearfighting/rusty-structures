use crate::serialization::serialize;
use data_structures::{BPlusTreeMap, BPlusTreeSet, BPlusTreeSnapshot, BPlusTreeSnapshotNode};
use serde::Serialize;
use std::collections::{BTreeMap as OrderedMap, BTreeSet as OrderedSet};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TreeView {
    kind: &'static str,
    family: &'static str,
    mode: &'static str,
    root: Option<Box<NodeView>>,
    len: usize,
    height: usize,
    leaf_ids: Vec<String>,
}
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct NodeView {
    id: String,
    node_kind: &'static str,
    entries: Vec<EntryView>,
    children: Vec<NodeView>,
    next_id: Option<String>,
}
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct EntryView {
    id: String,
    key: i32,
    value: Option<i32>,
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
        key: i32,
        key_index: usize,
        ordering: &'static str,
    },
    LeafSplit {
        left_id: String,
        right_id: String,
        separator_key: i32,
    },
    InternalSplit {
        node_id: String,
        separator_key: i32,
    },
    LeafLinked {
        from_id: String,
        to_id: Option<String>,
    },
    LeafMerged {
        removed_leaf_id: String,
        into_leaf_id: String,
    },
    LeafBorrowed {
        from_leaf_id: String,
        to_leaf_id: String,
        key: i32,
    },
    SeparatorUpdated {
        node_id: String,
        key_index: usize,
        old_key: i32,
        new_key: i32,
    },
    Inserted {
        entry_id: String,
        key: i32,
        value: Option<i32>,
    },
    Updated {
        entry_id: String,
        key: i32,
        old_value: Option<i32>,
        value: Option<i32>,
    },
    Duplicate {
        key: i32,
    },
    Found {
        key: i32,
        value: Option<i32>,
    },
    Missing {
        key: i32,
    },
    Removed {
        entry_id: String,
        key: i32,
        value: Option<i32>,
    },
    ScanEntry {
        entry_id: String,
        key: i32,
        value: Option<i32>,
    },
}

#[wasm_bindgen]
pub struct BPlusSession {
    map: BPlusTreeMap<i32, i32>,
    node_ids: OrderedMap<usize, String>,
    entry_ids: OrderedMap<i32, String>,
    next_node: u64,
}
#[wasm_bindgen]
impl BPlusSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            map: BPlusTreeMap::new(),
            node_ids: OrderedMap::new(),
            entry_ids: OrderedMap::new(),
            next_node: 0,
        }
    }
    pub fn snapshot(&mut self) -> Result<JsValue, JsValue> {
        let snap = self.map.visualization_snapshot();
        let view = self.view(&snap, "map");
        serialize(&view)
    }
    pub fn reset(&mut self) -> Result<JsValue, JsValue> {
        self.map.clear();
        self.node_ids.clear();
        self.entry_ids.clear();
        self.snapshot()
    }
    pub fn load_example(&mut self) -> Result<JsValue, JsValue> {
        self.map.clear();
        self.node_ids.clear();
        self.entry_ids.clear();
        for (k, v) in [
            (40, 400),
            (10, 100),
            (70, 700),
            (20, 200),
            (30, 300),
            (50, 500),
            (60, 600),
            (80, 800),
            (90, 900),
            (15, 150),
        ] {
            self.map.insert(k, v);
            self.entry_id(k);
        }
        self.snapshot()
    }
    pub fn insert(&mut self, key: i32, value: i32) -> Result<JsValue, JsValue> {
        let before_snap = self.map.visualization_snapshot();
        let before = self.view(&before_snap, "map");
        let mut events = self.search_events(&before_snap, key);
        let old = self.map.insert(key, value);
        let after_snap = self.map.visualization_snapshot();
        self.verify(&after_snap)?;
        self.sync_node_ids(&after_snap);
        let after = self.view(&after_snap, "map");
        events.extend(diff_structure(&before, &after));
        let id = self.entry_id(key);
        if let Some(old_value) = old {
            events.push(EventView::Updated {
                entry_id: id,
                key,
                old_value: Some(old_value),
                value: Some(value),
            });
        } else {
            events.push(EventView::Inserted {
                entry_id: id,
                key,
                value: Some(value),
            });
        }
        serialize(&TraceView {
            before,
            after,
            events,
        })
    }
    pub fn get(&mut self, key: i32) -> Result<JsValue, JsValue> {
        let snap = self.map.visualization_snapshot();
        let before = self.view(&snap, "map");
        let mut events = self.search_events(&snap, key);
        match self.map.get(&key) {
            Some(value) => events.push(EventView::Found {
                key,
                value: Some(*value),
            }),
            None => events.push(EventView::Missing { key }),
        };
        let after = self.view(&snap, "map");
        serialize(&TraceView {
            before,
            after,
            events,
        })
    }
    pub fn remove(&mut self, key: i32) -> Result<JsValue, JsValue> {
        let before_snap = self.map.visualization_snapshot();
        let before = self.view(&before_snap, "map");
        let mut events = self.search_events(&before_snap, key);
        let removed = self.map.remove(&key);
        let after_snap = self.map.visualization_snapshot();
        self.verify(&after_snap)?;
        self.sync_node_ids(&after_snap);
        let after = self.view(&after_snap, "map");
        events.extend(diff_structure(&before, &after));
        if let Some(value) = removed {
            events.push(EventView::Removed {
                entry_id: self.entry_id(key),
                key,
                value: Some(value),
            });
        } else {
            events.push(EventView::Missing { key });
        }
        serialize(&TraceView {
            before,
            after,
            events,
        })
    }
    pub fn scan(&mut self) -> Result<JsValue, JsValue> {
        let snap = self.map.visualization_snapshot();
        let before = self.view(&snap, "map");
        let pairs: Vec<_> = self.map.iter().map(|(key, value)| (*key, *value)).collect();
        let events = pairs
            .into_iter()
            .map(|(key, value)| EventView::ScanEntry {
                entry_id: self.entry_id(key),
                key,
                value: Some(value),
            })
            .collect();
        let after = self.view(&snap, "map");
        serialize(&TraceView {
            before,
            after,
            events,
        })
    }
}
impl Default for BPlusSession {
    fn default() -> Self {
        Self::new()
    }
}
impl BPlusSession {
    fn verify(&self, s: &BPlusTreeSnapshot<i32, i32>) -> Result<(), JsValue> {
        let mut leaves = Vec::new();
        let mut expected_height = 0;
        if let Some(root) = s.root {
            fn check(
                id: usize,
                s: &BPlusTreeSnapshot<i32, i32>,
                depth: usize,
                is_root: bool,
                leaf_depth: &mut Option<usize>,
                leaves: &mut Vec<usize>,
            ) -> Result<Option<i32>, JsValue> {
                let node = s
                    .nodes
                    .iter()
                    .find(|n| match n {
                        BPlusTreeSnapshotNode::Leaf { id: n, .. }
                        | BPlusTreeSnapshotNode::Internal { id: n, .. } => *n == id,
                    })
                    .ok_or_else(|| JsValue::from_str("B+ snapshot references a missing node"))?;
                match node {
                    BPlusTreeSnapshotNode::Leaf { entries, .. } => {
                        if entries.len() > 3 || (!is_root && entries.len() < 2) {
                            return Err(JsValue::from_str("B+ leaf occupancy invariant failed"));
                        }
                        if let Some(expected) = leaf_depth {
                            if *expected != depth {
                                return Err(JsValue::from_str("B+ leaves have different depths"));
                            }
                        } else {
                            *leaf_depth = Some(depth);
                        }
                        leaves.push(id);
                        Ok(entries.first().map(|(key, _)| *key))
                    }
                    BPlusTreeSnapshotNode::Internal {
                        separators,
                        children,
                        ..
                    } => {
                        if children.len() > 4
                            || children.len() != separators.len() + 1
                            || (!is_root && children.len() < 2)
                            || (is_root && children.len() < 2)
                        {
                            return Err(JsValue::from_str(
                                "B+ internal occupancy invariant failed",
                            ));
                        }
                        let mut first = None;
                        let mut child_first = Vec::new();
                        for child in children {
                            let value = check(*child, s, depth + 1, false, leaf_depth, leaves)?;
                            if first.is_none() {
                                first = value;
                            }
                            child_first.push(value);
                        }
                        for (i, sep) in separators.iter().enumerate() {
                            if child_first[i + 1] != Some(*sep) {
                                return Err(JsValue::from_str(
                                    "B+ separator does not match right child minimum",
                                ));
                            }
                        }
                        Ok(first)
                    }
                }
            }
            let mut leaf_depth = None;
            check(root, s, 1, true, &mut leaf_depth, &mut leaves)?;
            expected_height = leaf_depth.unwrap_or(0);
        }
        let mut chain = Vec::new();
        let mut id = s.first_leaf;
        while let Some(leaf_id) = id {
            let Some(BPlusTreeSnapshotNode::Leaf { entries, next, .. }) = s
                .nodes
                .iter()
                .find(|n| matches!(n,BPlusTreeSnapshotNode::Leaf{id,..}if *id==leaf_id))
            else {
                return Err(JsValue::from_str("B+ leaf chain points to a non-leaf"));
            };
            chain.extend(entries.iter().map(|(k, v)| (*k, *v)));
            id = *next;
        }
        let expected: Vec<_> = self.map.iter().map(|(k, v)| (*k, *v)).collect();
        if chain != expected
            || expected_height != s.height
            || leaves.len()
                != s.nodes
                    .iter()
                    .filter(|n| matches!(n, BPlusTreeSnapshotNode::Leaf { .. }))
                    .count()
        {
            return Err(JsValue::from_str(
                "B+ visual snapshot does not match map order or height",
            ));
        }
        Ok(())
    }
    fn entry_id(&mut self, key: i32) -> String {
        self.entry_ids
            .entry(key)
            .or_insert_with(|| format!("bplus-entry-{key}"))
            .clone()
    }
    fn sync_node_ids<K, V>(&mut self, s: &BPlusTreeSnapshot<K, V>) {
        let active: OrderedSet<_> = s
            .nodes
            .iter()
            .map(|n| match n {
                BPlusTreeSnapshotNode::Leaf { id, .. }
                | BPlusTreeSnapshotNode::Internal { id, .. } => *id,
            })
            .collect();
        self.node_ids.retain(|id, _| active.contains(id));
        for id in active {
            self.node_ids.entry(id).or_insert_with(|| {
                let name = format!("bplus-node-{}", self.next_node);
                self.next_node += 1;
                name
            });
        }
    }
    fn view(&mut self, s: &BPlusTreeSnapshot<i32, i32>, mode: &'static str) -> TreeView {
        self.sync_node_ids(s);
        let root = s
            .root
            .map(|id| snapshot_node(id, s, &self.node_ids, &self.entry_ids, mode))
            .transpose()
            .unwrap();
        let mut leaf_ids = Vec::new();
        let mut leaf = s.first_leaf;
        while let Some(id) = leaf {
            leaf_ids.push(self.node_ids[&id].clone());
            leaf = s.nodes.iter().find_map(|node| match node {
                BPlusTreeSnapshotNode::Leaf {
                    id: node_id, next, ..
                } if *node_id == id => *next,
                _ => None,
            });
        }
        TreeView {
            kind: "multiwayTree",
            family: "bPlus",
            mode,
            root,
            len: s.len,
            height: s.height,
            leaf_ids,
        }
    }
    fn search_events(&mut self, s: &BPlusTreeSnapshot<i32, i32>, key: i32) -> Vec<EventView> {
        let mut events = Vec::new();
        let mut id = s.root;
        while let Some(node_id) = id {
            match s
                .nodes
                .iter()
                .find(|node| matches!(node,BPlusTreeSnapshotNode::Internal{id,..}if *id==node_id))
            {
                Some(BPlusTreeSnapshotNode::Internal {
                    separators,
                    children,
                    ..
                }) => {
                    let mut index = 0;
                    for sep in separators {
                        let ordering = if key < *sep { "less" } else { "greater" };
                        let key_index = index;
                        events.push(EventView::Compared {
                            node_id: self.node_ids[&node_id].clone(),
                            key,
                            key_index,
                            ordering,
                        });
                        if key < *sep {
                            break;
                        }
                        index += 1;
                    }
                    id = Some(children[index]);
                }
                _ => {
                    let Some(BPlusTreeSnapshotNode::Leaf { entries, .. }) = s
                        .nodes
                        .iter()
                        .find(|n| matches!(n,BPlusTreeSnapshotNode::Leaf{id,..}if *id==node_id))
                    else {
                        break;
                    };
                    match entries.binary_search_by_key(&key, |(k, _)| *k) {
                        Ok(i) => events.push(EventView::Compared {
                            node_id: self.node_ids[&node_id].clone(),
                            key,
                            key_index: i,
                            ordering: "equal",
                        }),
                        Err(i) => events.push(EventView::Compared {
                            node_id: self.node_ids[&node_id].clone(),
                            key,
                            key_index: i,
                            ordering: if i < entries.len() { "less" } else { "greater" },
                        }),
                    }
                    id = None;
                }
            }
        }
        events
    }
}
fn snapshot_node(
    sid: usize,
    s: &BPlusTreeSnapshot<i32, i32>,
    ids: &OrderedMap<usize, String>,
    entry_ids: &OrderedMap<i32, String>,
    mode: &str,
) -> Result<Box<NodeView>, JsValue> {
    let id = ids
        .get(&sid)
        .cloned()
        .ok_or_else(|| JsValue::from_str("Missing B+ node visual ID"))?;
    let node =
        s.nodes
            .iter()
            .find(|n| match n {
                BPlusTreeSnapshotNode::Leaf { id, .. }
                | BPlusTreeSnapshotNode::Internal { id, .. } => *id == sid,
            })
            .ok_or_else(|| JsValue::from_str("Missing B+ node snapshot"))?;
    match node {
        BPlusTreeSnapshotNode::Leaf { entries, next, .. } => Ok(Box::new(NodeView {
            id,
            node_kind: "leaf",
            entries: entries
                .iter()
                .map(|(key, value)| EntryView {
                    id: entry_ids
                        .get(key)
                        .cloned()
                        .unwrap_or_else(|| format!("bplus-entry-{key}")),
                    key: *key,
                    value: if mode == "map" { Some(*value) } else { None },
                })
                .collect(),
            children: Vec::new(),
            next_id: next.map(|next| ids[&next].clone()),
        })),
        BPlusTreeSnapshotNode::Internal {
            separators,
            children,
            ..
        } => Ok(Box::new(NodeView {
            id,
            node_kind: "internal",
            entries: separators
                .iter()
                .map(|key| EntryView {
                    id: format!("bplus-separator-{key}"),
                    key: *key,
                    value: None,
                })
                .collect(),
            children: children
                .iter()
                .map(|child| *snapshot_node(*child, s, ids, entry_ids, mode).unwrap())
                .collect(),
            next_id: None,
        })),
    }
}
fn flatten(root: &Option<Box<NodeView>>, out: &mut Vec<NodeView>) {
    fn visit(n: &NodeView, out: &mut Vec<NodeView>) {
        out.push(n.clone());
        for child in &n.children {
            visit(child, out);
        }
    }
    if let Some(n) = root {
        visit(n, out);
    }
}
fn diff_structure(before: &TreeView, after: &TreeView) -> Vec<EventView> {
    let mut a = Vec::new();
    let mut b = Vec::new();
    flatten(&before.root, &mut a);
    flatten(&after.root, &mut b);
    let old: OrderedMap<_, _> = a.iter().map(|n| (n.id.clone(), n)).collect();
    let new: OrderedMap<_, _> = b.iter().map(|n| (n.id.clone(), n)).collect();
    let mut events = Vec::new();
    for n in &b {
        if !old.contains_key(&n.id) {
            if n.node_kind == "leaf" {
                let position = after
                    .leaf_ids
                    .iter()
                    .position(|id| id == &n.id)
                    .unwrap_or(0);
                events.push(EventView::LeafSplit {
                    left_id: position
                        .checked_sub(1)
                        .and_then(|i| after.leaf_ids.get(i))
                        .cloned()
                        .unwrap_or_default(),
                    right_id: n.id.clone(),
                    separator_key: n.entries.first().map_or(0, |e| e.key),
                });
            } else {
                events.push(EventView::InternalSplit {
                    node_id: n.id.clone(),
                    separator_key: n.entries.first().map_or(0, |e| e.key),
                });
            }
        }
        if let Some(prev) = old.get(&n.id) {
            if n.node_kind == "internal" {
                for (i, (x, y)) in prev.entries.iter().zip(&n.entries).enumerate() {
                    if x.key != y.key {
                        events.push(EventView::SeparatorUpdated {
                            node_id: n.id.clone(),
                            key_index: i,
                            old_key: x.key,
                            new_key: y.key,
                        });
                    }
                }
            }
            if n.node_kind == "leaf" {
                let prior: OrderedSet<_> = prev.entries.iter().map(|e| e.key).collect();
                let curr: OrderedSet<_> = n.entries.iter().map(|e| e.key).collect();
                if let Some(k) = curr.difference(&prior).next() {
                    if let Some(from) = a.iter().find(|p| {
                        p.id != n.id
                            && p.node_kind == "leaf"
                            && p.entries.iter().any(|e| e.key == *k)
                    }) {
                        events.push(EventView::LeafBorrowed {
                            from_leaf_id: from.id.clone(),
                            to_leaf_id: n.id.clone(),
                            key: *k,
                        });
                    }
                }
            }
        }
    }
    for n in &a {
        if n.node_kind == "leaf" && !new.contains_key(&n.id) {
            let old_position = before
                .leaf_ids
                .iter()
                .position(|id| id == &n.id)
                .unwrap_or(0);
            let into = old_position
                .checked_sub(1)
                .and_then(|i| after.leaf_ids.get(i))
                .or_else(|| after.leaf_ids.first())
                .cloned()
                .unwrap_or_default();
            events.push(EventView::LeafMerged {
                removed_leaf_id: n.id.clone(),
                into_leaf_id: into,
            });
        }
    }
    for n in &b {
        if n.node_kind == "leaf" && old.get(&n.id).is_none_or(|prev| prev.next_id != n.next_id) {
            events.push(EventView::LeafLinked {
                from_id: n.id.clone(),
                to_id: n.next_id.clone(),
            });
        }
    }
    events
}

#[wasm_bindgen]
pub struct BPlusSetSession {
    set: BPlusTreeSet<i32>,
    model: BPlusSession,
}
#[wasm_bindgen]
impl BPlusSetSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            set: BPlusTreeSet::new(),
            model: BPlusSession::new(),
        }
    }
    pub fn snapshot(&mut self) -> Result<JsValue, JsValue> {
        let snap = self.model.map.visualization_snapshot();
        serialize(&self.model.view(&snap, "set"))
    }
    pub fn reset(&mut self) -> Result<JsValue, JsValue> {
        self.set.clear();
        self.model.map.clear();
        self.model.node_ids.clear();
        let snap = self.model.map.visualization_snapshot();
        self.model.verify(&snap)?;
        serialize(&self.model.view(&snap, "set"))
    }
    pub fn load_example(&mut self) -> Result<JsValue, JsValue> {
        self.set.clear();
        self.model.map.clear();
        self.model.node_ids.clear();
        for key in [40, 10, 70, 20, 30, 50, 60, 80, 90, 15] {
            self.set.insert(key);
            self.model.map.insert(key, 0);
            self.model.entry_id(key);
        }
        let snap = self.model.map.visualization_snapshot();
        self.model.verify(&snap)?;
        serialize(&self.model.view(&snap, "set"))
    }
    pub fn insert(&mut self, key: i32) -> Result<JsValue, JsValue> {
        let before_snap = self.model.map.visualization_snapshot();
        let before = self.model.view(&before_snap, "set");
        let mut events = self.model.search_events(&before_snap, key);
        let inserted = self.set.insert(key);
        let old = self.model.map.insert(key, 0);
        let after_snap = self.model.map.visualization_snapshot();
        self.model.verify(&after_snap)?;
        if self.set.iter().copied().collect::<Vec<_>>()
            != self.model.map.keys().copied().collect::<Vec<_>>()
        {
            return Err(JsValue::from_str(
                "BPlusTreeSet diverged from its Rust model.",
            ));
        }
        let after = self.model.view(&after_snap, "set");
        events.extend(diff_structure(&before, &after));
        if inserted {
            events.push(EventView::Inserted {
                entry_id: self.model.entry_id(key),
                key,
                value: None,
            });
        } else {
            events.push(EventView::Duplicate { key });
        }
        let _ = old;
        serialize(&TraceView {
            before,
            after,
            events,
        })
    }
    pub fn get(&mut self, key: i32) -> Result<JsValue, JsValue> {
        let snap = self.model.map.visualization_snapshot();
        let before = self.model.view(&snap, "set");
        let mut events = self.model.search_events(&snap, key);
        if self.set.contains(&key) {
            events.push(EventView::Found { key, value: None });
        } else {
            events.push(EventView::Missing { key });
        }
        serialize(&TraceView {
            before: before.clone(),
            after: before,
            events,
        })
    }
    pub fn remove(&mut self, key: i32) -> Result<JsValue, JsValue> {
        let before_snap = self.model.map.visualization_snapshot();
        let before = self.model.view(&before_snap, "set");
        let mut events = self.model.search_events(&before_snap, key);
        let removed = self.set.remove(&key);
        self.model.map.remove(&key);
        let after_snap = self.model.map.visualization_snapshot();
        self.model.verify(&after_snap)?;
        self.model.node_ids.retain(|id, _| {
            after_snap.nodes.iter().any(|n| match n {
                BPlusTreeSnapshotNode::Leaf { id: n, .. }
                | BPlusTreeSnapshotNode::Internal { id: n, .. } => n == id,
            })
        });
        let after = self.model.view(&after_snap, "set");
        if self.set.iter().copied().collect::<Vec<_>>()
            != self.model.map.keys().copied().collect::<Vec<_>>()
        {
            return Err(JsValue::from_str(
                "BPlusTreeSet diverged from its Rust model.",
            ));
        }
        events.extend(diff_structure(&before, &after));
        if removed {
            events.push(EventView::Removed {
                entry_id: self.model.entry_id(key),
                key,
                value: None,
            });
        } else {
            events.push(EventView::Missing { key });
        }
        serialize(&TraceView {
            before,
            after,
            events,
        })
    }
    pub fn scan(&mut self) -> Result<JsValue, JsValue> {
        let snap = self.model.map.visualization_snapshot();
        let before = self.model.view(&snap, "set");
        let events = self
            .set
            .iter()
            .map(|key| EventView::ScanEntry {
                entry_id: format!("bplus-entry-{key}"),
                key: *key,
                value: None,
            })
            .collect();
        serialize(&TraceView {
            before: before.clone(),
            after: before,
            events,
        })
    }
}
impl Default for BPlusSetSession {
    fn default() -> Self {
        Self::new()
    }
}
