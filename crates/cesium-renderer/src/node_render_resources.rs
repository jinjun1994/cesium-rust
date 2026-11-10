//! NodeRenderResources matching CesiumJS `Scene/Model/NodeRenderResources.js` (Apache-2.0).

use std::collections::HashMap;
use cesium_math::DeveloperError;

use crate::alpha_pipeline_stage::ModelAlphaOptions;
use crate::model_render_resources::{ModelRenderResources, ModelUniformCallback};
use crate::model_runtime_node::ModelRuntimeNode;
use crate::render_state::RenderState;
use crate::shader_builder::ShaderBuilder;
use crate::vertex_array::VertexAttribute;

/// Render resources associated with a single scene graph node.
pub struct NodeRenderResources<'a, M> {
    /// A reference to the model. Inherited from the model render resources.
    pub model: &'a M,

    /// An object used to build a shader incrementally. Cloned from the model render resources.
    pub shader_builder: ShaderBuilder,

    /// A dictionary mapping uniform name to functions that return uniform values.
    pub uniform_map: HashMap<String, ModelUniformCallback>,

    /// Options for configuring the alpha stage.
    pub alpha_options: ModelAlphaOptions,

    /// Render state options for creating a RenderState. Cloned from model render resources.
    pub render_state_options: RenderState,

    /// Whether the model has a silhouette.
    pub has_silhouette: bool,

    /// Whether the model is part of a tileset that uses skipLevelOfDetail.
    pub has_skip_level_of_detail: bool,

    /// A reference to the runtime node.
    pub runtime_node: &'a ModelRuntimeNode,

    /// Vertex attributes accumulated at the node level.
    pub attributes: Vec<VertexAttribute>,

    /// The index to give to the next vertex attribute added to the attributes array. POSITION is 0.
    pub attribute_index: u32,

    /// The set index to assign to feature ID vertex attributes created from offset/repeat.
    pub feature_id_vertex_attribute_set_index: u32,

    /// The number of instances.
    pub instance_count: usize,
}

impl<'a, M: std::fmt::Debug> std::fmt::Debug for NodeRenderResources<'a, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeRenderResources")
            .field("model", &self.model)
            .field("render_state_options", &self.render_state_options)
            .field("alpha_options", &self.alpha_options)
            .field("has_silhouette", &self.has_silhouette)
            .field("has_skip_level_of_detail", &self.has_skip_level_of_detail)
            .field("attributes", &self.attributes)
            .field("attribute_index", &self.attribute_index)
            .field("instance_count", &self.instance_count)
            .finish()
    }
}

impl<'a, M> NodeRenderResources<'a, M> {
    /// Attempts to construct a new `NodeRenderResources` checking for required parameters.
    pub fn try_new(
        model_render_resources: Option<&'a ModelRenderResources<M>>,
        runtime_node: Option<&'a ModelRuntimeNode>,
    ) -> Result<Self, DeveloperError> {
        let model_resources = model_render_resources
            .ok_or_else(|| DeveloperError::new("modelRenderResources is required."))?;
        let runtime_node =
            runtime_node.ok_or_else(|| DeveloperError::new("runtimeNode is required."))?;

        Ok(Self {
            model: &model_resources.model,
            shader_builder: model_resources.shader_builder.clone(),
            uniform_map: HashMap::new(),
            alpha_options: model_resources.alpha_options.clone(),
            render_state_options: model_resources.render_state_options.clone(),
            has_silhouette: model_resources.has_silhouette,
            has_skip_level_of_detail: model_resources.has_skip_level_of_detail,
            runtime_node,
            attributes: Vec::new(),
            attribute_index: 1,
            feature_id_vertex_attribute_set_index: 0,
            instance_count: 0,
        })
    }

    /// Constructs a new `NodeRenderResources` inheriting from the given model resources.
    pub fn new(
        model_render_resources: &'a ModelRenderResources<M>,
        runtime_node: &'a ModelRuntimeNode,
    ) -> Self {
        Self::try_new(Some(model_render_resources), Some(runtime_node))
            .expect("Valid modelRenderResources and runtimeNode are required")
    }
}
