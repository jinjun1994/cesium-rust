// A wrapper around arrays so that the internal length of the array can be manually managed.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedArray<T> {
    array: Vec<Option<T>>,
    length: usize,
}

impl<T> Default for ManagedArray<T> {
    fn default() -> Self {
        Self::new(0)
    }
}

impl<T> ManagedArray<T> {
    pub fn new(length: usize) -> Self {
        let mut array = Vec::with_capacity(length);
        for _ in 0..length {
            array.push(None);
        }
        Self { array, length }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.length
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.array.len()
    }

    pub fn values(&self) -> &[Option<T>] {
        &self.array
    }

    pub fn active_values(&self) -> &[Option<T>] {
        &self.array[..self.length]
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.length {
            None
        } else {
            self.array[index].as_ref()
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.length {
            None
        } else {
            self.array[index].as_mut()
        }
    }

    pub fn set(&mut self, index: usize, element: T) {
        if index >= self.length {
            self.resize(index + 1);
        }
        self.array[index] = Some(element);
    }

    pub fn peek(&self) -> Option<&T> {
        if self.length == 0 {
            None
        } else {
            self.array[self.length - 1].as_ref()
        }
    }

    pub fn push(&mut self, element: T) {
        let index = self.length;
        self.length += 1;
        if index < self.array.len() {
            self.array[index] = Some(element);
        } else {
            self.array.push(Some(element));
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.length == 0 {
            return None;
        }
        self.length -= 1;
        self.array[self.length].take()
    }

    pub fn reserve(&mut self, length: usize) {
        if length > self.array.len() {
            let diff = length - self.array.len();
            for _ in 0..diff {
                self.array.push(None);
            }
        }
    }

    pub fn resize(&mut self, length: usize) {
        if length < self.length {
            for item in &mut self.array[length..self.length] {
                *item = None;
            }
        } else if length > self.array.len() {
            self.reserve(length);
        }
        self.length = length;
    }

    pub fn trim(&mut self, length: Option<usize>) {
        let target = length.unwrap_or(self.length);
        if target > self.array.len() {
            let diff = target - self.array.len();
            for _ in 0..diff {
                self.array.push(None);
            }
        } else {
            self.array.truncate(target);
        }
        if self.length > self.array.len() {
            self.length = self.array.len();
        }
    }
}
