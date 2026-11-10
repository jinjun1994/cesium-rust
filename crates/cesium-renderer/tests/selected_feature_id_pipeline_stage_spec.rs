use cesium_renderer::model_components::{
    FeatureId, FeatureIdAttribute, FeatureIdTexture, Instances, Node, Primitive,
};
use cesium_renderer::selected_feature_id_pipeline_stage::{
    MockSelectedFeatureIdModel, MockSelectedFeatureIdRenderResources,
    SelectedFeatureIdPipelineStage, SELECTED_FEATURE_ID_STAGE_COMMON,
};
use cesium_renderer::shader_builder::{ShaderBuilder, ShaderDestination};

fn create_default_shader_builder() -> ShaderBuilder {
    let mut sb = ShaderBuilder::new();
    sb.add_struct(
        SelectedFeatureIdPipelineStage::STRUCT_ID_SELECTED_FEATURE,
        SelectedFeatureIdPipelineStage::STRUCT_NAME_SELECTED_FEATURE,
        ShaderDestination::BOTH,
    );
    sb
}

#[test]
fn test_selected_feature_id_pipeline_stage_constants() {
    assert_eq!(
        SelectedFeatureIdPipelineStage::NAME,
        "SelectedFeatureIdPipelineStage"
    );
    assert_eq!(
        SelectedFeatureIdPipelineStage::STRUCT_ID_SELECTED_FEATURE,
        "SelectedFeature"
    );
    assert_eq!(
        SelectedFeatureIdPipelineStage::STRUCT_NAME_SELECTED_FEATURE,
        "SelectedFeature"
    );
}

#[test]
fn test_selects_primitive_feature_ids() {
    let mut render_resources = MockSelectedFeatureIdRenderResources {
        shader_builder: create_default_shader_builder(),
        model: MockSelectedFeatureIdModel {
            feature_id_label: Some("defaultIdsTest".to_string()),
            instance_feature_id_label: None,
        },
        ..Default::default()
    };

    let node = Node::default();
    let primitive = Primitive {
        feature_ids: vec![FeatureId::Attribute(FeatureIdAttribute {
            feature_count: 10,
            label: Some("defaultIdsTest".to_string()),
            ..Default::default()
        })],
        ..Default::default()
    };

    SelectedFeatureIdPipelineStage::process(&mut render_resources, &node, &primitive);

    assert!(render_resources.has_property_table);

    let sb = &render_resources.shader_builder;
    assert!(sb.vertex_define_lines().contains(&"HAS_SELECTED_FEATURE_ID".to_string()));
    assert!(sb.vertex_define_lines().contains(&"HAS_SELECTED_FEATURE_ID_ATTRIBUTE".to_string()));
    assert!(sb.vertex_define_lines().contains(&"SELECTED_FEATURE_ID defaultIdsTest".to_string()));

    assert!(sb.fragment_define_lines().contains(&"HAS_SELECTED_FEATURE_ID".to_string()));
    assert!(sb.fragment_define_lines().contains(&"HAS_SELECTED_FEATURE_ID_ATTRIBUTE".to_string()));
    assert!(sb.fragment_define_lines().contains(&"SELECTED_FEATURE_ID defaultIdsTest".to_string()));

    assert!(sb.vertex_lines().contains(&SELECTED_FEATURE_ID_STAGE_COMMON.to_string()));
    assert!(sb.fragment_lines().contains(&SELECTED_FEATURE_ID_STAGE_COMMON.to_string()));
}

#[test]
fn test_selects_feature_id_texture() {
    let mut render_resources = MockSelectedFeatureIdRenderResources {
        shader_builder: create_default_shader_builder(),
        model: MockSelectedFeatureIdModel {
            feature_id_label: Some("landCover".to_string()),
            instance_feature_id_label: None,
        },
        ..Default::default()
    };

    let node = Node::default();
    let primitive = Primitive {
        feature_ids: vec![FeatureId::Texture(FeatureIdTexture {
            feature_count: 5,
            label: Some("landCover".to_string()),
            ..Default::default()
        })],
        ..Default::default()
    };

    SelectedFeatureIdPipelineStage::process(&mut render_resources, &node, &primitive);

    assert!(render_resources.has_property_table);

    let sb = &render_resources.shader_builder;
    // Vertex shader has no texture defines/lines
    assert!(sb.vertex_define_lines().is_empty());
    assert!(sb.vertex_lines().is_empty());

    // Fragment shader has texture defines and common code
    assert!(sb.fragment_define_lines().contains(&"HAS_SELECTED_FEATURE_ID".to_string()));
    assert!(sb.fragment_define_lines().contains(&"HAS_SELECTED_FEATURE_ID_TEXTURE".to_string()));
    assert!(sb.fragment_define_lines().contains(&"SELECTED_FEATURE_ID landCover".to_string()));
    assert!(sb.fragment_lines().contains(&SELECTED_FEATURE_ID_STAGE_COMMON.to_string()));
}

#[test]
fn test_selects_instance_feature_ids() {
    let mut render_resources = MockSelectedFeatureIdRenderResources {
        shader_builder: create_default_shader_builder(),
        model: MockSelectedFeatureIdModel {
            feature_id_label: None,
            instance_feature_id_label: Some("section".to_string()),
        },
        ..Default::default()
    };

    let node = Node {
        instances: Some(Instances {
            feature_ids: vec![FeatureId::Attribute(FeatureIdAttribute {
                feature_count: 4,
                label: Some("section".to_string()),
                ..Default::default()
            })],
            ..Default::default()
        }),
        ..Default::default()
    };
    let primitive = Primitive::default();

    SelectedFeatureIdPipelineStage::process(&mut render_resources, &node, &primitive);

    assert!(render_resources.has_property_table);

    let sb = &render_resources.shader_builder;
    assert!(sb.vertex_define_lines().contains(&"HAS_SELECTED_FEATURE_ID".to_string()));
    assert!(sb.vertex_define_lines().contains(&"HAS_SELECTED_FEATURE_ID_ATTRIBUTE".to_string()));
    assert!(sb.vertex_define_lines().contains(&"SELECTED_FEATURE_ID section".to_string()));

    assert!(sb.fragment_define_lines().contains(&"HAS_SELECTED_FEATURE_ID".to_string()));
    assert!(sb.fragment_define_lines().contains(&"HAS_SELECTED_FEATURE_ID_ATTRIBUTE".to_string()));
    assert!(sb.fragment_define_lines().contains(&"SELECTED_FEATURE_ID section".to_string()));

    assert!(sb.vertex_lines().contains(&SELECTED_FEATURE_ID_STAGE_COMMON.to_string()));
    assert!(sb.fragment_lines().contains(&SELECTED_FEATURE_ID_STAGE_COMMON.to_string()));
}

#[test]
fn test_handles_null_feature_id_when_present() {
    let mut render_resources = MockSelectedFeatureIdRenderResources {
        shader_builder: create_default_shader_builder(),
        model: MockSelectedFeatureIdModel {
            feature_id_label: Some("idsGWithNull".to_string()),
            instance_feature_id_label: None,
        },
        ..Default::default()
    };

    let node = Node::default();
    let primitive = Primitive {
        feature_ids: vec![FeatureId::Texture(FeatureIdTexture {
            feature_count: 100,
            label: Some("idsGWithNull".to_string()),
            null_feature_id: Some(10),
            ..Default::default()
        })],
        ..Default::default()
    };

    SelectedFeatureIdPipelineStage::process(&mut render_resources, &node, &primitive);

    assert!(render_resources.has_property_table);

    let sb = &render_resources.shader_builder;
    assert!(sb.vertex_define_lines().is_empty());
    assert!(sb.fragment_define_lines().contains(&"HAS_NULL_FEATURE_ID".to_string()));
    assert!(sb.fragment_define_lines().contains(&"HAS_SELECTED_FEATURE_ID".to_string()));
    assert!(sb.fragment_define_lines().contains(&"HAS_SELECTED_FEATURE_ID_TEXTURE".to_string()));
    assert!(sb.fragment_define_lines().contains(&"SELECTED_FEATURE_ID idsGWithNull".to_string()));

    assert_eq!(render_resources.bound_null_feature_id, Some(10));
    let uniform_cb = render_resources.uniform_map.get("model_nullFeatureId").unwrap();
    assert_eq!(uniform_cb(), 10.0);
}
