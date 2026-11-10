//! InstancingPipelineStage spec tests matching CesiumJS `Scene/Model/InstancingPipelineStageSpec.js`.

use std::collections::HashMap;

use cesium_math::{
    AttributeType, Cartesian3, ComponentDatatype, Matrix4, Quaternion, SceneMode,
};
use cesium_renderer::instancing_pipeline_stage::{
    InstancingFrameState, InstancingModel, InstancingPipelineStage, InstancingRenderResources,
    InstancingRuntimeNode,
};
use cesium_renderer::model_components::{Attribute, Instances, Node};
use cesium_renderer::shader_builder::ShaderBuilder;

fn f32_to_bytes(slice: &[f32]) -> Vec<u8> {
    slice.iter().flat_map(|f| f.to_ne_bytes()).collect()
}

fn create_mock_node_with_rotation(count: usize) -> Node {
    let mut translations = Vec::with_capacity(count * 3);
    let mut rotations = Vec::with_capacity(count * 4);
    let mut scales = Vec::with_capacity(count * 3);

    for i in 0..count {
        let sign = if i % 2 == 0 { 1.0_f32 } else { -1.0_f32 };
        translations.extend_from_slice(&[sign * 2.0, sign * 2.0, 0.0]);
        rotations.extend_from_slice(&[
            0.0,
            0.0,
            std::f32::consts::FRAC_1_SQRT_2,
            std::f32::consts::FRAC_1_SQRT_2,
        ]);
        scales.extend_from_slice(&[1.0, 1.0, 1.0]);
    }

    let trans_attr = Attribute {
        semantic: Some("TRANSLATION".to_string()),
        count,
        component_datatype: Some(ComponentDatatype::Float),
        attribute_type: Some(AttributeType::Vec3),
        typed_array: Some(f32_to_bytes(&translations)),
        ..Default::default()
    };

    let rot_attr = Attribute {
        semantic: Some("ROTATION".to_string()),
        count,
        component_datatype: Some(ComponentDatatype::Float),
        attribute_type: Some(AttributeType::Vec4),
        typed_array: Some(f32_to_bytes(&rotations)),
        ..Default::default()
    };

    let scale_attr = Attribute {
        semantic: Some("SCALE".to_string()),
        count,
        component_datatype: Some(ComponentDatatype::Float),
        attribute_type: Some(AttributeType::Vec3),
        typed_array: Some(f32_to_bytes(&scales)),
        ..Default::default()
    };

    let feat_id_attr = Attribute {
        semantic: Some("_FEATURE_ID".to_string()),
        set_index: Some(0),
        count,
        component_datatype: Some(ComponentDatatype::Float),
        attribute_type: Some(AttributeType::Scalar),
        typed_array: Some(f32_to_bytes(&vec![0.0_f32; count])),
        ..Default::default()
    };

    let instances = Instances {
        attributes: vec![trans_attr, rot_attr, scale_attr, feat_id_attr],
        feature_ids: Vec::new(),
        transform_in_world_space: false,
    };

    Node {
        instances: Some(instances),
        ..Default::default()
    }
}

fn create_mock_node_translation_only(count: usize, with_min_max: bool) -> Node {
    let mut translations = Vec::with_capacity(count * 3);
    for i in 0..count {
        let val = if i == 0 { -2.0_f32 } else { 2.0_f32 };
        translations.extend_from_slice(&[val, val, 0.0]);
    }

    let (min, max) = if with_min_max {
        (Some(vec![-2.0, -2.0, 0.0]), Some(vec![2.0, 2.0, 0.0]))
    } else {
        (None, None)
    };

    let trans_attr = Attribute {
        semantic: Some("TRANSLATION".to_string()),
        count,
        component_datatype: Some(ComponentDatatype::Float),
        attribute_type: Some(AttributeType::Vec3),
        typed_array: Some(f32_to_bytes(&translations)),
        min,
        max,
        ..Default::default()
    };

    let instances = Instances {
        attributes: vec![trans_attr],
        feature_ids: Vec::new(),
        transform_in_world_space: false,
    };

    Node {
        instances: Some(instances),
        ..Default::default()
    }
}

#[test]
fn test_computes_instancing_translation_min_and_max_from_typed_arrays() {
    let mut node = create_mock_node_with_rotation(4);
    let mut sb = ShaderBuilder::default();
    let mut uniform_map = HashMap::new();
    let mut attributes = Vec::new();
    let mut attribute_index = 1;
    let mut model = InstancingModel::default();
    let mut runtime_node = InstancingRuntimeNode::default();
    let frame_state = InstancingFrameState::default();

    {
        let mut rr = InstancingRenderResources {
            attribute_index: &mut attribute_index,
            attributes: &mut attributes,
            shader_builder: &mut sb,
            uniform_map: &mut uniform_map,
            model: &mut model,
            runtime_node: &mut runtime_node,
            instance_count: 0,
            feature_id_vertex_attribute_set_index: 0,
        };
        InstancingPipelineStage::process(&mut rr, &mut node, &frame_state);
    }

    assert_eq!(attributes.len(), 4);
    assert_eq!(
        runtime_node.instancing_translation_min,
        Some(Cartesian3::new(-2.0, -2.0, 0.0))
    );
    assert_eq!(
        runtime_node.instancing_translation_max,
        Some(Cartesian3::new(2.0, 2.0, 0.0))
    );

    // Re-running process preserves min / max
    {
        let mut rr = InstancingRenderResources {
            attribute_index: &mut attribute_index,
            attributes: &mut attributes,
            shader_builder: &mut sb,
            uniform_map: &mut uniform_map,
            model: &mut model,
            runtime_node: &mut runtime_node,
            instance_count: 0,
            feature_id_vertex_attribute_set_index: 0,
        };
        InstancingPipelineStage::process(&mut rr, &mut node, &frame_state);
    }

    assert_eq!(
        runtime_node.instancing_translation_min,
        Some(Cartesian3::new(-2.0, -2.0, 0.0))
    );
    assert_eq!(
        runtime_node.instancing_translation_max,
        Some(Cartesian3::new(2.0, 2.0, 0.0))
    );
}

#[test]
fn test_sets_instancing_translation_min_and_max_from_attributes() {
    let mut node = create_mock_node_translation_only(4, true);
    let mut sb = ShaderBuilder::default();
    let mut uniform_map = HashMap::new();
    let mut attributes = Vec::new();
    let mut attribute_index = 1;
    let mut model = InstancingModel::default();
    let mut runtime_node = InstancingRuntimeNode::default();

    let mut rr = InstancingRenderResources {
        attribute_index: &mut attribute_index,
        attributes: &mut attributes,
        shader_builder: &mut sb,
        uniform_map: &mut uniform_map,
        model: &mut model,
        runtime_node: &mut runtime_node,
        instance_count: 0,
        feature_id_vertex_attribute_set_index: 0,
    };

    let frame_state = InstancingFrameState::default();
    InstancingPipelineStage::process(&mut rr, &mut node, &frame_state);

    assert_eq!(attributes.len(), 1);
    assert_eq!(
        runtime_node.instancing_translation_min,
        Some(Cartesian3::new(-2.0, -2.0, 0.0))
    );
    assert_eq!(
        runtime_node.instancing_translation_max,
        Some(Cartesian3::new(2.0, 2.0, 0.0))
    );
}

#[test]
fn test_creates_instancing_matrices_vertex_attributes_when_rotation_is_present() {
    let mut node = create_mock_node_with_rotation(4);
    let mut sb = ShaderBuilder::default();
    let mut uniform_map = HashMap::new();
    let mut attributes = Vec::new();
    let mut attribute_index = 1;
    let mut model = InstancingModel::default();
    let mut runtime_node = InstancingRuntimeNode::default();

    let mut rr = InstancingRenderResources {
        attribute_index: &mut attribute_index,
        attributes: &mut attributes,
        shader_builder: &mut sb,
        uniform_map: &mut uniform_map,
        model: &mut model,
        runtime_node: &mut runtime_node,
        instance_count: 0,
        feature_id_vertex_attribute_set_index: 0,
    };

    let frame_state = InstancingFrameState::default();
    InstancingPipelineStage::process(&mut rr, &mut node, &frame_state);

    assert_eq!(attributes.len(), 4);

    let vs = sb.create_vertex_shader_source();
    let vs_text = vs.sources.join("\n");
    assert!(vs.defines.iter().any(|d| d == "HAS_INSTANCING"));
    assert!(vs.defines.iter().any(|d| d == "HAS_INSTANCE_MATRICES"));
    assert!(vs_text.contains("a_instancingTransformRow0"));
    assert!(vs_text.contains("a_instancingTransformRow1"));
    assert!(vs_text.contains("a_instancingTransformRow2"));
    assert!(vs_text.contains("a_instanceFeatureId_0"));

    assert!(runtime_node.instancing_transforms_buffer.is_some());
    // Verify typed arrays unloaded
    for attr in &node.instances.as_ref().unwrap().attributes {
        if attr.semantic.as_deref() != Some("_FEATURE_ID") {
            assert!(attr.typed_array.is_none());
        }
    }
}

#[test]
fn test_creates_instancing_matrices_vertex_attributes_for_2d() {
    let mut node = create_mock_node_with_rotation(4);
    let mut sb = ShaderBuilder::default();
    let mut uniform_map = HashMap::new();
    let mut attributes = Vec::new();
    let mut attribute_index = 1;
    let mut model = InstancingModel {
        project_to_2d: true,
        ..Default::default()
    };
    let mut runtime_node = InstancingRuntimeNode::default();

    let mut rr = InstancingRenderResources {
        attribute_index: &mut attribute_index,
        attributes: &mut attributes,
        shader_builder: &mut sb,
        uniform_map: &mut uniform_map,
        model: &mut model,
        runtime_node: &mut runtime_node,
        instance_count: 0,
        feature_id_vertex_attribute_set_index: 0,
    };

    let frame_state = InstancingFrameState {
        mode: SceneMode::Scene2D,
        ..Default::default()
    };

    InstancingPipelineStage::process(&mut rr, &mut node, &frame_state);

    // 3 rows for 3D + 3 rows for 2D + 1 feature ID = 7 attributes
    assert_eq!(attributes.len(), 7);

    let vs = sb.create_vertex_shader_source();
    let vs_text = vs.sources.join("\n");
    assert!(vs.defines.iter().any(|d| d == "HAS_INSTANCING"));
    assert!(vs.defines.iter().any(|d| d == "HAS_INSTANCE_MATRICES"));
    assert!(vs.defines.iter().any(|d| d == "USE_2D_INSTANCING"));

    assert!(vs_text.contains("a_instancingTransformRow0"));
    assert!(vs_text.contains("a_instancingTransform2DRow0"));
    assert!(vs_text.contains("u_modelView2D"));

    assert!(runtime_node.instancing_transforms_buffer.is_some());
    assert!(runtime_node.instancing_transforms_buffer_2d.is_some());
    assert!(runtime_node.instancing_reference_point_2d.is_some());
    assert!(uniform_map.contains_key("u_modelView2D"));
}

#[test]
fn test_correctly_creates_transform_matrices() {
    let translation = Cartesian3::new(10.0, 20.0, 30.0);
    let rotation = Quaternion::from_axis_angle(Cartesian3::UNIT_Z, std::f64::consts::FRAC_PI_2);
    let scale = Cartesian3::new(2.0, 3.0, 4.0);

    let mat = Matrix4::from_translation_quaternion_rotation_scale(translation, rotation, scale);
    let typed = InstancingPipelineStage::transforms_to_typed_array(&[mat]);

    assert_eq!(typed.len(), 12);
    // Row 0: [mat[0], mat[4], mat[8], mat[12]]
    assert!((typed[0] - mat.values[0] as f32).abs() < 1e-6);
    assert!((typed[1] - mat.values[4] as f32).abs() < 1e-6);
    assert!((typed[2] - mat.values[8] as f32).abs() < 1e-6);
    assert!((typed[3] - mat.values[12] as f32).abs() < 1e-6);
    // Row 1: [mat[1], mat[5], mat[9], mat[13]]
    assert!((typed[4] - mat.values[1] as f32).abs() < 1e-6);
    assert!((typed[5] - mat.values[5] as f32).abs() < 1e-6);
    assert!((typed[6] - mat.values[9] as f32).abs() < 1e-6);
    assert!((typed[7] - mat.values[13] as f32).abs() < 1e-6);
    // Row 2: [mat[2], mat[6], mat[10], mat[14]]
    assert!((typed[8] - mat.values[2] as f32).abs() < 1e-6);
    assert!((typed[9] - mat.values[6] as f32).abs() < 1e-6);
    assert!((typed[10] - mat.values[10] as f32).abs() < 1e-6);
    assert!((typed[11] - mat.values[14] as f32).abs() < 1e-6);
}

#[test]
fn test_dequantizes_normalized_rotations() {
    let raw_shorts: [i16; 4] = [0, 0, 0, 32767];
    let bytes: Vec<u8> = raw_shorts
        .iter()
        .flat_map(|s| s.to_ne_bytes())
        .collect();

    let rot_attr = Attribute {
        semantic: Some("ROTATION".to_string()),
        count: 1,
        component_datatype: Some(ComponentDatatype::Short),
        attribute_type: Some(AttributeType::Vec4),
        normalized: true,
        typed_array: Some(bytes),
        ..Default::default()
    };

    let mut instances = Instances {
        attributes: vec![rot_attr],
        feature_ids: Vec::new(),
        transform_in_world_space: false,
    };

    let mut runtime_node = InstancingRuntimeNode::default();
    let transforms = InstancingPipelineStage::get_instance_transforms_as_matrices(
        &mut instances,
        1,
        &mut runtime_node,
    );

    assert_eq!(transforms.len(), 1);
    // Identity matrix expected for quat (0, 0, 0, 1) and default translation/scale
    for i in 0..16 {
        assert!((transforms[0].values[i] - Matrix4::IDENTITY.values[i]).abs() < 1e-4);
    }
}

#[test]
fn test_creates_translation_vertex_attributes_with_min_max_present() {
    let mut node = create_mock_node_translation_only(2, true);
    let mut sb = ShaderBuilder::default();
    let mut uniform_map = HashMap::new();
    let mut attributes = Vec::new();
    let mut attribute_index = 1;
    let mut model = InstancingModel::default();
    let mut runtime_node = InstancingRuntimeNode::default();

    let mut rr = InstancingRenderResources {
        attribute_index: &mut attribute_index,
        attributes: &mut attributes,
        shader_builder: &mut sb,
        uniform_map: &mut uniform_map,
        model: &mut model,
        runtime_node: &mut runtime_node,
        instance_count: 0,
        feature_id_vertex_attribute_set_index: 0,
    };

    let frame_state = InstancingFrameState::default();
    InstancingPipelineStage::process(&mut rr, &mut node, &frame_state);

    assert_eq!(attributes.len(), 1);
    let vs = sb.create_vertex_shader_source();
    let vs_text = vs.sources.join("\n");
    assert!(vs.defines.iter().any(|d| d == "HAS_INSTANCING"));
    assert!(vs.defines.iter().any(|d| d == "HAS_INSTANCE_TRANSLATION"));
    assert!(vs_text.contains("a_instanceTranslation"));
}

#[test]
fn test_creates_translation_vertex_attributes_for_2d() {
    let mut node = create_mock_node_translation_only(2, false);
    let mut sb = ShaderBuilder::default();
    let mut uniform_map = HashMap::new();
    let mut attributes = Vec::new();
    let mut attribute_index = 1;
    let mut model = InstancingModel {
        project_to_2d: true,
        ..Default::default()
    };
    let mut runtime_node = InstancingRuntimeNode::default();

    let mut rr = InstancingRenderResources {
        attribute_index: &mut attribute_index,
        attributes: &mut attributes,
        shader_builder: &mut sb,
        uniform_map: &mut uniform_map,
        model: &mut model,
        runtime_node: &mut runtime_node,
        instance_count: 0,
        feature_id_vertex_attribute_set_index: 0,
    };

    let frame_state = InstancingFrameState {
        mode: SceneMode::Scene2D,
        ..Default::default()
    };

    InstancingPipelineStage::process(&mut rr, &mut node, &frame_state);

    // 1 translation 3D + 1 translation 2D = 2 attributes
    assert_eq!(attributes.len(), 2);
    let vs = sb.create_vertex_shader_source();
    let vs_text = vs.sources.join("\n");
    assert!(vs.defines.iter().any(|d| d == "USE_2D_INSTANCING"));
    assert!(vs_text.contains("a_instanceTranslation"));
    assert!(vs_text.contains("a_instanceTranslation2D"));
    assert!(uniform_map.contains_key("u_modelView2D"));
}

#[test]
fn test_adds_uniforms_for_legacy_instancing_path() {
    let mut node = create_mock_node_with_rotation(2);
    node.instances.as_mut().unwrap().transform_in_world_space = true;

    let mut sb = ShaderBuilder::default();
    let mut uniform_map = HashMap::new();
    let mut attributes = Vec::new();
    let mut attribute_index = 1;
    let mut model = InstancingModel {
        model_matrix: Matrix4::from_uniform_scale(2.0),
        scene_graph: cesium_renderer::instancing_pipeline_stage::InstancingSceneGraph {
            computed_model_matrix: Matrix4::IDENTITY,
            axis_correction_matrix: Matrix4::IDENTITY,
            components_transform: Matrix4::from_translation(Cartesian3::new(10.0, 0.0, 0.0)),
        },
        ..Default::default()
    };
    let mut runtime_node = InstancingRuntimeNode {
        computed_transform: Matrix4::from_translation(Cartesian3::new(0.0, 2.0, 0.0)),
        ..Default::default()
    };

    let mut rr = InstancingRenderResources {
        attribute_index: &mut attribute_index,
        attributes: &mut attributes,
        shader_builder: &mut sb,
        uniform_map: &mut uniform_map,
        model: &mut model,
        runtime_node: &mut runtime_node,
        instance_count: 0,
        feature_id_vertex_attribute_set_index: 0,
    };

    let frame_state = InstancingFrameState::default();
    InstancingPipelineStage::process(&mut rr, &mut node, &frame_state);

    let vs = sb.create_vertex_shader_source();
    let vs_text = vs.sources.join("\n");
    assert!(vs.defines.iter().any(|d| d == "USE_LEGACY_INSTANCING"));
    assert!(vs_text.contains("u_instance_modifiedModelView"));
    assert!(vs_text.contains("u_instance_nodeTransform"));
    assert!(vs_text.contains("legacyInstancingStage"));

    assert!(uniform_map.contains_key("u_instance_modifiedModelView"));
    assert!(uniform_map.contains_key("u_instance_nodeTransform"));
}
