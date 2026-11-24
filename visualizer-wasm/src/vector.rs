use crate::serialization::serialize;
use data_structures::Vector;
use serde::Serialize;
use wasm_bindgen::prelude::*;

const MAX_ITEMS: usize = 64;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SequenceView {
    kind: &'static str,
    items: Vec<ItemView>,
    len: usize,
    capacity: usize,
}

#[derive(Serialize)]
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
    events: Vec<EventView>,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum EventView {
    CapacityChanged {
        old_capacity: usize,
        new_capacity: usize,
    },
    Shift {
        direction: &'static str,
        item_id: String,
        value: i32,
        from_index: usize,
        to_index: usize,
    },
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
    MissingIndex {
        index: usize,
        len: usize,
    },
    Empty,
    ReserveUnchanged {
        capacity: usize,
    },
}

#[wasm_bindgen]
pub struct VectorSession {
    values: Vector<i32>,
    item_ids: Vec<String>,
    next_item_id: u64,
}

#[wasm_bindgen]
impl VectorSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            values: Vector::new(),
            item_ids: Vec::new(),
            next_item_id: 0,
        }
    }

    pub fn load_example(&mut self) -> Result<JsValue, JsValue> {
        self.values.clear();
        self.item_ids.clear();
        for value in [3, 7, 11, 15] {
            self.values.push(value);
            let item_id = self.allocate_item_id();
            self.item_ids.push(item_id);
        }
        serialize(&self.snapshot_view())
    }

    pub fn reset(&mut self) -> Result<JsValue, JsValue> {
        self.values.clear();
        self.item_ids.clear();
        serialize(&self.snapshot_view())
    }

    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        serialize(&self.snapshot_view())
    }

    pub fn push(&mut self, value: i32) -> Result<JsValue, JsValue> {
        self.ensure_room(1)?;
        let before = self.snapshot_view();
        let old_capacity = self.values.capacity();
        self.values
            .try_push(value)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let item_id = self.allocate_item_id();
        self.item_ids.push(item_id.clone());
        let mut events = self.capacity_event(old_capacity);
        events.push(EventView::Inserted {
            item_id,
            index: self.values.len() - 1,
            value,
        });
        self.trace(before, events)
    }

    pub fn pop(&mut self) -> Result<JsValue, JsValue> {
        let before = self.snapshot_view();
        let mut events = Vec::new();
        if let Some(index) = self.values.len().checked_sub(1) {
            let value = self.values.pop().expect("the vector has a last value");
            let item_id = self.item_ids.pop().expect("every value has a visual ID");
            events.push(EventView::Removed {
                item_id,
                index,
                value,
            });
        } else {
            events.push(EventView::Empty);
        }
        self.trace(before, events)
    }

    pub fn insert_at(&mut self, index: u32, value: i32) -> Result<JsValue, JsValue> {
        let index = index as usize;
        if index > self.values.len() {
            return Err(JsValue::from_str("插入索引不能大于当前长度。"));
        }
        self.ensure_room(1)?;
        let before = self.snapshot_view();
        let old_capacity = self.values.capacity();
        self.values
            .try_reserve(1)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let mut events = self.capacity_event(old_capacity);
        for from_index in (index..self.values.len()).rev() {
            events.push(EventView::Shift {
                direction: "right",
                item_id: self.item_ids[from_index].clone(),
                value: self.values[from_index],
                from_index,
                to_index: from_index + 1,
            });
        }
        let item_id = self.allocate_item_id();
        self.values.insert(index, value);
        self.item_ids.insert(index, item_id.clone());
        events.push(EventView::Inserted {
            item_id,
            index,
            value,
        });
        self.trace(before, events)
    }

    pub fn remove_at(&mut self, index: u32) -> Result<JsValue, JsValue> {
        let index = index as usize;
        let before = self.snapshot_view();
        let mut events = Vec::new();
        if index >= self.values.len() {
            events.push(EventView::MissingIndex {
                index,
                len: self.values.len(),
            });
            return self.trace(before, events);
        }

        let value = self.values.remove(index).expect("index was validated");
        let item_id = self.item_ids.remove(index);
        events.push(EventView::Removed {
            item_id,
            index,
            value,
        });
        for from_index in index + 1..before.len {
            events.push(EventView::Shift {
                direction: "left",
                item_id: self.item_ids[from_index - 1].clone(),
                value: self.values[from_index - 1],
                from_index,
                to_index: from_index - 1,
            });
        }
        self.trace(before, events)
    }

    pub fn get(&self, index: u32) -> Result<JsValue, JsValue> {
        let index = index as usize;
        let before = self.snapshot_view();
        let events = match self.values.get(index) {
            Some(value) => vec![EventView::Accessed {
                item_id: self.item_ids[index].clone(),
                index,
                value: *value,
            }],
            None => vec![EventView::MissingIndex {
                index,
                len: self.values.len(),
            }],
        };
        self.trace(before, events)
    }

    pub fn reserve(&mut self, additional: u32) -> Result<JsValue, JsValue> {
        let additional = additional as usize;
        self.ensure_room(additional)?;
        let before = self.snapshot_view();
        let old_capacity = self.values.capacity();
        self.values
            .try_reserve(additional)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let mut events = self.capacity_event(old_capacity);
        if events.is_empty() {
            events.push(EventView::ReserveUnchanged {
                capacity: self.values.capacity(),
            });
        }
        self.trace(before, events)
    }
}

impl Default for VectorSession {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorSession {
    fn allocate_item_id(&mut self) -> String {
        let id = format!("vector-item-{}", self.next_item_id);
        self.next_item_id += 1;
        id
    }

    fn ensure_room(&self, additional: usize) -> Result<(), JsValue> {
        if self.values.len().saturating_add(additional) > MAX_ITEMS {
            return Err(JsValue::from_str("可视化示例最多支持 64 个元素。"));
        }
        Ok(())
    }

    fn snapshot_view(&self) -> SequenceView {
        SequenceView {
            kind: "sequence",
            items: self
                .values
                .iter()
                .zip(&self.item_ids)
                .map(|(value, id)| ItemView {
                    id: id.clone(),
                    label: value.to_string(),
                    value: *value,
                })
                .collect(),
            len: self.values.len(),
            capacity: self.values.capacity(),
        }
    }

    fn capacity_event(&self, old_capacity: usize) -> Vec<EventView> {
        let new_capacity = self.values.capacity();
        if new_capacity == old_capacity {
            Vec::new()
        } else {
            vec![EventView::CapacityChanged {
                old_capacity,
                new_capacity,
            }]
        }
    }

    fn trace(&self, before: SequenceView, events: Vec<EventView>) -> Result<JsValue, JsValue> {
        serialize(&TraceView {
            before,
            after: self.snapshot_view(),
            events,
        })
    }
}
