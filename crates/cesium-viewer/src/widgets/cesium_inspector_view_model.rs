use std::cell::{Cell, RefCell};
use std::rc::Rc;

use cesium_math::DeveloperError;

/// Mock / model of an inspectable Primitive for CesiumInspector.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InspectorPrimitive {
    pub id: String,
    debug_show_bounding_volume: Rc<Cell<bool>>,
}

impl InspectorPrimitive {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            debug_show_bounding_volume: Rc::new(Cell::new(false)),
        }
    }

    pub fn debug_show_bounding_volume(&self) -> bool {
        self.debug_show_bounding_volume.get()
    }

    pub fn set_debug_show_bounding_volume(&self, value: bool) {
        self.debug_show_bounding_volume.set(value);
    }
}

/// Mock / model of an inspectable Quadtree Tile for CesiumInspector.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InspectorTile {
    pub id: String,
    pub level: u32,
    pub x: u32,
    pub y: u32,
    pub renderable: bool,
}

impl InspectorTile {
    pub fn new(id: impl Into<String>, level: u32, x: u32, y: u32, renderable: bool) -> Self {
        Self {
            id: id.into(),
            level,
            x,
            y,
            renderable,
        }
    }
}

/// Scene debug state mirrored by CesiumInspectorViewModel.
#[derive(Clone, Debug, Default)]
pub struct SceneInspectorState {
    pub debug_show_frustums: bool,
    pub performance_active: bool,
    pub debug_command_filter_active: bool,
    pub wireframe: bool,
    pub suspend_lod_update: bool,
    pub tile_coordinates_layer_active: bool,
    pub bounding_sphere_tile: Option<InspectorTile>,
    pub tiles_to_render: Vec<InspectorTile>,
    pub reference_frame_count: usize,
}

/// View model for `CesiumInspector`.
#[derive(Clone, Debug)]
pub struct CesiumInspectorViewModel {
    scene_id: String,
    performance_container_id: String,
    state: Rc<RefCell<SceneInspectorState>>,
    primitive: RefCell<Option<InspectorPrimitive>>,
    tile: RefCell<Option<InspectorTile>>,
    filter_primitive: Cell<bool>,
    primitive_reference_frame: Cell<bool>,
    filter_tile: Cell<bool>,
    is_destroyed: Cell<bool>,
}

impl CesiumInspectorViewModel {
    /// Creates a new `CesiumInspectorViewModel`.
    pub fn new(
        scene_id: Option<impl Into<String>>,
        performance_container_id: Option<impl Into<String>>,
    ) -> Result<Self, DeveloperError> {
        let scene_id = scene_id
            .map(Into::into)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| DeveloperError::new("scene is required."))?;
        let perf_id = performance_container_id
            .map(Into::into)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| DeveloperError::new("performanceContainer is required."))?;

        Ok(Self {
            scene_id,
            performance_container_id: perf_id,
            state: Rc::new(RefCell::new(SceneInspectorState::default())),
            primitive: RefCell::new(None),
            tile: RefCell::new(None),
            filter_primitive: Cell::new(false),
            primitive_reference_frame: Cell::new(false),
            filter_tile: Cell::new(false),
            is_destroyed: Cell::new(false),
        })
    }

    /// Gets scene ID.
    pub fn scene_id(&self) -> &str {
        &self.scene_id
    }

    /// Gets performance container ID.
    pub fn performance_container_id(&self) -> &str {
        &self.performance_container_id
    }

    /// Gets frustums display status.
    pub fn frustums(&self) -> bool {
        self.state.borrow().debug_show_frustums
    }

    /// Sets frustums display status.
    pub fn set_frustums(&self, value: bool) {
        self.state.borrow_mut().debug_show_frustums = value;
    }

    /// Gets performance monitoring status.
    pub fn performance(&self) -> bool {
        self.state.borrow().performance_active
    }

    /// Sets performance monitoring status.
    pub fn set_performance(&self, value: bool) {
        self.state.borrow_mut().performance_active = value;
    }

    /// Gets current inspected primitive.
    pub fn primitive(&self) -> Option<InspectorPrimitive> {
        self.primitive.borrow().clone()
    }

    /// Sets current inspected primitive.
    pub fn set_primitive(&self, prim: Option<InspectorPrimitive>) {
        *self.primitive.borrow_mut() = prim;
    }

    /// Gets primitive bounding sphere visibility.
    pub fn primitive_bounding_sphere(&self) -> bool {
        self.primitive
            .borrow()
            .as_ref()
            .map(|p| p.debug_show_bounding_volume())
            .unwrap_or(false)
    }

    /// Sets primitive bounding sphere visibility.
    pub fn set_primitive_bounding_sphere(&self, value: bool) {
        if let Some(ref p) = *self.primitive.borrow() {
            p.set_debug_show_bounding_volume(value);
        }
    }

    /// Gets primitive filter active.
    pub fn filter_primitive(&self) -> bool {
        self.filter_primitive.get()
    }

    /// Sets primitive filter active.
    pub fn set_filter_primitive(&self, value: bool) {
        self.filter_primitive.set(value);
        self.state.borrow_mut().debug_command_filter_active = value;
    }

    /// Evaluates if the current command owner passes filter.
    pub fn is_command_accepted(&self, owner_id: &str) -> bool {
        if !self.filter_primitive.get() {
            return true;
        }
        self.primitive
            .borrow()
            .as_ref()
            .map(|p| p.id == owner_id)
            .unwrap_or(false)
    }

    /// Gets primitive reference frame active status.
    pub fn primitive_reference_frame(&self) -> bool {
        self.primitive_reference_frame.get()
    }

    /// Sets primitive reference frame active status.
    pub fn set_primitive_reference_frame(&self, value: bool) {
        self.primitive_reference_frame.set(value);
        let mut st = self.state.borrow_mut();
        st.reference_frame_count = if value { 1 } else { 0 };
    }

    /// Gets wireframe status.
    pub fn wireframe(&self) -> bool {
        self.state.borrow().wireframe
    }

    /// Sets wireframe status.
    pub fn set_wireframe(&self, value: bool) {
        self.state.borrow_mut().wireframe = value;
    }

    /// Gets suspend updates status.
    pub fn suspend_updates(&self) -> bool {
        self.state.borrow().suspend_lod_update
    }

    /// Sets suspend updates status.
    pub fn set_suspend_updates(&self, value: bool) {
        self.state.borrow_mut().suspend_lod_update = value;
    }

    /// Gets tile coordinates layer active status.
    pub fn tile_coordinates(&self) -> bool {
        self.state.borrow().tile_coordinates_layer_active
    }

    /// Sets tile coordinates layer active status.
    pub fn set_tile_coordinates(&self, value: bool) {
        self.state.borrow_mut().tile_coordinates_layer_active = value;
    }

    /// Gets current tile.
    pub fn tile(&self) -> Option<InspectorTile> {
        self.tile.borrow().clone()
    }

    /// Sets current tile.
    pub fn set_tile(&self, tile: Option<InspectorTile>) {
        *self.tile.borrow_mut() = tile;
    }

    /// Gets tile bounding sphere active status.
    pub fn tile_bounding_sphere(&self) -> bool {
        self.state.borrow().bounding_sphere_tile.is_some()
    }

    /// Sets tile bounding sphere active status.
    pub fn set_tile_bounding_sphere(&self, value: bool) {
        let mut st = self.state.borrow_mut();
        if value {
            st.bounding_sphere_tile = self.tile.borrow().clone();
        } else {
            st.bounding_sphere_tile = None;
        }
    }

    /// Gets tile filter status.
    pub fn filter_tile(&self) -> bool {
        self.filter_tile.get()
    }

    /// Sets tile filter status.
    pub fn set_filter_tile(&self, value: bool) {
        self.filter_tile.set(value);
        let mut st = self.state.borrow_mut();
        st.suspend_lod_update = value;
        st.tiles_to_render.clear();
        if value {
            if let Some(ref t) = *self.tile.borrow() {
                if t.renderable {
                    st.tiles_to_render.push(t.clone());
                }
            }
        }
    }

    /// Gets number of tiles currently scheduled to render.
    pub fn tiles_to_render_count(&self) -> usize {
        self.state.borrow().tiles_to_render.len()
    }

    /// Returns whether the view model has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed.get()
    }

    /// Destroys the view model.
    pub fn destroy(&self) {
        self.is_destroyed.set(true);
    }
}
