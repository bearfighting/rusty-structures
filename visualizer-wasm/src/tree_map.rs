use crate::serialization::serialize;
use data_structures::{AvlTreeMap, BinarySearchTreeMap, RedBlackTreeMap};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct View {
    kind: &'static str,
    backend: String,
    entries: Vec<Entry>,
    len: usize,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Entry {
    id: String,
    key: i32,
    value: i32,
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
    Inserted {
        key: i32,
        value: i32,
    },
    Updated {
        key: i32,
        old_value: i32,
        value: i32,
    },
    Found {
        key: i32,
        value: i32,
    },
    Missing {
        key: i32,
    },
    Removed {
        key: i32,
        value: i32,
    },
    ScanEntry {
        key: i32,
        value: i32,
    },
}

enum Map {
    Bst(BinarySearchTreeMap<i32, i32>),
    Avl(AvlTreeMap<i32, i32>),
    RedBlack(RedBlackTreeMap<i32, i32>),
}
impl Map {
    fn new(backend: &str) -> Result<Self, JsValue> {
        match backend {
            "binary-search-tree-map" => Ok(Self::Bst(BinarySearchTreeMap::new())),
            "avl-tree-map" => Ok(Self::Avl(AvlTreeMap::new())),
            "red-black-tree-map" => Ok(Self::RedBlack(RedBlackTreeMap::new())),
            _ => Err(JsValue::from_str("Unknown tree map backend.")),
        }
    }
    fn insert(&mut self, k: i32, v: i32) -> Option<i32> {
        match self {
            Self::Bst(m) => m.insert(k, v),
            Self::Avl(m) => m.insert(k, v),
            Self::RedBlack(m) => m.insert(k, v),
        }
    }
    fn get(&self, k: &i32) -> Option<&i32> {
        match self {
            Self::Bst(m) => m.get(k),
            Self::Avl(m) => m.get(k),
            Self::RedBlack(m) => m.get(k),
        }
    }
    fn remove(&mut self, k: &i32) -> Option<i32> {
        match self {
            Self::Bst(m) => m.remove(k),
            Self::Avl(m) => m.remove(k),
            Self::RedBlack(m) => m.remove(k),
        }
    }
    fn view(&self, backend: &str) -> View {
        let entries: Vec<_> = match self {
            Self::Bst(m) => m.iter().map(|(k, v)| entry(*k, *v)).collect(),
            Self::Avl(m) => m.iter().map(|(k, v)| entry(*k, *v)).collect(),
            Self::RedBlack(m) => m.iter().map(|(k, v)| entry(*k, *v)).collect(),
        };
        View {
            kind: "treeMap",
            backend: backend.into(),
            len: entries.len(),
            entries,
        }
    }
}
fn entry(key: i32, value: i32) -> Entry {
    Entry {
        id: format!("key-{key}"),
        key,
        value,
    }
}

#[wasm_bindgen]
pub struct TreeMapSession {
    backend: String,
    map: Map,
}
#[wasm_bindgen]
impl TreeMapSession {
    #[wasm_bindgen(constructor)]
    pub fn new(backend: &str) -> Result<TreeMapSession, JsValue> {
        Ok(Self {
            backend: backend.into(),
            map: Map::new(backend)?,
        })
    }
    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        serialize(&self.map.view(&self.backend))
    }
    pub fn reset(&mut self) -> Result<JsValue, JsValue> {
        self.map = Map::new(&self.backend)?;
        self.snapshot()
    }
    pub fn load_example(&mut self) -> Result<JsValue, JsValue> {
        self.map = Map::new(&self.backend)?;
        for (k, v) in [
            (8, 80),
            (4, 40),
            (12, 120),
            (2, 20),
            (6, 60),
            (10, 100),
            (14, 140),
        ] {
            self.map.insert(k, v);
        }
        self.snapshot()
    }
    pub fn insert(&mut self, key: i32, value: i32) -> Result<JsValue, JsValue> {
        let before = self.map.view(&self.backend);
        let old = self.map.insert(key, value);
        let after = self.map.view(&self.backend);
        let event = match old {
            Some(old_value) => Event::Updated {
                key,
                old_value,
                value,
            },
            None => Event::Inserted { key, value },
        };
        serialize(&Trace {
            before,
            after,
            events: vec![event],
        })
    }
    pub fn get(&self, key: i32) -> Result<JsValue, JsValue> {
        let view = self.map.view(&self.backend);
        let event = match self.map.get(&key) {
            Some(value) => Event::Found { key, value: *value },
            None => Event::Missing { key },
        };
        serialize(&Trace {
            before: view.clone(),
            after: view,
            events: vec![event],
        })
    }
    pub fn remove(&mut self, key: i32) -> Result<JsValue, JsValue> {
        let before = self.map.view(&self.backend);
        let removed = self.map.remove(&key);
        let after = self.map.view(&self.backend);
        let event = match removed {
            Some(value) => Event::Removed { key, value },
            None => Event::Missing { key },
        };
        serialize(&Trace {
            before,
            after,
            events: vec![event],
        })
    }
    pub fn scan(&self) -> Result<JsValue, JsValue> {
        let view = self.map.view(&self.backend);
        let events = view
            .entries
            .iter()
            .map(|e| Event::ScanEntry {
                key: e.key,
                value: e.value,
            })
            .collect();
        serialize(&Trace {
            before: view.clone(),
            after: view,
            events,
        })
    }
}
