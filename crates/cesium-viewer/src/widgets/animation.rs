use std::cell::Cell;

use super::animation_view_model::AnimationViewModel;

/// Represents an abstract or DOM container for the `Animation` widget.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AnimationContainer {
    pub id: String,
    pub in_dom: bool,
}

impl AnimationContainer {
    pub fn new(id: impl Into<String>, in_dom: bool) -> Self {
        Self {
            id: id.into(),
            in_dom,
        }
    }
}

impl<S: Into<String>> From<S> for AnimationContainer {
    fn from(s: S) -> Self {
        Self {
            id: s.into(),
            in_dom: true,
        }
    }
}

/// The Animation widget for playback, speed and clock controls.
pub struct Animation {
    container: AnimationContainer,
    view_model: AnimationViewModel,
    theme_changes_count: Cell<usize>,
    is_destroyed: Cell<bool>,
}

impl Animation {
    /// Creates a new `Animation` widget bound to a container and viewModel.
    pub fn new(container: impl Into<AnimationContainer>, view_model: AnimationViewModel) -> Self {
        Self {
            container: container.into(),
            view_model,
            theme_changes_count: Cell::new(0),
            is_destroyed: Cell::new(false),
        }
    }

    /// Gets the container associated with this widget.
    pub fn container(&self) -> &AnimationContainer {
        &self.container
    }

    /// Gets the associated viewModel.
    pub fn view_model(&self) -> &AnimationViewModel {
        &self.view_model
    }

    /// Applies theme changes to widget styling.
    pub fn apply_theme_changes(&self) {
        self.theme_changes_count
            .set(self.theme_changes_count.get() + 1);
    }

    /// Gets how many times theme changes were applied.
    pub fn apply_theme_changes_count(&self) -> usize {
        self.theme_changes_count.get()
    }

    /// Returns whether the widget has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed.get()
    }

    /// Destroys the widget and cleans up resources.
    pub fn destroy(&self) {
        self.is_destroyed.set(true);
    }
}
