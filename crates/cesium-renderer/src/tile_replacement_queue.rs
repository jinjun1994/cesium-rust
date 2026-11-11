//! TileReplacementQueue matching CesiumJS `Scene/TileReplacementQueue.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use crate::quadtree_tile::QuadtreeTile;

/// A priority queue of tiles to be replaced, if necessary, to make room for new tiles.
/// The queue is maintained in LRU order (head = newest, tail = oldest).
#[derive(Clone, Default)]
pub struct TileReplacementQueue {
    tiles: Vec<Rc<RefCell<QuadtreeTile>>>,
    last_before_start_of_frame: Option<Rc<RefCell<QuadtreeTile>>>,
}

impl TileReplacementQueue {
    /// Creates a new, empty `TileReplacementQueue`.
    pub fn new() -> Self {
        Self::default()
    }

    /// The number of tiles in the queue.
    pub fn count(&self) -> usize {
        self.tiles.len()
    }

    /// The most recently rendered tile (head of the queue).
    pub fn head(&self) -> Option<Rc<RefCell<QuadtreeTile>>> {
        self.tiles.first().cloned()
    }

    /// The least recently rendered tile (tail of the queue).
    pub fn tail(&self) -> Option<Rc<RefCell<QuadtreeTile>>> {
        self.tiles.last().cloned()
    }

    /// Marks the start of the render frame. Tiles before this point (rendered during this frame)
    /// will not be trimmed.
    pub fn mark_start_of_render_frame(&mut self) {
        self.last_before_start_of_frame = self.head();
    }

    /// Removes an item at index `pos` and updates `last_before_start_of_frame` if needed.
    fn remove_internal(&mut self, pos: usize) -> Rc<RefCell<QuadtreeTile>> {
        let item = self.tiles.remove(pos);
        if let Some(ref last) = self.last_before_start_of_frame {
            if Rc::ptr_eq(last, &item) {
                // In CesiumJS: _lastBeforeStartOfFrame = item.replacementNext;
                // In our array, replacementNext is the element now at index `pos`.
                self.last_before_start_of_frame = self.tiles.get(pos).cloned();
            }
        }
        item
    }

    /// Marks a tile as rendered this frame and moves it to the head of the queue.
    pub fn mark_tile_rendered(&mut self, item: &Rc<RefCell<QuadtreeTile>>) {
        if let Some(first) = self.tiles.first() {
            if Rc::ptr_eq(first, item) {
                if let Some(ref last) = self.last_before_start_of_frame {
                    if Rc::ptr_eq(last, item) {
                        self.last_before_start_of_frame = self.tiles.get(1).cloned();
                    }
                }
                return;
            }
        }

        if let Some(pos) = self.tiles.iter().position(|t| Rc::ptr_eq(t, item)) {
            self.remove_internal(pos);
        }

        self.tiles.insert(0, item.clone());
    }

    /// Reduces the size of the queue to a specified size by unloading the least-recently used tiles.
    /// Tiles that were used last frame will not be unloaded, even if that puts the number of tiles
    /// above the specified maximum.
    pub fn trim_tiles(&mut self, maximum_tiles: usize) {
        let mut current = self.tiles.last().cloned();
        let mut keep_trimming = true;

        while keep_trimming
            && self.last_before_start_of_frame.is_some()
            && self.count() > maximum_tiles
            && current.is_some()
        {
            let tile = current.as_ref().unwrap().clone();
            let is_last = self
                .last_before_start_of_frame
                .as_ref()
                .is_some_and(|last| Rc::ptr_eq(last, &tile));

            keep_trimming = !is_last;

            let pos = self.tiles.iter().position(|t| Rc::ptr_eq(t, &tile));

            let previous = if let Some(p) = pos {
                if p > 0 {
                    self.tiles.get(p - 1).cloned()
                } else {
                    None
                }
            } else {
                None
            };

            let eligible = tile.borrow().eligible_for_unloading;
            if eligible {
                tile.borrow_mut().free_resources();
                if let Some(p) = pos {
                    self.remove_internal(p);
                }
            }

            current = previous;
        }
    }
}
