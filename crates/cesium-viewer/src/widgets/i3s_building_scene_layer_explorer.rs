//! I3SBuildingSceneLayerExplorer matching CesiumJS `Widgets/I3SBuildingSceneLayerExplorer/I3SBuildingSceneLayerExplorer.js` (Apache-2.0).

use crate::widgets::i3s_building_scene_layer_explorer_view_model::{
    I3SBuildingSceneLayerExplorerViewModel, I3SDataProviderSource,
};
use cesium_math::DeveloperError;
use cesium_renderer::i3s_data_provider::I3SSublayer;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// A node in the virtual DOM representation of the I3S Building Scene Layer explorer widget.
#[derive(Clone, Debug)]
pub struct I3SExplorerNode {
    pub tag: String,
    pub id: Option<String>,
    pub class_name: String,
    pub text_content: String,
    pub children: Vec<Rc<RefCell<I3SExplorerNode>>>,
    pub parent: Option<Weak<RefCell<I3SExplorerNode>>>,
}

impl I3SExplorerNode {
    /// Creates a new virtual DOM node.
    pub fn new(tag: impl Into<String>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            tag: tag.into(),
            id: None,
            class_name: String::new(),
            text_content: String::new(),
            children: Vec::new(),
            parent: None,
        }))
    }

    /// Appends a child to this node, updating child's parent link.
    pub fn append_child(parent_rc: &Rc<RefCell<Self>>, child_rc: &Rc<RefCell<Self>>) {
        child_rc.borrow_mut().parent = Some(Rc::downgrade(parent_rc));
        parent_rc.borrow_mut().children.push(Rc::clone(child_rc));
    }

    /// Gets the tag / local name of this element in lowercase.
    pub fn local_name(&self) -> &str {
        &self.tag
    }

    /// Gets the class name of this element.
    pub fn class_name(&self) -> &str {
        &self.class_name
    }

    /// Gets the text content of this element.
    pub fn text_content(&self) -> &str {
        &self.text_content
    }

    /// Sets the text content.
    pub fn set_text_content(&mut self, text: impl Into<String>) {
        self.text_content = text.into();
    }

    /// Gets the element id, if any.
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// Gets child element count.
    pub fn child_element_count(&self) -> usize {
        self.children.len()
    }

    /// Gets parent element if linked.
    pub fn parent_element(&self) -> Option<Rc<RefCell<I3SExplorerNode>>> {
        self.parent.as_ref().and_then(Weak::upgrade)
    }

    /// Recursively queries for selector (.class or #id or tag).
    pub fn query_selector(&self, selector: &str) -> Option<Rc<RefCell<I3SExplorerNode>>> {
        for child in &self.children {
            if matches_selector(&child.borrow(), selector) {
                return Some(Rc::clone(child));
            }
            if let Some(found) = child.borrow().query_selector(selector) {
                return Some(found);
            }
        }
        None
    }
}

fn matches_selector(node: &I3SExplorerNode, selector: &str) -> bool {
    if let Some(class_target) = selector.strip_prefix('.') {
        node.class_name
            .split_whitespace()
            .any(|c| c == class_target)
    } else if let Some(id_target) = selector.strip_prefix('#') {
        node.id.as_deref() == Some(id_target)
    } else {
        node.tag.eq_ignore_ascii_case(selector)
    }
}

/// Simulated container element for the I3S Building Scene Layer explorer widget.
#[derive(Clone, Debug, Default)]
pub struct I3SBuildingSceneLayerExplorerContainer {
    pub id: String,
    pub children: Vec<Rc<RefCell<I3SExplorerNode>>>,
}

impl I3SBuildingSceneLayerExplorerContainer {
    /// Creates a new container with the given DOM id.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            children: Vec::new(),
        }
    }

    /// Gets child element count.
    pub fn child_element_count(&self) -> usize {
        self.children.len()
    }

    /// Recursively queries the container children for a selector.
    pub fn query_selector(&self, selector: &str) -> Option<Rc<RefCell<I3SExplorerNode>>> {
        for child in &self.children {
            if matches_selector(&child.borrow(), selector) {
                return Some(Rc::clone(child));
            }
            if let Some(found) = child.borrow().query_selector(selector) {
                return Some(found);
            }
        }
        None
    }
}

/// The I3S Building Scene Layer Explorer widget.
pub struct I3SBuildingSceneLayerExplorer {
    container: Option<Rc<RefCell<I3SBuildingSceneLayerExplorerContainer>>>,
    view_model: I3SBuildingSceneLayerExplorerViewModel,
    is_destroyed: bool,
}

impl I3SBuildingSceneLayerExplorer {
    /// Creates a new `I3SBuildingSceneLayerExplorer`.
    pub fn new(
        container: Option<Rc<RefCell<I3SBuildingSceneLayerExplorerContainer>>>,
        provider: Option<Rc<RefCell<dyn I3SDataProviderSource>>>,
    ) -> Result<Self, DeveloperError> {
        let container = container.ok_or_else(|| DeveloperError::new("container is required."))?;
        let provider = provider.ok_or_else(|| DeveloperError::new("i3sProvider is required."))?;

        let mut view_model = I3SBuildingSceneLayerExplorerViewModel::new(provider)?;

        // Build simulated DOM hierarchy
        let html_wrapper = I3SExplorerNode::new("div");
        html_wrapper.borrow_mut().class_name = "cesium-viewer-i3s-explorer".to_string();

        let h3_title = I3SExplorerNode::new("h3");
        h3_title.borrow_mut().text_content = "Building explorer".to_string();
        I3SExplorerNode::append_child(&html_wrapper, &h3_title);

        let select_top = I3SExplorerNode::new("select");
        let top_layer_title = if view_model.top_layers().is_empty()
            || view_model.top_layers()[0].name == "Building layers not found"
        {
            "Building layers not found".to_string()
        } else {
            view_model.top_layers()[0].name.clone()
        };
        select_top.borrow_mut().text_content = top_layer_title;
        I3SExplorerNode::append_child(&html_wrapper, &select_top);

        let bsl_wrapper = I3SExplorerNode::new("div");
        bsl_wrapper.borrow_mut().id = Some("bsl-wrapper".to_string());

        let h3_level = I3SExplorerNode::new("h3");
        h3_level.borrow_mut().text_content = "Select Level".to_string();
        I3SExplorerNode::append_child(&bsl_wrapper, &h3_level);

        let select_level = I3SExplorerNode::new("select");
        I3SExplorerNode::append_child(&bsl_wrapper, &select_level);

        let h3_categories = I3SExplorerNode::new("h3");
        h3_categories.borrow_mut().text_content = "Disciplines & Categories".to_string();
        I3SExplorerNode::append_child(&bsl_wrapper, &h3_categories);

        // Build tree from sublayers
        let ul_root = I3SExplorerNode::new("ul");
        ul_root.borrow_mut().class_name = "layersList".to_string();

        for top_sub in view_model.sublayers() {
            let ul_sub = I3SExplorerNode::new("ul");
            ul_sub.borrow_mut().class_name = "layersList".to_string();
            let mut sorted_sublayers = top_sub.sublayers.clone();
            sorted_sublayers.sort_by(|a, b| a.name.cmp(&b.name));
            for category in &sorted_sublayers {
                build_category_tree(&ul_sub, category);
            }
            I3SExplorerNode::append_child(&ul_root, &ul_sub);
        }

        I3SExplorerNode::append_child(&bsl_wrapper, &ul_root);
        I3SExplorerNode::append_child(&html_wrapper, &bsl_wrapper);

        container.borrow_mut().children.push(html_wrapper);

        if let Some(def) = view_model.default_layer().cloned() {
            view_model.set_current_layer(def);
        }

        Ok(Self {
            container: Some(container),
            view_model,
            is_destroyed: false,
        })
    }

    /// Gets the container associated with this widget.
    pub fn container(&self) -> Option<&Rc<RefCell<I3SBuildingSceneLayerExplorerContainer>>> {
        self.container.as_ref()
    }

    /// Gets the view model.
    pub fn view_model(&self) -> &I3SBuildingSceneLayerExplorerViewModel {
        &self.view_model
    }

    /// Gets the mutable view model.
    pub fn view_model_mut(&mut self) -> &mut I3SBuildingSceneLayerExplorerViewModel {
        &mut self.view_model
    }

    /// Simulates a click on an expander node (`.expandItem`), toggling its `+`/`-` text and child nested list class.
    pub fn fire_click(expander: &Rc<RefCell<I3SExplorerNode>>) {
        let new_text = {
            let exp = expander.borrow();
            if exp.text_content == "+" { "-" } else { "+" }.to_string()
        };
        expander.borrow_mut().text_content = new_text;

        let grandparent = {
            let parent = expander.borrow().parent_element();
            parent.and_then(|p| p.borrow().parent_element())
        };

        if let Some(gp) = grandparent {
            let children = gp.borrow().children.clone();
            for child in children {
                let mut b = child.borrow_mut();
                if b.tag == "ul" {
                    if b.class_name.contains("active") {
                        b.class_name = "nested".to_string();
                    } else {
                        b.class_name = "nested active".to_string();
                    }
                }
            }
        }
    }

    /// Returns true if this object was destroyed.
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

fn build_category_tree(parent_node: &Rc<RefCell<I3SExplorerNode>>, sublayer: &I3SSublayer) {
    let li = I3SExplorerNode::new("li");
    let li_wrapper = I3SExplorerNode::new("div");
    li_wrapper.borrow_mut().class_name = "li-wrapper".to_string();

    let expand_item = I3SExplorerNode::new("span");
    expand_item.borrow_mut().class_name = "expandItem".to_string();
    expand_item.borrow_mut().text_content = "+".to_string();
    I3SExplorerNode::append_child(&li_wrapper, &expand_item);

    let checkbox = I3SExplorerNode::new("input");
    I3SExplorerNode::append_child(&li_wrapper, &checkbox);

    let label = I3SExplorerNode::new("label");
    let label_span = I3SExplorerNode::new("span");
    label_span.borrow_mut().text_content = sublayer.name.clone();
    I3SExplorerNode::append_child(&label, &label_span);
    I3SExplorerNode::append_child(&li_wrapper, &label);

    I3SExplorerNode::append_child(&li, &li_wrapper);

    let nested_ul = I3SExplorerNode::new("ul");
    nested_ul.borrow_mut().class_name = "nested".to_string();
    nested_ul.borrow_mut().id = Some(format!("{}-expander", sublayer.name));
    let mut sorted_children = sublayer.sublayers.clone();
    sorted_children.sort_by(|a, b| a.name.cmp(&b.name));
    for child_sub in &sorted_children {
        let child_li = I3SExplorerNode::new("li");
        let child_li_wrapper = I3SExplorerNode::new("div");
        child_li_wrapper.borrow_mut().class_name = "li-wrapper".to_string();

        let child_checkbox = I3SExplorerNode::new("input");
        I3SExplorerNode::append_child(&child_li_wrapper, &child_checkbox);

        let child_label = I3SExplorerNode::new("label");
        let child_span = I3SExplorerNode::new("span");
        child_span.borrow_mut().text_content = child_sub.name.clone();
        I3SExplorerNode::append_child(&child_label, &child_span);
        I3SExplorerNode::append_child(&child_li_wrapper, &child_label);

        I3SExplorerNode::append_child(&child_li, &child_li_wrapper);
        I3SExplorerNode::append_child(&nested_ul, &child_li);
    }
    I3SExplorerNode::append_child(&li, &nested_ul);

    I3SExplorerNode::append_child(parent_node, &li);
}
