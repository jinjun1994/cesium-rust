use std::cell::Cell;

use cesium_math::DeveloperError;

use super::base_layer_picker_view_model::{
    BaseLayerPickerViewModel, BaseLayerPickerViewModelOptions, GlobeModel, StandaloneGlobe,
};

/// An abstract UI container representation for `BaseLayerPicker`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaseLayerPickerContainer {
    pub id: String,
    pub exists: bool,
}

impl BaseLayerPickerContainer {
    pub fn new(id: impl Into<String>, exists: bool) -> Self {
        Self {
            id: id.into(),
            exists,
        }
    }
}

/// The `BaseLayerPicker` widget displaying imagery and terrain options.
pub struct BaseLayerPicker<G: GlobeModel + 'static = StandaloneGlobe, R: Clone + 'static = ()> {
    container: BaseLayerPickerContainer,
    view_model: BaseLayerPickerViewModel<G, R>,
    is_destroyed: Cell<bool>,
}

impl<G: GlobeModel + 'static, R: Clone + 'static> BaseLayerPicker<G, R> {
    /// Creates a new `BaseLayerPicker` widget.
    pub fn new(
        container: Option<BaseLayerPickerContainer>,
        options: Option<BaseLayerPickerViewModelOptions<G, R>>,
    ) -> Result<Self, DeveloperError> {
        let container = container.ok_or_else(|| DeveloperError::new("container is required."))?;
        if !container.exists {
            return Err(DeveloperError::new(
                "Element with given id does not exist in the document.",
            ));
        }

        let options = options.ok_or_else(|| DeveloperError::new("options are required."))?;
        let view_model = BaseLayerPickerViewModel::new(options)?;

        Ok(Self {
            container,
            view_model,
            is_destroyed: Cell::new(false),
        })
    }

    /// Gets the container.
    pub fn container(&self) -> &BaseLayerPickerContainer {
        &self.container
    }

    /// Gets the associated viewModel.
    pub fn view_model(&self) -> &BaseLayerPickerViewModel<G, R> {
        &self.view_model
    }

    /// Simulates an input/pointer/mouse event on a target target_id.
    /// If the target is outside this widget's container, closes the dropdown.
    pub fn handle_pointer_down(&self, target_id: &str) {
        if target_id != self.container.id && !target_id.starts_with(&format!("{}_child", self.container.id)) {
            self.view_model.set_drop_down_visible(false);
        }
    }

    /// Returns whether the widget has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed.get()
    }

    /// Destroys the widget.
    pub fn destroy(&self) {
        self.is_destroyed.set(true);
    }
}

impl<G: GlobeModel + 'static, R: Clone + 'static> std::fmt::Debug for BaseLayerPicker<G, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BaseLayerPicker")
            .field("container", &self.container)
            .field("is_destroyed", &self.is_destroyed.get())
            .finish()
    }
}

