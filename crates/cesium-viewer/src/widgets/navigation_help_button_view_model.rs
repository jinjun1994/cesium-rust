//! NavigationHelpButtonViewModel matching CesiumJS `Widgets/NavigationHelpButton/NavigationHelpButtonViewModel.js` (Apache-2.0).

use crate::widgets::create_command::{create_command, Command};
use std::cell::Cell;
use std::rc::Rc;

/// View model for [`NavigationHelpButton`].
pub struct NavigationHelpButtonViewModel {
    show_instructions: Rc<Cell<bool>>,
    touch: Rc<Cell<bool>>,
    pub tooltip: String,
    command: Command,
    show_click: Command,
    show_touch: Command,
}

impl Default for NavigationHelpButtonViewModel {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationHelpButtonViewModel {
    /// Creates a new `NavigationHelpButtonViewModel`.
    pub fn new() -> Self {
        let show_instructions = Rc::new(Cell::new(false));
        let touch = Rc::new(Cell::new(false));

        let show_inst_clone = Rc::clone(&show_instructions);
        let command = create_command(move |_| {
            show_inst_clone.set(!show_inst_clone.get());
        });

        let touch_clone1 = Rc::clone(&touch);
        let show_click = create_command(move |_| {
            touch_clone1.set(false);
        });

        let touch_clone2 = Rc::clone(&touch);
        let show_touch = create_command(move |_| {
            touch_clone2.set(true);
        });

        Self {
            show_instructions,
            touch,
            tooltip: "Navigation Instructions".to_string(),
            command,
            show_click,
            show_touch,
        }
    }

    /// Gets whether navigation instructions are currently shown.
    pub fn show_instructions(&self) -> bool {
        self.show_instructions.get()
    }

    /// Sets whether navigation instructions are currently shown.
    pub fn set_show_instructions(&mut self, show: bool) {
        self.show_instructions.set(show);
    }

    /// Gets whether touch mode instructions are shown.
    pub fn is_touch(&self) -> bool {
        self.touch.get()
    }

    /// Sets whether touch mode instructions are shown.
    pub fn set_touch(&mut self, touch: bool) {
        self.touch.set(touch);
    }

    /// Gets the command that toggles instruction visibility.
    pub fn command(&self) -> Command {
        self.command.clone()
    }

    /// Gets the command that selects mouse instructions.
    pub fn show_click(&self) -> Command {
        self.show_click.clone()
    }

    /// Gets the command that selects touch instructions.
    pub fn show_touch(&self) -> Command {
        self.show_touch.clone()
    }
}
