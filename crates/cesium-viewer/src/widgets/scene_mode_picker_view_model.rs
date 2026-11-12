//! SceneModePickerViewModel matching CesiumJS `Widgets/SceneModePicker/SceneModePickerViewModel.js` (Apache-2.0).

use crate::widgets::home_button_view_model::SceneMode;
use cesium_math::{DeveloperError, Event, ListenerId};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// Simulated Scene state for scene mode picking.
#[derive(Clone, Debug)]
pub struct SceneModePickerScene {
    pub mode: SceneMode,
    target_mode: SceneMode,
    pub morph_start: Event<(SceneMode, SceneMode)>,
    pub is_destroyed: bool,
}

impl Default for SceneModePickerScene {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneModePickerScene {
    /// Creates a new `SceneModePickerScene`.
    pub fn new() -> Self {
        Self {
            mode: SceneMode::Scene3D,
            target_mode: SceneMode::Scene3D,
            morph_start: Event::new(),
            is_destroyed: false,
        }
    }

    /// Morphs to 2D mode.
    pub fn morph_to_2d(&mut self, _duration: f64) {
        let old = self.mode;
        self.target_mode = SceneMode::Scene2D;
        self.morph_start.raise_event(&(old, SceneMode::Scene2D));
    }

    /// Morphs to 3D mode.
    pub fn morph_to_3d(&mut self, _duration: f64) {
        let old = self.mode;
        self.target_mode = SceneMode::Scene3D;
        self.morph_start.raise_event(&(old, SceneMode::Scene3D));
    }

    /// Morphs to Columbus View mode.
    pub fn morph_to_columbus_view(&mut self, _duration: f64) {
        let old = self.mode;
        self.target_mode = SceneMode::ColumbusView;
        self.morph_start.raise_event(&(old, SceneMode::ColumbusView));
    }

    /// Completes active morph transition.
    pub fn complete_morph(&mut self) {
        self.mode = self.target_mode;
    }

    /// Destroys scene resources.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }
}

/// View model for [`SceneModePicker`].
pub struct SceneModePickerViewModel {
    scene: Rc<RefCell<SceneModePickerScene>>,
    duration: f64,
    scene_mode: Rc<Cell<SceneMode>>,
    drop_down_visible: Rc<Cell<bool>>,
    pub tooltip_2d: String,
    pub tooltip_3d: String,
    pub tooltip_columbus_view: String,
    morph_listener_id: Option<ListenerId>,
    is_destroyed: bool,
}

impl SceneModePickerViewModel {
    /// Creates a new `SceneModePickerViewModel`.
    pub fn new(
        scene: Option<Rc<RefCell<SceneModePickerScene>>>,
        duration: Option<f64>,
    ) -> Result<Self, DeveloperError> {
        let scene = scene.ok_or_else(|| DeveloperError::new("scene is required."))?;
        let dur = duration.unwrap_or(2.0);
        if dur < 0.0 {
            return Err(DeveloperError::new("duration value must be positive."));
        }

        let scene_mode = Rc::new(Cell::new(scene.borrow().mode));
        let drop_down_visible = Rc::new(Cell::new(false));

        let mode_clone = Rc::clone(&scene_mode);
        let drop_clone = Rc::clone(&drop_down_visible);
        let morph_listener_id = scene
            .borrow()
            .morph_start
            .add_listener(move |(_old_mode, new_mode)| {
                mode_clone.set(*new_mode);
                drop_clone.set(false);
            });

        Ok(Self {
            scene,
            duration: dur,
            scene_mode,
            drop_down_visible,
            tooltip_2d: "2D".to_string(),
            tooltip_3d: "3D".to_string(),
            tooltip_columbus_view: "Columbus View".to_string(),
            morph_listener_id: Some(morph_listener_id),
            is_destroyed: false,
        })
    }

    /// Gets the scene.
    pub fn scene(&self) -> Rc<RefCell<SceneModePickerScene>> {
        Rc::clone(&self.scene)
    }

    /// Gets animation duration.
    pub fn duration(&self) -> f64 {
        self.duration
    }

    /// Sets animation duration.
    pub fn set_duration(&mut self, value: f64) -> Result<(), DeveloperError> {
        if value < 0.0 {
            return Err(DeveloperError::new("duration value must be positive."));
        }
        self.duration = value;
        Ok(())
    }

    /// Gets the current scene mode.
    pub fn scene_mode(&self) -> SceneMode {
        self.scene_mode.get()
    }

    /// Gets whether the dropdown is visible.
    pub fn drop_down_visible(&self) -> bool {
        self.drop_down_visible.get()
    }

    /// Sets whether the dropdown is visible.
    pub fn set_drop_down_visible(&mut self, visible: bool) {
        self.drop_down_visible.set(visible);
    }

    /// Toggles dropdown visibility.
    pub fn toggle_drop_down(&mut self) {
        self.drop_down_visible.set(!self.drop_down_visible.get());
    }

    /// Morphs to 2D.
    pub fn morph_to_2d(&mut self) {
        self.scene.borrow_mut().morph_to_2d(self.duration);
    }

    /// Morphs to 3D.
    pub fn morph_to_3d(&mut self) {
        self.scene.borrow_mut().morph_to_3d(self.duration);
    }

    /// Morphs to Columbus View.
    pub fn morph_to_columbus_view(&mut self) {
        self.scene.borrow_mut().morph_to_columbus_view(self.duration);
    }

    /// Gets the currently active tooltip text.
    pub fn selected_tooltip(&self) -> &str {
        match self.scene_mode.get() {
            SceneMode::Scene2D => &self.tooltip_2d,
            SceneMode::Scene3D => &self.tooltip_3d,
            _ => &self.tooltip_columbus_view,
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
                self.scene.borrow().morph_start.remove_listener(id);
            }
        }
    }
}

impl Drop for SceneModePickerViewModel {
    fn drop(&mut self) {
        self.destroy();
    }
}
