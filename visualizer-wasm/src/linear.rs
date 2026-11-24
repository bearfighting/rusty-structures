use crate::serialization::serialize;
use data_structures::{Deque, Queue, Stack};
use serde::Serialize;
use wasm_bindgen::prelude::*;

const MAX_ITEMS: usize = 64;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SequenceView {
    kind: &'static str,
    structure: &'static str,
    items: Vec<ItemView>,
    len: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    capacity: Option<usize>,
}
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ItemView {
    id: String,
    label: String,
    value: i32,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TraceView {
    before: SequenceView,
    after: SequenceView,
    events: Vec<Event>,
}
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum Event {
    Inserted {
        item_id: String,
        index: usize,
        value: i32,
    },
    Removed {
        item_id: String,
        index: usize,
        value: i32,
    },
    Accessed {
        item_id: String,
        index: usize,
        value: i32,
    },
    Empty,
    CapacityChanged {
        old_capacity: usize,
        new_capacity: usize,
    },
}

enum Container {
    Stack(Stack<i32>),
    Queue(Queue<i32>),
    Deque(Deque<i32>),
}
#[wasm_bindgen]
pub struct LinearSession {
    structure: String,
    container: Container,
    ids: Vec<String>,
    next_id: u64,
}
#[wasm_bindgen]
impl LinearSession {
    #[wasm_bindgen(constructor)]
    pub fn new(structure: &str) -> Result<LinearSession, JsValue> {
        let container = match structure {
            "stack" => Container::Stack(Stack::new()),
            "queue" => Container::Queue(Queue::new()),
            "deque" => Container::Deque(Deque::new()),
            _ => return Err(JsValue::from_str("Unknown linear container.")),
        };
        Ok(Self {
            structure: structure.into(),
            container,
            ids: Vec::new(),
            next_id: 0,
        })
    }
    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        serialize(&self.view())
    }
    pub fn reset(&mut self) -> Result<JsValue, JsValue> {
        self.clear();
        self.serialize_view()
    }
    pub fn load_example(&mut self) -> Result<JsValue, JsValue> {
        self.clear();
        let end = if self.structure == "stack" {
            "front"
        } else {
            "back"
        };
        for value in [10, 20, 30, 40] {
            self.push_value(value, end)?;
        }
        self.serialize_view()
    }
    pub fn push(&mut self, value: i32) -> Result<JsValue, JsValue> {
        self.push_value(value, "front")
    }
    pub fn enqueue(&mut self, value: i32) -> Result<JsValue, JsValue> {
        self.push_value(value, "back")
    }
    pub fn push_front(&mut self, value: i32) -> Result<JsValue, JsValue> {
        self.push_value(value, "front")
    }
    pub fn push_back(&mut self, value: i32) -> Result<JsValue, JsValue> {
        self.push_value(value, "back")
    }
    pub fn pop(&mut self) -> Result<JsValue, JsValue> {
        match self.structure.as_str() {
            "stack" => self.remove_end("front"),
            "queue" => self.remove_end("front"),
            _ => Err(JsValue::from_str("Pop is supported by Stack only.")),
        }
    }
    pub fn dequeue(&mut self) -> Result<JsValue, JsValue> {
        self.remove_end("front")
    }
    pub fn pop_front(&mut self) -> Result<JsValue, JsValue> {
        self.remove_end("front")
    }
    pub fn pop_back(&mut self) -> Result<JsValue, JsValue> {
        self.remove_end("back")
    }
    pub fn peek(&self) -> Result<JsValue, JsValue> {
        if self.structure != "stack" {
            return Err(JsValue::from_str("Peek is supported by Stack only."));
        }
        self.access(0)
    }
    pub fn front(&self) -> Result<JsValue, JsValue> {
        self.access(0)
    }
    pub fn back(&self) -> Result<JsValue, JsValue> {
        let index = self.ids.len().saturating_sub(1);
        self.access(index)
    }
}
impl LinearSession {
    fn clear(&mut self) {
        match &mut self.container {
            Container::Stack(c) => c.clear(),
            Container::Queue(c) => c.clear(),
            Container::Deque(c) => c.clear(),
        }
        self.ids.clear();
    }
    fn serialize_view(&self) -> Result<JsValue, JsValue> {
        serialize(&self.view())
    }
    fn view(&self) -> SequenceView {
        let values: Vec<i32> = match &self.container {
            Container::Stack(c) => c.iter().copied().collect(),
            Container::Queue(c) => c.iter().copied().collect(),
            Container::Deque(c) => c.iter().copied().collect(),
        };
        let capacity = match &self.container {
            Container::Stack(_) => None,
            Container::Queue(c) => Some(c.capacity()),
            Container::Deque(c) => Some(c.capacity()),
        };
        let items = values
            .into_iter()
            .zip(&self.ids)
            .map(|(value, id)| ItemView {
                id: id.clone(),
                label: value.to_string(),
                value,
            })
            .collect();
        SequenceView {
            kind: "sequence",
            structure: match self.structure.as_str() {
                "stack" => "stack",
                "queue" => "queue",
                _ => "deque",
            },
            len: self.ids.len(),
            capacity,
            items,
        }
    }
    fn push_value(&mut self, value: i32, end: &str) -> Result<JsValue, JsValue> {
        if self.ids.len() >= MAX_ITEMS {
            return Err(JsValue::from_str("可视化示例最多支持 64 个元素。"));
        }
        let before = self.view();
        let old_capacity = before.capacity;
        let result = match &mut self.container {
            Container::Stack(c) => {
                if end == "front" {
                    c.push(value);
                } else {
                    return Err(JsValue::from_str("Stack accepts pushes at the top."));
                }
                Ok(())
            }
            Container::Queue(c) => c
                .try_enqueue(value)
                .map_err(|e| JsValue::from_str(&e.to_string())),
            Container::Deque(c) => {
                if end == "front" {
                    c.try_push_front(value)
                        .map_err(|e| JsValue::from_str(&e.to_string()))
                } else {
                    c.try_push_back(value)
                        .map_err(|e| JsValue::from_str(&e.to_string()))
                }
            }
        };
        result?;
        let id = self.allocate_id();
        if self.structure == "stack" || self.structure == "deque" && end == "front" {
            self.ids.insert(0, id.clone());
        } else {
            self.ids.push(id.clone());
        }
        let after = self.view();
        let index = if self.structure == "stack" || self.structure == "deque" && end == "front" {
            0
        } else {
            after.len - 1
        };
        let mut events = Vec::new();
        if let (Some(old), Some(new)) = (old_capacity, after.capacity) {
            if old != new {
                events.push(Event::CapacityChanged {
                    old_capacity: old,
                    new_capacity: new,
                });
            }
        }
        events.push(Event::Inserted {
            item_id: id,
            index,
            value,
        });
        serialize(&TraceView {
            before,
            after,
            events,
        })
    }
    fn remove_end(&mut self, end: &str) -> Result<JsValue, JsValue> {
        let before = self.view();
        let index = if end == "front" {
            0
        } else {
            self.ids.len().saturating_sub(1)
        };
        let removed = match &mut self.container {
            Container::Stack(c) => c.pop(),
            Container::Queue(c) => {
                if end == "front" {
                    c.dequeue()
                } else {
                    return Err(JsValue::from_str("Queue removes values from the front."));
                }
            }
            Container::Deque(c) => {
                if end == "front" {
                    c.pop_front()
                } else {
                    c.pop_back()
                }
            }
        };
        let events = if let Some(value) = removed {
            let id = self.ids.remove(index);
            vec![Event::Removed {
                item_id: id,
                index,
                value,
            }]
        } else {
            vec![Event::Empty]
        };
        let after = self.view();
        serialize(&TraceView {
            before,
            after,
            events,
        })
    }
    fn access(&self, index: usize) -> Result<JsValue, JsValue> {
        let before = self.view();
        let value = match &self.container {
            Container::Stack(c) => c.iter().nth(index).copied(),
            Container::Queue(c) => c.iter().nth(index).copied(),
            Container::Deque(c) => c.iter().nth(index).copied(),
        };
        let events = if let Some(value) = value {
            vec![Event::Accessed {
                item_id: self.ids[index].clone(),
                index,
                value,
            }]
        } else {
            vec![Event::Empty]
        };
        serialize(&TraceView {
            before: before.clone(),
            after: before,
            events,
        })
    }
    fn allocate_id(&mut self) -> String {
        let id = format!("{}-item-{}", self.structure, self.next_id);
        self.next_id += 1;
        id
    }
}
impl Default for LinearSession {
    fn default() -> Self {
        Self::new("stack").expect("stack is a valid linear container")
    }
}
