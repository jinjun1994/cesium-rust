//! SceneModePicker widget matching CesiumJS `Widgets/SceneModePicker/SceneModePicker.js` (Apache-2.0).

use crate::widgets::scene_mode_picker_view_model::{
    SceneModePickerScene, SceneModePickerViewModel,
};
use cesium_math::DeveloperError;
use std::cell::RefCell;
use std::rc::Rc;

/// Simulated DOM element inside SceneModePicker.
#[derive(Clone, Debug, Default)]
pub struct SceneModePickerElement {
    pub tag: String,
    pub class_name: String,
    pub children: Vec<Rc<RefCell<SceneModePickerElement>>>,
}

impl SceneModePickerElement {
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

/// Simulated container DOM element for SceneModePicker.
#[derive(Clone, Debug, Default)]
pub struct SceneModePickerContainer {
    pub id: String,
    pub children: Vec<Rc<RefCell<SceneModePickerElement>>>,
}

impl SceneModePickerContainer {
    /// Creates a new container with an ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            children: Vec::new(),
        }
    }

    /// Gets the first child element of the container.
    pub fn first_child(&self) -> Option<Rc<RefCell<SceneModePickerElement>>> {
        self.children.first().cloned()
    }
}

/// Single button widget for switching between scene modes (2D, 3D, Columbus View).
pub struct SceneModePicker {
    container: Option<Rc<RefCell<SceneModePickerContainer>>>,
    wrapper: Rc<RefCell<SceneModePickerElement>>,
    view_model: SceneModePickerViewModel,
    is_destroyed: bool,
}

impl SceneModePicker {
    /// Creates a new `SceneModePicker`.
    pub fn new(
        container: Option<Rc<RefCell<SceneModePickerContainer>>>,
        scene: Option<Rc<RefCell<SceneModePickerScene>>>,
        duration: Option<f64>,
    ) -> Result<Self, DeveloperError> {
        let container = container.ok_or_else(|| DeveloperError::new("container is required."))?;
        let scene = scene.ok_or_else(|| DeveloperError::new("scene is required."))?;

        let view_model = SceneModePickerViewModel::new(Some(scene), duration)?;

        let wrapper = SceneModePickerElement::new("span");
        wrapper.borrow_mut().class_name =
            "cesium-sceneModePicker-wrapper cesium-toolbar-button".to_string();

        let button = SceneModePickerElement::new("button");
        button.borrow_mut().class_name = "cesium-button cesium-toolbar-button".to_string();
        wrapper.borrow_mut().children.push(button);

        let morph_3d = SceneModePickerElement::new("button");
        morph_3d.borrow_mut().class_name =
            "cesium-button cesium-toolbar-button cesium-sceneModePicker-dropDown-icon".to_string();
        wrapper.borrow_mut().children.push(morph_3d);

        let morph_2d = SceneModePickerElement::new("button");
        morph_2d.borrow_mut().class_name =
            "cesium-button cesium-toolbar-button cesium-sceneModePicker-dropDown-icon".to_string();
        wrapper.borrow_mut().children.push(morph_2d);

        let morph_columbus = SceneModePickerElement::new("button");
        morph_columbus.borrow_mut().class_name =
            "cesium-button cesium-toolbar-button cesium-sceneModePicker-dropDown-icon".to_string();
        wrapper.borrow_mut().children.push(morph_columbus);

        container.borrow_mut().children.push(Rc::clone(&wrapper));

        Ok(Self {
            container: Some(container),
            wrapper,
            view_model,
            is_destroyed: false,
        })
    }

    /// Creates a `SceneModePicker` from container ID.
    pub fn from_id(
        container_id: &str,
        scene: Option<Rc<RefCell<SceneModePickerScene>>>,
        duration: Option<f64>,
        lookup: impl Fn(&str) -> Option<Rc<RefCell<SceneModePickerContainer>>>,
    ) -> Result<Self, DeveloperError> {
        if container_id.is_empty() {
            return Err(DeveloperError::new("container is required."));
        }
        let container = lookup(container_id).ok_or_else(|| {
            DeveloperError::new(format!(
                "Element with id \"{container_id}\" does not exist in the document."
            ))
        })?;
        Self::new(Some(container), scene, duration)
    }

    /// Gets the container element.
    pub fn container(&self) -> Option<Rc<RefCell<SceneModePickerContainer>>> {
        self.container.clone()
    }

    /// Gets the view model.
    pub fn view_model(&self) -> &SceneModePickerViewModel {
        &self.view_model
    }

    /// Gets the mutable view model.
    pub fn view_model_mut(&mut self) -> &mut SceneModePickerViewModel {
        &mut self.view_model
    }

    /// Handles pointer/mouse input event outside wrapper to auto-close dropdown.
    pub fn close_on_input(&mut self, target_node: &Rc<RefCell<SceneModePickerElement>>) {
        if !SceneModePickerElement::contains(&self.wrapper, target_node) {
            self.view_model.set_drop_down_visible(false);
        }
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
