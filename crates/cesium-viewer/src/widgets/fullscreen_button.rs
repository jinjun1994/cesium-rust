use std::cell::Cell;

use cesium_math::DeveloperError;

use super::fullscreen_button_view_model::FullscreenButtonViewModel;

/// Abstract UI container representation for `FullscreenButton`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullscreenButtonContainer {
    pub id: String,
    pub exists: bool,
}

impl FullscreenButtonContainer {
    pub fn new(id: impl Into<String>, exists: bool) -> Self {
        Self {
            id: id.into(),
            exists,
        }
    }
}

/// The `FullscreenButton` widget.
#[derive(Debug)]
pub struct FullscreenButton {
    container: FullscreenButtonContainer,
    view_model: FullscreenButtonViewModel,
    is_destroyed: Cell<bool>,
}

impl FullscreenButton {
    /// Creates a new `FullscreenButton` widget.
    pub fn new(
        container: Option<FullscreenButtonContainer>,
        fullscreen_element: Option<impl Into<String>>,
    ) -> Result<Self, DeveloperError> {
        let container = container.ok_or_else(|| DeveloperError::new("container is required."))?;
        if !container.exists {
            return Err(DeveloperError::new(
                "Element with given id does not exist in the document.",
            ));
        }

        let view_model = FullscreenButtonViewModel::new(fullscreen_element);

        Ok(Self {
            container,
            view_model,
            is_destroyed: Cell::new(false),
        })
    }

    /// Gets container.
    pub fn container(&self) -> &FullscreenButtonContainer {
        &self.container
    }

    /// Gets view model.
    pub fn view_model(&self) -> &FullscreenButtonViewModel {
        &self.view_model
    }

    /// Returns whether the widget has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed.get()
    }

    /// Destroys the widget.
    pub fn destroy(&self) {
        self.view_model.destroy();
        self.is_destroyed.set(true);
    }
}
