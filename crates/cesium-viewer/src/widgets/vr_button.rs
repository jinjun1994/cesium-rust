//! VRButton widget matching CesiumJS `Widgets/VRButton/VRButton.js` (Apache-2.0).

use cesium_math::DeveloperError;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::vr_button_view_model::{VRButtonViewModel, VRScene};

/// Abstract UI container representation for `VRButton`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VRButtonContainer {
    pub id: String,
    pub exists: bool,
}

impl VRButtonContainer {
    pub fn new(id: impl Into<String>, exists: bool) -> Self {
        Self {
            id: id.into(),
            exists,
        }
    }
}

/// The `VRButton` widget.
#[derive(Debug)]
pub struct VRButton {
    container: VRButtonContainer,
    view_model: VRButtonViewModel,
    is_destroyed: Cell<bool>,
}

impl VRButton {
    /// Creates a new `VRButton` widget.
    pub fn new(
        container: Option<VRButtonContainer>,
        scene: Option<Rc<RefCell<VRScene>>>,
        vr_element: Option<impl Into<String>>,
    ) -> Result<Self, DeveloperError> {
        let container = container.ok_or_else(|| DeveloperError::new("container is required."))?;
        if !container.exists {
            return Err(DeveloperError::new(
                "Element with given id does not exist in the document.",
            ));
        }

        let view_model = VRButtonViewModel::new(scene, vr_element)?;

        Ok(Self {
            container,
            view_model,
            is_destroyed: Cell::new(false),
        })
    }

    /// Creates a `VRButton` from a container ID.
    pub fn from_id(
        container_id: &str,
        scene: Option<Rc<RefCell<VRScene>>>,
        vr_element: Option<impl Into<String>>,
        lookup: impl Fn(&str) -> Option<VRButtonContainer>,
    ) -> Result<Self, DeveloperError> {
        if container_id.is_empty() {
            return Err(DeveloperError::new("container is required."));
        }
        let container = lookup(container_id).ok_or_else(|| {
            DeveloperError::new(format!(
                "Element with id \"{container_id}\" does not exist in the document."
            ))
        })?;
        Self::new(Some(container), scene, vr_element)
    }

    /// Gets the container.
    pub fn container(&self) -> &VRButtonContainer {
        &self.container
    }

    /// Gets the view model.
    pub fn view_model(&self) -> &VRButtonViewModel {
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
