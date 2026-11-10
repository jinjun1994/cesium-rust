//! PickingPipelineStage matching CesiumJS `Scene/Model/PickingPipelineStage.js` (Apache-2.0).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use cesium_math::{Color, ComponentDatatype};

use crate::buffer::{Buffer, BufferOptions};
use crate::model_components::Primitive;
use crate::model_statistics::ModelStatistics;
use crate::model_type::ModelType;
use crate::shader_builder::{ShaderBuilder, ShaderDestination};
use crate::vertex_array::VertexAttribute;

static NEXT_PICK_KEY: AtomicU32 = AtomicU32::new(1);

/// A generated pick ID matching CesiumJS Context `PickId`.
#[derive(Clone, Debug, PartialEq)]
pub struct PickId {
    pub key: String,
    pub color: Color,
}

impl PickId {
    pub fn new_with_index(index: u32) -> Self {
        let r = ((index >> 24) & 0xFF) as f32 / 255.0;
        let g = ((index >> 16) & 0xFF) as f32 / 255.0;
        let b = ((index >> 8) & 0xFF) as f32 / 255.0;
        let a = (index & 0xFF) as f32 / 255.0;
        Self {
            key: format!("pick_{index}"),
            color: Color::new(r, g, b, a),
        }
    }
}

/// A registry mapping pick keys to objects matching CesiumJS Context `_pickObjects`.
#[derive(Default)]
pub struct PickContext {
    pub pick_objects: HashMap<String, Arc<dyn std::any::Any + Send + Sync>>,
}

impl PickContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_pick_id<T: 'static + Send + Sync>(&mut self, object: Arc<T>) -> PickId {
        let index = NEXT_PICK_KEY.fetch_add(1, Ordering::Relaxed);
        let pick_id = PickId::new_with_index(index);
        self.pick_objects.insert(pick_id.key.clone(), object);
        pick_id
    }
}

/// Detail description of picked model node and primitive.
#[derive(Clone, Debug, PartialEq)]
pub struct DetailPickObject {
    pub model_id: Option<String>,
    pub node_name: Option<String>,
    pub primitive_index: Option<usize>,
}

/// Model pick object description matching CesiumJS `buildPickObject`.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelPickObject {
    pub id: Option<String>,
    pub primitive_id: Option<String>,
    pub content_id: Option<String>,
    pub instance_id: Option<usize>,
    pub is_3d_tiles: bool,
    pub detail: DetailPickObject,
}

/// Builds the GLSL expression for the float snap payload written during a snapping pass.
pub fn snap_id_from_pick_id(pick_id: &str) -> String {
    format!("vec4(rgba8UnormToUint32({pick_id}), isEdge ? 1.0 : 0.0, -v_positionEC.z, 0.0)")
}

/// Options and resources passed to [`PickingPipelineStage`].
pub struct PickingRenderResources<'a> {
    pub attributes: &'a mut Vec<VertexAttribute>,
    pub attribute_index: &'a mut u32,
    pub pick_id: &'a mut Option<String>,
    pub snap_id: &'a mut Option<String>,
    pub shader_builder: &'a mut ShaderBuilder,
    pub statistics: &'a mut ModelStatistics,
    pub uniform_map: &'a mut HashMap<String, String>,
    pub model_type: ModelType,
    pub model_id: Option<String>,
    pub custom_pick_object: Option<ModelPickObject>,
    pub node_name: Option<String>,
    pub primitive_index: Option<usize>,
    pub has_property_table: bool,
    pub instance_count: usize,
    pub has_draped_vectors: bool,
    pub classification_type: Option<u32>,
    pub pick_texture_uniform: Option<String>,
    pub pipeline_resources_count: &'a mut usize,
    pub pick_ids: &'a mut Vec<PickId>,
}

/// The picking pipeline stage is responsible for handling picking of primitives.
pub struct PickingPipelineStage;

impl PickingPipelineStage {
    pub const NAME: &'static str = "PickingPipelineStage";

    /// Process a primitive for picking.
    pub fn process(
        resources: &mut PickingRenderResources<'_>,
        _primitive: &Primitive,
        context: &mut PickContext,
    ) {
        let is_instanced = resources.instance_count > 0;

        if resources.has_property_table {
            Self::process_pick_texture(resources);
        } else if is_instanced {
            Self::process_instanced_pick_ids(resources, context);
        } else {
            Self::process_uniform_pick_id(resources, context);
        }

        if resources.classification_type.is_none() && resources.has_draped_vectors {
            if let Some(ref current_pick_id) = resources.pick_id {
                *resources.pick_id = Some(format!("vectorPickColorOver({current_pick_id})"));
            }
        }
    }

    fn process_uniform_pick_id(
        resources: &mut PickingRenderResources<'_>,
        context: &mut PickContext,
    ) {
        let pick_object = if let Some(ref custom) = resources.custom_pick_object {
            custom.clone()
        } else {
            ModelPickObject {
                id: resources.model_id.clone(),
                primitive_id: Some("model".to_string()),
                content_id: if ModelType::is_3d_tiles(resources.model_type) {
                    Some("content".to_string())
                } else {
                    None
                },
                instance_id: None,
                is_3d_tiles: ModelType::is_3d_tiles(resources.model_type),
                detail: DetailPickObject {
                    model_id: resources.model_id.clone(),
                    node_name: resources.node_name.clone(),
                    primitive_index: resources.primitive_index,
                },
            }
        };

        let pick_id = context.create_pick_id(Arc::new(pick_object));
        *resources.pipeline_resources_count += 1;
        resources.pick_ids.push(pick_id.clone());

        resources.shader_builder.add_uniform(
            "vec4",
            "czm_pickColor",
            Some(ShaderDestination::FRAGMENT),
        );

        resources.uniform_map.insert(
            "czm_pickColor".to_string(),
            format!("{:?}", pick_id.color),
        );

        let pick_id_str = "czm_pickColor".to_string();
        *resources.snap_id = Some(snap_id_from_pick_id(&pick_id_str));
        *resources.pick_id = Some(pick_id_str);
    }

    fn process_instanced_pick_ids(
        resources: &mut PickingRenderResources<'_>,
        context: &mut PickContext,
    ) {
        let instance_count = resources.instance_count;
        let mut pick_ids_bytes = Vec::with_capacity(instance_count * 4);

        for i in 0..instance_count {
            let pick_object = ModelPickObject {
                id: resources.model_id.clone(),
                primitive_id: Some("model".to_string()),
                content_id: if ModelType::is_3d_tiles(resources.model_type) {
                    Some("content".to_string())
                } else {
                    None
                },
                instance_id: Some(i),
                is_3d_tiles: ModelType::is_3d_tiles(resources.model_type),
                detail: DetailPickObject {
                    model_id: resources.model_id.clone(),
                    node_name: resources.node_name.clone(),
                    primitive_index: resources.primitive_index,
                },
            };

            let pick_id = context.create_pick_id(Arc::new(pick_object));
            resources.pick_ids.push(pick_id.clone());

            pick_ids_bytes.push(Color::float_to_byte(pick_id.color.red));
            pick_ids_bytes.push(Color::float_to_byte(pick_id.color.green));
            pick_ids_bytes.push(Color::float_to_byte(pick_id.color.blue));
            pick_ids_bytes.push(Color::float_to_byte(pick_id.color.alpha));
        }

        *resources.pipeline_resources_count += instance_count + 1; // pick IDs + buffer

        let pick_ids_buffer = Buffer::create_vertex_buffer(BufferOptions {
            size_in_bytes: 0,
            typed_array: Some(pick_ids_bytes),
            ..Default::default()
        })
        .expect("Buffer creation");

        resources.statistics.add_buffer(&pick_ids_buffer, false);

        let current_index = *resources.attribute_index;
        *resources.attribute_index += 1;

        let pick_ids_vertex_attribute = VertexAttribute {
            index: current_index,
            enabled: true,
            vertex_buffer: Some(pick_ids_buffer),
            components_per_attribute: 4,
            component_datatype: ComponentDatatype::UnsignedByte,
            normalize: true,
            offset_in_bytes: 0,
            stride_in_bytes: 0,
            instance_divisor: 1,
        };

        resources.attributes.push(pick_ids_vertex_attribute);

        resources.shader_builder.add_define(
            "USE_PICKING",
            None,
            Some(ShaderDestination::BOTH),
        );
        resources.shader_builder.add_attribute("vec4", "a_pickColor");
        resources.shader_builder.add_varying("vec4", "v_pickColor", None);

        let pick_id_str = "v_pickColor".to_string();
        *resources.snap_id = Some(snap_id_from_pick_id(&pick_id_str));
        *resources.pick_id = Some(pick_id_str);
    }

    fn process_pick_texture(resources: &mut PickingRenderResources<'_>) {
        resources.shader_builder.add_uniform(
            "sampler2D",
            "model_pickTexture",
            Some(ShaderDestination::FRAGMENT),
        );

        if let Some(ref tex) = resources.pick_texture_uniform {
            resources
                .uniform_map
                .insert("model_pickTexture".to_string(), tex.clone());
        }

        let pick_id_str = "((selectedFeature.id < int(model_featuresLength)) ? texture(model_pickTexture, selectedFeature.st) : vec4(0.0))".to_string();
        *resources.snap_id = Some(snap_id_from_pick_id(&pick_id_str));
        *resources.pick_id = Some(pick_id_str);
    }
}
