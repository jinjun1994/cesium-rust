//! AssociativeArray matching CesiumJS `Core/AssociativeArray`.
//! Adapted from CesiumJS (Apache-2.0).

use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone, Debug)]
pub struct AssociativeArray<K, V> {
    hash: HashMap<K, V>,
    array: Vec<V>,
}

impl<K, V> Default for AssociativeArray<K, V>
where
    K: Eq + Hash + Clone,
    V: PartialEq + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> AssociativeArray<K, V>
where
    K: Eq + Hash + Clone,
    V: PartialEq + Clone,
{
    pub fn new() -> Self {
        Self {
            hash: HashMap::new(),
            array: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.array.len()
    }

    pub fn is_empty(&self) -> bool {
        self.array.is_empty()
    }

    pub fn values(&self) -> &[V] {
        &self.array
    }

    pub fn contains(&self, key: &K) -> bool {
        self.hash.contains_key(key)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.hash.get(key)
    }

    pub fn set(&mut self, key: K, value: V) {
        let old_value = self.hash.get(&key);
        if old_value != Some(&value) {
            self.remove(&key);
            self.hash.insert(key, value.clone());
            self.array.push(value);
        }
    }

    pub fn remove(&mut self, key: &K) -> bool {
        if let Some(removed_val) = self.hash.remove(key) {
            if let Some(pos) = self.array.iter().position(|x| *x == removed_val) {
                self.array.remove(pos);
            }
            true
        } else {
            false
        }
    }

    pub fn remove_all(&mut self) {
        self.hash.clear();
        self.array.clear();
    }
}
