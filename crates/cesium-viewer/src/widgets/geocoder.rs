use std::cell::Cell;

use cesium_math::DeveloperError;

use super::geocoder_view_model::{GeocoderViewModel, GeocoderViewModelOptions};

/// Container representation for `Geocoder`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeocoderContainer {
    pub id: String,
    pub exists: bool,
}

impl GeocoderContainer {
    pub fn new(id: impl Into<String>, exists: bool) -> Self {
        Self {
            id: id.into(),
            exists,
        }
    }
}

/// The `Geocoder` widget.
pub struct Geocoder {
    container: GeocoderContainer,
    view_model: GeocoderViewModel,
    is_destroyed: Cell<bool>,
}

impl Geocoder {
    /// Creates a new `Geocoder` widget.
    pub fn new(
        container: Option<GeocoderContainer>,
        options: Option<GeocoderViewModelOptions>,
    ) -> Result<Self, DeveloperError> {
        let container = container.ok_or_else(|| DeveloperError::new("container is required."))?;
        if !container.exists {
            return Err(DeveloperError::new(
                "Element with given id does not exist in the document.",
            ));
        }

        let options = options.ok_or_else(|| DeveloperError::new("options are required."))?;
        let view_model = GeocoderViewModel::new(options)?;

        Ok(Self {
            container,
            view_model,
            is_destroyed: Cell::new(false),
        })
    }

    /// Gets container.
    pub fn container(&self) -> &GeocoderContainer {
        &self.container
    }

    /// Gets view model.
    pub fn view_model(&self) -> &GeocoderViewModel {
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

impl std::fmt::Debug for Geocoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Geocoder")
            .field("container", &self.container)
            .field("is_destroyed", &self.is_destroyed.get())
            .finish()
    }
}

