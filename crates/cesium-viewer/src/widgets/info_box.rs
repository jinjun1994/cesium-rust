//! InfoBox widget matching CesiumJS `Widgets/InfoBox/InfoBox.js` (Apache-2.0).

use crate::widgets::info_box_view_model::InfoBoxViewModel;
use cesium_math::DeveloperError;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A virtual DOM node inside the InfoBox frame document.
#[derive(Clone, Debug, Default)]
pub struct InfoBoxFrameNode {
    pub tag: String,
    pub class_name: String,
    pub inner_html: String,
}

/// Simulated content document inside the InfoBox iframe.
#[derive(Clone, Debug, Default)]
pub struct InfoBoxContentDocument {
    pub body_nodes: Vec<Rc<RefCell<InfoBoxFrameNode>>>,
}

impl InfoBoxContentDocument {
    /// Gets the first child of document body.
    pub fn first_child(&self) -> Option<Rc<RefCell<InfoBoxFrameNode>>> {
        self.body_nodes.first().cloned()
    }
}

/// Simulated iframe element for InfoBox.
#[derive(Clone, Debug, Default)]
pub struct InfoBoxFrame {
    pub class_name: String,
    pub content_document: InfoBoxContentDocument,
}

impl InfoBoxFrame {
    /// Creates a new `InfoBoxFrame`.
    pub fn new() -> Self {
        let node = Rc::new(RefCell::new(InfoBoxFrameNode {
            tag: "div".to_string(),
            class_name: "cesium-infoBox-description".to_string(),
            inner_html: String::new(),
        }));
        Self {
            class_name: "cesium-infoBox-iframe".to_string(),
            content_document: InfoBoxContentDocument {
                body_nodes: vec![node],
            },
        }
    }
}

/// Simulated DOM element for the root InfoBox container child.
#[derive(Clone, Debug, Default)]
pub struct InfoBoxElement {
    pub class_name: String,
    pub style: HashMap<String, String>,
}

impl InfoBoxElement {
    /// Creates a new `InfoBoxElement`.
    pub fn new() -> Self {
        Self {
            class_name: "cesium-infoBox".to_string(),
            style: HashMap::new(),
        }
    }
}

/// Simulated container DOM element for InfoBox.
#[derive(Clone, Debug, Default)]
pub struct InfoBoxContainer {
    pub id: String,
    pub children: Vec<Rc<RefCell<InfoBoxElement>>>,
}

impl InfoBoxContainer {
    /// Creates a new container with an ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            children: Vec::new(),
        }
    }

    /// Gets the first child element of the container.
    pub fn first_child(&self) -> Option<Rc<RefCell<InfoBoxElement>>> {
        self.children.first().cloned()
    }
}

/// The InfoBox widget for displaying information or descriptions.
pub struct InfoBox {
    container: Option<Rc<RefCell<InfoBoxContainer>>>,
    element: Rc<RefCell<InfoBoxElement>>,
    frame: Rc<RefCell<InfoBoxFrame>>,
    view_model: InfoBoxViewModel,
    is_destroyed: bool,
}

impl InfoBox {
    /// Creates a new `InfoBox`.
    pub fn new(container: Option<Rc<RefCell<InfoBoxContainer>>>) -> Result<Self, DeveloperError> {
        let container = container.ok_or_else(|| DeveloperError::new("container is required."))?;

        let element = Rc::new(RefCell::new(InfoBoxElement::new()));
        let frame = Rc::new(RefCell::new(InfoBoxFrame::new()));
        let view_model = InfoBoxViewModel::new();

        container.borrow_mut().children.push(Rc::clone(&element));

        Ok(Self {
            container: Some(container),
            element,
            frame,
            view_model,
            is_destroyed: false,
        })
    }

    /// Creates an `InfoBox` from a container ID.
    pub fn from_id(
        container_id: &str,
        lookup: impl Fn(&str) -> Option<Rc<RefCell<InfoBoxContainer>>>,
    ) -> Result<Self, DeveloperError> {
        if container_id.is_empty() {
            return Err(DeveloperError::new("container is required."));
        }
        let container = lookup(container_id).ok_or_else(|| {
            DeveloperError::new(format!(
                "Element with id \"{container_id}\" does not exist in the document."
            ))
        })?;
        Self::new(Some(container))
    }

    /// Gets the container element.
    pub fn container(&self) -> Option<Rc<RefCell<InfoBoxContainer>>> {
        self.container.clone()
    }

    /// Gets the view model.
    pub fn view_model(&self) -> &InfoBoxViewModel {
        &self.view_model
    }

    /// Gets the mutable view model.
    pub fn view_model_mut(&mut self) -> &mut InfoBoxViewModel {
        &mut self.view_model
    }

    /// Sets the description and synchronizes the iframe content document and styling.
    pub fn set_description(&mut self, description: impl Into<String>) {
        let desc = description.into();
        self.view_model.description = desc.clone();

        // Update iframe body node
        if let Some(node) = self.frame.borrow().content_document.first_child() {
            node.borrow_mut().inner_html = desc.clone();
        }

        // Extract background-color from style attribute if single element
        let mut bg_color = String::new();
        if let Some(idx) = desc.find("background-color:") {
            let after = &desc[idx + "background-color:".len()..];
            let end_idx = after.find(';').or_else(|| after.find('"')).unwrap_or(after.len());
            bg_color = after[..end_idx].trim().to_string();
        }

        self.element
            .borrow_mut()
            .style
            .insert("background-color".to_string(), bg_color);
    }

    /// Gets the iframe frame element.
    pub fn frame(&self) -> Rc<RefCell<InfoBoxFrame>> {
        self.frame.clone()
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
