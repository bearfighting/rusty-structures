//! Shared private storage for [`Queue`](crate::Queue) and [`Deque`](crate::Deque).
//!
//! The buffer stores a fixed set of initialized `Option<T>` slots. Values are
//! kept in one logical run beginning at `head`; when that run reaches the end
//! of the storage, it continues again at slot zero.

use crate::{TryReserveError, Vector};
use std::iter::FusedIterator;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RingBuffer<T> {
    slots: Vector<Option<T>>,
    head: usize,
    len: usize,
}

impl<T> RingBuffer<T> {
    pub(crate) const fn new() -> Self {
        Self {
            slots: Vector::new(),
            head: 0,
            len: 0,
        }
    }

    pub(crate) fn try_with_capacity(capacity: usize) -> Result<Self, TryReserveError> {
        let mut slots = Vector::try_with_capacity(capacity)?;
        for _ in 0..capacity {
            slots.push(None);
        }
        Ok(Self {
            slots,
            head: 0,
            len: 0,
        })
    }

    pub(crate) const fn len(&self) -> usize {
        self.len
    }

    pub(crate) const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) const fn capacity(&self) -> usize {
        self.slots.len()
    }

    pub(crate) fn try_push_back(&mut self, value: T) -> Result<(), (T, TryReserveError)> {
        if self.len == self.capacity()
            && let Err(error) = self.grow()
        {
            return Err((value, error));
        }

        let index = self.index_at_offset(self.len);
        self.slots[index] = Some(value);
        self.len += 1;
        Ok(())
    }

    pub(crate) fn try_push_front(&mut self, value: T) -> Result<(), (T, TryReserveError)> {
        if self.len == self.capacity()
            && let Err(error) = self.grow()
        {
            return Err((value, error));
        }

        if self.head == 0 {
            self.head = self.capacity() - 1;
        } else {
            self.head -= 1;
        }
        self.slots[self.head] = Some(value);
        self.len += 1;
        Ok(())
    }

    pub(crate) fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let value = self.slots[self.head]
            .take()
            .expect("every live ring slot contains a value");
        self.len -= 1;
        self.advance_head();
        Some(value)
    }

    pub(crate) fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let index = self.index_at_offset(self.len - 1);
        let value = self.slots[index]
            .take()
            .expect("every live ring slot contains a value");
        self.len -= 1;
        if self.is_empty() {
            self.head = 0;
        }
        Some(value)
    }

    pub(crate) fn front(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            self.slots[self.head].as_ref()
        }
    }

    pub(crate) fn front_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            self.slots[self.head].as_mut()
        }
    }

    pub(crate) fn back(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            self.slots[self.index_at_offset(self.len - 1)].as_ref()
        }
    }

    pub(crate) fn back_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            let index = self.index_at_offset(self.len - 1);
            self.slots[index].as_mut()
        }
    }

    pub(crate) fn clear(&mut self) {
        while let Some(value) = self.pop_front() {
            drop(value);
        }
    }

    pub(crate) fn iter(&self) -> RingIter<'_, T> {
        RingIter::new(self)
    }

    pub(crate) fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut T> + FusedIterator {
        let capacity = self.capacity();
        let first_len = self.len.min(capacity - self.head);
        let second_len = self.len - first_len;
        let (before_head, from_head) = self.slots.as_mut_slice().split_at_mut(self.head);

        from_head[..first_len]
            .iter_mut()
            .map(occupied_slot)
            .chain(before_head[..second_len].iter_mut().map(occupied_slot))
    }

    fn advance_head(&mut self) {
        if self.is_empty() || self.head + 1 == self.capacity() {
            self.head = 0;
        } else {
            self.head += 1;
        }
    }

    fn index_at_offset(&self, offset: usize) -> usize {
        let slots_until_end = self.capacity() - self.head;
        if offset >= slots_until_end {
            offset - slots_until_end
        } else {
            self.head + offset
        }
    }

    fn grow(&mut self) -> Result<(), TryReserveError> {
        let new_capacity = if self.capacity() == 0 {
            4
        } else {
            self.capacity()
                .checked_mul(2)
                .ok_or(TryReserveError::CapacityOverflow)?
        };
        let mut new_slots = Vector::try_with_capacity(new_capacity)?;
        for _ in 0..new_capacity {
            new_slots.push(None);
        }

        for offset in 0..self.len {
            let old_index = self.index_at_offset(offset);
            let value = self.slots[old_index]
                .take()
                .expect("every live ring slot contains a value");
            new_slots[offset] = Some(value);
        }

        self.slots = new_slots;
        self.head = 0;
        Ok(())
    }
}

fn occupied_slot<T>(slot: &mut Option<T>) -> &mut T {
    slot.as_mut()
        .expect("every live ring slot contains a value")
}

/// A shared iterator over values in a ring buffer's logical order.
pub struct RingIter<'a, T> {
    buffer: &'a RingBuffer<T>,
    front: usize,
    back: usize,
}

impl<'a, T> RingIter<'a, T> {
    fn new(buffer: &'a RingBuffer<T>) -> Self {
        Self {
            buffer,
            front: 0,
            back: buffer.len,
        }
    }
}

impl<'a, T> Iterator for RingIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            return None;
        }
        let index = self.buffer.index_at_offset(self.front);
        self.front += 1;
        Some(
            self.buffer.slots[index]
                .as_ref()
                .expect("every live ring slot contains a value"),
        )
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.back - self.front;
        (remaining, Some(remaining))
    }
}

impl<T> DoubleEndedIterator for RingIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            return None;
        }
        self.back -= 1;
        let index = self.buffer.index_at_offset(self.back);
        Some(
            self.buffer.slots[index]
                .as_ref()
                .expect("every live ring slot contains a value"),
        )
    }
}

impl<T> ExactSizeIterator for RingIter<'_, T> {}
impl<T> FusedIterator for RingIter<'_, T> {}
