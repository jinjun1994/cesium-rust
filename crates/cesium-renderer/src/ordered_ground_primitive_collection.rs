//! OrderedGroundPrimitiveCollection matching CesiumJS `Scene/OrderedGroundPrimitiveCollection.js` (Apache-2.0).

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use cesium_math::DeveloperError;

static NEXT_GROUND_PRIMITIVE_ID: AtomicU64 = AtomicU64::new(1);

/// Generates a globally unique identifier for ground primitives.
pub fn next_ground_primitive_id() -> u64 {
    NEXT_GROUND_PRIMITIVE_ID.fetch_add(1, Ordering::Relaxed)
}

/// Trait for any ground primitive stored in an `OrderedGroundPrimitiveCollection`.
pub trait GroundPrimitive: Send + Sync {
    /// Globally unique ID of this primitive.
    fn id(&self) -> u64;
    /// Called during the update pass.
    fn update(&self);
    /// Destroys resources held by this primitive.
    fn destroy(&self);
    /// Returns true if this primitive has been destroyed.
    fn is_destroyed(&self) -> bool;
}

/// A standard mock or concrete primitive implementation for ground rendering.
#[derive(Debug, Clone)]
pub struct MockGroundPrimitive {
    pub id: u64,
    pub destroyed: Arc<AtomicBool>,
    pub update_logger: Option<Arc<std::sync::Mutex<Vec<u64>>>>,
}

impl Default for MockGroundPrimitive {
    fn default() -> Self {
        Self::new()
    }
}

impl MockGroundPrimitive {
    pub fn new() -> Self {
        Self {
            id: next_ground_primitive_id(),
            destroyed: Arc::new(AtomicBool::new(false)),
            update_logger: None,
        }
    }

    pub fn with_logger(logger: Arc<std::sync::Mutex<Vec<u64>>>) -> Self {
        Self {
            id: next_ground_primitive_id(),
            destroyed: Arc::new(AtomicBool::new(false)),
            update_logger: Some(logger),
        }
    }
}

impl GroundPrimitive for MockGroundPrimitive {
    fn id(&self) -> u64 {
        self.id
    }

    fn update(&self) {
        if let Some(logger) = &self.update_logger {
            if let Ok(mut lock) = logger.lock() {
                lock.push(self.id);
            }
        }
    }

    fn destroy(&self) {
        self.destroyed.store(true, Ordering::SeqCst);
    }

    fn is_destroyed(&self) -> bool {
        self.destroyed.load(Ordering::SeqCst)
    }
}

/// A subcollection of ground primitives sharing the same z-index.
#[derive(Clone)]
pub struct PrimitiveSubcollection {
    pub z_index: i32,
    pub primitives: Vec<Arc<dyn GroundPrimitive>>,
}

impl PrimitiveSubcollection {
    pub fn new(z_index: i32) -> Self {
        Self {
            z_index,
            primitives: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.primitives.len()
    }

    pub fn is_empty(&self) -> bool {
        self.primitives.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&Arc<dyn GroundPrimitive>> {
        self.primitives.get(index)
    }
}

/// A primitive collection for maintaining the rendering order of ground primitives based on a z-index.
pub struct OrderedGroundPrimitiveCollection {
    length: usize,
    collections_array: Vec<PrimitiveSubcollection>,
    primitive_z_index: HashMap<u64, i32>,
    pub show: bool,
    is_destroyed: bool,
}

impl Default for OrderedGroundPrimitiveCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderedGroundPrimitiveCollection {
    /// Creates a new `OrderedGroundPrimitiveCollection`.
    pub fn new() -> Self {
        Self {
            length: 0,
            collections_array: Vec::new(),
            primitive_z_index: HashMap::new(),
            show: true,
            is_destroyed: false,
        }
    }

    fn check_destroyed(&self) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ))
        } else {
            Ok(())
        }
    }

    /// Number of primitives in the collection.
    pub fn length(&self) -> usize {
        self.length
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Read-only access to subcollections sorted by z-index.
    pub fn collections_array(&self) -> &[PrimitiveSubcollection] {
        &self.collections_array
    }

    /// Adds a primitive with an optional z-index (default 0).
    pub fn add(
        &mut self,
        primitive: Arc<dyn GroundPrimitive>,
        z_index: Option<i32>,
    ) -> Result<Arc<dyn GroundPrimitive>, DeveloperError> {
        self.check_destroyed()?;

        let z = z_index.unwrap_or(0);
        let id = primitive.id();

        // If already present, remove it first
        if self.primitive_z_index.contains_key(&id) {
            self.remove(&primitive, true);
        }

        let array_idx = match self
            .collections_array
            .binary_search_by_key(&z, |c| c.z_index)
        {
            Ok(idx) => idx,
            Err(idx) => {
                self.collections_array
                    .insert(idx, PrimitiveSubcollection::new(z));
                idx
            }
        };

        self.collections_array[array_idx]
            .primitives
            .push(primitive.clone());
        self.primitive_z_index.insert(id, z);
        self.length += 1;

        Ok(primitive)
    }

    /// Sets or adjusts the z-index of a primitive. Adds it if not already present.
    pub fn set(
        &mut self,
        primitive: &Arc<dyn GroundPrimitive>,
        z_index: i32,
    ) -> Result<(), DeveloperError> {
        self.check_destroyed()?;

        let id = primitive.id();
        if let Some(&current_z) = self.primitive_z_index.get(&id) {
            if current_z == z_index {
                return Ok(());
            }
            self.remove(primitive, true);
        }

        self.add(primitive.clone(), Some(z_index))?;
        Ok(())
    }

    /// Removes a primitive from the collection.
    pub fn remove(&mut self, primitive: &Arc<dyn GroundPrimitive>, do_not_destroy: bool) -> bool {
        if self.is_destroyed {
            return false;
        }

        let id = primitive.id();
        let z = match self.primitive_z_index.remove(&id) {
            Some(z) => z,
            None => return false,
        };

        let mut remove_sub_idx = None;
        let mut removed = false;

        if let Ok(idx) = self
            .collections_array
            .binary_search_by_key(&z, |c| c.z_index)
        {
            let sub = &mut self.collections_array[idx];
            if let Some(pos) = sub.primitives.iter().position(|p| p.id() == id) {
                let p = sub.primitives.remove(pos);
                if !do_not_destroy {
                    p.destroy();
                }
                self.length = self.length.saturating_sub(1);
                removed = true;
            }

            if sub.primitives.is_empty() {
                remove_sub_idx = Some(idx);
            }
        }

        if let Some(idx) = remove_sub_idx {
            self.collections_array.remove(idx);
        }

        removed
    }

    /// Removes all primitives and destroys them.
    pub fn remove_all(&mut self) {
        for sub in &self.collections_array {
            for p in &sub.primitives {
                p.destroy();
            }
        }

        self.collections_array.clear();
        self.primitive_z_index.clear();
        self.length = 0;
    }

    /// Checks if a primitive is in the collection.
    pub fn contains(&self, primitive: &Arc<dyn GroundPrimitive>) -> bool {
        self.primitive_z_index.contains_key(&primitive.id())
    }

    /// Updates all primitives in ascending z-index order.
    pub fn update(&self) {
        if !self.show || self.is_destroyed {
            return;
        }

        for sub in &self.collections_array {
            for p in &sub.primitives {
                p.update();
            }
        }
    }

    /// Destroys the collection and all contained primitives.
    pub fn destroy(&mut self) {
        self.remove_all();
        self.is_destroyed = true;
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }
}
