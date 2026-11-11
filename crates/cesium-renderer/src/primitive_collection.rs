//! PrimitiveCollection matching CesiumJS `Scene/PrimitiveCollection.js` (Apache-2.0).

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use cesium_math::DeveloperError;

static NEXT_PRIMITIVE_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_COLLECTION_ID: AtomicU64 = AtomicU64::new(1);

/// Global external tracker for primitive reference counts and composite memberships.
static EXTERNAL_TRACKER: Mutex<Option<HashMap<u64, PrimitiveExternalInfo>>> = Mutex::new(None);

#[derive(Default, Clone, Debug)]
struct PrimitiveExternalInfo {
    reference_count: Option<usize>,
    composites: HashMap<String, ()>,
}

fn with_tracker<R>(f: impl FnOnce(&mut HashMap<u64, PrimitiveExternalInfo>) -> R) -> R {
    let mut guard = EXTERNAL_TRACKER.lock().unwrap();
    if guard.is_none() {
        *guard = Some(HashMap::new());
    }
    f(guard.as_mut().unwrap())
}

/// Generates a globally unique identifier for scene primitives.
pub fn next_primitive_id() -> u64 {
    NEXT_PRIMITIVE_ID.fetch_add(1, Ordering::Relaxed)
}

/// Trait for any primitive stored in a `PrimitiveCollection`.
pub trait ScenePrimitive: Send + Sync {
    /// Globally unique ID of this primitive.
    fn id(&self) -> u64;
    /// Destroys resources held by this primitive.
    fn destroy(&self);
    /// Returns true if this primitive has been destroyed.
    fn is_destroyed(&self) -> bool;
    /// Called during update pass.
    fn update(&self) {}
    /// Whether the primitive is shown.
    fn show(&self) -> bool {
        true
    }
    /// Sets whether the primitive is shown.
    fn set_show(&self, _show: bool) {}
}

/// Standard concrete primitive for testing or scene rendering.
#[derive(Clone, Debug)]
pub struct MockPrimitive {
    pub id: u64,
    pub destroyed: Arc<AtomicBool>,
    pub show: Arc<AtomicBool>,
}

impl Default for MockPrimitive {
    fn default() -> Self {
        Self::new()
    }
}

impl MockPrimitive {
    pub fn new() -> Self {
        Self {
            id: next_primitive_id(),
            destroyed: Arc::new(AtomicBool::new(false)),
            show: Arc::new(AtomicBool::new(true)),
        }
    }
}

impl ScenePrimitive for MockPrimitive {
    fn id(&self) -> u64 {
        self.id
    }

    fn destroy(&self) {
        self.destroyed.store(true, Ordering::SeqCst);
    }

    fn is_destroyed(&self) -> bool {
        self.destroyed.load(Ordering::SeqCst)
    }

    fn show(&self) -> bool {
        self.show.load(Ordering::SeqCst)
    }

    fn set_show(&self, show: bool) {
        self.show.store(show, Ordering::SeqCst);
    }
}

/// Options for configuring a `PrimitiveCollection`.
#[derive(Clone, Debug)]
pub struct PrimitiveCollectionOptions {
    pub show: Option<bool>,
    pub destroy_primitives: Option<bool>,
    pub count_references: Option<bool>,
}

impl Default for PrimitiveCollectionOptions {
    fn default() -> Self {
        Self {
            show: Some(true),
            destroy_primitives: Some(true),
            count_references: Some(false),
        }
    }
}

type EventListener = Arc<dyn Fn(&Arc<dyn ScenePrimitive>) + Send + Sync>;

/// A collection of primitives. `PrimitiveCollection` is also a primitive itself so collections can
/// be added to collections forming a hierarchy.
pub struct PrimitiveCollection {
    pub id: u64,
    pub show: bool,
    pub destroy_primitives: bool,
    pub count_references: bool,
    guid: String,
    primitives: Vec<Arc<dyn ScenePrimitive>>,
    is_destroyed: AtomicBool,
    on_primitive_added: Vec<EventListener>,
    on_primitive_removed: Vec<EventListener>,
}

impl Default for PrimitiveCollection {
    fn default() -> Self {
        Self::new(PrimitiveCollectionOptions::default())
    }
}

impl PrimitiveCollection {
    /// Creates a new `PrimitiveCollection`.
    pub fn new(options: PrimitiveCollectionOptions) -> Self {
        let id = next_primitive_id();
        let coll_idx = NEXT_COLLECTION_ID.fetch_add(1, Ordering::Relaxed);
        let guid = format!("prim_coll_{coll_idx}");
        Self {
            id,
            show: options.show.unwrap_or(true),
            destroy_primitives: options.destroy_primitives.unwrap_or(true),
            count_references: options.count_references.unwrap_or(false),
            guid,
            primitives: Vec::new(),
            is_destroyed: AtomicBool::new(false),
            on_primitive_added: Vec::new(),
            on_primitive_removed: Vec::new(),
        }
    }

    /// Gets the number of primitives in the collection.
    pub fn len(&self) -> usize {
        self.primitives.len()
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.primitives.is_empty()
    }

    /// Adds a listener for primitive added event.
    pub fn add_on_primitive_added(
        &mut self,
        listener: impl Fn(&Arc<dyn ScenePrimitive>) + Send + Sync + 'static,
    ) {
        self.on_primitive_added.push(Arc::new(listener));
    }

    /// Adds a listener for primitive removed event.
    pub fn add_on_primitive_removed(
        &mut self,
        listener: impl Fn(&Arc<dyn ScenePrimitive>) + Send + Sync + 'static,
    ) {
        self.on_primitive_removed.push(Arc::new(listener));
    }

    /// Adds a primitive to the collection, optionally at an index.
    pub fn add(
        &mut self,
        primitive: Arc<dyn ScenePrimitive>,
        index: Option<usize>,
    ) -> Result<Arc<dyn ScenePrimitive>, DeveloperError> {
        if self.is_destroyed() {
            return Err(DeveloperError::new("This object was destroyed."));
        }

        if let Some(idx) = index {
            if idx > self.primitives.len() {
                return Err(DeveloperError::new(
                    "index must be less than or equal to the number of primitives.",
                ));
            }
            self.primitives.insert(idx, primitive.clone());
        } else {
            self.primitives.push(primitive.clone());
        }

        let prim_id = primitive.id();
        let guid = self.guid.clone();
        let count_refs = self.count_references;

        with_tracker(|tracker| {
            let entry = tracker.entry(prim_id).or_default();
            entry.composites.insert(guid, ());
            if count_refs {
                entry.reference_count = Some(entry.reference_count.unwrap_or(0) + 1);
            }
        });

        for listener in &self.on_primitive_added {
            listener(&primitive);
        }

        Ok(primitive)
    }

    /// Determines if this collection contains a primitive.
    pub fn contains(&self, primitive: &dyn ScenePrimitive) -> bool {
        let prim_id = primitive.id();
        with_tracker(|tracker| {
            if let Some(entry) = tracker.get(&prim_id) {
                entry.composites.contains_key(&self.guid)
            } else {
                false
            }
        })
    }

    /// Removes a primitive from the collection.
    pub fn remove(&mut self, primitive: &dyn ScenePrimitive) -> bool {
        if !self.contains(primitive) {
            return false;
        }

        let prim_id = primitive.id();
        if let Some(pos) = self.primitives.iter().position(|p| p.id() == prim_id) {
            let removed = self.primitives.remove(pos);
            let guid = &self.guid;
            let count_refs = self.count_references;
            let destroy_prims = self.destroy_primitives;

            let should_destroy = with_tracker(|tracker| {
                if let Some(entry) = tracker.get_mut(&prim_id) {
                    entry.composites.remove(guid);
                    if count_refs {
                        if let Some(rc) = entry.reference_count.as_mut() {
                            *rc = rc.saturating_sub(1);
                        }
                    }
                    let ref_count = entry.reference_count.unwrap_or(0);
                    destroy_prims && (!count_refs || ref_count == 0)
                } else {
                    false
                }
            });

            if should_destroy {
                removed.destroy();
            }

            for listener in &self.on_primitive_removed {
                listener(&removed);
            }

            true
        } else {
            false
        }
    }

    /// Removes all primitives from the collection.
    pub fn remove_all(&mut self) {
        let primitives = std::mem::take(&mut self.primitives);
        let guid = &self.guid;
        let count_refs = self.count_references;
        let destroy_prims = self.destroy_primitives;

        for primitive in primitives {
            let prim_id = primitive.id();
            let should_destroy = with_tracker(|tracker| {
                if let Some(entry) = tracker.get_mut(&prim_id) {
                    entry.composites.remove(guid);
                    if count_refs {
                        if let Some(rc) = entry.reference_count.as_mut() {
                            *rc = rc.saturating_sub(1);
                        }
                    }
                    let ref_count = entry.reference_count.unwrap_or(0);
                    destroy_prims && (!count_refs || ref_count == 0)
                } else {
                    false
                }
            });

            if should_destroy {
                primitive.destroy();
            }

            for listener in &self.on_primitive_removed {
                listener(&primitive);
            }
        }
    }

    /// Gets the primitive at index.
    pub fn get(&self, index: usize) -> Result<Arc<dyn ScenePrimitive>, DeveloperError> {
        self.primitives
            .get(index)
            .cloned()
            .ok_or_else(|| DeveloperError::new("Index out of bounds."))
    }

    /// Raises a primitive up one in render order.
    pub fn raise(&mut self, primitive: &dyn ScenePrimitive) -> Result<(), DeveloperError> {
        if !self.contains(primitive) {
            return Err(DeveloperError::new("primitive is not in this collection."));
        }
        let pos = self
            .primitives
            .iter()
            .position(|p| p.id() == primitive.id())
            .unwrap();
        if pos < self.primitives.len() - 1 {
            self.primitives.swap(pos, pos + 1);
        }
        Ok(())
    }

    /// Raises a primitive to the top of the collection.
    pub fn raise_to_top(&mut self, primitive: &dyn ScenePrimitive) -> Result<(), DeveloperError> {
        if !self.contains(primitive) {
            return Err(DeveloperError::new("primitive is not in this collection."));
        }
        let pos = self
            .primitives
            .iter()
            .position(|p| p.id() == primitive.id())
            .unwrap();
        if pos < self.primitives.len() - 1 {
            let p = self.primitives.remove(pos);
            self.primitives.push(p);
        }
        Ok(())
    }

    /// Lowers a primitive down one in render order.
    pub fn lower(&mut self, primitive: &dyn ScenePrimitive) -> Result<(), DeveloperError> {
        if !self.contains(primitive) {
            return Err(DeveloperError::new("primitive is not in this collection."));
        }
        let pos = self
            .primitives
            .iter()
            .position(|p| p.id() == primitive.id())
            .unwrap();
        if pos > 0 {
            self.primitives.swap(pos, pos - 1);
        }
        Ok(())
    }

    /// Lowers a primitive to the bottom of the collection.
    pub fn lower_to_bottom(
        &mut self,
        primitive: &dyn ScenePrimitive,
    ) -> Result<(), DeveloperError> {
        if !self.contains(primitive) {
            return Err(DeveloperError::new("primitive is not in this collection."));
        }
        let pos = self
            .primitives
            .iter()
            .position(|p| p.id() == primitive.id())
            .unwrap();
        if pos > 0 {
            let p = self.primitives.remove(pos);
            self.primitives.insert(0, p);
        }
        Ok(())
    }

    /// Returns the reference count of a primitive if tracking is active.
    pub fn get_reference_count(primitive: &dyn ScenePrimitive) -> Option<usize> {
        let prim_id = primitive.id();
        with_tracker(|tracker| tracker.get(&prim_id).and_then(|e| e.reference_count))
    }

    /// Destroys this collection and optionally contained primitives.
    pub fn destroy(&self) {
        <Self as ScenePrimitive>::destroy(self);
    }

    /// Returns true if destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed.load(Ordering::SeqCst)
    }
}

impl ScenePrimitive for PrimitiveCollection {
    fn id(&self) -> u64 {
        self.id
    }

    fn destroy(&self) {
        if self.is_destroyed.swap(true, Ordering::SeqCst) {
            return;
        }
        let guid = &self.guid;
        let count_refs = self.count_references;
        let destroy_prims = self.destroy_primitives;

        for primitive in &self.primitives {
            let prim_id = primitive.id();
            let should_destroy = with_tracker(|tracker| {
                if let Some(entry) = tracker.get_mut(&prim_id) {
                    entry.composites.remove(guid);
                    if count_refs {
                        if let Some(rc) = entry.reference_count.as_mut() {
                            *rc = rc.saturating_sub(1);
                        }
                    }
                    let ref_count = entry.reference_count.unwrap_or(0);
                    destroy_prims && (!count_refs || ref_count == 0)
                } else {
                    false
                }
            });

            if should_destroy {
                primitive.destroy();
            }
        }
    }

    fn is_destroyed(&self) -> bool {
        self.is_destroyed.load(Ordering::SeqCst)
    }

    fn show(&self) -> bool {
        self.show
    }

    fn set_show(&self, _show: bool) {}
}
