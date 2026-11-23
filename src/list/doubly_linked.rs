//! An owning doubly linked list.
//!
//! Nodes are owned by a forward chain of `Box` values. Each node also stores a
//! non-owning pointer to its predecessor, and the list stores a non-owning
//! pointer to its final node. This supports O(1) insertion and removal at both
//! ends without exposing pointers to callers.

mod iter;
mod traits;

pub use iter::{IntoIter, Iter, IterMut};
use std::ptr::NonNull;

struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
    prev: Option<NonNull<Node<T>>>,
}

/// A doubly linked list that owns its elements.
///
/// Insertion, removal, and inspection at either end take O(1) time. Traversal
/// takes O(n) time. The forward links own nodes; backward links are private,
/// non-owning pointers maintained by the list.
///
/// # Examples
///
/// ```
/// use data_structures::DoublyLinkedList;
///
/// let mut values = DoublyLinkedList::new();
/// values.push_back(2);
/// values.push_front(1);
/// values.push_back(3);
///
/// assert_eq!(values.iter().copied().collect::<Vec<_>>(), [1, 2, 3]);
/// assert_eq!(values.iter().rev().copied().collect::<Vec<_>>(), [3, 2, 1]);
/// assert_eq!(values.pop_back(), Some(3));
/// ```
pub struct DoublyLinkedList<T> {
    head: Option<Box<Node<T>>>,
    // SAFETY INVARIANT: if Some, tail points to the final node owned by the
    // forward chain starting at head. It is updated before that node is removed.
    tail: Option<NonNull<Node<T>>>,
    len: usize,
}

// SAFETY: the list exclusively owns every node through Box. Moving the list
// leaves its heap allocated nodes, and the tail pointer target, at stable places.
unsafe impl<T: Send> Send for DoublyLinkedList<T> {}

// SAFETY: shared list access only exposes shared references to elements.
// Mutating the nodes requires &mut self, which prevents concurrent mutation.
unsafe impl<T: Sync> Sync for DoublyLinkedList<T> {}

impl<T> DoublyLinkedList<T> {
    /// Creates an empty list.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
        }
    }

    /// Returns the number of elements in the list.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the list contains no elements.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Adds an element to the front of the list.
    pub fn push_front(&mut self, value: T) {
        let mut node = Box::new(Node {
            value,
            next: self.head.take(),
            prev: None,
        });
        let node_pointer = NonNull::from(node.as_mut());

        if let Some(old_head) = node.next.as_deref_mut() {
            old_head.prev = Some(node_pointer);
        } else {
            self.tail = Some(node_pointer);
        }

        self.head = Some(node);
        self.len += 1;
    }

    /// Adds an element to the back of the list.
    pub fn push_back(&mut self, value: T) {
        let mut node = Box::new(Node {
            value,
            next: None,
            prev: self.tail,
        });
        let node_pointer = NonNull::from(node.as_mut());

        if let Some(mut old_tail) = self.tail {
            // SAFETY: old_tail points to the final node owned by head. The
            // exclusive borrow of self lets us update that node's next link.
            unsafe { old_tail.as_mut().next = Some(node) };
        } else {
            self.head = Some(node);
        }

        self.tail = Some(node_pointer);
        self.len += 1;
    }

    /// Removes and returns the first element, or `None` if the list is empty.
    pub fn pop_front(&mut self) -> Option<T> {
        let mut first = self.head.take()?;
        self.head = first.next.take();
        self.len -= 1;

        if let Some(new_head) = self.head.as_deref_mut() {
            new_head.prev = None;
        } else {
            self.tail = None;
        }

        Some(first.value)
    }

    /// Removes and returns the last element, or `None` if the list is empty.
    pub fn pop_back(&mut self) -> Option<T> {
        let old_tail = self.tail?;

        if self.len == 1 {
            let last = self.head.take().expect("a one-element list has a head");
            self.tail = None;
            self.len = 0;
            return Some(last.value);
        }

        // SAFETY: old_tail points to the final node owned by head, and a list
        // with more than one node gives it a valid predecessor in the chain.
        let mut previous = unsafe {
            old_tail
                .as_ref()
                .prev
                .expect("the tail of a multi-element list has a predecessor")
        };

        // SAFETY: previous is a live node in the owned chain. Taking its next
        // link detaches and transfers ownership of the final Box to `last`.
        let last = unsafe {
            previous
                .as_mut()
                .next
                .take()
                .expect("the predecessor owns the final node")
        };

        self.tail = Some(previous);
        self.len -= 1;
        Some(last.value)
    }

    /// Returns a shared reference to the first element, or `None` if empty.
    #[must_use]
    pub fn front(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.value)
    }

    /// Returns a mutable reference to the first element, or `None` if empty.
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| &mut node.value)
    }

    /// Returns a shared reference to the last element, or `None` if empty.
    #[must_use]
    pub fn back(&self) -> Option<&T> {
        self.tail.map(|tail| {
            // SAFETY: the tail invariant guarantees this points to a live node.
            unsafe { &tail.as_ref().value }
        })
    }

    /// Returns a mutable reference to the last element, or `None` if empty.
    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.tail.map(|mut tail| {
            // SAFETY: the tail invariant guarantees this points to a live node;
            // the exclusive borrow of self guarantees unique access.
            unsafe { &mut tail.as_mut().value }
        })
    }

    /// Removes all elements from the list.
    pub fn clear(&mut self) {
        while self.pop_front().is_some() {}
    }

    /// Returns a double-ended iterator from the first element to the last.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(self)
    }

    /// Returns a mutable iterator from the first element to the last.
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            next: self.head.as_deref_mut(),
            remaining: self.len,
        }
    }
}

impl<T> Drop for DoublyLinkedList<T> {
    fn drop(&mut self) {
        // Remove the forward links one at a time to avoid recursive destruction
        // of a long chain. Backward pointers never own nodes.
        while let Some(mut node) = self.head.take() {
            self.head = node.next.take();
        }
        self.tail = None;
        self.len = 0;
    }
}
