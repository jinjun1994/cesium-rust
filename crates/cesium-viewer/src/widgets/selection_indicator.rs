//! SelectionIndicator widget matching CesiumJS `Widgets/SelectionIndicator/SelectionIndicator.js` (Apache-2.0).

use crate::widgets::selection_indicator_view_model::{
    SelectionIndicatorContainer, SelectionIndicatorElement, SelectionIndicatorScene,
    SelectionIndicatorViewModel,
};
use cesium_math::DeveloperError;
use std::cell::RefCell;
use std::rc::Rc;

/// A widget for displaying an indicator on a selected object.
pub struct SelectionIndicator {
    container: Option<Rc<RefCell<SelectionIndicatorContainer>>>,
    element: Rc<RefCell<SelectionIndicatorElement>>,
    view_model: SelectionIndicatorViewModel,
    is_destroyed: bool,
}

impl SelectionIndicator {
    /// Creates a new `SelectionIndicator`.
    pub fn new(
        container: Option<Rc<RefCell<SelectionIndicatorContainer>>>,
        scene: Option<Rc<RefCell<SelectionIndicatorScene>>>,
    ) -> Result<Self, DeveloperError> {
        let container = container.ok_or_else(|| DeveloperError::new("container is required."))?;
        let scene = scene.ok_or_else(|| DeveloperError::new("scene is required."))?;

        let element = Rc::new(RefCell::new(SelectionIndicatorElement::default()));
        let view_model = SelectionIndicatorViewModel::new(
            Some(scene),
            Some(Rc::clone(&element)),
            Some(Rc::clone(&container)),
        )?;

        Ok(Self {
            container: Some(container),
            element,
            view_model,
            is_destroyed: false,
        })
    }

    /// Creates a `SelectionIndicator` from container ID.
    pub fn from_id(
        container_id: &str,
        scene: Option<Rc<RefCell<SelectionIndicatorScene>>>,
        lookup: impl Fn(&str) -> Option<Rc<RefCell<SelectionIndicatorContainer>>>,
    ) -> Result<Self, DeveloperError> {
        if container_id.is_empty() {
            return Err(DeveloperError::new("container is required."));
        }
        let container = lookup(container_id).ok_or_else(|| {
            DeveloperError::new(format!(
                "Element with id \"{container_id}\" does not exist in the document."
            ))
        })?;
        Self::new(Some(container), scene)
    }

    /// Gets the container element.
    pub fn container(&self) -> Option<Rc<RefCell<SelectionIndicatorContainer>>> {
        self.container.clone()
    }

    /// Gets the visual indicator DOM element.
    pub fn element(&self) -> Rc<RefCell<SelectionIndicatorElement>> {
        Rc::clone(&self.element)
    }

    /// Gets the view model.
    pub fn view_model(&self) -> &SelectionIndicatorViewModel {
        &self.view_model
    }

    /// Gets the mutable view model.
    pub fn view_model_mut(&mut self) -> &mut SelectionIndicatorViewModel {
        &mut self.view_model
    }

    /// Returns true if this widget was destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys this widget.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        self.container = None;
    }
}
