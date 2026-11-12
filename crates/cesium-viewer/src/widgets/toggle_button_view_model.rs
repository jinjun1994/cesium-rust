use std::rc::Rc;

use super::create_command::Command;

/// A view model representing a toggle button in Cesium UI widgets.
#[derive(Clone)]
pub struct ToggleButtonViewModel {
    pub command: Command<(), ()>,
    toggled_fn: Rc<dyn Fn() -> bool>,
    tooltip_fn: Rc<dyn Fn() -> String>,
}

impl ToggleButtonViewModel {
    /// Creates a new `ToggleButtonViewModel` with custom dynamic closures for `toggled` and `tooltip`.
    pub fn new<T, S>(command: Command<(), ()>, toggled: T, tooltip: S) -> Self
    where
        T: Fn() -> bool + 'static,
        S: Fn() -> String + 'static,
    {
        Self {
            command,
            toggled_fn: Rc::new(toggled),
            tooltip_fn: Rc::new(tooltip),
        }
    }

    /// Creates a `ToggleButtonViewModel` with static tooltip string and dynamic toggled state.
    pub fn with_static_tooltip<T>(command: Command<(), ()>, toggled: T, tooltip: impl Into<String>) -> Self
    where
        T: Fn() -> bool + 'static,
    {
        let tooltip_str = tooltip.into();
        Self::new(command, toggled, move || tooltip_str.clone())
    }

    /// Returns whether the button is currently toggled.
    #[inline]
    pub fn toggled(&self) -> bool {
        (self.toggled_fn)()
    }

    /// Returns the current tooltip text.
    #[inline]
    pub fn tooltip(&self) -> String {
        (self.tooltip_fn)()
    }
}
