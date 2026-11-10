//! PrimitiveRenderResources matching CesiumJS `Scene/Model/PrimitiveRenderResources.js` (Apache-2.0).

use std::collections::HashMap;
use cesium_math::{BoundingSphere, Cartesian3, DeveloperError, PrimitiveType};

use crate::alpha_pipeline_stage::ModelAlphaOptions;
use crate::buffer::Buffer;
use crate::classification_pipeline_stage::VertexAttributeSemantic;
use crate::lighting_pipeline_stage::ModelLightingOptions;
use crate::model_components::Indices;
use crate::model_render_resources::ModelUniformCallback;
use crate::model_runtime_node::ModelRuntimeNode;
use crate::model_runtime_primitive::ModelRuntimePrimitive;
use crate::node_render_resources::NodeRenderResources;
use crate::render_state::RenderState;
use crate::shader_builder::ShaderBuilder;
use crate::vertex_array::VertexAttribute;

/// Render resources associated with a single primitive.
pub struct PrimitiveRenderResources<'a, M> {
    /// A reference to the parent model.
    pub model: &'a M,

    /// A reference to the runtime node.
    pub runtime_node: &'a ModelRuntimeNode,

    /// A reference to the runtime primitive.
    pub runtime_primitive: &'a ModelRuntimePrimitive,

    /// Vertex attributes accumulated at the primitive level.
    pub attributes: Vec<VertexAttribute>,

    /// The index to give to the next vertex attribute added.
    pub attribute_index: u32,

    /// The set index to assign to feature ID vertex attributes.
    pub feature_id_vertex_attribute_set_index: u32,

    /// A dictionary mapping uniform name to functions returning uniform values.
    pub uniform_map: HashMap<String, ModelUniformCallback>,

    /// Options for configuring the alpha stage.
    pub alpha_options: ModelAlphaOptions,

    /// Render state options for creating a RenderState.
    pub render_state_options: RenderState,

    /// Whether the model has a silhouette.
    pub has_silhouette: bool,

    /// Whether the model is part of a tileset that uses skipLevelOfDetail.
    pub has_skip_level_of_detail: bool,

    /// Whether this primitive writes feature IDs in a planar fill pass.
    pub planar_fill_id_pass: bool,

    /// Shader builder for this primitive.
    pub shader_builder: ShaderBuilder,

    /// The number of instances.
    pub instance_count: usize,

    /// The number of indices / vertices in the primitive.
    pub count: usize,

    /// Whether or not this primitive has a property table for storing metadata.
    pub has_property_table: bool,

    /// The indices for this primitive.
    pub indices: Option<Indices>,

    /// Additional index buffer for wireframe mode.
    pub wireframe_index_buffer: Option<Buffer>,

    /// The primitive type such as TRIANGLES or POINTS.
    pub primitive_type: Option<PrimitiveType>,

    /// The minimum position value for this primitive.
    pub position_min: Cartesian3,

    /// The maximum position value for this primitive.
    pub position_max: Cartesian3,

    /// The bounding sphere containing all vertices in this primitive.
    pub bounding_sphere: BoundingSphere,

    /// Options for configuring the lighting stage.
    pub lighting_options: ModelLightingOptions,

    /// The shader variable to use for picking.
    pub pick_id: Option<String>,

    /// The shader expression for snap payload.
    pub snap_id: Option<String>,
}

impl<'a, M: std::fmt::Debug> std::fmt::Debug for PrimitiveRenderResources<'a, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrimitiveRenderResources")
            .field("model", &self.model)
            .field("render_state_options", &self.render_state_options)
            .field("alpha_options", &self.alpha_options)
            .field("has_silhouette", &self.has_silhouette)
            .field("has_skip_level_of_detail", &self.has_skip_level_of_detail)
            .field("attributes", &self.attributes)
            .field("attribute_index", &self.attribute_index)
            .field("count", &self.count)
            .field("position_min", &self.position_min)
            .field("position_max", &self.position_max)
            .finish()
    }
}

impl<'a, M> PrimitiveRenderResources<'a, M> {
    /// Attempts to construct a new `PrimitiveRenderResources` checking for required parameters.
    pub fn try_new(
        node_render_resources: Option<&'a NodeRenderResources<'a, M>>,
        runtime_primitive: Option<&'a ModelRuntimePrimitive>,
    ) -> Result<Self, DeveloperError> {
        let node_resources = node_render_resources
            .ok_or_else(|| DeveloperError::new("nodeRenderResources is required."))?;
        let runtime_primitive =
            runtime_primitive.ok_or_else(|| DeveloperError::new("runtimePrimitive is required."))?;

        let primitive_fallback = crate::model_components::Primitive::default();
        let primitive = runtime_primitive
            .primitive
            .as_ref()
            .unwrap_or(&primitive_fallback);

        let position_attr = primitive
            .attributes
            .iter()
            .find(|a| a.semantic.as_deref() == Some(VertexAttributeSemantic::POSITION));

        let count = if let Some(ref indices) = primitive.indices {
            indices.count
        } else if let Some(pos) = position_attr {
            pos.count
        } else {
            0
        };

        let position_min = position_attr
            .and_then(|a| a.min.as_ref())
            .filter(|min| min.len() >= 3)
            .map(|min| Cartesian3::new(min[0], min[1], min[2]))
            .unwrap_or(Cartesian3::new(-1.0, -1.0, -1.0));

        let position_max = position_attr
            .and_then(|a| a.max.as_ref())
            .filter(|max| max.len() >= 3)
            .map(|max| Cartesian3::new(max[0], max[1], max[2]))
            .unwrap_or(Cartesian3::new(1.0, 1.0, 1.0));

        let bounding_sphere = BoundingSphere::from_corner_points(
            position_min.into(),
            position_max.into(),
        );

        Ok(Self {
            model: node_resources.model,
            runtime_node: node_resources.runtime_node,
            runtime_primitive,
            attributes: node_resources.attributes.clone(),
            attribute_index: node_resources.attribute_index,
            feature_id_vertex_attribute_set_index: node_resources.feature_id_vertex_attribute_set_index,
            uniform_map: HashMap::new(),
            alpha_options: node_resources.alpha_options.clone(),
            render_state_options: node_resources.render_state_options.clone(),
            has_silhouette: node_resources.has_silhouette,
            has_skip_level_of_detail: node_resources.has_skip_level_of_detail,
            planar_fill_id_pass: false,
            shader_builder: node_resources.shader_builder.clone(),
            instance_count: node_resources.instance_count,
            count,
            has_property_table: false,
            indices: primitive.indices.clone(),
            wireframe_index_buffer: None,
            primitive_type: primitive.primitive_type,
            position_min,
            position_max,
            bounding_sphere,
            lighting_options: ModelLightingOptions::default(),
            pick_id: None,
            snap_id: None,
        })
    }

    /// Constructs a new `PrimitiveRenderResources` inheriting from node render resources.
    pub fn new(
        node_render_resources: &'a NodeRenderResources<'a, M>,
        runtime_primitive: &'a ModelRuntimePrimitive,
    ) -> Self {
        Self::try_new(Some(node_render_resources), Some(runtime_primitive))
            .expect("Valid nodeRenderResources and runtimePrimitive are required")
    }
}
