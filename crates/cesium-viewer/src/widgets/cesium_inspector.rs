use std::cell::Cell;

use cesium_math::DeveloperError;

use super::cesium_inspector_view_model::CesiumInspectorViewModel;

/// Container model for `CesiumInspector`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CesiumInspectorContainer {
    pub id: String,
    pub exists: bool,
}

impl CesiumInspectorContainer {
    pub fn new(id: impl Into<String>, exists: bool) -> Self {
        Self {
            id: id.into(),
            exists,
        }
    }
}

/// The `CesiumInspector` widget.
#[derive(Debug)]
pub struct CesiumInspector {
    container: CesiumInspectorContainer,
    view_model: CesiumInspectorViewModel,
    is_destroyed: Cell<bool>,
}

impl CesiumInspector {
    /// Creates a new `CesiumInspector` widget.
    pub fn new(
        container: Option<CesiumInspectorContainer>,
        scene_id: Option<impl Into<String>>,
    ) -> Result<Self, DeveloperError> {
        let container = container.ok_or_else(|| DeveloperError::new("container is required."))?;
        if !container.exists {
            return Err(DeveloperError::new(
                "Element with given id does not exist in the document.",
            ));
        }

        let scene_id = scene_id
            .map(Into::into)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| DeveloperError::new("scene is required."))?;

        let perf_id = format!("{}_perf", container.id);
        let view_model = CesiumInspectorViewModel::new(Some(scene_id), Some(perf_id))?;

        Ok(Self {
            container,
            view_model,
            is_destroyed: Cell::new(false),
        })
    }

    /// Gets container.
    pub fn container(&self) -> &CesiumInspectorContainer {
        &self.container
    }

    /// Gets view model.
    pub fn view_model(&self) -> &CesiumInspectorViewModel {
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
