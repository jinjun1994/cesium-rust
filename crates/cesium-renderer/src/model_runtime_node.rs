//! ModelRuntimeNode and ModelRuntimePrimitive matching CesiumJS `Scene/Model/ModelRuntimeNode.js` and `ModelRuntimePrimitive.js` (Apache-2.0).

use cesium_math::{BoundingSphere, Cartesian3, DeveloperError, Matrix4, Quaternion};

use crate::model_components::Node;
use crate::model_draw_command::ModelDrawCommand;

/// Pipeline stage enum for runtime nodes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NodePipelineStage {
    Instancing,
    NodeStatistics,
}

impl NodePipelineStage {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Instancing => "InstancingPipelineStage",
            Self::NodeStatistics => "NodeStatisticsPipelineStage",
        }
    }
}

/// Update stage enum for runtime nodes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NodeUpdateStage {
    ModelMatrixUpdate,
}

impl NodeUpdateStage {
    pub fn name(&self) -> &'static str {
        match self {
            Self::ModelMatrixUpdate => "ModelMatrixUpdateStage",
        }
    }
}

/// Runtime primitive belonging to a `ModelRuntimeNode`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelRuntimePrimitive {
    pub draw_command: Option<ModelDrawCommand>,
    pub bounding_sphere: Option<BoundingSphere>,
    pub bounding_sphere_2d: Option<BoundingSphere>,
}

impl ModelRuntimePrimitive {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_command(
        draw_command: ModelDrawCommand,
        bounding_sphere: Option<BoundingSphere>,
    ) -> Self {
        Self {
            draw_command: Some(draw_command),
            bounding_sphere,
            bounding_sphere_2d: None,
        }
    }
}

/// Options to instantiate a [`ModelRuntimeNode`].
#[derive(Clone, Debug)]
pub struct ModelRuntimeNodeOptions {
    pub name: String,
    pub id: usize,
    pub transform: Matrix4,
    pub transform_to_root: Matrix4,
    pub children: Vec<usize>,
    pub show: Option<bool>,
}

impl Default for ModelRuntimeNodeOptions {
    fn default() -> Self {
        Self {
            name: String::new(),
            id: 0,
            transform: Matrix4::IDENTITY,
            transform_to_root: Matrix4::IDENTITY,
            children: Vec::new(),
            show: None,
        }
    }
}

/// Builder arguments for constructing a `ModelRuntimeNode` matching CesiumJS strict validation.
#[derive(Clone, Debug, Default)]
pub struct ModelRuntimeNodeArgs {
    pub node: Option<Node>,
    pub transform: Option<Matrix4>,
    pub transform_to_root: Option<Matrix4>,
    pub children: Option<Vec<usize>>,
    pub scene_graph: Option<()>,
}

/// An in-memory representation of a node in a model scene graph.
#[derive(Clone, Debug)]
pub struct ModelRuntimeNode {
    pub name: String,
    pub id: usize,
    pub node: Option<Node>,
    pub has_transform_parameters: bool,
    pub translation: Cartesian3,
    pub rotation: Quaternion,
    pub scale: Cartesian3,
    pub morph_weights: Vec<f64>,
    pub original_transform: Matrix4,
    pub transform: Matrix4,
    pub transform_to_root: Matrix4,
    pub computed_transform: Matrix4,
    pub transform_dirty: bool,
    pub show: bool,
    pub user_animated: bool,
    pub children: Vec<usize>,
    pub runtime_primitives: Vec<ModelRuntimePrimitive>,
    pub pipeline_stages: Vec<NodePipelineStage>,
    pub update_stages: Vec<NodeUpdateStage>,
    pub is_instanced: bool,
}

impl PartialEq for ModelRuntimeNode {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.id == other.id
            && self.has_transform_parameters == other.has_transform_parameters
            && self.translation == other.translation
            && self.rotation == other.rotation
            && self.scale == other.scale
            && self.morph_weights == other.morph_weights
            && self.original_transform == other.original_transform
            && self.transform == other.transform
            && self.transform_to_root == other.transform_to_root
            && self.computed_transform == other.computed_transform
            && self.transform_dirty == other.transform_dirty
            && self.show == other.show
            && self.user_animated == other.user_animated
            && self.children == other.children
            && self.runtime_primitives == other.runtime_primitives
            && self.pipeline_stages == other.pipeline_stages
            && self.update_stages == other.update_stages
            && self.is_instanced == other.is_instanced
    }
}

impl Default for ModelRuntimeNode {
    fn default() -> Self {
        Self {
            name: String::new(),
            id: 0,
            node: None,
            has_transform_parameters: true,
            translation: Cartesian3::ZERO,
            rotation: Quaternion::IDENTITY,
            scale: Cartesian3::new(1.0, 1.0, 1.0),
            morph_weights: Vec::new(),
            original_transform: Matrix4::IDENTITY,
            transform: Matrix4::IDENTITY,
            transform_to_root: Matrix4::IDENTITY,
            computed_transform: Matrix4::IDENTITY,
            transform_dirty: false,
            show: true,
            user_animated: false,
            children: Vec::new(),
            runtime_primitives: Vec::new(),
            pipeline_stages: Vec::new(),
            update_stages: Vec::new(),
            is_instanced: false,
        }
    }
}

impl ModelRuntimeNode {
    pub fn new() -> Self {
        Self::default()
    }

    /// Strict constructor checking that all required arguments are present.
    pub fn try_new(args: ModelRuntimeNodeArgs) -> Result<Self, DeveloperError> {
        let node = args
            .node
            .ok_or_else(|| DeveloperError::new("options.node is required."))?;
        let transform = args
            .transform
            .ok_or_else(|| DeveloperError::new("options.transform is required."))?;
        let transform_to_root = args
            .transform_to_root
            .ok_or_else(|| DeveloperError::new("options.transformToRoot is required."))?;
        args.scene_graph
            .ok_or_else(|| DeveloperError::new("options.sceneGraph is required."))?;
        let children = args
            .children
            .ok_or_else(|| DeveloperError::new("options.children is required."))?;

        let mut runtime_node = Self::from_node(node, transform, transform_to_root, children);
        runtime_node.configure_pipeline();
        Ok(runtime_node)
    }

    /// Constructs a `ModelRuntimeNode` from a `Node` component and transforms.
    pub fn from_node(
        node: Node,
        transform: Matrix4,
        transform_to_root: Matrix4,
        children: Vec<usize>,
    ) -> Self {
        let name = node.name.clone().unwrap_or_default();
        let id = node.index.unwrap_or(0);
        let has_matrix = node.matrix.is_some();
        let has_transform_parameters = !has_matrix;

        let translation = node.translation.unwrap_or(Cartesian3::ZERO);
        let rotation = node.rotation.unwrap_or(Quaternion::IDENTITY);
        let scale = node.scale.unwrap_or_else(|| Cartesian3::new(1.0, 1.0, 1.0));
        let morph_weights = node.morph_weights.clone();

        // KHR_node_visibility logic: show = false only when visible is explicitly false
        let mut show = true;
        if let Some(ref vis) = node.node_visibility {
            if vis.visible == Some(false) {
                show = false;
            }
        }

        let computed_transform = transform_to_root.multiply_transformation(&transform);
        let is_instanced = node.instances.is_some();

        Self {
            name,
            id,
            node: Some(node),
            has_transform_parameters,
            translation,
            rotation,
            scale,
            morph_weights,
            original_transform: transform,
            transform,
            transform_to_root,
            computed_transform,
            transform_dirty: false,
            show,
            user_animated: false,
            children,
            runtime_primitives: Vec::new(),
            pipeline_stages: Vec::new(),
            update_stages: Vec::new(),
            is_instanced,
        }
    }

    pub fn from_options(options: ModelRuntimeNodeOptions) -> Self {
        let original_transform = options.transform;
        let transform = options.transform;
        let transform_to_root = options.transform_to_root;
        let computed_transform = transform_to_root.multiply_transformation(&transform);

        Self {
            name: options.name,
            id: options.id,
            node: None,
            has_transform_parameters: true,
            translation: Cartesian3::ZERO,
            rotation: Quaternion::IDENTITY,
            scale: Cartesian3::new(1.0, 1.0, 1.0),
            morph_weights: Vec::new(),
            original_transform,
            transform,
            transform_to_root,
            computed_transform,
            transform_dirty: false,
            show: options.show.unwrap_or(true),
            user_animated: false,
            children: options.children,
            runtime_primitives: Vec::new(),
            pipeline_stages: Vec::new(),
            update_stages: Vec::new(),
            is_instanced: false,
        }
    }

    pub fn with_transform(transform: Matrix4) -> Self {
        Self {
            original_transform: transform,
            transform,
            computed_transform: transform,
            ..Default::default()
        }
    }

    pub fn computed_transform(&self) -> Matrix4 {
        Matrix4::from_translation_quaternion_rotation_scale(
            self.translation,
            self.rotation,
            self.scale,
        )
    }

    /// Sets the node transform and marks it dirty without replacing `original_transform`.
    pub fn set_transform(&mut self, transform: Matrix4) {
        self.transform = transform;
        self.transform_dirty = true;
    }

    /// Recalculates computed transform using internal `transform_to_root * transform`.
    pub fn update_computed_transform(&mut self) -> Matrix4 {
        self.computed_transform = self.transform_to_root.multiply_transformation(&self.transform);
        self.transform_dirty = false;
        self.computed_transform
    }

    /// Recalculates `computed_transform = root_matrix * transform_to_root * transform`.
    pub fn update_computed_transform_with_root(&mut self, root_matrix: &Matrix4) {
        let combined = root_matrix.multiply(&self.transform_to_root);
        self.computed_transform = combined.multiply(&self.transform);
        self.transform_dirty = false;
    }

    /// Configures the node pipeline and update stages.
    pub fn configure_pipeline(&mut self) {
        self.pipeline_stages.clear();
        self.update_stages.clear();

        if self.is_instanced {
            self.pipeline_stages.push(NodePipelineStage::Instancing);
        }

        self.pipeline_stages.push(NodePipelineStage::NodeStatistics);
        self.update_stages.push(NodeUpdateStage::ModelMatrixUpdate);
    }

    /// Returns the local space translation if defined via parameters.
    pub fn get_translation(&self) -> Option<Cartesian3> {
        if self.has_transform_parameters {
            Some(self.translation)
        } else {
            None
        }
    }

    /// Sets the local translation and updates the local transform matrix.
    pub fn set_translation(&mut self, translation: Cartesian3) -> Result<(), DeveloperError> {
        if !self.has_transform_parameters {
            return Err(DeveloperError::new(
                "The translation of a node cannot be set if it was defined using a matrix in the model.",
            ));
        }
        if self.translation == translation {
            return Ok(());
        }
        self.translation = translation;
        self.update_transform_from_parameters();
        Ok(())
    }

    /// Returns the local space rotation if defined via parameters.
    pub fn get_rotation(&self) -> Option<Quaternion> {
        if self.has_transform_parameters {
            Some(self.rotation)
        } else {
            None
        }
    }

    /// Sets the local rotation and updates the local transform matrix.
    pub fn set_rotation(&mut self, rotation: Quaternion) -> Result<(), DeveloperError> {
        if !self.has_transform_parameters {
            return Err(DeveloperError::new(
                "The rotation of a node cannot be set if it was defined using a matrix in the model.",
            ));
        }
        if self.rotation == rotation {
            return Ok(());
        }
        self.rotation = rotation;
        self.update_transform_from_parameters();
        Ok(())
    }

    /// Returns the local space scale if defined via parameters.
    pub fn get_scale(&self) -> Option<Cartesian3> {
        if self.has_transform_parameters {
            Some(self.scale)
        } else {
            None
        }
    }

    /// Sets the local scale and updates the local transform matrix.
    pub fn set_scale(&mut self, scale: Cartesian3) -> Result<(), DeveloperError> {
        if !self.has_transform_parameters {
            return Err(DeveloperError::new(
                "The scale of a node cannot be set if it was defined using a matrix in the model.",
            ));
        }
        if self.scale == scale {
            return Ok(());
        }
        self.scale = scale;
        self.update_transform_from_parameters();
        Ok(())
    }

    /// Updates the local transform matrix from TRS parameters.
    fn update_transform_from_parameters(&mut self) {
        self.transform_dirty = true;
        self.transform = Matrix4::from_translation_quaternion_rotation_scale(
            self.translation,
            self.rotation,
            self.scale,
        );
    }

    /// Sets morph weights, verifying that the length matches the original weights array.
    pub fn set_morph_weights(&mut self, weights: &[f64]) -> Result<(), DeveloperError> {
        if self.morph_weights.len() != weights.len() {
            return Err(DeveloperError::new(
                "value must have the same length as the original weights array.",
            ));
        }
        self.morph_weights.copy_from_slice(weights);
        Ok(())
    }

    /// Returns the child index at `index`.
    pub fn get_child(&self, index: usize) -> Result<usize, DeveloperError> {
        if index >= self.children.len() {
            return Err(DeveloperError::new(
                "index must be greater than or equal to 0 and less than the number of children.",
            ));
        }
        Ok(self.children[index])
    }
}
