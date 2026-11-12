//! NavigationHelpButton widget matching CesiumJS `Widgets/NavigationHelpButton/NavigationHelpButton.js` (Apache-2.0).

use crate::widgets::navigation_help_button_view_model::NavigationHelpButtonViewModel;
use cesium_math::DeveloperError;
use std::cell::RefCell;
use std::rc::Rc;

/// DOM node element inside the NavigationHelpButton.
#[derive(Clone, Debug, Default)]
pub struct NavigationHelpElement {
    pub tag: String,
    pub id: Option<String>,
    pub class_name: String,
    pub children: Vec<Rc<RefCell<NavigationHelpElement>>>,
}

impl NavigationHelpElement {
    /// Creates a new element.
    pub fn new(tag: impl Into<String>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            tag: tag.into(),
            id: None,
            class_name: String::new(),
            children: Vec::new(),
        }))
    }

    /// Checks whether this element or any of its descendants matches the given target element.
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

/// Simulated container DOM element for NavigationHelpButton.
#[derive(Clone, Debug, Default)]
pub struct NavigationHelpButtonContainer {
    pub id: String,
    pub children: Vec<Rc<RefCell<NavigationHelpElement>>>,
}

impl NavigationHelpButtonContainer {
    /// Creates a new container with an ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            children: Vec::new(),
        }
    }

    /// Gets the first child element of the container.
    pub fn first_child(&self) -> Option<Rc<RefCell<NavigationHelpElement>>> {
        self.children.first().cloned()
    }
}

/// Initialization options for [`NavigationHelpButton`].
#[derive(Clone, Debug, Default)]
pub struct NavigationHelpButtonOptions {
    /// The DOM container element or handle.
    pub container: Option<Rc<RefCell<NavigationHelpButtonContainer>>>,
    /// Whether navigation instructions should initially be visible. Defaults to false.
    pub instructions_initially_visible: Option<bool>,
}

/// The NavigationHelpButton widget for displaying instructions for navigating the globe.
pub struct NavigationHelpButton {
    container: Option<Rc<RefCell<NavigationHelpButtonContainer>>>,
    wrapper: Rc<RefCell<NavigationHelpElement>>,
    view_model: NavigationHelpButtonViewModel,
    is_destroyed: bool,
}

impl NavigationHelpButton {
    /// Creates a new `NavigationHelpButton`.
    pub fn new(options: Option<NavigationHelpButtonOptions>) -> Result<Self, DeveloperError> {
        let opts = options.ok_or_else(|| DeveloperError::new("options.container is required."))?;
        let container = opts
            .container
            .ok_or_else(|| DeveloperError::new("options.container is required."))?;

        let mut view_model = NavigationHelpButtonViewModel::new();
        let show_instructions_default = opts.instructions_initially_visible.unwrap_or(false);
        view_model.set_show_instructions(show_instructions_default);

        let wrapper = NavigationHelpElement::new("span");
        wrapper.borrow_mut().class_name = "cesium-navigationHelpButton-wrapper".to_string();

        let button = NavigationHelpElement::new("button");
        button.borrow_mut().class_name =
            "cesium-button cesium-toolbar-button cesium-navigation-help-button".to_string();
        wrapper.borrow_mut().children.push(button);

        let instruction_container = NavigationHelpElement::new("div");
        instruction_container.borrow_mut().class_name = "cesium-navigation-help".to_string();
        wrapper.borrow_mut().children.push(instruction_container);

        container.borrow_mut().children.push(Rc::clone(&wrapper));

        Ok(Self {
            container: Some(container),
            wrapper,
            view_model,
            is_destroyed: false,
        })
    }

    /// Creates a `NavigationHelpButton` from container ID.
    pub fn from_id(
        container_id: &str,
        options: Option<NavigationHelpButtonOptions>,
        lookup: impl Fn(&str) -> Option<Rc<RefCell<NavigationHelpButtonContainer>>>,
    ) -> Result<Self, DeveloperError> {
        if container_id.is_empty() {
            return Err(DeveloperError::new("options.container is required."));
        }
        let container = lookup(container_id).ok_or_else(|| {
            DeveloperError::new(format!(
                "Element with id \"{container_id}\" does not exist in the document."
            ))
        })?;
        let mut opts = options.unwrap_or_default();
        opts.container = Some(container);
        Self::new(Some(opts))
    }

    /// Gets the container associated with this widget.
    pub fn container(&self) -> Option<Rc<RefCell<NavigationHelpButtonContainer>>> {
        self.container.clone()
    }

    /// Gets the view model.
    pub fn view_model(&self) -> &NavigationHelpButtonViewModel {
        &self.view_model
    }

    /// Gets the mutable view model.
    pub fn view_model_mut(&mut self) -> &mut NavigationHelpButtonViewModel {
        &mut self.view_model
    }

    /// Handles pointer/mouse input event outside wrapper to auto-close instructions.
    pub fn close_on_input(&mut self, target_node: &Rc<RefCell<NavigationHelpElement>>) {
        if !NavigationHelpElement::contains(&self.wrapper, target_node) {
            self.view_model.set_show_instructions(false);
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
    }
}
