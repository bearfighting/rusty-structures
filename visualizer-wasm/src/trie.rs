use crate::serialization::serialize;
use data_structures::{Trie, TrieSnapshotNode};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use wasm_bindgen::prelude::*;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct View {
    kind: &'static str,
    root: NodeView,
    len: usize,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct NodeView {
    id: String,
    prefix: String,
    edge_label: Option<String>,
    is_word: bool,
    children: Vec<NodeView>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Trace {
    before: View,
    after: View,
    events: Vec<Event>,
}
#[derive(Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
enum Event {
    Visited {
        node_id: String,
        prefix: String,
        character: Option<String>,
        matched: bool,
    },
    NodeCreated {
        node_id: String,
        prefix: String,
        character: String,
    },
    WordMarked {
        node_id: String,
        word: String,
    },
    TerminalCleared {
        node_id: String,
        word: String,
    },
    NodePruned {
        node_id: String,
        prefix: String,
    },
    Duplicate {
        word: String,
    },
    Found {
        word: String,
    },
    PrefixFound {
        prefix: String,
        matches: usize,
    },
    ScanWord {
        word: String,
    },
    Missing {
        query: String,
    },
    Removed {
        word: String,
    },
}

#[wasm_bindgen]
pub struct TrieSession {
    trie: Trie,
    ids: BTreeMap<String, String>,
    next_id: u64,
}
#[wasm_bindgen]
impl TrieSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            trie: Trie::new(),
            ids: BTreeMap::new(),
            next_id: 0,
        }
    }
    pub fn snapshot(&mut self) -> Result<JsValue, JsValue> {
        let v = self.view();
        serialize(&v)
    }
    pub fn reset(&mut self) -> Result<JsValue, JsValue> {
        self.trie.clear();
        self.serialize_view()
    }
    pub fn load_example(&mut self) -> Result<JsValue, JsValue> {
        self.trie.clear();
        for word in ["car", "cart", "cat", "dog", "do", "猫", "猫咪"] {
            self.trie.insert(word);
        }
        self.serialize_view()
    }
    pub fn insert(&mut self, word: String) -> Result<JsValue, JsValue> {
        let before_snapshot = self.trie.visualization_snapshot();
        let before = self.view();
        let existed = self.trie.contains(&word);
        self.trie.insert(&word);
        let after_snapshot = self.trie.visualization_snapshot();
        let after = self.view();
        let mut events = walk_events(&before_snapshot.root, &word, &self.ids);
        if existed {
            events.push(Event::Duplicate { word });
        } else {
            let old: BTreeSet<_> = collect_prefixes(&before_snapshot.root)
                .into_iter()
                .collect();
            for node in flatten(&after.root) {
                if !old.contains(&node.prefix) {
                    events.push(Event::NodeCreated {
                        node_id: node.id.clone(),
                        prefix: node.prefix.clone(),
                        character: node.edge_label.clone().unwrap_or_default(),
                    });
                }
            }
            events.push(Event::WordMarked {
                node_id: self.id_for(&word),
                word: word.clone(),
            });
        }
        self.verify(&after_snapshot)?;
        serialize(&Trace {
            before,
            after,
            events,
        })
    }
    pub fn contains(&mut self, word: String) -> Result<JsValue, JsValue> {
        self.query(word, false)
    }
    pub fn starts_with(&mut self, prefix: String) -> Result<JsValue, JsValue> {
        self.query(prefix, true)
    }
    pub fn remove(&mut self, word: String) -> Result<JsValue, JsValue> {
        let before_snapshot = self.trie.visualization_snapshot();
        let before = self.view();
        let existed = self.trie.contains(&word);
        self.trie.remove(&word);
        let after_snapshot = self.trie.visualization_snapshot();
        let after = self.view();
        let mut events = walk_events(&before_snapshot.root, &word, &self.ids);
        if existed {
            events.push(Event::TerminalCleared {
                node_id: self.id_for(&word),
                word: word.clone(),
            });
            let current: BTreeSet<_> = collect_prefixes(&after_snapshot.root).into_iter().collect();
            let mut pruned: Vec<_> = flatten(&before.root)
                .into_iter()
                .filter(|node| !current.contains(&node.prefix) && !node.prefix.is_empty())
                .collect();
            pruned.sort_by_key(|node| std::cmp::Reverse(node.prefix.chars().count()));
            for node in pruned {
                events.push(Event::NodePruned {
                    node_id: node.id,
                    prefix: node.prefix,
                });
            }
            events.push(Event::Removed { word });
        } else {
            events.push(Event::Missing { query: word });
        }
        self.verify(&after_snapshot)?;
        serialize(&Trace {
            before,
            after,
            events,
        })
    }
    pub fn scan(&mut self, prefix: String) -> Result<JsValue, JsValue> {
        let snap = self.trie.visualization_snapshot();
        let before = self.view();
        let mut events = walk_events(&snap.root, &prefix, &self.ids);
        let matches: Vec<_> = self
            .trie
            .iter()
            .filter(|word| word.starts_with(&prefix))
            .collect();
        if prefix.is_empty() && !self.trie.is_empty()
            || !prefix.is_empty() && snap_has_prefix(&snap.root, &prefix)
        {
            events.push(Event::PrefixFound {
                prefix: prefix.clone(),
                matches: matches.len(),
            });
        } else {
            events.push(Event::Missing {
                query: prefix.clone(),
            });
        }
        events.extend(matches.into_iter().map(|word| Event::ScanWord { word }));
        serialize(&Trace {
            before: before.clone(),
            after: before,
            events,
        })
    }
}

impl TrieSession {
    fn serialize_view(&mut self) -> Result<JsValue, JsValue> {
        let v = self.view();
        serialize(&v)
    }
    fn query(&mut self, query: String, prefix: bool) -> Result<JsValue, JsValue> {
        let snap = self.trie.visualization_snapshot();
        let view = self.view();
        let mut events = walk_events(&snap.root, &query, &self.ids);
        let result = if prefix {
            !self.trie.is_empty() && snap_has_prefix(&snap.root, &query)
        } else {
            self.trie.contains(&query)
        };
        if result {
            if prefix {
                let n = self
                    .trie
                    .iter()
                    .filter(|word| word.starts_with(&query))
                    .count();
                events.push(Event::PrefixFound {
                    prefix: query,
                    matches: n,
                });
            } else {
                events.push(Event::Found { word: query });
            }
        } else {
            events.push(Event::Missing { query });
        }
        serialize(&Trace {
            before: view.clone(),
            after: view,
            events,
        })
    }
    fn view(&mut self) -> View {
        fn node(
            n: &TrieSnapshotNode,
            edge: Option<char>,
            ids: &mut BTreeMap<String, String>,
            next: &mut u64,
        ) -> NodeView {
            let id = ids
                .entry(n.prefix.clone())
                .or_insert_with(|| {
                    let id = format!("trie-{}", *next);
                    *next += 1;
                    id
                })
                .clone();
            NodeView {
                id,
                prefix: n.prefix.clone(),
                edge_label: edge.map(|c| c.to_string()),
                is_word: n.is_word,
                children: n
                    .children
                    .iter()
                    .map(|(c, ch)| node(ch, Some(*c), ids, next))
                    .collect(),
            }
        }
        let s = self.trie.visualization_snapshot();
        View {
            kind: "trie",
            root: node(&s.root, None, &mut self.ids, &mut self.next_id),
            len: s.len,
        }
    }
    fn id_for(&self, prefix: &str) -> String {
        self.ids
            .get(prefix)
            .cloned()
            .unwrap_or_else(|| "trie-unknown".into())
    }
    fn verify(&self, s: &data_structures::TrieSnapshot) -> Result<(), JsValue> {
        let mut words = BTreeSet::new();
        fn visit(n: &TrieSnapshotNode, out: &mut BTreeSet<String>) {
            if n.is_word {
                out.insert(n.prefix.clone());
            }
            for (_, child) in &n.children {
                visit(child, out);
            }
        }
        visit(&s.root, &mut words);
        let iter: BTreeSet<_> = self.trie.iter().collect();
        if words != iter || s.len != iter.len() {
            return Err(JsValue::from_str(
                "Trie snapshot diverged from Rust iteration.",
            ));
        }
        Ok(())
    }
}
impl Default for TrieSession {
    fn default() -> Self {
        Self::new()
    }
}
fn walk_events(root: &TrieSnapshotNode, query: &str, ids: &BTreeMap<String, String>) -> Vec<Event> {
    let mut events = Vec::new();
    let mut prefix = String::new();
    for c in query.chars() {
        let Some((_, child)) =
            root_at(&prefix, root).and_then(|n| n.children.iter().find(|(edge, _)| *edge == c))
        else {
            events.push(Event::Visited {
                node_id: ids.get(&prefix).cloned().unwrap_or_default(),
                prefix,
                character: Some(c.to_string()),
                matched: false,
            });
            return events;
        };
        events.push(Event::Visited {
            node_id: ids.get(&prefix).cloned().unwrap_or_default(),
            prefix: prefix.clone(),
            character: Some(c.to_string()),
            matched: true,
        });
        prefix.push(c);
        if child.prefix != prefix {
            return events;
        }
    }
    events
}
fn root_at<'a>(prefix: &str, root: &'a TrieSnapshotNode) -> Option<&'a TrieSnapshotNode> {
    if root.prefix == prefix {
        return Some(root);
    }
    for (_, child) in &root.children {
        if let Some(n) = root_at(prefix, child) {
            return Some(n);
        }
    }
    None
}
fn snap_has_prefix(root: &TrieSnapshotNode, prefix: &str) -> bool {
    root_at(prefix, root).is_some()
}
fn collect_prefixes(root: &TrieSnapshotNode) -> Vec<String> {
    let mut out = Vec::new();
    fn walk(n: &TrieSnapshotNode, out: &mut Vec<String>) {
        out.push(n.prefix.clone());
        for (_, c) in &n.children {
            walk(c, out);
        }
    }
    walk(root, &mut out);
    out
}
fn flatten(root: &NodeView) -> Vec<NodeView> {
    let mut out = vec![root.clone()];
    for child in &root.children {
        out.extend(flatten(child));
    }
    out
}
