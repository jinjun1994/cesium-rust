use std::cell::Cell;

use cesium_math::DeveloperError;

use super::home_button_view_model::HomeButtonViewModel;

/// Container model for `HomeButton`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HomeButtonContainer {
    pub id: String,
    pub exists: bool,
}

impl HomeButtonContainer {
    pub fn new(id: impl Into<String>, exists: bool) -> Self {
        Self {
            id: id.into(),
            exists,
        }
    }
}

/// The `HomeButton` widget.
#[derive(Debug)]
pub struct HomeButton {
    container: HomeButtonContainer,
    view_model: HomeButtonViewModel,
    is_destroyed: Cell<bool>,
}

impl HomeButton {
    /// Creates a new `HomeButton` widget.
    pub fn new(
        container: Option<HomeButtonContainer>,
        scene_id: Option<impl Into<String>>,
        duration: Option<f64>,
    ) -> Result<Self, DeveloperError> {
        let container = container.ok_or_else(|| DeveloperError::new("container is required."))?;
        if !container.exists {
            return Err(DeveloperError::new(
                "Element with given id does not exist in the document.",
            ));
        }

        let view_model = HomeButtonViewModel::new(scene_id, duration)?;

        Ok(Self {
            container,
            view_model,
            is_destroyed: Cell::new(false),
        })
    }

    /// Gets container.
    pub fn container(&self) -> &HomeButtonContainer {
        &self.container
    }

    /// Gets view model.
    pub fn view_model(&self) -> &HomeButtonViewModel {
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
