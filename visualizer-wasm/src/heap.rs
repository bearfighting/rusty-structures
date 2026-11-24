use crate::serialization::serialize;
use data_structures::BinaryHeap;
use serde::Serialize;
use wasm_bindgen::prelude::*;

const MAX_ITEMS: usize = 64;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct HeapView {
    kind: &'static str,
    items: Vec<ItemView>,
    len: usize,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemView {
    id: String,
    value: i32,
    index: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TraceView {
    before: HeapView,
    after: HeapView,
    events: Vec<EventView>,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum EventView {
    Inserted {
        item_id: String,
        index: usize,
        value: i32,
    },
    Compared {
        parent_id: String,
        child_id: String,
        parent_index: usize,
        child_index: usize,
        ordering: &'static str,
    },
    ComparedChildren {
        left_id: String,
        right_id: String,
        left_index: usize,
        right_index: usize,
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
        value: i32,
        replacement_id: Option<String>,
        replacement_value: Option<i32>,
    },
    Accessed {
        item_id: String,
        value: i32,
    },
    Empty,
}

#[wasm_bindgen]
pub struct HeapSession {
    heap: BinaryHeap<i32>,
    item_ids: Vec<String>,
    next_id: u64,
}

#[wasm_bindgen]
impl HeapSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            item_ids: Vec::new(),
            next_id: 0,
        }
    }

    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        serialize(&self.snapshot_view())
    }
    pub fn reset(&mut self) -> Result<JsValue, JsValue> {
        self.heap.clear();
        self.item_ids.clear();
        serialize(&self.snapshot_view())
    }
    pub fn load_example(&mut self) -> Result<JsValue, JsValue> {
        self.heap.clear();
        self.item_ids.clear();
        for value in [14, 6, 12, 2, 4, 8, 10] {
            self.heap.push(value);
            let id = self.allocate_id();
            self.item_ids.push(id);
        }
        serialize(&self.snapshot_view())
    }
    pub fn push(&mut self, value: i32) -> Result<JsValue, JsValue> {
        if self.heap.len() >= MAX_ITEMS {
            return Err(JsValue::from_str("可视化示例最多支持 64 个元素。"));
        }
        let before = self.snapshot_view();
        let id = self.allocate_id();
        let mut values: Vec<i32> = self.heap.iter().copied().collect();
        let mut ids = self.item_ids.clone();
        values.push(value);
        ids.push(id.clone());
        let index = values.len() - 1;
        let mut events = vec![EventView::Inserted {
            item_id: id,
            index,
            value,
        }];
        let mut child = index;
        while child > 0 {
            let parent = (child - 1) / 2;
            events.push(EventView::Compared {
                parent_id: ids[parent].clone(),
                child_id: ids[child].clone(),
                parent_index: parent,
                child_index: child,
                ordering: if values[parent] < values[child] {
                    "less"
                } else {
                    "greaterOrEqual"
                },
            });
            if values[parent] >= values[child] {
                break;
            }
            events.push(EventView::Swapped {
                first_id: ids[parent].clone(),
                second_id: ids[child].clone(),
                first_index: parent,
                second_index: child,
            });
            values.swap(parent, child);
            ids.swap(parent, child);
            child = parent;
        }
        self.heap.push(value);
        self.item_ids = ids;
        self.trace(before, events)
    }
    pub fn pop(&mut self) -> Result<JsValue, JsValue> {
        let before = self.snapshot_view();
        if self.heap.is_empty() {
            return self.trace(before, vec![EventView::Empty]);
        }
        let mut values: Vec<i32> = self.heap.iter().copied().collect();
        let mut ids = self.item_ids.clone();
        let removed_value = values[0];
        let removed_id = ids[0].clone();
        values.pop();
        ids.pop();
        let replacement_value = values.first().copied();
        let replacement_id = ids.first().cloned();
        let mut events = vec![EventView::RootRemoved {
            item_id: removed_id,
            value: removed_value,
            replacement_id,
            replacement_value,
        }];
        if !values.is_empty() {
            let mut parent = 0;
            loop {
                let left = parent * 2 + 1;
                if left >= values.len() {
                    break;
                }
                let right = left + 1;
                if right < values.len() {
                    events.push(EventView::ComparedChildren {
                        left_id: ids[left].clone(),
                        right_id: ids[right].clone(),
                        left_index: left,
                        right_index: right,
                        ordering: if values[left] >= values[right] {
                            "leftGreaterOrEqual"
                        } else {
                            "rightGreater"
                        },
                    });
                }
                let greater = if right < values.len() && values[right] > values[left] {
                    right
                } else {
                    left
                };
                events.push(EventView::Compared {
                    parent_id: ids[parent].clone(),
                    child_id: ids[greater].clone(),
                    parent_index: parent,
                    child_index: greater,
                    ordering: if values[parent] < values[greater] {
                        "less"
                    } else {
                        "greaterOrEqual"
                    },
                });
                if values[parent] >= values[greater] {
                    break;
                }
                events.push(EventView::Swapped {
                    first_id: ids[parent].clone(),
                    second_id: ids[greater].clone(),
                    first_index: parent,
                    second_index: greater,
                });
                values.swap(parent, greater);
                ids.swap(parent, greater);
                parent = greater;
            }
        }
        self.heap.pop();
        self.item_ids = ids;
        self.trace(before, events)
    }
    pub fn peek(&self) -> Result<JsValue, JsValue> {
        let before = self.snapshot_view();
        let events = match (self.heap.peek(), self.item_ids.first()) {
            (Some(value), Some(id)) => vec![EventView::Accessed {
                item_id: id.clone(),
                value: *value,
            }],
            _ => vec![EventView::Empty],
        };
        self.trace(before, events)
    }
}

impl Default for HeapSession {
    fn default() -> Self {
        Self::new()
    }
}
impl HeapSession {
    fn allocate_id(&mut self) -> String {
        let id = format!("heap-item-{}", self.next_id);
        self.next_id += 1;
        id
    }
    fn snapshot_view(&self) -> HeapView {
        HeapView {
            kind: "heap",
            items: self
                .heap
                .iter()
                .zip(&self.item_ids)
                .enumerate()
                .map(|(index, (value, id))| ItemView {
                    id: id.clone(),
                    value: *value,
                    index,
                })
                .collect(),
            len: self.heap.len(),
        }
    }
    fn trace(&self, before: HeapView, events: Vec<EventView>) -> Result<JsValue, JsValue> {
        serialize(&TraceView {
            before,
            after: self.snapshot_view(),
            events,
        })
    }
}
