//! A priority queue whose entries are addressed by stable integer indices.

/// A max-priority queue with at most one priority per index in `0..capacity`.
///
/// The heap contains indices, while `priorities` stores each present index's
/// priority and `positions` maps an index back to its heap position. These
/// arrays are updated together on every heap swap. `set`, `remove`, and `pop`
/// take O(log n); `get`, `contains`, and `peek` take O(1). Storage is O(capacity).
/// Greater priorities are returned first; ties are unspecified.
///
/// # Examples
///
/// ```
/// use data_structures::IndexPriorityQueue;
///
/// let mut queue = IndexPriorityQueue::new(4);
/// queue.set(2, 10);
/// queue.set(1, 7);
/// queue.set(1, 12);
/// assert_eq!(queue.pop(), Some((1, 12)));
/// assert_eq!(queue.pop(), Some((2, 10)));
/// ```
#[derive(Clone, Debug)]
pub struct IndexPriorityQueue<P> {
    priorities: Vec<Option<P>>,
    heap: Vec<usize>,
    positions: Vec<Option<usize>>,
}

impl<P> IndexPriorityQueue<P> {
    /// Creates an empty queue that accepts indices in `0..capacity`.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            priorities: std::iter::repeat_with(|| None).take(capacity).collect(),
            heap: Vec::new(),
            positions: vec![None; capacity],
        }
    }

    /// Returns the number of indices currently in the queue.
    #[must_use]
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    /// Returns the number of indices the queue can address.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.priorities.len()
    }

    /// Returns whether the queue has no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Returns whether `index` has a priority in the queue.
    #[must_use]
    pub fn contains(&self, index: usize) -> bool {
        self.positions.get(index).is_some_and(Option::is_some)
    }

    /// Returns the priority stored for `index`, or `None` if absent or invalid.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&P> {
        self.priorities.get(index)?.as_ref()
    }

    /// Returns the index and priority with greatest priority without removing it.
    #[must_use]
    pub fn peek(&self) -> Option<(usize, &P)> {
        let index = *self.heap.first()?;
        Some((
            index,
            self.priorities[index]
                .as_ref()
                .expect("heap index is present"),
        ))
    }

    /// Removes every entry while retaining the fixed index capacity.
    pub fn clear(&mut self) {
        for index in self.heap.drain(..) {
            self.priorities[index] = None;
            self.positions[index] = None;
        }
    }
}

impl<P: Ord> IndexPriorityQueue<P> {
    /// Inserts or updates `index` and returns its previous priority.
    ///
    /// Returns `Err(priority)` when `index` is outside `0..capacity`, preserving
    /// ownership of the rejected priority.
    pub fn set(&mut self, index: usize, priority: P) -> Result<Option<P>, P> {
        if index >= self.capacity() {
            return Err(priority);
        }
        let old = self.priorities[index].replace(priority);
        match self.positions[index] {
            Some(position) => self.repair(position),
            None => {
                self.heap.push(index);
                let position = self.heap.len() - 1;
                self.positions[index] = Some(position);
                self.sift_up(position);
            }
        }
        Ok(old)
    }

    /// Removes and returns the priority for `index`, if present.
    pub fn remove(&mut self, index: usize) -> Option<P> {
        let position = self.positions.get(index).copied().flatten()?;
        let last_position = self.heap.len() - 1;
        self.swap_positions(position, last_position);
        self.heap.pop();
        self.positions[index] = None;
        let priority = self.priorities[index].take();
        if position < self.heap.len() {
            self.repair(position);
        }
        priority
    }

    /// Removes and returns the index with greatest priority and its priority.
    pub fn pop(&mut self) -> Option<(usize, P)> {
        let index = *self.heap.first()?;
        let priority = self.remove(index).expect("heap index is present");
        Some((index, priority))
    }

    fn repair(&mut self, position: usize) {
        if position > 0 && self.greater(position, (position - 1) / 2) {
            self.sift_up(position);
        } else {
            self.sift_down(position);
        }
    }

    fn greater(&self, left: usize, right: usize) -> bool {
        let left_index = self.heap[left];
        let right_index = self.heap[right];
        self.priorities[left_index]
            .as_ref()
            .expect("heap index is present")
            > self.priorities[right_index]
                .as_ref()
                .expect("heap index is present")
    }

    fn swap_positions(&mut self, left: usize, right: usize) {
        self.heap.swap(left, right);
        self.positions[self.heap[left]] = Some(left);
        self.positions[self.heap[right]] = Some(right);
    }

    fn sift_up(&mut self, mut position: usize) {
        while position > 0 {
            let parent = (position - 1) / 2;
            if !self.greater(position, parent) {
                break;
            }
            self.swap_positions(position, parent);
            position = parent;
        }
    }

    fn sift_down(&mut self, mut position: usize) {
        loop {
            let left = position * 2 + 1;
            if left >= self.heap.len() {
                break;
            }
            let right = left + 1;
            let greater_child = if right < self.heap.len() && self.greater(right, left) {
                right
            } else {
                left
            };
            if !self.greater(greater_child, position) {
                break;
            }
            self.swap_positions(position, greater_child);
            position = greater_child;
        }
    }
}
