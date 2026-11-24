use crate::serialization::serialize;
use data_structures::PriorityQueue;
use serde::Serialize;
use wasm_bindgen::prelude::*;

const MAX_ITEMS: usize = 64;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct View {
    kind: &'static str,
    items: Vec<Item>,
    len: usize,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    id: String,
    priority: i32,
    value: i32,
    index: usize,
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
        item_id: String,
        index: usize,
        priority: i32,
        value: i32,
    },
    Compared {
        parent_id: String,
        child_id: String,
        parent_index: usize,
        child_index: usize,
        parent_priority: i32,
        child_priority: i32,
        ordering: &'static str,
    },
    ComparedChildren {
        left_id: String,
        right_id: String,
        left_index: usize,
        right_index: usize,
        left_priority: i32,
        right_priority: i32,
        ordering: &'static str,
    },
    Swapped {
        first_id: String,
        second_id: String,
        first_index: usize,
        second_index: usize,
    },
    RootRemoved {
        item_id: String,
        priority: i32,
        value: i32,
        replacement_id: Option<String>,
        replacement_priority: Option<i32>,
        replacement_value: Option<i32>,
    },
    Accessed {
        item_id: String,
        priority: i32,
        value: i32,
    },
    Empty,
}

#[wasm_bindgen]
pub struct PriorityQueueSession {
    queue: PriorityQueue<i32, i32>,
    ids: Vec<String>,
    next_id: u64,
}
#[wasm_bindgen]
impl PriorityQueueSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            queue: PriorityQueue::new(),
            ids: Vec::new(),
            next_id: 0,
        }
    }
    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        serialize(&self.view())
    }
    pub fn reset(&mut self) -> Result<JsValue, JsValue> {
        self.queue.clear();
        self.ids.clear();
        self.serialize_view()
    }
    pub fn load_example(&mut self) -> Result<JsValue, JsValue> {
        self.queue.clear();
        self.ids.clear();
        for (priority, value) in [(3, 30), (5, 50), (2, 20), (5, 55), (1, 10)] {
            self.push(priority, value)?;
        }
        self.serialize_view()
    }
    pub fn push(&mut self, priority: i32, value: i32) -> Result<JsValue, JsValue> {
        if self.queue.len() >= MAX_ITEMS {
            return Err(JsValue::from_str("可视化示例最多支持 64 个元素。"));
        }
        let before = self.view();
        let mut items = self.entries();
        let mut ids = self.ids.clone();
        let id = self.allocate_id();
        let index = items.len();
        items.push((priority, value));
        ids.push(id.clone());
        let mut events = vec![Event::Inserted {
            item_id: id,
            index,
            priority,
            value,
        }];
        let mut child = index;
        while child > 0 {
            let parent = (child - 1) / 2;
            events.push(Event::Compared {
                parent_id: ids[parent].clone(),
                child_id: ids[child].clone(),
                parent_index: parent,
                child_index: child,
                parent_priority: items[parent].0,
                child_priority: items[child].0,
                ordering: if items[parent].0 < items[child].0 {
                    "less"
                } else {
                    "greaterOrEqual"
                },
            });
            if items[parent].0 >= items[child].0 {
                break;
            }
            events.push(Event::Swapped {
                first_id: ids[parent].clone(),
                second_id: ids[child].clone(),
                first_index: parent,
                second_index: child,
            });
            items.swap(parent, child);
            ids.swap(parent, child);
            child = parent;
        }
        self.queue.push(priority, value);
        self.ids = ids;
        self.verify(&items)?;
        self.trace(before, events)
    }
    pub fn pop(&mut self) -> Result<JsValue, JsValue> {
        let before = self.view();
        let mut items = self.entries();
        let mut ids = self.ids.clone();
        if items.is_empty() {
            return self.trace(before, vec![Event::Empty]);
        }
        let (priority, value) = items[0];
        let removed_id = ids[0].clone();
        let last_item = items.pop().expect("heap root exists");
        let last_id = ids.pop().expect("heap root has a visual ID");
        let replacement = if items.is_empty() {
            None
        } else {
            items[0] = last_item;
            ids[0] = last_id;
            Some(items[0])
        };
        let replacement_id = if items.is_empty() {
            None
        } else {
            Some(ids[0].clone())
        };
        let mut events = vec![Event::RootRemoved {
            item_id: removed_id,
            priority,
            value,
            replacement_id,
            replacement_priority: replacement.map(|x| x.0),
            replacement_value: replacement.map(|x| x.1),
        }];
        if !items.is_empty() {
            let mut parent = 0;
            loop {
                let left = parent * 2 + 1;
                if left >= items.len() {
                    break;
                }
                let right = left + 1;
                if right < items.len() {
                    events.push(Event::ComparedChildren {
                        left_id: ids[left].clone(),
                        right_id: ids[right].clone(),
                        left_index: left,
                        right_index: right,
                        left_priority: items[left].0,
                        right_priority: items[right].0,
                        ordering: if items[left].0 >= items[right].0 {
                            "leftGreaterOrEqual"
                        } else {
                            "rightGreater"
                        },
                    });
                }
                let greater = if right < items.len() && items[right].0 > items[left].0 {
                    right
                } else {
                    left
                };
                events.push(Event::Compared {
                    parent_id: ids[parent].clone(),
                    child_id: ids[greater].clone(),
                    parent_index: parent,
                    child_index: greater,
                    parent_priority: items[parent].0,
                    child_priority: items[greater].0,
                    ordering: if items[parent].0 < items[greater].0 {
                        "less"
                    } else {
                        "greaterOrEqual"
                    },
                });
                if items[parent].0 >= items[greater].0 {
                    break;
                }
                events.push(Event::Swapped {
                    first_id: ids[parent].clone(),
                    second_id: ids[greater].clone(),
                    first_index: parent,
                    second_index: greater,
                });
                items.swap(parent, greater);
                ids.swap(parent, greater);
                parent = greater;
            }
        }
        let removed = self.queue.pop();
        if removed != Some((priority, value)) {
            return Err(JsValue::from_str(
                "PriorityQueue result diverged from its heap trace.",
            ));
        }
        self.ids = ids;
        self.verify(&items)?;
        self.trace(before, events)
    }
    pub fn peek(&self) -> Result<JsValue, JsValue> {
        let before = self.view();
        let events = match (self.queue.peek(), self.ids.first()) {
            (Some((priority, value)), Some(id)) => vec![Event::Accessed {
                item_id: id.clone(),
                priority: *priority,
                value: *value,
            }],
            _ => vec![Event::Empty],
        };
        self.trace(before, events)
    }
}
impl PriorityQueueSession {
    fn entries(&self) -> Vec<(i32, i32)> {
        self.queue.iter().map(|(p, v)| (*p, *v)).collect()
    }
    fn view(&self) -> View {
        View {
            kind: "priorityQueue",
            len: self.queue.len(),
            items: self
                .queue
                .iter()
                .zip(&self.ids)
                .enumerate()
                .map(|(index, ((priority, value), id))| Item {
                    id: id.clone(),
                    priority: *priority,
                    value: *value,
                    index,
                })
                .collect(),
        }
    }
    fn serialize_view(&self) -> Result<JsValue, JsValue> {
        serialize(&self.view())
    }
    fn trace(&self, before: View, events: Vec<Event>) -> Result<JsValue, JsValue> {
        serialize(&Trace {
            before,
            after: self.view(),
            events,
        })
    }
    fn allocate_id(&mut self) -> String {
        let id = format!("priority-item-{}", self.next_id);
        self.next_id += 1;
        id
    }
    fn verify(&self, expected: &[(i32, i32)]) -> Result<(), JsValue> {
        let actual = self.entries();
        if actual != expected || self.queue.len() != self.ids.len() {
            return Err(JsValue::from_str(
                "PriorityQueue layout diverged from its Rust heap.",
            ));
        }
        Ok(())
    }
}
impl Default for PriorityQueueSession {
    fn default() -> Self {
        Self::new()
    }
}
