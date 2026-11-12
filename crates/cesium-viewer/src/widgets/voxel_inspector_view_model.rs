//! VoxelInspectorViewModel matching CesiumJS `Widgets/VoxelInspector/VoxelInspectorViewModel.js` (Apache-2.0).

use cesium_math::DeveloperError;
use std::cell::Cell;

/// The view model for `VoxelInspector`.
#[derive(Clone, Debug)]
pub struct VoxelInspectorViewModel {
    scene_id: String,
    inspector_visible: Cell<bool>,
    display_visible: Cell<bool>,
    transform_visible: Cell<bool>,
    clipping_visible: Cell<bool>,
    shader_visible: Cell<bool>,
    depth_test: Cell<bool>,
    show: Cell<bool>,
    disable_update: Cell<bool>,
    debug_draw: Cell<bool>,
    jitter: Cell<bool>,
    nearest_sampling: Cell<bool>,
    screen_space_error: Cell<f64>,
    step_size: Cell<f64>,
    is_destroyed: Cell<bool>,
}

impl VoxelInspectorViewModel {
    /// Creates a new `VoxelInspectorViewModel`.
    pub fn new(scene_id: Option<impl Into<String>>) -> Result<Self, DeveloperError> {
        let scene_id = scene_id
            .map(Into::into)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| DeveloperError::new("scene is required."))?;

        Ok(Self {
            scene_id,
            inspector_visible: Cell::new(true),
            display_visible: Cell::new(true),
            transform_visible: Cell::new(false),
            clipping_visible: Cell::new(false),
            shader_visible: Cell::new(false),
            depth_test: Cell::new(true),
            show: Cell::new(true),
            disable_update: Cell::new(false),
            debug_draw: Cell::new(false),
            jitter: Cell::new(true),
            nearest_sampling: Cell::new(false),
            screen_space_error: Cell::new(16.0),
            step_size: Cell::new(1.0),
            is_destroyed: Cell::new(false),
        })
    }

    /// Gets the scene identifier.
    pub fn scene(&self) -> &str {
        &self.scene_id
    }

    /// Toggles the main inspector visibility.
    pub fn toggle_inspector(&self) {
        self.inspector_visible.set(!self.inspector_visible.get());
    }

    /// Gets inspector visibility.
    pub fn inspector_visible(&self) -> bool {
        self.inspector_visible.get()
    }

    /// Toggles Display section.
    pub fn toggle_display(&self) {
        self.display_visible.set(!self.display_visible.get());
    }

    /// Gets Display section visibility.
    pub fn display_visible(&self) -> bool {
        self.display_visible.get()
    }

    /// Toggles Transform section.
    pub fn toggle_transform(&self) {
        self.transform_visible.set(!self.transform_visible.get());
    }

    /// Gets Transform section visibility.
    pub fn transform_visible(&self) -> bool {
        self.transform_visible.get()
    }

    /// Toggles Clipping section.
    pub fn toggle_clipping(&self) {
        self.clipping_visible.set(!self.clipping_visible.get());
    }

    /// Gets Clipping section visibility.
    pub fn clipping_visible(&self) -> bool {
        self.clipping_visible.get()
    }

    /// Toggles Shader section.
    pub fn toggle_shader(&self) {
        self.shader_visible.set(!self.shader_visible.get());
    }

    /// Gets Shader section visibility.
    pub fn shader_visible(&self) -> bool {
        self.shader_visible.get()
    }

    /// Gets depth test status.
    pub fn depth_test(&self) -> bool {
        self.depth_test.get()
    }

    /// Sets depth test status.
    pub fn set_depth_test(&self, val: bool) {
        self.depth_test.set(val);
    }

    /// Gets show status.
    pub fn show(&self) -> bool {
        self.show.get()
    }

    /// Sets show status.
    pub fn set_show(&self, val: bool) {
        self.show.set(val);
    }

    /// Gets disable update status.
    pub fn disable_update(&self) -> bool {
        self.disable_update.get()
    }

    /// Sets disable update status.
    pub fn set_disable_update(&self, val: bool) {
        self.disable_update.set(val);
    }

    /// Gets debug draw status.
    pub fn debug_draw(&self) -> bool {
        self.debug_draw.get()
    }

    /// Sets debug draw status.
    pub fn set_debug_draw(&self, val: bool) {
        self.debug_draw.set(val);
    }

    /// Gets jitter status.
    pub fn jitter(&self) -> bool {
        self.jitter.get()
    }

    /// Sets jitter status.
    pub fn set_jitter(&self, val: bool) {
        self.jitter.set(val);
    }

    /// Gets nearest sampling status.
    pub fn nearest_sampling(&self) -> bool {
        self.nearest_sampling.get()
    }

    /// Sets nearest sampling status.
    pub fn set_nearest_sampling(&self, val: bool) {
        self.nearest_sampling.set(val);
    }

    /// Gets screen space error.
    pub fn screen_space_error(&self) -> f64 {
        self.screen_space_error.get()
    }

    /// Sets screen space error.
    pub fn set_screen_space_error(&self, val: f64) {
        self.screen_space_error.set(val);
    }

    /// Gets step size.
    pub fn step_size(&self) -> f64 {
        self.step_size.get()
    }

    /// Sets step size.
    pub fn set_step_size(&self, val: f64) {
        self.step_size.set(val);
    }

    /// Returns true if destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed.get()
    }

    /// Destroys the view model.
    pub fn destroy(&self) {
        self.is_destroyed.set(true);
    }
}
