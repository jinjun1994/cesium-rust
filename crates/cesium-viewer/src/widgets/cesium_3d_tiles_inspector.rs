use std::cell::Cell;

use cesium_math::DeveloperError;

use super::cesium_3d_tiles_inspector_view_model::Cesium3DTilesInspectorViewModel;

/// Abstract UI container representation for `Cesium3DTilesInspector`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cesium3DTilesInspectorContainer {
    pub id: String,
    pub exists: bool,
}

impl Cesium3DTilesInspectorContainer {
    pub fn new(id: impl Into<String>, exists: bool) -> Self {
        Self {
            id: id.into(),
            exists,
        }
    }
}

/// The `Cesium3DTilesInspector` widget.
#[derive(Debug)]
pub struct Cesium3DTilesInspector {
    container: Cesium3DTilesInspectorContainer,
    view_model: Cesium3DTilesInspectorViewModel,
    is_destroyed: Cell<bool>,
}

impl Cesium3DTilesInspector {
    /// Creates a new `Cesium3DTilesInspector` widget.
    pub fn new(
        container: Option<Cesium3DTilesInspectorContainer>,
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

        let perf_id = format!("{}_performance", container.id);
        let view_model = Cesium3DTilesInspectorViewModel::new(Some(scene_id), Some(perf_id))?;

        Ok(Self {
            container,
            view_model,
            is_destroyed: Cell::new(false),
        })
    }

    /// Gets container.
    pub fn container(&self) -> &Cesium3DTilesInspectorContainer {
        &self.container
    }

    /// Gets view model.
    pub fn view_model(&self) -> &Cesium3DTilesInspectorViewModel {
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
