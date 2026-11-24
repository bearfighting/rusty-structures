use crate::serialization::serialize;
use data_structures::BTreeMap;
use data_structures::BTreeSet;
use serde::Serialize;
use wasm_bindgen::prelude::*;

const MIN_DEGREE: usize = 2;
const MAX_KEYS: usize = 3;

#[derive(Clone)]
struct Entry {
    key: i32,
    value: i32,
    id: String,
}
#[derive(Clone)]
struct Node {
    id: String,
    entries: Vec<Entry>,
    children: Vec<Box<Node>>,
}
impl Node {
    fn leaf(id: String) -> Self {
        Self {
            id,
            entries: Vec::new(),
            children: Vec::new(),
        }
    }
    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TreeView {
    kind: &'static str,
    family: &'static str,
    mode: &'static str,
    root: Option<Box<NodeView>>,
    len: usize,
    height: usize,
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
    Split {
        parent_id: String,
        left_id: String,
        right_id: String,
        promoted_key: i32,
    },
    Borrowed {
        parent_id: String,
        child_id: String,
        sibling_id: String,
        direction: &'static str,
        key: i32,
    },
    Merged {
        parent_id: String,
        left_id: String,
        right_id: String,
        separator_key: i32,
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
    ReplacementSelected {
        key: i32,
        replacement: i32,
        direction: &'static str,
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
}

#[wasm_bindgen]
pub struct BTreeSession {
    map: BTreeMap<i32, i32>,
    root: Option<Box<Node>>,
    next_node: u64,
    next_entry: u64,
}
#[wasm_bindgen]
impl BTreeSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            root: None,
            next_node: 0,
            next_entry: 0,
        }
    }
    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        serialize(&self.view())
    }
    pub fn reset(&mut self) -> Result<JsValue, JsValue> {
        self.map.clear();
        self.root = None;
        serialize(&self.view())
    }
    pub fn load_example(&mut self) -> Result<JsValue, JsValue> {
        self.map.clear();
        self.root = None;
        for (k, v) in [
            (50, 500),
            (20, 200),
            (70, 700),
            (10, 100),
            (30, 300),
            (60, 600),
            (80, 800),
            (25, 250),
            (35, 350),
            (65, 650),
            (75, 750),
        ] {
            self.insert_inner(k, v, &mut Vec::new());
        }
        serialize(&self.view())
    }
    pub fn insert(&mut self, key: i32, value: i32) -> Result<JsValue, JsValue> {
        let before = self.view();
        let mut events = Vec::new();
        self.insert_inner(key, value, &mut events);
        self.verify()?;
        self.trace(before, events)
    }
    pub fn get(&self, key: i32) -> Result<JsValue, JsValue> {
        let before = self.view();
        let mut events = Vec::new();
        let mut node = self.root.as_deref();
        while let Some(n) = node {
            match n.entries.binary_search_by_key(&key, |e| e.key) {
                Ok(i) => {
                    events.push(EventView::Compared {
                        node_id: n.id.clone(),
                        key,
                        key_index: i,
                        ordering: "equal",
                    });
                    events.push(EventView::Found {
                        key,
                        value: Some(n.entries[i].value),
                    });
                    return self.trace(before, events);
                }
                Err(i) => {
                    events.push(EventView::Compared {
                        node_id: n.id.clone(),
                        key,
                        key_index: i,
                        ordering: if n.entries.get(i).is_some_and(|e| key < e.key) {
                            "less"
                        } else {
                            "greater"
                        },
                    });
                    if n.is_leaf() {
                        break;
                    }
                    node = Some(&n.children[i]);
                }
            }
        }
        events.push(EventView::Missing { key });
        self.trace(before, events)
    }
    pub fn remove(&mut self, key: i32) -> Result<JsValue, JsValue> {
        let before = self.view();
        let mut events = Vec::new();
        let removed = match self.root.as_mut() {
            Some(root) => remove_from(root, key, &mut events),
            None => None,
        };
        let actual = self.map.remove(&key);
        if actual != removed.as_ref().map(|e| e.value) {
            return Err(JsValue::from_str(
                "B-tree visual model diverged from Rust map.",
            ));
        }
        if removed.is_some() {
            if self.root.as_ref().is_some_and(|n| n.entries.is_empty()) {
                let mut root = self.root.take().expect("root");
                self.root = root.children.pop();
            }
            let e = removed.unwrap();
            events.push(EventView::Removed {
                entry_id: e.id,
                key: e.key,
                value: Some(e.value),
            });
        } else {
            events.push(EventView::Missing { key });
        }
        self.verify()?;
        self.trace(before, events)
    }
}
impl Default for BTreeSession {
    fn default() -> Self {
        Self::new()
    }
}
impl BTreeSession {
    fn node_id(&mut self) -> String {
        alloc_node_id(&mut self.next_node)
    }
    fn entry_id(&mut self) -> String {
        let id = format!("btree-entry-{}", self.next_entry);
        self.next_entry += 1;
        id
    }
    fn insert_inner(&mut self, key: i32, value: i32, events: &mut Vec<EventView>) {
        if let Some(old) = self.map.get(&key).copied() {
            let entry_id = find_entry(&self.root, key)
                .map(|e| e.id.clone())
                .unwrap_or_default();
            self.map.insert(key, value);
            if let Some(entry) = find_entry_mut(&mut self.root, key) {
                entry.value = value;
            }
            events.push(EventView::Updated {
                entry_id,
                key,
                old_value: Some(old),
                value: Some(value),
            });
            return;
        }
        let entry = Entry {
            key,
            value,
            id: self.entry_id(),
        };
        if self.root.is_none() {
            let mut root = Node::leaf(self.node_id());
            root.entries.push(entry.clone());
            self.root = Some(Box::new(root));
            self.map.insert(key, value);
            events.push(EventView::Inserted {
                entry_id: entry.id,
                key,
                value: Some(value),
            });
            return;
        }
        if self
            .root
            .as_ref()
            .is_some_and(|r| r.entries.len() == MAX_KEYS)
        {
            let old = self.root.take().unwrap();
            let mut root = Node::leaf(self.node_id());
            root.children.push(old);
            split_child(&mut root, 0, &mut self.next_node, events);
            self.root = Some(Box::new(root));
        }
        insert_nonfull(
            self.root.as_deref_mut().unwrap(),
            entry,
            &mut self.next_node,
            events,
        );
        self.map.insert(key, value);
        events.push(EventView::Inserted {
            entry_id: find_entry(&self.root, key).unwrap().id.clone(),
            key,
            value: Some(value),
        });
    }
    fn view(&self) -> TreeView {
        self.view_mode("map")
    }
    fn view_mode(&self, mode: &'static str) -> TreeView {
        TreeView {
            kind: "multiwayTree",
            family: "bTree",
            mode,
            root: node_view_mode(&self.root, mode == "map"),
            len: self.map.len(),
            height: height(&self.root),
        }
    }
    fn trace(&self, before: TreeView, events: Vec<EventView>) -> Result<JsValue, JsValue> {
        self.trace_mode(before, events, "map")
    }
    fn trace_mode(
        &self,
        before: TreeView,
        events: Vec<EventView>,
        mode: &'static str,
    ) -> Result<JsValue, JsValue> {
        serialize(&TraceView {
            before,
            after: self.view_mode(mode),
            events,
        })
    }
    fn verify(&self) -> Result<(), JsValue> {
        let mut pairs = Vec::new();
        inorder(&self.root, &mut pairs);
        let expected: Vec<_> = self.map.iter().map(|(k, v)| (*k, *v)).collect();
        if pairs != expected || !valid(&self.root, None, None, 0, &mut None) {
            return Err(JsValue::from_str(
                "B-tree visual model diverged from Rust map.",
            ));
        }
        Ok(())
    }
}
fn alloc_node_id(next: &mut u64) -> String {
    let id = format!("btree-node-{next}");
    *next += 1;
    id
}
fn split_child(p: &mut Node, i: usize, next: &mut u64, ev: &mut Vec<EventView>) {
    let right_id = alloc_node_id(next);
    let child = &mut p.children[i];
    let mut right = Box::new(Node {
        id: right_id.clone(),
        entries: child.entries.split_off(MIN_DEGREE),
        children: Vec::new(),
    });
    let median = child.entries.pop().unwrap();
    if !child.is_leaf() {
        right.children = child.children.split_off(MIN_DEGREE);
    }
    let left_id = child.id.clone();
    let key = median.key;
    p.entries.insert(i, median);
    p.children.insert(i + 1, right);
    ev.push(EventView::Split {
        parent_id: p.id.clone(),
        left_id,
        right_id,
        promoted_key: key,
    });
}
fn insert_nonfull(n: &mut Node, e: Entry, next: &mut u64, ev: &mut Vec<EventView>) {
    let i = n
        .entries
        .binary_search_by_key(&e.key, |x| x.key)
        .unwrap_err();
    let (key_index, ordering) = if i < n.entries.len() {
        (i, "less")
    } else {
        (n.entries.len().saturating_sub(1), "greater")
    };
    ev.push(EventView::Compared {
        node_id: n.id.clone(),
        key: e.key,
        key_index,
        ordering,
    });
    if n.is_leaf() {
        n.entries.insert(i, e);
        return;
    }
    if n.children[i].entries.len() == MAX_KEYS {
        split_child(n, i, next, ev);
        if e.key > n.entries[i].key {
            insert_nonfull(&mut n.children[i + 1], e, next, ev)
        } else {
            insert_nonfull(&mut n.children[i], e, next, ev)
        }
    } else {
        insert_nonfull(&mut n.children[i], e, next, ev)
    }
}
fn remove_from(n: &mut Node, key: i32, ev: &mut Vec<EventView>) -> Option<Entry> {
    let search = n.entries.binary_search_by_key(&key, |e| e.key);
    let (key_index, ordering) = match search {
        Ok(i) => (i, "equal"),
        Err(i) if i < n.entries.len() => (i, "less"),
        Err(i) if i > 0 => (i - 1, "greater"),
        Err(_) => (0, "greater"),
    };
    ev.push(EventView::Compared {
        node_id: n.id.clone(),
        key,
        key_index,
        ordering,
    });
    if let Ok(i) = search {
        if n.is_leaf() {
            return Some(n.entries.remove(i));
        }
        if n.children[i].entries.len() >= MIN_DEGREE {
            let pred = remove_max(&mut n.children[i], ev);
            let old = std::mem::replace(&mut n.entries[i], pred);
            ev.push(EventView::ReplacementSelected {
                key,
                replacement: n.entries[i].key,
                direction: "predecessor",
            });
            return Some(old);
        }
        if n.children[i + 1].entries.len() >= MIN_DEGREE {
            let succ = remove_min(&mut n.children[i + 1], ev);
            let old = std::mem::replace(&mut n.entries[i], succ);
            ev.push(EventView::ReplacementSelected {
                key,
                replacement: n.entries[i].key,
                direction: "successor",
            });
            return Some(old);
        }
        merge(n, i, ev);
        return remove_from(&mut n.children[i], key, ev);
    }
    let mut ci = search.unwrap_err();
    if n.is_leaf() {
        return None;
    }
    if n.children[ci].entries.len() == 1 {
        if ci > 0 && n.children[ci - 1].entries.len() >= MIN_DEGREE {
            borrow_prev(n, ci, ev)
        } else if ci < n.entries.len() && n.children[ci + 1].entries.len() >= MIN_DEGREE {
            borrow_next(n, ci, ev)
        } else if ci < n.entries.len() {
            merge(n, ci, ev)
        } else {
            merge(n, ci - 1, ev);
            ci -= 1;
        }
    }
    remove_from(&mut n.children[ci], key, ev)
}
fn remove_min(n: &mut Node, ev: &mut Vec<EventView>) -> Entry {
    if n.is_leaf() {
        return n.entries.remove(0);
    }
    if n.children[0].entries.len() == 1 {
        if n.children[1].entries.len() >= MIN_DEGREE {
            borrow_next(n, 0, ev)
        } else {
            merge(n, 0, ev)
        }
    }
    remove_min(&mut n.children[0], ev)
}
fn remove_max(n: &mut Node, ev: &mut Vec<EventView>) -> Entry {
    let last = n.entries.len() - 1;
    if n.is_leaf() {
        return n.entries.remove(last);
    }
    if n.children[last + 1].entries.len() == 1 {
        if n.children[last].entries.len() >= MIN_DEGREE {
            borrow_prev(n, last + 1, ev)
        } else {
            merge(n, last, ev)
        }
    }
    let i = n.children.len() - 1;
    remove_max(&mut n.children[i], ev)
}
fn borrow_prev(p: &mut Node, ci: usize, ev: &mut Vec<EventView>) {
    let sep = p.entries.remove(ci - 1);
    let (ls, cs) = p.children.split_at_mut(ci);
    let l = ls.last_mut().unwrap();
    let c = &mut cs[0];
    let key = sep.key;
    let lid = l.id.clone();
    let cid = c.id.clone();
    c.entries.insert(0, sep);
    p.entries.insert(ci - 1, l.entries.pop().unwrap());
    if !l.is_leaf() {
        c.children.insert(0, l.children.pop().unwrap());
    }
    ev.push(EventView::Borrowed {
        parent_id: p.id.clone(),
        child_id: cid,
        sibling_id: lid,
        direction: "left",
        key,
    });
}
fn borrow_next(p: &mut Node, ci: usize, ev: &mut Vec<EventView>) {
    let sep = p.entries.remove(ci);
    let (ls, rs) = p.children.split_at_mut(ci + 1);
    let c = ls.last_mut().unwrap();
    let r = &mut rs[0];
    let key = sep.key;
    let cid = c.id.clone();
    let rid = r.id.clone();
    c.entries.push(sep);
    p.entries.insert(ci, r.entries.remove(0));
    if !r.is_leaf() {
        c.children.push(r.children.remove(0));
    }
    ev.push(EventView::Borrowed {
        parent_id: p.id.clone(),
        child_id: cid,
        sibling_id: rid,
        direction: "right",
        key,
    });
}
fn merge(p: &mut Node, i: usize, ev: &mut Vec<EventView>) {
    let sep = p.entries.remove(i);
    let r = p.children.remove(i + 1);
    let l = &mut p.children[i];
    let key = sep.key;
    let lid = l.id.clone();
    let rid = r.id.clone();
    l.entries.push(sep);
    l.entries.extend(r.entries);
    l.children.extend(r.children);
    ev.push(EventView::Merged {
        parent_id: p.id.clone(),
        left_id: lid,
        right_id: rid,
        separator_key: key,
    });
}
fn find_entry(n: &Option<Box<Node>>, key: i32) -> Option<&Entry> {
    find_entry_node(n.as_deref()?, key)
}
fn find_entry_node(n: &Node, key: i32) -> Option<&Entry> {
    match n.entries.binary_search_by_key(&key, |e| e.key) {
        Ok(i) => Some(&n.entries[i]),
        Err(i) => {
            if n.is_leaf() {
                None
            } else {
                find_entry_node(&n.children[i], key)
            }
        }
    }
}
fn find_entry_mut(n: &mut Option<Box<Node>>, key: i32) -> Option<&mut Entry> {
    find_entry_node_mut(n.as_deref_mut()?, key)
}
fn find_entry_node_mut(n: &mut Node, key: i32) -> Option<&mut Entry> {
    match n.entries.binary_search_by_key(&key, |e| e.key) {
        Ok(i) => Some(&mut n.entries[i]),
        Err(i) => {
            if n.is_leaf() {
                None
            } else {
                find_entry_node_mut(&mut n.children[i], key)
            }
        }
    }
}
fn height(n: &Option<Box<Node>>) -> usize {
    n.as_ref()
        .map_or(0, |x| 1 + height(&x.children.first().cloned()))
}
fn node_view_mode(n: &Option<Box<Node>>, include_values: bool) -> Option<Box<NodeView>> {
    n.as_ref().map(|x| {
        Box::new(NodeView {
            id: x.id.clone(),
            node_kind: "bTree",
            entries: x
                .entries
                .iter()
                .map(|e| EntryView {
                    id: e.id.clone(),
                    key: e.key,
                    value: if include_values { Some(e.value) } else { None },
                })
                .collect(),
            children: x
                .children
                .iter()
                .map(|c| *node_view_mode(&Some(c.clone()), include_values).unwrap())
                .collect(),
            next_id: None,
        })
    })
}

#[wasm_bindgen]
pub struct BTreeSetSession {
    set: BTreeSet<i32>,
    model: BTreeSession,
}
#[wasm_bindgen]
impl BTreeSetSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            set: BTreeSet::new(),
            model: BTreeSession::new(),
        }
    }
    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        serialize(&self.model.view_mode("set"))
    }
    pub fn reset(&mut self) -> Result<JsValue, JsValue> {
        self.set.clear();
        self.model.map.clear();
        self.model.root = None;
        serialize(&self.model.view_mode("set"))
    }
    pub fn load_example(&mut self) -> Result<JsValue, JsValue> {
        self.set.clear();
        self.model.map.clear();
        self.model.root = None;
        for key in [50, 20, 70, 10, 30, 60, 80, 25, 35, 65, 75] {
            self.set.insert(key);
            self.model.insert_inner(key, 0, &mut Vec::new());
        }
        serialize(&self.model.view_mode("set"))
    }
    pub fn insert(&mut self, key: i32) -> Result<JsValue, JsValue> {
        let before = self.model.view_mode("set");
        let mut events = Vec::new();
        self.model.insert_inner(key, 0, &mut events);
        let inserted = self.set.insert(key);
        if !inserted {
            events.pop();
            events.push(EventView::Duplicate { key });
        }
        self.verify()?;
        self.model.trace_mode(before, events, "set")
    }
    pub fn get(&self, key: i32) -> Result<JsValue, JsValue> {
        let before = self.model.view_mode("set");
        let mut events = Vec::new();
        let mut n = self.model.root.as_deref();
        while let Some(node) = n {
            match node.entries.binary_search_by_key(&key, |e| e.key) {
                Ok(i) => {
                    events.push(EventView::Compared {
                        node_id: node.id.clone(),
                        key,
                        key_index: i,
                        ordering: "equal",
                    });
                    events.push(EventView::Found { key, value: None });
                    return self.model.trace_mode(before, events, "set");
                }
                Err(i) => {
                    events.push(EventView::Compared {
                        node_id: node.id.clone(),
                        key,
                        key_index: i,
                        ordering: if node.entries.get(i).is_some_and(|e| key < e.key) {
                            "less"
                        } else {
                            "greater"
                        },
                    });
                    if node.is_leaf() {
                        break;
                    }
                    n = Some(&node.children[i]);
                }
            }
        }
        events.push(EventView::Missing { key });
        self.model.trace_mode(before, events, "set")
    }
    pub fn remove(&mut self, key: i32) -> Result<JsValue, JsValue> {
        let before = self.model.view_mode("set");
        let mut events = Vec::new();
        let removed = match self.model.root.as_mut() {
            Some(root) => remove_from(root, key, &mut events),
            None => None,
        };
        self.model.map.remove(&key);
        let actual = self.set.remove(&key);
        if actual != removed.is_some() {
            return Err(JsValue::from_str(
                "BTreeSet visual model diverged from Rust set.",
            ));
        }
        if let Some(entry) = removed {
            if self
                .model
                .root
                .as_ref()
                .is_some_and(|n| n.entries.is_empty())
            {
                let mut root = self.model.root.take().unwrap();
                self.model.root = root.children.pop();
            }
            events.push(EventView::Removed {
                entry_id: entry.id,
                key,
                value: None,
            });
        } else {
            events.push(EventView::Missing { key });
        }
        self.verify()?;
        self.model.trace_mode(before, events, "set")
    }
}
impl Default for BTreeSetSession {
    fn default() -> Self {
        Self::new()
    }
}
impl BTreeSetSession {
    fn verify(&self) -> Result<(), JsValue> {
        self.model.verify()?;
        if self.set.iter().copied().collect::<Vec<_>>()
            != self.model.map.keys().copied().collect::<Vec<_>>()
        {
            return Err(JsValue::from_str(
                "BTreeSet values diverged from Rust model.",
            ));
        }
        Ok(())
    }
}
fn inorder(n: &Option<Box<Node>>, out: &mut Vec<(i32, i32)>) {
    if let Some(n) = n {
        for i in 0..n.entries.len() {
            if !n.is_leaf() {
                inorder(&Some(n.children[i].clone()), out);
            }
            out.push((n.entries[i].key, n.entries[i].value));
        }
        if !n.is_leaf() {
            inorder(&Some(n.children[n.entries.len()].clone()), out);
        }
    }
}
fn valid(
    n: &Option<Box<Node>>,
    min: Option<i32>,
    max: Option<i32>,
    depth: usize,
    leaf: &mut Option<usize>,
) -> bool {
    let Some(n) = n else { return true };
    if n.entries.is_empty()
        || n.entries.len() > MAX_KEYS
        || n.entries.windows(2).any(|w| w[0].key >= w[1].key)
        || n.entries
            .iter()
            .any(|e| min.is_some_and(|x| e.key <= x) || max.is_some_and(|x| e.key >= x))
    {
        return false;
    }
    if n.is_leaf() {
        if let Some(d) = leaf {
            if *d != depth {
                return false;
            }
        } else {
            *leaf = Some(depth)
        }
        return true;
    }
    if n.children.len() != n.entries.len() + 1 {
        return false;
    }
    for i in 0..n.children.len() {
        if !valid(
            &Some(n.children[i].clone()),
            if i == 0 {
                min
            } else {
                Some(n.entries[i - 1].key)
            },
            if i == n.entries.len() {
                max
            } else {
                Some(n.entries[i].key)
            },
            depth + 1,
            leaf,
        ) {
            return false;
        }
    }
    true
}
