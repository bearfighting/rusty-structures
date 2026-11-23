//! Traceable operations for visual teaching tools.

use super::{BinarySearchTree, Node};
use std::cmp::Ordering;

/// A semantic step emitted while a BST operation runs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BstEvent<T> {
    /// Compared the query key with a node.
    Compare { node: T, key: T, ordering: Ordering },
    /// Continued into the left child.
    DescendLeft { from: T },
    /// Continued into the right child.
    DescendRight { from: T },
    /// Added a new node.
    Inserted { key: T },
    /// The key already existed during insertion.
    Duplicate { key: T },
    /// A search found its key.
    Found { key: T },
    /// A search or removal did not find its key.
    Missing { key: T },
    /// A two-child deletion selected its in-order successor.
    SuccessorSelected { removed: T, successor: T },
    /// Removed the matching node.
    Removed { key: T },
}

/// A read-only copy of a node used by a visualizer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BstTreeSnapshotNode<T> {
    /// The unique value stored at this node.
    pub value: T,
    /// Snapshot of the left child.
    pub left: Option<Box<Self>>,
    /// Snapshot of the right child.
    pub right: Option<Box<Self>>,
}

/// The shape and size of a BST at one point in time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BstTreeSnapshot<T> {
    /// Root node, or `None` for an empty tree.
    pub root: Option<Box<BstTreeSnapshotNode<T>>>,
    /// Number of stored values.
    pub len: usize,
}

/// Events and final structure produced by one real BST operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BstTrace<T> {
    /// Algorithm steps in execution order.
    pub events: Vec<BstEvent<T>>,
    /// The tree shape after the operation completes.
    pub snapshot: BstTreeSnapshot<T>,
}

impl<T: Ord + Clone> BinarySearchTree<T> {
    /// Copies the current tree shape into a renderer-friendly snapshot.
    pub fn snapshot_for_visualization(&self) -> BstTreeSnapshot<T> {
        BstTreeSnapshot {
            root: snapshot_node(self.root.as_deref()),
            len: self.len,
        }
    }

    /// Inserts a value while recording the comparisons and path taken.
    pub fn insert_traced(&mut self, value: T) -> BstTrace<T> {
        let mut events = Vec::new();
        let mut link = &mut self.root;
        loop {
            match link {
                Some(node) => {
                    let ordering = value.cmp(&node.value);
                    events.push(BstEvent::Compare {
                        node: node.value.clone(),
                        key: value.clone(),
                        ordering,
                    });
                    match ordering {
                        Ordering::Less => {
                            events.push(BstEvent::DescendLeft {
                                from: node.value.clone(),
                            });
                            link = &mut node.left;
                        }
                        Ordering::Greater => {
                            events.push(BstEvent::DescendRight {
                                from: node.value.clone(),
                            });
                            link = &mut node.right;
                        }
                        Ordering::Equal => {
                            events.push(BstEvent::Duplicate { key: value });
                            break;
                        }
                    }
                }
                None => {
                    *link = Some(Box::new(Node {
                        value: value.clone(),
                        left: None,
                        right: None,
                    }));
                    self.len += 1;
                    events.push(BstEvent::Inserted { key: value });
                    break;
                }
            }
        }
        self.trace(events)
    }

    /// Searches for a value while recording every comparison and branch.
    pub fn search_traced(&self, key: &T) -> BstTrace<T> {
        let mut events = Vec::new();
        let mut node = self.root.as_deref();
        while let Some(current) = node {
            let ordering = key.cmp(&current.value);
            events.push(BstEvent::Compare {
                node: current.value.clone(),
                key: key.clone(),
                ordering,
            });
            match ordering {
                Ordering::Less => {
                    events.push(BstEvent::DescendLeft {
                        from: current.value.clone(),
                    });
                    node = current.left.as_deref();
                }
                Ordering::Greater => {
                    events.push(BstEvent::DescendRight {
                        from: current.value.clone(),
                    });
                    node = current.right.as_deref();
                }
                Ordering::Equal => {
                    events.push(BstEvent::Found { key: key.clone() });
                    return self.trace(events);
                }
            }
        }
        events.push(BstEvent::Missing { key: key.clone() });
        self.trace(events)
    }

    /// Removes a value while recording the search path and successor choice.
    pub fn remove_traced(&mut self, key: &T) -> BstTrace<T> {
        let mut events = Vec::new();
        let mut link = &mut self.root;
        loop {
            let ordering = match link.as_ref() {
                Some(node) => {
                    let ordering = key.cmp(&node.value);
                    events.push(BstEvent::Compare {
                        node: node.value.clone(),
                        key: key.clone(),
                        ordering,
                    });
                    ordering
                }
                None => {
                    events.push(BstEvent::Missing { key: key.clone() });
                    return self.trace(events);
                }
            };
            match ordering {
                Ordering::Less => {
                    let node = link.as_mut().expect("link contains a node");
                    events.push(BstEvent::DescendLeft {
                        from: node.value.clone(),
                    });
                    link = &mut node.left;
                }
                Ordering::Greater => {
                    let node = link.as_mut().expect("link contains a node");
                    events.push(BstEvent::DescendRight {
                        from: node.value.clone(),
                    });
                    link = &mut node.right;
                }
                Ordering::Equal => {
                    let mut removed = link.take().expect("matched link contains a node");
                    let replacement = match (removed.left.take(), removed.right.take()) {
                        (None, right) => right,
                        (left, None) => left,
                        (Some(left), Some(right)) => {
                            let successor_value = min_value(&right).clone();
                            events.push(BstEvent::SuccessorSelected {
                                removed: removed.value.clone(),
                                successor: successor_value,
                            });
                            let mut right = Some(right);
                            let mut successor = super::take_min_iterative(&mut right);
                            successor.left = Some(left);
                            successor.right = right;
                            Some(successor)
                        }
                    };
                    *link = replacement;
                    self.len -= 1;
                    events.push(BstEvent::Removed { key: removed.value });
                    return self.trace(events);
                }
            }
        }
    }

    fn trace(&self, events: Vec<BstEvent<T>>) -> BstTrace<T> {
        BstTrace {
            events,
            snapshot: BstTreeSnapshot {
                root: snapshot_node(self.root.as_deref()),
                len: self.len,
            },
        }
    }
}

fn min_value<T>(mut node: &Node<T>) -> &T {
    while let Some(left) = node.left.as_deref() {
        node = left;
    }
    &node.value
}

fn snapshot_node<T: Clone>(node: Option<&Node<T>>) -> Option<Box<BstTreeSnapshotNode<T>>> {
    node.map(|node| {
        Box::new(BstTreeSnapshotNode {
            value: node.value.clone(),
            left: snapshot_node(node.left.as_deref()),
            right: snapshot_node(node.right.as_deref()),
        })
    })
}

#[cfg(test)]
mod tests {
    use super::{BinarySearchTree, BstEvent};
    use std::cmp::Ordering;

    #[test]
    fn traced_operations_match_tree_state_and_report_key_paths() {
        let mut tree = BinarySearchTree::new();
        tree.insert_traced(8);
        tree.insert_traced(4);
        tree.insert_traced(12);

        let duplicate = tree.insert_traced(4);
        assert!(matches!(
            duplicate.events.last(),
            Some(BstEvent::Duplicate { key: 4 })
        ));
        assert_eq!(duplicate.snapshot.len, 3);

        let search = tree.search_traced(&10);
        assert!(search.events.contains(&BstEvent::Compare {
            node: 8,
            key: 10,
            ordering: Ordering::Greater
        }));
        assert!(search.events.contains(&BstEvent::Missing { key: 10 }));
        assert_eq!(search.snapshot.len, 3);

        let removal = tree.remove_traced(&8);
        assert!(removal.events.contains(&BstEvent::SuccessorSelected {
            removed: 8,
            successor: 12
        }));
        assert!(removal.events.contains(&BstEvent::Removed { key: 8 }));
        assert_eq!(removal.snapshot.len, 2);
        assert_eq!(tree.iter().copied().collect::<Vec<_>>(), [4, 12]);
    }
}
