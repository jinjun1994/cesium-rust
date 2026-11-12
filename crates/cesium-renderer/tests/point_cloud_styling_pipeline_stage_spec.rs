use cesium_math::{Cartesian3, Cartesian4, Matrix4, SceneMode};
use cesium_renderer::alpha_pipeline_stage::ModelAlphaOptions;
use cesium_renderer::commands::Pass;
use cesium_renderer::point_cloud_shading::PointCloudShading;
use cesium_renderer::point_cloud_styling_pipeline_stage::{
    PointCloudAttributeInfo, PointCloudFrameState, PointCloudModelType, PointCloudPrimitiveInfo,
    PointCloudStylingModel, PointCloudStylingPipelineStage, PointCloudStylingRenderResources,
    PointCloudStylingUniformMap, PointCloudTileContentInfo, PointCloudTileRefine,
};
use cesium_renderer::shader_builder::ShaderBuilder;
use cesium_renderer::vertex_attribute_semantic::VertexAttributeSemantic;
use cesium_renderer::Cesium3DTileStyleDocument;
use serde_json::json;

fn mock_primitive(with_normals: bool) -> PointCloudPrimitiveInfo {
    let mut attributes = vec![PointCloudAttributeInfo {
        semantic: VertexAttributeSemantic::POSITION.to_string(),
        min: Cartesian3::new(0.0, 0.0, 0.0),
        max: Cartesian3::new(1.0, 1.0, 1.0),
        count: 64,
    }];
    if with_normals {
        attributes.push(PointCloudAttributeInfo {
            semantic: VertexAttributeSemantic::NORMAL.to_string(),
            min: Cartesian3::new(-1.0, -1.0, -1.0),
            max: Cartesian3::new(1.0, 1.0, 1.0),
            count: 64,
        });
    }
    PointCloudPrimitiveInfo { attributes }
}

fn node_transform() -> Matrix4 {
    Matrix4::from_column_major_array([
        2.0, 0.0, 0.0, 0.0, // col 0
        0.0, 2.0, 0.0, 0.0, // col 1
        0.0, 0.0, 2.0, 0.0, // col 2
        1.0, 0.0, 0.0, 1.0, // col 3
    ])
}

#[test]
fn processes_gltf_point_cloud_with_shading_and_geometric_error() {
    let mut shader_builder = ShaderBuilder::new();
    let mut uniform_map = PointCloudStylingUniformMap::default();
    let mut alpha_options = ModelAlphaOptions::default();

    let model = PointCloudStylingModel {
        model_type: PointCloudModelType::Gltf,
        point_cloud_shading: PointCloudShading {
            attenuation: true,
            back_face_culling: true,
            geometric_error_scale: 2.0,
            ..Default::default()
        },
        style: None,
        property_attribute_names: Vec::new(),
        has_feature_table_with_features: false,
        content: None,
    };

    let mut resources = PointCloudStylingRenderResources {
        shader_builder: &mut shader_builder,
        uniform_map: &mut uniform_map,
        alpha_options: &mut alpha_options,
        runtime_node_transform: node_transform(),
        model: &model,
    };

    let frame_state = PointCloudFrameState {
        mode: SceneMode::Scene3D,
        is_orthographic: false,
        drawing_buffer_height: 800.0,
        sse_denominator: 0.5,
        pixel_ratio: 2.0,
    };

    PointCloudStylingPipelineStage::process(&mut resources, &mock_primitive(false), &frame_state)
        .unwrap();

    assert!(shader_builder
        .vertex_define_lines()
        .iter()
        .any(|d| d.contains("HAS_POINT_CLOUD_ATTENUATION")));
    assert!(shader_builder
        .vertex_define_lines()
        .iter()
        .any(|d| d.contains("HAS_POINT_CLOUD_BACK_FACE_CULLING")));
    assert!(shader_builder
        .vertex_uniform_lines()
        .iter()
        .any(|u| u.contains("model_pointCloudParameters")));

    // volume = (2*2*2) = 8, count = 64 -> cbrt(8/64) = 0.5; * geometricErrorScale 2.0 = 1.0
    assert_eq!(
        uniform_map.model_point_cloud_parameters,
        Cartesian4::new(2.0, 1.0, 1600.0, 0.0)
    );
}

#[test]
fn processes_pnts_style_and_translucent_alpha_pass() {
    let mut shader_builder = ShaderBuilder::new();
    let mut uniform_map = PointCloudStylingUniformMap::default();
    let mut alpha_options = ModelAlphaOptions::default();

    let style = Cesium3DTileStyleDocument::from_json(&json!({
        "color": "rgba(255, 0, 0, 0.5)",
        "show": "${height} > 10",
        "pointSize": "3.0"
    }))
    .unwrap();

    let model = PointCloudStylingModel {
        model_type: PointCloudModelType::TilePnts,
        point_cloud_shading: PointCloudShading::default(),
        style: Some(style),
        property_attribute_names: vec!["height".to_string()],
        has_feature_table_with_features: true,
        content: Some(PointCloudTileContentInfo {
            refine: PointCloudTileRefine::Add,
            geometric_error: 4.0,
            memory_adjusted_screen_space_error: 16.0,
            time_since_load: 12.5,
        }),
    };

    let mut resources = PointCloudStylingRenderResources {
        shader_builder: &mut shader_builder,
        uniform_map: &mut uniform_map,
        alpha_options: &mut alpha_options,
        runtime_node_transform: Matrix4::IDENTITY,
        model: &model,
    };

    let frame_state = PointCloudFrameState {
        mode: SceneMode::Scene2D,
        is_orthographic: true,
        ..Default::default()
    };

    PointCloudStylingPipelineStage::process(&mut resources, &mock_primitive(false), &frame_state)
        .unwrap();

    assert_eq!(alpha_options.pass, Some(Pass::Translucent));
    assert!(shader_builder
        .vertex_define_lines()
        .iter()
        .any(|d| d.contains("HAS_POINT_CLOUD_COLOR_STYLE")));
    assert!(shader_builder
        .vertex_define_lines()
        .iter()
        .any(|d| d.contains("HAS_POINT_CLOUD_SHOW_STYLE")));
    assert!(shader_builder
        .vertex_define_lines()
        .iter()
        .any(|d| d.contains("HAS_POINT_CLOUD_POINT_SIZE_STYLE")));
    assert!(shader_builder
        .vertex_define_lines()
        .iter()
        .any(|d| d.contains("COMPUTE_POSITION_WC_STYLE")));
    assert_eq!(
        uniform_map.model_point_cloud_parameters,
        Cartesian4::new(5.0, 4.0, f64::INFINITY, 12.5)
    );
}

#[test]
fn throws_when_style_uses_normal_without_normal_attribute() {
    let mut shader_builder = ShaderBuilder::new();
    let mut uniform_map = PointCloudStylingUniformMap::default();
    let mut alpha_options = ModelAlphaOptions::default();

    let style = Cesium3DTileStyleDocument::from_json(&json!({
        "color": "vec4(${NORMAL}, 1.0)"
    }))
    .unwrap();

    let model = PointCloudStylingModel {
        model_type: PointCloudModelType::TilePnts,
        point_cloud_shading: PointCloudShading::default(),
        style: Some(style),
        property_attribute_names: Vec::new(),
        has_feature_table_with_features: false,
        content: None,
    };

    let mut resources = PointCloudStylingRenderResources {
        shader_builder: &mut shader_builder,
        uniform_map: &mut uniform_map,
        alpha_options: &mut alpha_options,
        runtime_node_transform: Matrix4::IDENTITY,
        model: &model,
    };

    let err = PointCloudStylingPipelineStage::process(
        &mut resources,
        &mock_primitive(false),
        &PointCloudFrameState::default(),
    )
    .unwrap_err();
    assert!(
        err.to_string()
            .contains("Style references the NORMAL semantic but the point cloud does not have normals"),
        "unexpected error: {err}"
    );
}

#[test]
fn skips_gpu_styling_when_batch_table_is_present() {
    let mut shader_builder = ShaderBuilder::new();
    let mut uniform_map = PointCloudStylingUniformMap::default();
    let mut alpha_options = ModelAlphaOptions::default();

    let style = Cesium3DTileStyleDocument::from_json(&json!({
        "color": "color('#ff0000')"
    }))
    .unwrap();

    let model = PointCloudStylingModel {
        model_type: PointCloudModelType::TilePnts,
        point_cloud_shading: PointCloudShading::default(),
        style: Some(style),
        property_attribute_names: Vec::new(),
        has_feature_table_with_features: true,
        content: None,
    };

    let mut resources = PointCloudStylingRenderResources {
        shader_builder: &mut shader_builder,
        uniform_map: &mut uniform_map,
        alpha_options: &mut alpha_options,
        runtime_node_transform: Matrix4::IDENTITY,
        model: &model,
    };

    PointCloudStylingPipelineStage::process(
        &mut resources,
        &mock_primitive(false),
        &PointCloudFrameState::default(),
    )
    .unwrap();

    assert!(!shader_builder
        .vertex_define_lines()
        .iter()
        .any(|d| d.contains("HAS_POINT_CLOUD_COLOR_STYLE")));
}
