use crate::serialization::serialize;
use data_structures::SinglyLinkedList;
use serde::Serialize;
use wasm_bindgen::prelude::*;

const MAX_ITEMS: usize = 40;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkedListView {
    kind: &'static str,
    link_mode: &'static str,
    nodes: Vec<NodeView>,
    head_id: Option<String>,
    tail_id: Option<String>,
    len: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NodeView {
    id: String,
    value: i32,
    next_id: Option<String>,
    prev_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TraceView {
    before: LinkedListView,
    after: LinkedListView,
    events: Vec<EventView>,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum EventView {
    NodeCreated {
        node_id: String,
        value: i32,
    },
    LinkCreated {
        direction: &'static str,
        from_id: String,
        to_id: String,
    },
    HeadChanged {
        from_id: Option<String>,
        to_id: Option<String>,
    },
    TailChanged {
        from_id: Option<String>,
        to_id: Option<String>,
    },
    NodeRemoved {
        node_id: String,
        index: usize,
        value: i32,
    },
    NodeVisited {
        node_id: String,
        endpoint: &'static str,
        value: i32,
    },
    Empty,
}

#[wasm_bindgen]
pub struct SinglyLinkedListSession {
    list: SinglyLinkedList<i32>,
    node_ids: Vec<String>,
    next_node_id: u64,
}

#[wasm_bindgen]
impl SinglyLinkedListSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            list: SinglyLinkedList::new(),
            node_ids: Vec::new(),
            next_node_id: 0,
        }
    }

    pub fn load_example(&mut self) -> Result<JsValue, JsValue> {
        self.list.clear();
        self.node_ids.clear();
        for value in [5, 9, 13] {
            self.list.push_back(value);
            let node_id = self.allocate_node_id();
            self.node_ids.push(node_id);
        }
        serialize(&self.view())
    }

    pub fn reset(&mut self) -> Result<JsValue, JsValue> {
        self.list.clear();
        self.node_ids.clear();
        serialize(&self.view())
    }

    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        serialize(&self.view())
    }

    pub fn push_front(&mut self, value: i32) -> Result<JsValue, JsValue> {
        self.ensure_room()?;
        let before = self.view();
        let old_head = self.node_ids.first().cloned();
        let old_tail = self.node_ids.last().cloned();
        let node_id = self.allocate_node_id();
        self.list.push_front(value);
        self.node_ids.insert(0, node_id.clone());

        let mut events = vec![EventView::NodeCreated {
            node_id: node_id.clone(),
            value,
        }];
        if let Some(old_head_id) = old_head.clone() {
            events.push(EventView::LinkCreated {
                direction: "next",
                from_id: node_id.clone(),
                to_id: old_head_id,
            });
        }
        events.push(EventView::HeadChanged {
            from_id: old_head,
            to_id: Some(node_id.clone()),
        });
        if old_tail.is_none() {
            events.push(EventView::TailChanged {
                from_id: None,
                to_id: Some(node_id),
            });
        }
        self.trace(before, events)
    }

    pub fn push_back(&mut self, value: i32) -> Result<JsValue, JsValue> {
        self.ensure_room()?;
        let before = self.view();
        let old_tail = self.node_ids.last().cloned();
        let node_id = self.allocate_node_id();
        self.list.push_back(value);
        self.node_ids.push(node_id.clone());

        let mut events = vec![EventView::NodeCreated {
            node_id: node_id.clone(),
            value,
        }];
        if let Some(old_tail_id) = old_tail.clone() {
            events.push(EventView::LinkCreated {
                direction: "next",
                from_id: old_tail_id,
                to_id: node_id.clone(),
            });
        } else {
            events.push(EventView::HeadChanged {
                from_id: None,
                to_id: Some(node_id.clone()),
            });
        }
        events.push(EventView::TailChanged {
            from_id: old_tail,
            to_id: Some(node_id),
        });
        self.trace(before, events)
    }

    pub fn pop_front(&mut self) -> Result<JsValue, JsValue> {
        let before = self.view();
        let Some(old_head) = self.node_ids.first().cloned() else {
            return self.trace(before, vec![EventView::Empty]);
        };
        let old_tail = self.node_ids.last().cloned();
        let next_head = self.node_ids.get(1).cloned();
        let index = 0;
        let value = self.list.pop_front().expect("a visual head exists");
        self.node_ids.remove(0);

        let mut events = vec![
            EventView::NodeRemoved {
                node_id: old_head.clone(),
                index,
                value,
            },
            EventView::HeadChanged {
                from_id: Some(old_head),
                to_id: next_head,
            },
        ];
        if self.node_ids.is_empty() {
            events.push(EventView::TailChanged {
                from_id: old_tail,
                to_id: None,
            });
        }
        self.trace(before, events)
    }

    pub fn front(&self) -> Result<JsValue, JsValue> {
        let before = self.view();
        let events = match (self.list.front(), self.node_ids.first()) {
            (Some(value), Some(node_id)) => vec![EventView::NodeVisited {
                node_id: node_id.clone(),
                endpoint: "head",
                value: *value,
            }],
            _ => vec![EventView::Empty],
        };
        self.trace(before, events)
    }

    pub fn back(&self) -> Result<JsValue, JsValue> {
        let before = self.view();
        let events = match (self.list.back(), self.node_ids.last()) {
            (Some(value), Some(node_id)) => vec![EventView::NodeVisited {
                node_id: node_id.clone(),
                endpoint: "tail",
                value: *value,
            }],
            _ => vec![EventView::Empty],
        };
        self.trace(before, events)
    }
}

impl Default for SinglyLinkedListSession {
    fn default() -> Self {
        Self::new()
    }
}

impl SinglyLinkedListSession {
    fn allocate_node_id(&mut self) -> String {
        let id = format!("singly-node-{}", self.next_node_id);
        self.next_node_id += 1;
        id
    }

    fn ensure_room(&self) -> Result<(), JsValue> {
        if self.list.len() >= MAX_ITEMS {
            return Err(JsValue::from_str("可视化示例最多支持 40 个节点。"));
        }
        Ok(())
    }

    fn view(&self) -> LinkedListView {
        let nodes = self
            .list
            .iter()
            .zip(&self.node_ids)
            .enumerate()
            .map(|(index, (value, id))| NodeView {
                id: id.clone(),
                value: *value,
                next_id: self.node_ids.get(index + 1).cloned(),
                prev_id: None,
            })
            .collect();
        LinkedListView {
            kind: "linkedList",
            link_mode: "singly",
            nodes,
            head_id: self.node_ids.first().cloned(),
            tail_id: self.node_ids.last().cloned(),
            len: self.list.len(),
        }
    }

    fn trace(&self, before: LinkedListView, events: Vec<EventView>) -> Result<JsValue, JsValue> {
        serialize(&TraceView {
            before,
            after: self.view(),
            events,
        })
    }
}
