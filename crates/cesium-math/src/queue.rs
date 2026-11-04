//! Queue matching CesiumJS `Core/Queue`.
//! Adapted from CesiumJS (Apache-2.0).

use std::collections::VecDeque;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Queue<T> {
    items: VecDeque<T>,
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Self {
            items: VecDeque::new(),
        }
    }

    pub fn enqueue(&mut self, item: T) {
        self.items.push_back(item);
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.items.pop_front()
    }

    pub fn peek(&self) -> Option<&T> {
        self.items.front()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&T, &T) -> std::cmp::Ordering,
    {
        let slice = self.items.make_contiguous();
        slice.sort_by(compare);
    }
}

impl<T: PartialEq> Queue<T> {
    pub fn contains(&self, item: &T) -> bool {
        self.items.contains(item)
    }
}
