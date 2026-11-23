//! A priority queue backed by the binary heap.

use super::BinaryHeap;

/// An entry whose priority determines its position in the heap.
#[derive(Clone, Debug)]
struct Entry<P, T> {
    priority: P,
    value: T,
}

impl<P: PartialEq, T> PartialEq for Entry<P, T> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl<P: Eq, T> Eq for Entry<P, T> {}

impl<P: PartialOrd, T> PartialOrd for Entry<P, T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

impl<P: Ord, T> Ord for Entry<P, T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

/// A max-priority queue of values that do not need to implement `Ord`.
///
/// Entries with greater priorities are returned first. The order of entries
/// with equal priorities is unspecified. Insertion and removal take O(log n);
/// peeking takes O(1).
///
/// # Examples
///
/// ```
/// use data_structures::PriorityQueue;
///
/// let mut jobs = PriorityQueue::new();
/// jobs.push(1, "background");
/// jobs.push(5, "urgent");
/// assert_eq!(jobs.pop(), Some((5, "urgent")));
/// ```
#[derive(Clone, Debug)]
pub struct PriorityQueue<P, T> {
    entries: BinaryHeap<Entry<P, T>>,
}

impl<P, T> PriorityQueue<P, T> {
    /// Creates an empty priority queue.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: BinaryHeap::new(),
        }
    }

    /// Creates an empty priority queue with room for at least `capacity` values.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: BinaryHeap::with_capacity(capacity),
        }
    }

    /// Returns the number of queued values.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns the number of queued values the queue can hold without reallocating.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.entries.capacity()
    }

    /// Returns whether the queue is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Removes all queued values.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl<P: Ord, T> PriorityQueue<P, T> {
    /// Adds a value with the given priority.
    pub fn push(&mut self, priority: P, value: T) {
        self.entries.push(Entry { priority, value });
    }

    /// Returns the value with the greatest priority without removing it.
    #[must_use]
    pub fn peek(&self) -> Option<(&P, &T)> {
        self.entries
            .peek()
            .map(|entry| (&entry.priority, &entry.value))
    }

    /// Removes and returns a value with the greatest priority.
    pub fn pop(&mut self) -> Option<(P, T)> {
        self.entries
            .pop()
            .map(|entry| (entry.priority, entry.value))
    }
}

impl<P, T> Default for PriorityQueue<P, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Ord, T> Extend<(P, T)> for PriorityQueue<P, T> {
    fn extend<I: IntoIterator<Item = (P, T)>>(&mut self, items: I) {
        for (priority, value) in items {
            self.push(priority, value);
        }
    }
}

impl<P: Ord, T> FromIterator<(P, T)> for PriorityQueue<P, T> {
    fn from_iter<I: IntoIterator<Item = (P, T)>>(items: I) -> Self {
        let mut queue = Self::new();
        queue.extend(items);
        queue
    }
}
