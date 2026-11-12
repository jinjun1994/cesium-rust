//! VoxelInspector widget matching CesiumJS `Widgets/VoxelInspector/VoxelInspector.js` (Apache-2.0).

use cesium_math::DeveloperError;
use std::cell::Cell;

use super::voxel_inspector_view_model::VoxelInspectorViewModel;

/// Container model for `VoxelInspector`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoxelInspectorContainer {
    pub id: String,
    pub exists: bool,
}

impl VoxelInspectorContainer {
    pub fn new(id: impl Into<String>, exists: bool) -> Self {
        Self {
            id: id.into(),
            exists,
        }
    }
}

/// The `VoxelInspector` widget.
#[derive(Debug)]
pub struct VoxelInspector {
    container: VoxelInspectorContainer,
    view_model: VoxelInspectorViewModel,
    is_destroyed: Cell<bool>,
}

impl VoxelInspector {
    /// Creates a new `VoxelInspector` widget.
    pub fn new(
        container: Option<VoxelInspectorContainer>,
        scene_id: Option<impl Into<String>>,
    ) -> Result<Self, DeveloperError> {
        let container = container.ok_or_else(|| DeveloperError::new("container is required."))?;
        if !container.exists {
            return Err(DeveloperError::new(
                "Element with given id does not exist in the document.",
            ));
        }

        let view_model = VoxelInspectorViewModel::new(scene_id)?;

        Ok(Self {
            container,
            view_model,
            is_destroyed: Cell::new(false),
        })
    }

    /// Creates a `VoxelInspector` from container ID.
    pub fn from_id(
        container_id: &str,
        scene_id: Option<impl Into<String>>,
        lookup: impl Fn(&str) -> Option<VoxelInspectorContainer>,
    ) -> Result<Self, DeveloperError> {
        if container_id.is_empty() {
            return Err(DeveloperError::new("container is required."));
        }
        let container = lookup(container_id).ok_or_else(|| {
            DeveloperError::new(format!(
                "Element with id \"{container_id}\" does not exist in the document."
            ))
        })?;
        Self::new(Some(container), scene_id)
    }

    /// Gets the container.
    pub fn container(&self) -> &VoxelInspectorContainer {
        &self.container
    }

    /// Gets the view model.
    pub fn view_model(&self) -> &VoxelInspectorViewModel {
        &self.view_model
    }

    /// Returns true if destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed.get()
    }

    /// Destroys this widget.
    pub fn destroy(&self) {
        self.is_destroyed.set(true);
        self.view_model.destroy();
    }
}
