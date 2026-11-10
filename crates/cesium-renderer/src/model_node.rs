//! ModelNode matching CesiumJS `Scene/Model/ModelNode.js` (Apache-2.0).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use cesium_math::{DeveloperError, Matrix4};

use crate::model_runtime_node::ModelRuntimeNode;

/// A public-facing node of a 3D model, providing access to properties such as node transform and visibility.
#[derive(Clone, Debug)]
pub struct ModelNode {
    runtime_node: Rc<RefCell<ModelRuntimeNode>>,
    model_user_animation_dirty: Option<Rc<Cell<bool>>>,
}

impl ModelNode {
    /// Creates a new `ModelNode` referencing a runtime node.
    pub fn new(
        has_model: bool,
        runtime_node: Option<Rc<RefCell<ModelRuntimeNode>>>,
        model_user_animation_dirty: Option<Rc<Cell<bool>>>,
    ) -> Result<Self, DeveloperError> {
        if !has_model {
            return Err(DeveloperError::new("model is required"));
        }
        let runtime_node = runtime_node.ok_or_else(|| DeveloperError::new("runtimeNode is required"))?;
        Ok(Self {
            runtime_node,
            model_user_animation_dirty,
        })
    }

    /// Gets the name of the node.
    pub fn name(&self) -> String {
        self.runtime_node.borrow().name.clone()
    }

    /// Gets the integer ID / index of the node.
    pub fn id(&self) -> usize {
        self.runtime_node.borrow().id
    }

    /// Gets whether this node is visible.
    pub fn show(&self) -> bool {
        self.runtime_node.borrow().show
    }

    /// Sets whether this node is visible.
    pub fn set_show(&mut self, show: bool) {
        self.runtime_node.borrow_mut().show = show;
    }

    /// Gets the current transform matrix of the node.
    pub fn matrix(&self) -> Matrix4 {
        self.runtime_node.borrow().transform
    }

    /// Gets the original transform matrix of the node from the glTF asset.
    pub fn original_matrix(&self) -> Matrix4 {
        self.runtime_node.borrow().original_transform
    }

    /// Sets the node transform. If `None` is provided, the node transform is reset to `original_matrix`.
    pub fn set_matrix(&mut self, matrix: Option<Matrix4>) {
        let mut node = self.runtime_node.borrow_mut();
        match matrix {
            Some(m) => {
                node.transform = m;
                node.user_animated = true;
                node.transform_dirty = true;
                if let Some(flag) = &self.model_user_animation_dirty {
                    flag.set(true);
                }
            }
            None => {
                node.transform = node.original_transform;
                node.user_animated = false;
                node.transform_dirty = true;
            }
        }
    }
}
