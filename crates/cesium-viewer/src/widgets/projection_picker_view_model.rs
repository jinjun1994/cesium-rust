//! ProjectionPickerViewModel matching CesiumJS `Widgets/ProjectionPicker/ProjectionPickerViewModel.js` (Apache-2.0).

use crate::widgets::home_button_view_model::SceneMode;
use cesium_math::{DeveloperError, Event, ListenerId};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// Camera frustum type for projection picking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrustumType {
    Perspective,
    Orthographic,
}

/// Simulated Camera state for projection picker.
#[derive(Clone, Debug)]
pub struct ProjectionPickerCamera {
    pub frustum: FrustumType,
    pub is_flight_in_progress: bool,
}

impl Default for ProjectionPickerCamera {
    fn default() -> Self {
        Self {
            frustum: FrustumType::Perspective,
            is_flight_in_progress: false,
        }
    }
}

impl ProjectionPickerCamera {
    /// Starts a camera flight.
    pub fn fly_home(&mut self, _duration: f64) {
        self.is_flight_in_progress = true;
    }

    /// Cancels active camera flight.
    pub fn cancel_flight(&mut self) {
        self.is_flight_in_progress = false;
    }

    /// Switches camera frustum to perspective.
    pub fn switch_to_perspective_frustum(&mut self) {
        self.frustum = FrustumType::Perspective;
    }

    /// Switches camera frustum to orthographic.
    pub fn switch_to_orthographic_frustum(&mut self) {
        self.frustum = FrustumType::Orthographic;
    }
}

/// Simulated Scene state for projection picker.
#[derive(Clone, Debug)]
pub struct ProjectionPickerScene {
    pub mode: SceneMode,
    pub camera: ProjectionPickerCamera,
    pub morph_complete: Event<(SceneMode, SceneMode)>,
    pub pre_render: Event<()>,
    pub is_destroyed: bool,
}

impl Default for ProjectionPickerScene {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectionPickerScene {
    /// Creates a new `ProjectionPickerScene`.
    pub fn new() -> Self {
        Self {
            mode: SceneMode::Scene3D,
            camera: ProjectionPickerCamera::default(),
            morph_complete: Event::new(),
            pre_render: Event::new(),
            is_destroyed: false,
        }
    }

    /// Morphs scene to 2D mode.
    pub fn morph_to_2d(&mut self, _duration: f64) {
        let old_mode = self.mode;
        self.mode = SceneMode::Scene2D;
        self.camera.frustum = FrustumType::Orthographic;
        self.morph_complete.raise_event(&(old_mode, SceneMode::Scene2D));
    }

    /// Morphs scene to 3D mode.
    pub fn morph_to_3d(&mut self, _duration: f64) {
        let old_mode = self.mode;
        self.mode = SceneMode::Scene3D;
        self.morph_complete.raise_event(&(old_mode, SceneMode::Scene3D));
    }

    /// Simulates pre-render tick.
    pub fn pre_render_tick(&self) {
        self.pre_render.raise_event(&());
    }

    /// Destroys scene resources.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }
}

/// The view model for [`ProjectionPicker`].
pub struct ProjectionPickerViewModel {
    scene: Rc<RefCell<ProjectionPickerScene>>,
    orthographic: Rc<Cell<bool>>,
    flight_in_progress: Rc<Cell<bool>>,
    scene_mode: Rc<Cell<SceneMode>>,
    pub drop_down_visible: bool,
    pub tooltip_perspective: String,
    pub tooltip_orthographic: String,
    morph_listener_id: Option<ListenerId>,
    pre_render_listener_id: Option<ListenerId>,
    is_destroyed: bool,
}

impl ProjectionPickerViewModel {
    /// Creates a new `ProjectionPickerViewModel`.
    pub fn new(
        scene: Option<Rc<RefCell<ProjectionPickerScene>>>,
    ) -> Result<Self, DeveloperError> {
        let scene = scene.ok_or_else(|| DeveloperError::new("scene is required."))?;

        let is_ortho = scene.borrow().camera.frustum == FrustumType::Orthographic
            || scene.borrow().mode == SceneMode::Scene2D;
        let orthographic = Rc::new(Cell::new(is_ortho));
        let flight_in_progress = Rc::new(Cell::new(scene.borrow().camera.is_flight_in_progress));
        let scene_mode = Rc::new(Cell::new(scene.borrow().mode));

        let ortho_clone = Rc::clone(&orthographic);
        let mode_clone = Rc::clone(&scene_mode);
        let scene_weak = Rc::downgrade(&scene);
        let morph_listener_id = scene
            .borrow()
            .morph_complete
            .add_listener(move |(_old_mode, new_mode)| {
                mode_clone.set(*new_mode);
                let ortho = if *new_mode == SceneMode::Scene2D {
                    true
                } else if let Some(sc) = scene_weak.upgrade() {
                    sc.borrow().camera.frustum == FrustumType::Orthographic
                } else {
                    false
                };
                ortho_clone.set(ortho);
            });

        let flight_clone = Rc::clone(&flight_in_progress);
        let scene_weak2 = Rc::downgrade(&scene);
        let pre_render_listener_id = scene.borrow().pre_render.add_listener(move |_| {
            if let Some(sc) = scene_weak2.upgrade() {
                flight_clone.set(sc.borrow().camera.is_flight_in_progress);
            }
        });

        Ok(Self {
            scene,
            orthographic,
            flight_in_progress,
            scene_mode,
            drop_down_visible: false,
            tooltip_perspective: "Perspective Projection".to_string(),
            tooltip_orthographic: "Orthographic Projection".to_string(),
            morph_listener_id: Some(morph_listener_id),
            pre_render_listener_id: Some(pre_render_listener_id),
            is_destroyed: false,
        })
    }

    /// Gets the scene.
    pub fn scene(&self) -> Rc<RefCell<ProjectionPickerScene>> {
        Rc::clone(&self.scene)
    }

    /// Gets whether the dropdown is visible.
    pub fn drop_down_visible(&self) -> bool {
        self.drop_down_visible
    }

    /// Sets whether the dropdown is visible.
    pub fn set_drop_down_visible(&mut self, visible: bool) {
        self.drop_down_visible = visible;
    }

    /// Gets whether camera flight is currently in progress.
    pub fn is_flight_in_progress(&self) -> bool {
        self.flight_in_progress.get()
    }

    /// Gets the current scene mode.
    pub fn scene_mode(&self) -> SceneMode {
        self.scene_mode.get()
    }

    /// Toggles the dropdown visibility. Does nothing if in 2D mode or during flight.
    pub fn toggle_drop_down(&mut self) {
        if self.scene_mode.get() == SceneMode::Scene2D || self.flight_in_progress.get() {
            return;
        }
        self.drop_down_visible = !self.drop_down_visible;
    }

    /// Switches to perspective projection.
    pub fn switch_to_perspective(&mut self) {
        if self.scene_mode.get() == SceneMode::Scene2D {
            return;
        }
        self.scene.borrow_mut().camera.switch_to_perspective_frustum();
        self.orthographic.set(false);
        self.drop_down_visible = false;
    }

    /// Switches to orthographic projection.
    pub fn switch_to_orthographic(&mut self) {
        if self.scene_mode.get() == SceneMode::Scene2D {
            return;
        }
        self.scene.borrow_mut().camera.switch_to_orthographic_frustum();
        self.orthographic.set(true);
        self.drop_down_visible = false;
    }

    /// Returns whether the scene is currently using an orthographic projection.
    pub fn is_orthographic_projection(&self) -> bool {
        self.orthographic.get()
    }

    /// Gets the currently selected tooltip text.
    pub fn selected_tooltip(&self) -> &str {
        if self.is_orthographic_projection() {
            &self.tooltip_orthographic
        } else {
            &self.tooltip_perspective
        }
    }

    /// Returns true if this object was destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys the view model and unregisters listeners.
    pub fn destroy(&mut self) {
        if !self.is_destroyed {
            self.is_destroyed = true;
            if let Some(id) = self.morph_listener_id.take() {
                self.scene.borrow().morph_complete.remove_listener(id);
            }
            if let Some(id) = self.pre_render_listener_id.take() {
                self.scene.borrow().pre_render.remove_listener(id);
            }
        }
    }
}

impl Drop for ProjectionPickerViewModel {
    fn drop(&mut self) {
        self.destroy();
    }
}
