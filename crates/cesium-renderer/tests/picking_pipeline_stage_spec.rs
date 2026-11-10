use std::collections::HashMap;

use cesium_math::ComponentDatatype;
use cesium_renderer::model_components::Primitive;
use cesium_renderer::model_statistics::ModelStatistics;
use cesium_renderer::model_type::ModelType;
use cesium_renderer::picking_pipeline_stage::{
    DetailPickObject, ModelPickObject, PickContext, PickId, PickingPipelineStage,
    PickingRenderResources,
};
use cesium_renderer::shader_builder::ShaderBuilder;
use cesium_renderer::vertex_array::VertexAttribute;

struct MockSetup {
    attributes: Vec<VertexAttribute>,
    attribute_index: u32,
    pick_id: Option<String>,
    snap_id: Option<String>,
    shader_builder: ShaderBuilder,
    statistics: ModelStatistics,
    uniform_map: HashMap<String, String>,
    pipeline_resources_count: usize,
    pick_ids: Vec<PickId>,
}

impl MockSetup {
    fn new() -> Self {
        Self {
            attributes: Vec::new(),
            attribute_index: 1,
            pick_id: None,
            snap_id: None,
            shader_builder: ShaderBuilder::new(),
            statistics: ModelStatistics::new(),
            uniform_map: HashMap::new(),
            pipeline_resources_count: 0,
            pick_ids: Vec::new(),
        }
    }

    fn as_resources(&mut self) -> PickingRenderResources<'_> {
        PickingRenderResources {
            attributes: &mut self.attributes,
            attribute_index: &mut self.attribute_index,
            pick_id: &mut self.pick_id,
            snap_id: &mut self.snap_id,
            shader_builder: &mut self.shader_builder,
            statistics: &mut self.statistics,
            uniform_map: &mut self.uniform_map,
            model_type: ModelType::Gltf,
            model_id: None,
            custom_pick_object: None,
            node_name: Some("test_node".to_string()),
            primitive_index: Some(0),
            has_property_table: false,
            instance_count: 0,
            has_draped_vectors: false,
            classification_type: None,
            pick_texture_uniform: None,
            pipeline_resources_count: &mut self.pipeline_resources_count,
            pick_ids: &mut self.pick_ids,
        }
    }
}

#[test]
fn test_returns_model_pick_object_if_defined() {
    let mut setup = MockSetup::new();
    let mut context = PickContext::new();

    let custom_pick = ModelPickObject {
        id: Some("custom_id".to_string()),
        primitive_id: Some("custom_primitive".to_string()),
        content_id: None,
        instance_id: None,
        is_3d_tiles: false,
        detail: DetailPickObject {
            model_id: Some("custom_id".to_string()),
            node_name: None,
            primitive_index: None,
        },
    };

    let mut resources = setup.as_resources();
    resources.custom_pick_object = Some(custom_pick.clone());

    let primitive = Primitive::default();
    PickingPipelineStage::process(&mut resources, &primitive, &mut context);

    assert_eq!(
        setup.shader_builder.fragment_uniform_lines(),
        &["uniform vec4 czm_pickColor;".to_string()]
    );

    assert_eq!(context.pick_objects.len(), 1);
    let obj = context.pick_objects.values().next().unwrap();
    let downcasted = obj.downcast_ref::<ModelPickObject>().unwrap();
    assert_eq!(downcasted, &custom_pick);

    assert!(setup.uniform_map.contains_key("czm_pickColor"));
    assert_eq!(setup.pipeline_resources_count, 1);
    assert_eq!(setup.pick_ids.len(), 1);
    assert_eq!(setup.pick_id.as_deref(), Some("czm_pickColor"));
}

#[test]
fn test_sets_picking_variables_for_3d_tiles() {
    let mut setup = MockSetup::new();
    let mut context = PickContext::new();

    let mut resources = setup.as_resources();
    resources.model_type = ModelType::TileGltf;

    let primitive = Primitive::default();
    PickingPipelineStage::process(&mut resources, &primitive, &mut context);

    assert_eq!(
        setup.shader_builder.fragment_uniform_lines(),
        &["uniform vec4 czm_pickColor;".to_string()]
    );

    let obj = context.pick_objects.values().next().unwrap();
    let downcasted = obj.downcast_ref::<ModelPickObject>().unwrap();
    assert!(downcasted.is_3d_tiles);
    assert_eq!(downcasted.content_id.as_deref(), Some("content"));

    assert_eq!(setup.pipeline_resources_count, 1);
    assert_eq!(setup.pick_ids.len(), 1);
    assert_eq!(setup.pick_id.as_deref(), Some("czm_pickColor"));
}

#[test]
fn test_sets_picking_variables_for_models() {
    let mut setup = MockSetup::new();
    let mut context = PickContext::new();

    let mut resources = setup.as_resources();
    resources.model_type = ModelType::Gltf;

    let primitive = Primitive::default();
    PickingPipelineStage::process(&mut resources, &primitive, &mut context);

    assert_eq!(
        setup.shader_builder.fragment_uniform_lines(),
        &["uniform vec4 czm_pickColor;".to_string()]
    );

    let obj = context.pick_objects.values().next().unwrap();
    let downcasted = obj.downcast_ref::<ModelPickObject>().unwrap();
    assert!(!downcasted.is_3d_tiles);
    assert_eq!(downcasted.primitive_id.as_deref(), Some("model"));

    assert_eq!(setup.pipeline_resources_count, 1);
    assert_eq!(setup.pick_ids.len(), 1);
    assert_eq!(setup.pick_id.as_deref(), Some("czm_pickColor"));
}

#[test]
fn test_sets_value_for_pick_object_id_if_model_has_id() {
    let mut setup = MockSetup::new();
    let mut context = PickContext::new();

    let mut resources = setup.as_resources();
    resources.model_id = Some("model_uuid".to_string());

    let primitive = Primitive::default();
    PickingPipelineStage::process(&mut resources, &primitive, &mut context);

    let obj = context.pick_objects.values().next().unwrap();
    let downcasted = obj.downcast_ref::<ModelPickObject>().unwrap();
    assert_eq!(downcasted.id.as_deref(), Some("model_uuid"));
    assert_eq!(setup.pick_id.as_deref(), Some("czm_pickColor"));
}

#[test]
fn test_sets_picking_variables_with_instancing() {
    let mut setup = MockSetup::new();
    let mut context = PickContext::new();

    let mut resources = setup.as_resources();
    resources.instance_count = 4;

    let primitive = Primitive::default();
    PickingPipelineStage::process(&mut resources, &primitive, &mut context);

    // Shader builder checks
    assert_eq!(
        setup.shader_builder.attribute_lines(),
        &["in vec4 a_pickColor;".to_string()]
    );
    assert_eq!(
        setup.shader_builder.vertex_varying_lines(),
        &["out vec4 v_pickColor;".to_string()]
    );
    assert_eq!(
        setup.shader_builder.fragment_varying_lines(),
        &["in vec4 v_pickColor;".to_string()]
    );

    // Attributes check
    assert_eq!(setup.attributes.len(), 1);
    let attr = &setup.attributes[0];
    assert_eq!(attr.index, 1);
    assert_eq!(setup.attribute_index, 2);
    assert!(attr.vertex_buffer.is_some());
    assert_eq!(attr.vertex_buffer.as_ref().unwrap().size_in_bytes, 4 * 4);
    assert_eq!(attr.components_per_attribute, 4);
    assert_eq!(attr.component_datatype, ComponentDatatype::UnsignedByte);
    assert!(attr.normalize);
    assert_eq!(attr.instance_divisor, 1);

    // Pipeline resources: 4 pickIds + 1 buffer = 5
    assert_eq!(setup.pipeline_resources_count, 5);
    assert_eq!(setup.pick_ids.len(), 4);
    assert_eq!(setup.statistics.geometry_byte_length, 4 * 4);
    assert_eq!(setup.pick_id.as_deref(), Some("v_pickColor"));
}

#[test]
fn test_sets_picking_variables_with_feature_id_textures() {
    let mut setup = MockSetup::new();
    let mut context = PickContext::new();

    let mut resources = setup.as_resources();
    resources.has_property_table = true;
    resources.pick_texture_uniform = Some("mockPickTexture".to_string());

    let primitive = Primitive::default();
    PickingPipelineStage::process(&mut resources, &primitive, &mut context);

    assert_eq!(
        setup.uniform_map.get("model_pickTexture").map(|s| s.as_str()),
        Some("mockPickTexture")
    );

    assert_eq!(
        setup.shader_builder.fragment_uniform_lines(),
        &["uniform sampler2D model_pickTexture;".to_string()]
    );

    assert_eq!(
        setup.pick_id.as_deref(),
        Some("((selectedFeature.id < int(model_featuresLength)) ? texture(model_pickTexture, selectedFeature.st) : vec4(0.0))")
    );
}
