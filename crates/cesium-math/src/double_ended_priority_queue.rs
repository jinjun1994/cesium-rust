//! DoubleEndedPriorityQueue matching CesiumJS `Core/DoubleEndedPriorityQueue`.
//! Adapted from CesiumJS (Apache-2.0).

use std::cmp::Ordering;

pub type Comparator<T> = Box<dyn Fn(&T, &T) -> Ordering + Send + Sync>;

pub struct DoubleEndedPriorityQueue<T> {
    array: Vec<T>,
    comparator: Comparator<T>,
    maximum_length: Option<usize>,
}

impl<T: std::fmt::Debug> std::fmt::Debug for DoubleEndedPriorityQueue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DoubleEndedPriorityQueue")
            .field("array", &self.array)
            .field("maximum_length", &self.maximum_length)
            .finish()
    }
}

impl<T> DoubleEndedPriorityQueue<T> {
    pub fn new<F>(comparator: F) -> Self
    where
        F: Fn(&T, &T) -> Ordering + Send + Sync + 'static,
    {
        Self {
            array: Vec::new(),
            comparator: Box::new(comparator),
            maximum_length: None,
        }
    }

    pub fn with_maximum_length<F>(comparator: F, maximum_length: usize) -> Self
    where
        F: Fn(&T, &T) -> Ordering + Send + Sync + 'static,
    {
        Self {
            array: Vec::with_capacity(maximum_length),
            comparator: Box::new(comparator),
            maximum_length: Some(maximum_length),
        }
    }

    pub fn len(&self) -> usize {
        self.array.len()
    }

    pub fn is_empty(&self) -> bool {
        self.array.is_empty()
    }

    pub fn maximum_length(&self) -> Option<usize> {
        self.maximum_length
    }

    pub fn set_maximum_length(&mut self, max_len: Option<usize>) {
        if let Some(val) = max_len {
            while self.array.len() > val {
                self.remove_minimum();
            }
        }
        self.maximum_length = max_len;
    }

    pub fn internal_array(&self) -> &[T] {
        &self.array
    }

    pub fn reset(&mut self) {
        self.array.clear();
    }

    pub fn resort(&mut self) {
        let len = self.array.len();
        for i in 0..len {
            push_up(&mut self.array, i, &*self.comparator);
        }
    }

    pub fn get_minimum(&self) -> Option<&T> {
        self.array.first()
    }

    pub fn get_maximum(&self) -> Option<&T> {
        let len = self.array.len();
        if len == 0 {
            None
        } else if len <= 2 {
            self.array.last()
        } else if (self.comparator)(&self.array[1], &self.array[2]) == Ordering::Greater {
            Some(&self.array[1])
        } else {
            Some(&self.array[2])
        }
    }

    pub fn insert(&mut self, element: T) -> Option<T> {
        let mut removed_element = None;
        if let Some(max_len) = self.maximum_length {
            if max_len == 0 {
                return None;
            } else if self.array.len() == max_len {
                let min_elem = &self.array[0];
                if (self.comparator)(&element, min_elem) != Ordering::Greater {
                    return Some(element);
                }
                removed_element = self.remove_minimum();
            }
        }

        let index = self.array.len();
        self.array.push(element);
        push_up(&mut self.array, index, &*self.comparator);
        removed_element
    }

    pub fn remove_minimum(&mut self) -> Option<T> {
        if self.array.is_empty() {
            return None;
        }
        let min_element = self.array.swap_remove(0);
        let len = self.array.len();
        if len > 0 {
            push_down(&mut self.array, 0, len, &*self.comparator);
        }
        Some(min_element)
    }

    pub fn remove_maximum(&mut self) -> Option<T> {
        let len = self.array.len();
        if len == 0 {
            return None;
        }
        if len <= 2 {
            return self.array.pop();
        }
        let max_idx = if (self.comparator)(&self.array[1], &self.array[2]) == Ordering::Greater {
            1
        } else {
            2
        };
        let max_element = self.array.swap_remove(max_idx);
        let len = self.array.len();
        if len >= 3 {
            push_down(&mut self.array, max_idx, len, &*self.comparator);
        }
        Some(max_element)
    }
}

fn is_min_level(index: usize) -> bool {
    let level = (index + 1).ilog2();
    level.is_multiple_of(2)
}

fn push_up<T, F>(array: &mut [T], mut index: usize, comparator: &F)
where
    F: Fn(&T, &T) -> Ordering + ?Sized,
{
    if index == 0 {
        return;
    }
    let on_min_level = is_min_level(index);
    let parent_index = (index - 1) / 2;
    let less_than_parent = comparator(&array[index], &array[parent_index]) == Ordering::Less;

    if less_than_parent != on_min_level {
        array.swap(index, parent_index);
        index = parent_index;
    }

    let curr_on_min_level = is_min_level(index);
    while index >= 3 {
        let grandparent_index = (index - 3) / 4;
        let cmp = comparator(&array[index], &array[grandparent_index]);
        let should_swap = if curr_on_min_level {
            cmp == Ordering::Less
        } else {
            cmp == Ordering::Greater
        };
        if !should_swap {
            break;
        }
        array.swap(index, grandparent_index);
        index = grandparent_index;
    }
}

fn push_down<T, F>(array: &mut [T], mut index: usize, length: usize, comparator: &F)
where
    F: Fn(&T, &T) -> Ordering + ?Sized,
{
    let on_min_level = is_min_level(index);

    while 2 * index + 1 < length {
        let left_child = 2 * index + 1;
        let mut target = left_child;
        let right_child = left_child + 1;
        if right_child < length {
            let right_is_candidate = if on_min_level {
                comparator(&array[right_child], &array[target]) == Ordering::Less
            } else {
                comparator(&array[right_child], &array[target]) == Ordering::Greater
            };
            if right_is_candidate {
                target = right_child;
            }
            let grandchild_start = 2 * left_child + 1;
            let grandchild_count = (length.saturating_sub(grandchild_start)).min(4);
            for i in 0..grandchild_count {
                let gc_idx = grandchild_start + i;
                let gc_is_candidate = if on_min_level {
                    comparator(&array[gc_idx], &array[target]) == Ordering::Less
                } else {
                    comparator(&array[gc_idx], &array[target]) == Ordering::Greater
                };
                if gc_is_candidate {
                    target = gc_idx;
                }
            }
        }

        let should_swap_target = if on_min_level {
            comparator(&array[target], &array[index]) == Ordering::Less
        } else {
            comparator(&array[target], &array[index]) == Ordering::Greater
        };

        if should_swap_target {
            array.swap(target, index);
            if target != left_child && target != right_child {
                let parent_of_grandchild = (target - 1) / 2;
                let should_swap_parent = if on_min_level {
                    comparator(&array[target], &array[parent_of_grandchild]) == Ordering::Greater
                } else {
                    comparator(&array[target], &array[parent_of_grandchild]) == Ordering::Less
                };
                if should_swap_parent {
                    array.swap(target, parent_of_grandchild);
                }
            }
            index = target;
        } else {
            break;
        }
    }
}
