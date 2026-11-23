//! An owning singly linked list with a maintained tail pointer.
//!
//! The head owns the chain of nodes through `Box`. The tail is a non-owning
//! pointer to the final node, which makes `push_back` and `back` O(1). The
//! pointer is only used while the node remains owned by the chain.

use std::ptr::NonNull;

mod iter;
mod traits;

pub use iter::{IntoIter, Iter, IterMut};

struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

/// A singly linked list that owns its elements.
///
/// `push_front`, `push_back`, `pop_front`, `front`, and `back` take O(1) time.
/// `back` is O(1) because the list maintains a pointer to its final node.
/// Traversal is O(n). Removing the last element is not provided because a
/// singly linked list cannot find its previous node in O(1) time.
///
/// # Examples
///
/// ```
/// use data_structures::SinglyLinkedList;
///
/// let mut values = SinglyLinkedList::new();
/// values.push_back(2);
/// values.push_front(1);
/// values.push_back(3);
///
/// assert_eq!(values.front(), Some(&1));
/// assert_eq!(values.back(), Some(&3));
/// assert_eq!(values.iter().copied().collect::<Vec<_>>(), [1, 2, 3]);
/// ```
pub struct SinglyLinkedList<T> {
    head: Option<Box<Node<T>>>,
    // SAFETY INVARIANT: when Some, tail points to the final node in the chain
    // owned by head. It never owns the node and is reset before that node is
    // removed or the list becomes empty.
    tail: Option<NonNull<Node<T>>>,
    len: usize,
}

// SAFETY: all nodes are exclusively owned by this list. Moving the list moves
// only the Box handles; the nodes and the tail target stay at stable addresses.
unsafe impl<T: Send> Send for SinglyLinkedList<T> {}

// SAFETY: shared access only exposes shared references to T. Mutation requires
// &mut self, which prevents concurrent access through the list.
unsafe impl<T: Sync> Sync for SinglyLinkedList<T> {}

impl<T> SinglyLinkedList<T> {
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
        });
        let node_pointer = NonNull::from(node.as_mut());

        if self.tail.is_none() {
            self.tail = Some(node_pointer);
        }
        self.head = Some(node);
        self.len += 1;
    }

    /// Adds an element to the back of the list.
    pub fn push_back(&mut self, value: T) {
        let mut node = Box::new(Node { value, next: None });
        let node_pointer = NonNull::from(node.as_mut());

        match self.tail {
            Some(mut tail) => {
                // SAFETY: tail points to the final node owned by head, and the
                // exclusive borrow of self allows us to update its next link.
                unsafe { tail.as_mut().next = Some(node) };
            }
            None => self.head = Some(node),
        }

        self.tail = Some(node_pointer);
        self.len += 1;
    }

    /// Removes and returns the first element, or `None` if the list is empty.
    pub fn pop_front(&mut self) -> Option<T> {
        let mut first = self.head.take()?;
        self.head = first.next.take();
        self.len -= 1;

        if self.head.is_none() {
            self.tail = None;
        }

        Some(first.value)
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
            // SAFETY: the tail invariant guarantees this pointer addresses the
            // final node still owned by the list.
            unsafe { &tail.as_ref().value }
        })
    }

    /// Returns a mutable reference to the last element, or `None` if empty.
    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.tail.map(|mut tail| {
            // SAFETY: the tail invariant guarantees this pointer addresses the
            // final node still owned by the list; &mut self gives unique access.
            unsafe { &mut tail.as_mut().value }
        })
    }

    /// Removes all elements from the list.
    pub fn clear(&mut self) {
        while self.pop_front().is_some() {}
    }

    /// Returns an iterator from the first element to the last.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(self)
    }

    /// Returns a mutable iterator from the first element to the last.
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut::new(self)
    }
}

impl<T> Drop for SinglyLinkedList<T> {
    fn drop(&mut self) {
        // Unlink one node at a time to avoid recursively dropping a long chain.
        while let Some(mut node) = self.head.take() {
            self.head = node.next.take();
        }
        self.tail = None;
        self.len = 0;
    }
}
