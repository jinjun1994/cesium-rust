use std::cell::{Cell, RefCell};
use std::rc::Rc;

use cesium_math::DeveloperError;

use super::create_command::{create_command, Command};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SceneMode {
    #[default]
    Scene3D,
    Scene2D,
    ColumbusView,
    Morphing,
}

/// A scene model interface that responds to camera flyHome requests.
#[derive(Clone, Debug, Default)]
pub struct HomeButtonSceneState {
    pub mode: SceneMode,
    pub home_flight_count: usize,
    pub last_duration: Option<f64>,
}

/// View model for `HomeButton`.
#[derive(Clone, Debug)]
pub struct HomeButtonViewModel {
    scene_id: String,
    duration: Cell<Option<f64>>,
    tooltip: RefCell<String>,
    scene_state: Rc<RefCell<HomeButtonSceneState>>,
    command: Command<(), ()>,
    is_destroyed: Cell<bool>,
}

impl HomeButtonViewModel {
    /// Creates a new `HomeButtonViewModel`.
    pub fn new(
        scene_id: Option<impl Into<String>>,
        duration: Option<f64>,
    ) -> Result<Self, DeveloperError> {
        let scene_id = scene_id
            .map(Into::into)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| DeveloperError::new("scene is required."))?;

        if let Some(dur) = duration {
            if dur < 0.0 {
                return Err(DeveloperError::new("duration must be positive."));
            }
        }

        let state = Rc::new(RefCell::new(HomeButtonSceneState::default()));
        let state_clone = Rc::clone(&state);
        let duration_cell = Cell::new(duration);

        let cmd = create_command(move |()| {
            let mut st = state_clone.borrow_mut();
            st.home_flight_count += 1;
            st.last_duration = duration_cell.get();
        });

        Ok(Self {
            scene_id,
            duration: Cell::new(duration),
            tooltip: RefCell::new("View Home".to_string()),
            scene_state: state,
            command: cmd,
            is_destroyed: Cell::new(false),
        })
    }

    /// Gets scene ID.
    pub fn scene_id(&self) -> &str {
        &self.scene_id
    }

    /// Gets camera flight duration.
    pub fn duration(&self) -> Option<f64> {
        self.duration.get()
    }

    /// Sets camera flight duration.
    pub fn set_duration(&self, duration: Option<f64>) -> Result<(), DeveloperError> {
        if let Some(dur) = duration {
            if dur < 0.0 {
                return Err(DeveloperError::new("duration must be positive."));
            }
        }
        self.duration.set(duration);
        Ok(())
    }

    /// Gets tooltip string.
    pub fn tooltip(&self) -> String {
        self.tooltip.borrow().clone()
    }

    /// Sets scene mode for testing/integration.
    pub fn set_scene_mode(&self, mode: SceneMode) {
        self.scene_state.borrow_mut().mode = mode;
    }

    /// Gets scene mode.
    pub fn scene_mode(&self) -> SceneMode {
        self.scene_state.borrow().mode
    }

    /// Gets count of flyHome executions.
    pub fn home_flight_count(&self) -> usize {
        self.scene_state.borrow().home_flight_count
    }

    /// Gets flyHome command.
    pub fn command(&self) -> &Command<(), ()> {
        &self.command
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
