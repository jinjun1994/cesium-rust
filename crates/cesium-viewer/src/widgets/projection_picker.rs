//! ProjectionPicker widget matching CesiumJS `Widgets/ProjectionPicker/ProjectionPicker.js` (Apache-2.0).

use crate::widgets::projection_picker_view_model::{
    ProjectionPickerScene, ProjectionPickerViewModel,
};
use cesium_math::DeveloperError;
use std::cell::RefCell;
use std::rc::Rc;

/// DOM node element inside ProjectionPicker.
#[derive(Clone, Debug, Default)]
pub struct ProjectionPickerElement {
    pub tag: String,
    pub class_name: String,
    pub children: Vec<Rc<RefCell<ProjectionPickerElement>>>,
}

impl ProjectionPickerElement {
    /// Creates a new element.
    pub fn new(tag: impl Into<String>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            tag: tag.into(),
            class_name: String::new(),
            children: Vec::new(),
        }))
    }

    /// Checks if this node contains target.
    pub fn contains(node_rc: &Rc<RefCell<Self>>, target_rc: &Rc<RefCell<Self>>) -> bool {
        if Rc::ptr_eq(node_rc, target_rc) {
            return true;
        }
        for child in &node_rc.borrow().children {
            if Self::contains(child, target_rc) {
                return true;
            }
        }
        false
    }
}

/// Simulated container DOM element for ProjectionPicker.
#[derive(Clone, Debug, Default)]
pub struct ProjectionPickerContainer {
    pub id: String,
    pub children: Vec<Rc<RefCell<ProjectionPickerElement>>>,
}

impl ProjectionPickerContainer {
    /// Creates a new container with an ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            children: Vec::new(),
        }
    }

    /// Gets the first child element of the container.
    pub fn first_child(&self) -> Option<Rc<RefCell<ProjectionPickerElement>>> {
        self.children.first().cloned()
    }
}

/// Single button widget for switching between perspective and orthographic projections.
pub struct ProjectionPicker {
    container: Option<Rc<RefCell<ProjectionPickerContainer>>>,
    wrapper: Rc<RefCell<ProjectionPickerElement>>,
    view_model: ProjectionPickerViewModel,
    is_destroyed: bool,
}

impl ProjectionPicker {
    /// Creates a new `ProjectionPicker`.
    pub fn new(
        container: Option<Rc<RefCell<ProjectionPickerContainer>>>,
        scene: Option<Rc<RefCell<ProjectionPickerScene>>>,
    ) -> Result<Self, DeveloperError> {
        let container = container.ok_or_else(|| DeveloperError::new("container is required."))?;
        let scene = scene.ok_or_else(|| DeveloperError::new("scene is required."))?;

        let view_model = ProjectionPickerViewModel::new(Some(scene))?;

        let wrapper = ProjectionPickerElement::new("span");
        wrapper.borrow_mut().class_name =
            "cesium-projectionPicker-wrapper cesium-toolbar-button".to_string();

        let button = ProjectionPickerElement::new("button");
        button.borrow_mut().class_name = "cesium-button cesium-toolbar-button".to_string();
        wrapper.borrow_mut().children.push(button);

        let perspective_button = ProjectionPickerElement::new("button");
        perspective_button.borrow_mut().class_name =
            "cesium-button cesium-toolbar-button cesium-projectionPicker-dropDown-icon".to_string();
        wrapper.borrow_mut().children.push(perspective_button);

        let orthographic_button = ProjectionPickerElement::new("button");
        orthographic_button.borrow_mut().class_name =
            "cesium-button cesium-toolbar-button cesium-projectionPicker-dropDown-icon".to_string();
        wrapper.borrow_mut().children.push(orthographic_button);

        container.borrow_mut().children.push(Rc::clone(&wrapper));

        Ok(Self {
            container: Some(container),
            wrapper,
            view_model,
            is_destroyed: false,
        })
    }

    /// Creates a `ProjectionPicker` from container ID.
    pub fn from_id(
        container_id: &str,
        scene: Option<Rc<RefCell<ProjectionPickerScene>>>,
        lookup: impl Fn(&str) -> Option<Rc<RefCell<ProjectionPickerContainer>>>,
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
    pub fn container(&self) -> Option<Rc<RefCell<ProjectionPickerContainer>>> {
        self.container.clone()
    }

    /// Gets the view model.
    pub fn view_model(&self) -> &ProjectionPickerViewModel {
        &self.view_model
    }

    /// Gets the mutable view model.
    pub fn view_model_mut(&mut self) -> &mut ProjectionPickerViewModel {
        &mut self.view_model
    }

    /// Handles pointer/mouse input event outside wrapper to auto-close dropdown.
    pub fn close_on_input(&mut self, target_node: &Rc<RefCell<ProjectionPickerElement>>) {
        if !ProjectionPickerElement::contains(&self.wrapper, target_node) {
            self.view_model.set_drop_down_visible(false);
        }
    }

    /// Simulates clicking the main picker button.
    pub fn handle_button_click(&mut self) {
        self.view_model.toggle_drop_down();
    }

    /// Returns true if this widget was destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys this widget and cleans up container children.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        if let Some(container) = self.container.take() {
            container.borrow_mut().children.clear();
        }
        self.view_model.destroy();
    }
}
