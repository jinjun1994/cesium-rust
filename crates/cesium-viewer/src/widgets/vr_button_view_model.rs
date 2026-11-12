//! VRButtonViewModel matching CesiumJS `Widgets/VRButton/VRButtonViewModel.js` (Apache-2.0).

use cesium_math::DeveloperError;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::create_command::{create_command_with_can_execute, Command};

/// Simulated scene for VR mode.
#[derive(Clone, Debug, Default)]
pub struct VRScene {
    pub use_web_vr: Cell<bool>,
    pub is_destroyed: Cell<bool>,
}

impl VRScene {
    pub fn new() -> Self {
        Self {
            use_web_vr: Cell::new(false),
            is_destroyed: Cell::new(false),
        }
    }

    pub fn destroy(&self) {
        self.is_destroyed.set(true);
    }
}

/// View model for `VRButton`.
#[derive(Clone, Debug)]
pub struct VRButtonViewModel {
    scene: Rc<RefCell<VRScene>>,
    vr_element: RefCell<String>,
    is_vr_mode: Rc<Cell<bool>>,
    is_vr_enabled: Rc<Cell<bool>>,
    command: Command<(), ()>,
    is_destroyed: Cell<bool>,
}

impl VRButtonViewModel {
    /// Creates a new `VRButtonViewModel`.
    pub fn new(
        scene: Option<Rc<RefCell<VRScene>>>,
        vr_element: Option<impl Into<String>>,
    ) -> Result<Self, DeveloperError> {
        let scene = scene.ok_or_else(|| DeveloperError::new("scene is required."))?;

        let element = vr_element
            .map(Into::into)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "document.body".to_string());

        let is_vr_mode = Rc::new(Cell::new(false));
        let is_vr_enabled = Rc::new(Cell::new(true));

        let vr_clone = Rc::clone(&is_vr_mode);
        let scene_clone = Rc::clone(&scene);
        let command = create_command_with_can_execute(
            move |()| {
                let next = !vr_clone.get();
                vr_clone.set(next);
                scene_clone.borrow().use_web_vr.set(next);
            },
            true,
        );

        Ok(Self {
            scene,
            vr_element: RefCell::new(element),
            is_vr_mode,
            is_vr_enabled,
            command,
            is_destroyed: Cell::new(false),
        })
    }

    /// Gets the scene.
    pub fn scene(&self) -> Rc<RefCell<VRScene>> {
        Rc::clone(&self.scene)
    }

    /// Gets target element identifier.
    pub fn vr_element(&self) -> String {
        self.vr_element.borrow().clone()
    }

    /// Sets target element identifier.
    pub fn set_vr_element(&self, element: impl Into<String>) -> Result<(), DeveloperError> {
        let el = element.into();
        if el.is_empty() {
            return Err(DeveloperError::new("vrElement is not an Element or valid ID"));
        }
        *self.vr_element.borrow_mut() = el;
        Ok(())
    }

    /// Gets whether VR is enabled on this system/browser.
    pub fn is_vr_enabled(&self) -> bool {
        self.is_vr_enabled.get()
    }

    /// Sets whether VR is enabled.
    pub fn set_vr_enabled(&self, enabled: bool) {
        self.is_vr_enabled.set(enabled);
    }

    /// Gets whether currently presenting in VR.
    pub fn is_vr_mode(&self) -> bool {
        self.is_vr_mode.get()
    }

    /// Gets the command.
    pub fn command(&self) -> &Command<(), ()> {
        &self.command
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
