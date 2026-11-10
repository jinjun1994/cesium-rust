use cesium_math::{Cartesian3, ComponentDatatype, Matrix4, SceneMode};
use cesium_renderer::model_components::{Attribute, Primitive, Quantization};
use cesium_renderer::scene_mode_2d_pipeline_stage::{
    MockSceneMode2DRenderResources, SceneMode2DFrameState, SceneMode2DPipelineStage,
};

#[test]
fn test_scene_mode_2d_pipeline_stage_name() {
    assert_eq!(SceneMode2DPipelineStage::NAME, "SceneMode2DPipelineStage");
}

#[test]
fn test_processes_resources_for_2d_for_primitive() {
    let mut render_resources = MockSceneMode2DRenderResources {
        position_min: Cartesian3::new(-0.5, -0.5, -0.5),
        position_max: Cartesian3::new(0.5, 0.5, 0.5),
        ..Default::default()
    };

    let positions: Vec<f32> = vec![
        -0.5, -0.5, 0.0,
        0.5, -0.5, 0.0,
        0.5, 0.5, 0.0,
        -0.5, 0.5, 0.0,
    ];
    let pos_bytes: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();

    let mut primitive = Primitive {
        attributes: vec![Attribute {
            name: Some("POSITION".to_string()),
            semantic: Some("POSITION".to_string()),
            component_datatype: Some(ComponentDatatype::Float),
            count: 4,
            typed_array: Some(pos_bytes),
            ..Default::default()
        }],
        ..Default::default()
    };

    let frame_state = SceneMode2DFrameState {
        mode: SceneMode::Scene2D,
        view: Matrix4::IDENTITY,
        ..Default::default()
    };

    SceneMode2DPipelineStage::process(&mut render_resources, &mut primitive, &frame_state);

    // 1. Check runtime primitive has boundingSphere2D
    let bs = render_resources.runtime_primitive.bounding_sphere_2d;
    assert!(bs.is_some());

    // 2. Check positionBuffer2D was created and tracked in model_resources
    let pos_buffer = render_resources
        .runtime_primitive
        .position_buffer_2d
        .as_ref()
        .unwrap();
    assert_eq!(render_resources.model.model_resources.len(), 1);
    assert_eq!(render_resources.model.model_resources[0].id, pos_buffer.id);

    // 3. Check position attribute's typedArray was unloaded
    assert!(primitive.attributes[0].typed_array.is_none());

    // 4. Check shader defines & uniforms
    let sb = &render_resources.shader_builder;
    assert!(sb.vertex_define_lines().contains(&"USE_2D_POSITIONS".to_string()));
    assert!(sb.vertex_uniform_lines().contains(&"uniform mat4 u_modelView2D;".to_string()));

    // 5. Check u_modelView2D matrix
    let expected_mat = Matrix4::from_translation(bs.unwrap().center.into());
    assert_eq!(render_resources.u_model_view_2d, Some(expected_mat));
}

#[test]
fn test_processes_resources_for_2d_for_primitive_with_quantization() {
    let mut render_resources = MockSceneMode2DRenderResources {
        position_min: Cartesian3::new(-1.0, -1.0, -1.0),
        position_max: Cartesian3::new(1.0, 1.0, 1.0),
        ..Default::default()
    };

    let raw_positions: Vec<f32> = vec![
        0.0, 0.0, 0.0,
        1.0, 1.0, 1.0,
    ];
    let pos_bytes: Vec<u8> = raw_positions.iter().flat_map(|f| f.to_le_bytes()).collect();

    let quantization = Quantization {
        quantized_volume_offset: Some(vec![-0.5, -0.5, -0.5]),
        quantized_volume_step_size: Some(vec![0.1, 0.1, 0.1]),
        ..Default::default()
    };

    let mut primitive = Primitive {
        attributes: vec![Attribute {
            name: Some("POSITION".to_string()),
            semantic: Some("POSITION".to_string()),
            count: 2,
            typed_array: Some(pos_bytes),
            quantization: Some(quantization),
            ..Default::default()
        }],
        ..Default::default()
    };

    let frame_state = SceneMode2DFrameState::default();

    SceneMode2DPipelineStage::process(&mut render_resources, &mut primitive, &frame_state);

    assert!(render_resources.runtime_primitive.bounding_sphere_2d.is_some());
    assert!(render_resources.runtime_primitive.position_buffer_2d.is_some());
    assert!(primitive.attributes[0].typed_array.is_none());
    assert!(render_resources.shader_builder.vertex_define_lines().contains(&"USE_2D_POSITIONS".to_string()));
}

#[test]
fn test_processes_resources_for_instanced_model_in_2d() {
    let mut render_resources = MockSceneMode2DRenderResources {
        position_min: Cartesian3::new(-0.5, -0.5, -0.5),
        position_max: Cartesian3::new(0.5, 0.5, 0.5),
        ..Default::default()
    };
    render_resources.runtime_node.is_instanced = true;

    let positions: Vec<f32> = vec![0.0, 0.0, 0.0];
    let pos_bytes: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();

    let mut primitive = Primitive {
        attributes: vec![Attribute {
            name: Some("POSITION".to_string()),
            semantic: Some("POSITION".to_string()),
            count: 1,
            typed_array: Some(pos_bytes),
            ..Default::default()
        }],
        ..Default::default()
    };

    let frame_state = SceneMode2DFrameState::default();

    SceneMode2DPipelineStage::process(&mut render_resources, &mut primitive, &frame_state);

    // Bounding sphere is computed
    assert!(render_resources.runtime_primitive.bounding_sphere_2d.is_some());
    // Position buffer is not created for instanced primitive here
    assert!(render_resources.runtime_primitive.position_buffer_2d.is_none());
    // Position typedArray is unloaded
    assert!(primitive.attributes[0].typed_array.is_none());
    // USE_2D_POSITIONS is not added (handled by instancing stage)
    assert!(render_resources.shader_builder.vertex_define_lines().is_empty());
    assert!(render_resources.u_model_view_2d.is_none());
}
