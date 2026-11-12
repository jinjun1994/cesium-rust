use std::cell::{Cell, RefCell};
use std::rc::Rc;

use cesium_math::DeveloperError;

use super::create_command::{create_command_with_can_execute, Command};

/// View model for `FullscreenButton`.
#[derive(Clone, Debug)]
pub struct FullscreenButtonViewModel {
    fullscreen_element: RefCell<String>,
    is_fullscreen: Rc<Cell<bool>>,
    is_fullscreen_enabled: Rc<Cell<bool>>,
    command: Command<(), ()>,
    is_destroyed: Cell<bool>,
}

impl Default for FullscreenButtonViewModel {
    fn default() -> Self {
        Self::new(None::<&str>)
    }
}

impl FullscreenButtonViewModel {
    /// Creates a new `FullscreenButtonViewModel` with optional target fullscreen element.
    pub fn new(fullscreen_element: Option<impl Into<String>>) -> Self {
        let element = fullscreen_element
            .map(Into::into)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "document.body".to_string());

        let is_fullscreen = Rc::new(Cell::new(false));
        let is_fullscreen_enabled = Rc::new(Cell::new(true));

        let fs_clone = Rc::clone(&is_fullscreen);
        let command = create_command_with_can_execute(
            move |()| {
                fs_clone.set(!fs_clone.get());
            },
            true,
        );

        Self {
            fullscreen_element: RefCell::new(element),
            is_fullscreen,
            is_fullscreen_enabled,
            command,
            is_destroyed: Cell::new(false),
        }
    }

    /// Gets whether fullscreen mode is currently active.
    pub fn is_fullscreen(&self) -> bool {
        self.is_fullscreen.get()
    }

    /// Sets whether fullscreen mode is currently active.
    pub fn set_is_fullscreen(&self, value: bool) {
        self.is_fullscreen.set(value);
    }

    /// Gets whether fullscreen capability is enabled.
    pub fn is_fullscreen_enabled(&self) -> bool {
        self.is_fullscreen_enabled.get()
    }

    /// Sets whether fullscreen capability is enabled and updates command execution status.
    pub fn set_is_fullscreen_enabled(&self, value: bool) {
        self.is_fullscreen_enabled.set(value);
        self.command.set_can_execute(value);
    }

    /// Gets the current fullscreen element.
    pub fn fullscreen_element(&self) -> String {
        self.fullscreen_element.borrow().clone()
    }

    /// Sets the target fullscreen element.
    pub fn set_fullscreen_element(&self, element: impl Into<String>) -> Result<(), DeveloperError> {
        let name = element.into();
        if name.trim().is_empty() {
            return Err(DeveloperError::new("value must be a valid Element."));
        }
        *self.fullscreen_element.borrow_mut() = name;
        Ok(())
    }

    /// Gets the tooltip.
    pub fn tooltip(&self) -> String {
        if !self.is_fullscreen_enabled() {
            "Full screen unavailable".to_string()
        } else if self.is_fullscreen() {
            "Exit full screen".to_string()
        } else {
            "Full screen".to_string()
        }
    }

    /// Gets the toggle fullscreen command.
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
