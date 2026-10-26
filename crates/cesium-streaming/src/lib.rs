//! Platform-neutral budget accounting, stale task rejection and pinned LRU.
use std::{collections::HashMap, hash::Hash};
use thiserror::Error;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskToken {
    pub scene_epoch: u64,
    pub device_epoch: u64,
    pub generation: u64,
}
#[derive(Default, Debug)]
pub struct Epochs {
    pub scene: u64,
    pub device: u64,
    pub generation: u64,
}
impl Epochs {
    pub fn token(&self) -> TaskToken {
        TaskToken {
            scene_epoch: self.scene,
            device_epoch: self.device,
            generation: self.generation,
        }
    }
    pub fn accepts(&self, t: TaskToken) -> bool {
        self.token() == t
    }
    pub fn cancel(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }
    pub fn unload(&mut self) {
        self.scene = self.scene.wrapping_add(1);
        self.cancel();
    }
    pub fn device_lost(&mut self) {
        self.device = self.device.wrapping_add(1);
        self.cancel();
    }
}
#[derive(Debug, Error)]
#[error("budget exhausted or invalid reservation")]
pub struct BudgetError;
#[derive(Debug)]
pub struct Budget {
    pub limit: usize,
    used: usize,
    reservations: HashMap<u64, usize>,
    next: u64,
}
impl Budget {
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            used: 0,
            reservations: HashMap::new(),
            next: 0,
        }
    }
    pub fn used(&self) -> usize {
        self.used
    }
    pub fn reserve(&mut self, bytes: usize) -> Result<u64, BudgetError> {
        let total = self
            .used
            .checked_add(bytes)
            .filter(|n| *n <= self.limit)
            .ok_or(BudgetError)?;
        let id = self.next;
        self.next = self.next.checked_add(1).ok_or(BudgetError)?;
        self.used = total;
        self.reservations.insert(id, bytes);
        Ok(id)
    }
    pub fn adjust(&mut self, id: u64, actual: usize) -> Result<(), BudgetError> {
        let old = *self.reservations.get(&id).ok_or(BudgetError)?;
        let total = (self.used - old)
            .checked_add(actual)
            .filter(|n| *n <= self.limit)
            .ok_or(BudgetError)?;
        self.used = total;
        self.reservations.insert(id, actual);
        Ok(())
    }
    pub fn release(&mut self, id: u64) -> Result<(), BudgetError> {
        let n = self.reservations.remove(&id).ok_or(BudgetError)?;
        self.used -= n;
        Ok(())
    }
}
#[derive(Debug)]
struct Entry {
    reservation: u64,
    last: u64,
    pinned: bool,
}
#[derive(Debug)]
pub struct Residency<K> {
    pub budget: Budget,
    entries: HashMap<K, Entry>,
    clock: u64,
}
impl<K: Copy + Eq + Hash> Residency<K> {
    pub fn new(limit: usize) -> Self {
        Self {
            budget: Budget::new(limit),
            entries: HashMap::new(),
            clock: 0,
        }
    }
    pub fn begin_frame(&mut self) {
        self.clock = self.clock.wrapping_add(1);
        for e in self.entries.values_mut() {
            e.pinned = false;
        }
    }
    pub fn pin(&mut self, key: K) {
        if let Some(e) = self.entries.get_mut(&key) {
            e.pinned = true;
            e.last = self.clock;
        }
    }
    /// Returns evictions for the owner to drop GPU/CPU resources before inserting.
    pub fn make_room(&mut self, bytes: usize) -> Result<Vec<K>, BudgetError> {
        if bytes > self.budget.limit {
            return Err(BudgetError);
        }
        let free = self.budget.limit - self.budget.used();
        let mut candidates: Vec<_> = self
            .entries
            .iter()
            .filter(|(_, e)| !e.pinned)
            .map(|(k, e)| (*k, e.last, self.budget.reservations[&e.reservation]))
            .collect();
        candidates.sort_by_key(|(_, last, _)| *last);
        // Transactional: insufficient unpinned space leaves cache untouched.
        let mut reclaimed = free;
        let mut selected = Vec::new();
        for (k, _, n) in candidates {
            if reclaimed >= bytes {
                break;
            }
            reclaimed = reclaimed.saturating_add(n);
            selected.push(k);
        }
        if reclaimed < bytes {
            return Err(BudgetError);
        }
        for k in &selected {
            let e = self.entries.remove(k).ok_or(BudgetError)?;
            self.budget.release(e.reservation)?;
        }
        Ok(selected)
    }
    pub fn insert(&mut self, key: K, bytes: usize) -> Result<(), BudgetError> {
        if self.entries.contains_key(&key) {
            return Err(BudgetError);
        }
        let reservation = self.budget.reserve(bytes)?;
        self.entries.insert(
            key,
            Entry {
                reservation,
                last: self.clock,
                pinned: false,
            },
        );
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reservation_conservation() {
        let mut b = Budget::new(100);
        let a = b.reserve(60).unwrap();
        assert!(b.reserve(50).is_err());
        b.adjust(a, 80).unwrap();
        assert!(b.adjust(a, 101).is_err());
        assert_eq!(b.used(), 80);
        b.release(a).unwrap();
        assert_eq!(b.used(), 0);
        assert!(b.release(a).is_err());
    }
    #[test]
    fn stale_tasks() {
        let mut e = Epochs::default();
        let t = e.token();
        assert!(e.accepts(t));
        e.cancel();
        assert!(!e.accepts(t));
        let t = e.token();
        e.device_lost();
        assert!(!e.accepts(t));
        let t = e.token();
        e.unload();
        assert!(!e.accepts(t));
    }
    #[test]
    fn pinned_lru_backpressure() {
        let mut c = Residency::new(100);
        c.insert(1, 60).unwrap();
        c.insert(2, 40).unwrap();
        c.begin_frame();
        c.pin(1);
        assert!(c.make_room(50).is_err());
        assert_eq!(c.budget.used(), 100);
        assert_eq!(c.make_room(40).unwrap(), vec![2]);
        assert_eq!(c.budget.used(), 60);
        c.insert(3, 40).unwrap();
    }
}
