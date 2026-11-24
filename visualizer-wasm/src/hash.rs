use crate::serialization::serialize;
use data_structures::{HashMap, HashSet};
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::BuildHasherDefault;
use wasm_bindgen::prelude::*;

type StableHash = BuildHasherDefault<DefaultHasher>;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct View {
    kind: &'static str,
    mode: &'static str,
    buckets: Vec<Bucket>,
    len: usize,
    capacity: usize,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Entry {
    id: String,
    key: i32,
    value: Option<i32>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Bucket {
    index: usize,
    entries: Vec<Entry>,
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
    Compared {
        bucket: usize,
        key: i32,
        compared_key: i32,
        matched: bool,
    },
    Collision {
        bucket: usize,
        key: i32,
        chain_len: usize,
    },
    Inserted {
        bucket: usize,
        key: i32,
        value: Option<i32>,
    },
    Updated {
        bucket: usize,
        key: i32,
        value: i32,
    },
    Removed {
        bucket: usize,
        key: i32,
    },
    Missing {
        bucket: usize,
        key: i32,
    },
    Resized {
        old_count: usize,
        new_count: usize,
    },
    Rehashed {
        key: i32,
        old_bucket: usize,
        new_bucket: usize,
    },
    Found {
        bucket: usize,
        key: i32,
        value: Option<i32>,
    },
    Duplicate {
        bucket: usize,
        key: i32,
    },
}

fn bucket_for(key: i32, count: usize) -> usize {
    let mut h = DefaultHasher::new();
    std::hash::Hash::hash(&key, &mut h);
    h.finish() as usize % count
}
use std::hash::Hasher;
fn map_view(map: &HashMap<i32, i32, StableHash>) -> View {
    let mut buckets: Vec<_> = (0..map.bucket_count())
        .map(|index| Bucket {
            index,
            entries: Vec::new(),
        })
        .collect();
    for (key, value) in map.iter() {
        buckets[bucket_for(*key, map.bucket_count())]
            .entries
            .push(Entry {
                id: format!("k{key}"),
                key: *key,
                value: Some(*value),
            });
    }
    View {
        kind: "hashTable",
        mode: "map",
        buckets,
        len: map.len(),
        capacity: map.capacity(),
    }
}
fn set_view(set: &HashSet<i32, StableHash>) -> View {
    let mut buckets: Vec<_> = (0..set.bucket_count())
        .map(|index| Bucket {
            index,
            entries: Vec::new(),
        })
        .collect();
    for key in set.iter() {
        buckets[bucket_for(*key, set.bucket_count())]
            .entries
            .push(Entry {
                id: format!("k{key}"),
                key: *key,
                value: None,
            });
    }
    View {
        kind: "hashTable",
        mode: "set",
        buckets,
        len: set.len(),
        capacity: set.capacity(),
    }
}
fn diff(
    before: &View,
    after: &View,
    changed: i32,
    value: Option<i32>,
    removed: bool,
) -> Vec<Event> {
    let mut out = Vec::new();
    if before.buckets.len() != after.buckets.len() {
        out.push(Event::Resized {
            old_count: before.buckets.len(),
            new_count: after.buckets.len(),
        });
        for (old_bucket, bucket) in before.buckets.iter().enumerate() {
            for e in &bucket.entries {
                let new_bucket = bucket_for(e.key, after.buckets.len());
                if old_bucket != new_bucket {
                    out.push(Event::Rehashed {
                        key: e.key,
                        old_bucket,
                        new_bucket,
                    });
                }
            }
        }
    }
    let b = bucket_for(changed, before.buckets.len());
    for entry in &before.buckets[b].entries {
        let matched = entry.key == changed;
        out.push(Event::Compared {
            bucket: b,
            key: changed,
            compared_key: entry.key,
            matched,
        });
        if matched {
            break;
        }
    }
    if !removed {
        let target_bucket = bucket_for(changed, after.buckets.len());
        let chain = after.buckets[target_bucket].entries.len().saturating_sub(1);
        if chain > 0 {
            out.push(Event::Collision {
                bucket: target_bucket,
                key: changed,
                chain_len: chain,
            });
        }
    }
    if removed {
        out.push(Event::Removed {
            bucket: b,
            key: changed,
        });
    } else {
        out.push(Event::Inserted {
            bucket: bucket_for(changed, after.buckets.len()),
            key: changed,
            value,
        });
    }
    out
}

#[wasm_bindgen]
pub struct HashMapSession {
    map: HashMap<i32, i32, StableHash>,
}
#[wasm_bindgen]
impl HashMapSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            map: HashMap::with_hasher(StableHash::default()),
        }
    }
    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        serialize(&map_view(&self.map))
    }
    pub fn reset(&mut self) -> Result<JsValue, JsValue> {
        self.map.clear();
        self.snapshot()
    }
    pub fn load_example(&mut self) -> Result<JsValue, JsValue> {
        self.map.clear();
        for (k, v) in [
            (12, 120),
            (20, 200),
            (28, 280),
            (3, 30),
            (11, 110),
            (19, 190),
        ] {
            self.map.insert(k, v);
        }
        self.snapshot()
    }
    pub fn insert(&mut self, key: i32, value: i32) -> Result<JsValue, JsValue> {
        let before = map_view(&self.map);
        let old = self.map.bucket_count();
        let previous = self.map.insert(key, value);
        let after = map_view(&self.map);
        let mut events = diff(&before, &after, key, Some(value), false);
        if let Some(v) = previous {
            events.retain(|e| !matches!(e, Event::Inserted { .. } | Event::Collision { .. }));
            events.push(Event::Updated {
                bucket: bucket_for(key, after.buckets.len()),
                key,
                value: v,
            });
        } else if old != self.map.bucket_count() {
        }
        serialize(&Trace {
            before,
            after,
            events,
        })
    }
    pub fn get(&self, key: i32) -> Result<JsValue, JsValue> {
        let view = map_view(&self.map);
        let bucket = bucket_for(key, view.buckets.len());
        let mut events: Vec<_> = view.buckets[bucket]
            .entries
            .iter()
            .take_while(|e| e.key != key)
            .map(|e| Event::Compared {
                bucket,
                key,
                compared_key: e.key,
                matched: false,
            })
            .collect();
        if let Some(v) = self.map.get(&key) {
            events.push(Event::Compared {
                bucket,
                key,
                compared_key: key,
                matched: true,
            });
            events.push(Event::Found {
                bucket,
                key,
                value: Some(*v),
            });
        } else {
            events.push(Event::Missing { bucket, key });
        }
        serialize(&Trace {
            before: view.clone(),
            after: view,
            events,
        })
    }
    pub fn remove(&mut self, key: i32) -> Result<JsValue, JsValue> {
        let before = map_view(&self.map);
        let removed = self.map.remove(&key);
        let after = map_view(&self.map);
        let events = if removed.is_some() {
            diff(&before, &after, key, None, true)
        } else {
            vec![Event::Missing {
                bucket: bucket_for(key, before.buckets.len()),
                key,
            }]
        };
        serialize(&Trace {
            before,
            after,
            events,
        })
    }
}

#[wasm_bindgen]
pub struct HashSetSession {
    set: HashSet<i32, StableHash>,
}
#[wasm_bindgen]
impl HashSetSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            set: HashSet::with_hasher(StableHash::default()),
        }
    }
    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        serialize(&set_view(&self.set))
    }
    pub fn reset(&mut self) -> Result<JsValue, JsValue> {
        self.set.clear();
        self.snapshot()
    }
    pub fn load_example(&mut self) -> Result<JsValue, JsValue> {
        self.set.clear();
        for k in [12, 20, 28, 3, 11, 19] {
            self.set.insert(k);
        }
        self.snapshot()
    }
    pub fn insert(&mut self, key: i32) -> Result<JsValue, JsValue> {
        let before = set_view(&self.set);
        let inserted = self.set.insert(key);
        let after = set_view(&self.set);
        let events = if inserted {
            diff(&before, &after, key, None, false)
        } else {
            vec![Event::Duplicate {
                bucket: bucket_for(key, after.buckets.len()),
                key,
            }]
        };
        serialize(&Trace {
            before,
            after,
            events,
        })
    }
    pub fn contains(&self, key: i32) -> Result<JsValue, JsValue> {
        let view = set_view(&self.set);
        let bucket = bucket_for(key, view.buckets.len());
        let events = if self.set.contains(&key) {
            vec![Event::Found {
                bucket,
                key,
                value: None,
            }]
        } else {
            vec![Event::Missing { bucket, key }]
        };
        serialize(&Trace {
            before: view.clone(),
            after: view,
            events,
        })
    }
    pub fn remove(&mut self, key: i32) -> Result<JsValue, JsValue> {
        let before = set_view(&self.set);
        let removed = self.set.remove(&key);
        let after = set_view(&self.set);
        let events = if removed {
            diff(&before, &after, key, None, true)
        } else {
            vec![Event::Missing {
                bucket: bucket_for(key, before.buckets.len()),
                key,
            }]
        };
        serialize(&Trace {
            before,
            after,
            events,
        })
    }
}
