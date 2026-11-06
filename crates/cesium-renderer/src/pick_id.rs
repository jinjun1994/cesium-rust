//! PickId matching CesiumJS `Renderer/PickId.js` (Apache-2.0).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use cesium_math::Color;

/// Represents a pickable object with a unique integer ID and picking color.
pub struct PickId<T> {
    pick_objects: Rc<RefCell<HashMap<u32, T>>>,
    pub key: u32,
    pub color: Color,
}

impl<T: Clone> PickId<T> {
    /// Reserved value indicating a null (missing) Pick ID.
    pub const NULL_PICK_ID: u32 = 0;

    /// Creates a new PickId associated with a dictionary of pickable objects.
    pub fn new(pick_objects: Rc<RefCell<HashMap<u32, T>>>, key: u32, color: Color) -> Self {
        Self {
            pick_objects,
            key,
            color,
        }
    }

    /// Gets the associated object from the pick objects map.
    pub fn object(&self) -> Option<T> {
        self.pick_objects.borrow().get(&self.key).cloned()
    }

    /// Sets or updates the associated object in the pick objects map.
    pub fn set_object(&self, value: T) {
        self.pick_objects.borrow_mut().insert(self.key, value);
    }

    /// Removes the object associated with this pick ID from the pick objects map.
    pub fn destroy(&self) {
        self.pick_objects.borrow_mut().remove(&self.key);
    }
}
