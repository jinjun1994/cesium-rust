// Array implementation of a heap with configurable comparator and optional maximum length.

#[derive(Debug, Clone)]
pub struct Heap<T, F>
where
    F: Fn(&T, &T) -> std::cmp::Ordering,
{
    comparator: F,
    array: Vec<T>,
    maximum_length: Option<usize>,
}

impl<T, F> Heap<T, F>
where
    F: Fn(&T, &T) -> std::cmp::Ordering,
{
    pub fn new(comparator: F) -> Self {
        Self {
            comparator,
            array: Vec::new(),
            maximum_length: None,
        }
    }

    pub fn with_maximum_length(comparator: F, maximum_length: usize) -> Self {
        Self {
            comparator,
            array: Vec::new(),
            maximum_length: Some(maximum_length),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.array.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.array.is_empty()
    }

    #[inline]
    pub fn maximum_length(&self) -> Option<usize> {
        self.maximum_length
    }

    pub fn set_maximum_length(&mut self, maximum_length: Option<usize>) {
        if let Some(max) = maximum_length {
            if max < self.array.len() {
                self.array.truncate(max);
            }
        }
        self.maximum_length = maximum_length;
    }

    #[inline]
    pub fn internal_array(&self) -> &[T] {
        &self.array
    }

    pub fn reserve(&mut self, length: usize) {
        if length > self.array.len() {
            self.array.reserve(length - self.array.len());
        }
    }

    pub fn heapify(&mut self, mut index: usize) {
        let length = self.array.len();
        let mut candidate;

        loop {
            let right = 2 * (index + 1);
            let left = right - 1;

            if left < length && (self.comparator)(&self.array[left], &self.array[index]).is_lt() {
                candidate = left;
            } else {
                candidate = index;
            }

            if right < length
                && (self.comparator)(&self.array[right], &self.array[candidate]).is_lt()
            {
                candidate = right;
            }

            if candidate != index {
                self.array.swap(candidate, index);
                index = candidate;
            } else {
                break;
            }
        }
    }

    pub fn resort(&mut self) {
        let length = self.array.len();
        if length <= 1 {
            return;
        }
        let half = length / 2;
        for i in (0..=half).rev() {
            self.heapify(i);
        }
    }

    pub fn insert(&mut self, element: T) -> Option<T> {
        let mut index = self.array.len();
        self.array.push(element);

        while index != 0 {
            let parent = (index - 1) / 2;
            if (self.comparator)(&self.array[index], &self.array[parent]).is_lt() {
                self.array.swap(index, parent);
                index = parent;
            } else {
                break;
            }
        }

        if let Some(max_len) = self.maximum_length {
            if self.array.len() > max_len {
                return self.array.pop();
            }
        }

        None
    }

    pub fn pop(&mut self, index: usize) -> Option<T> {
        if self.array.is_empty() || index >= self.array.len() {
            return None;
        }

        let last_index = self.array.len() - 1;
        self.array.swap(index, last_index);
        let root = self.array.pop();
        if !self.array.is_empty() && index < self.array.len() {
            self.heapify(index);
        }
        root
    }
}
