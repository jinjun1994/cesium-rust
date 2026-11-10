//! Tests for ModelClippingPlanesPipelineStage matching CesiumJS `Scene/Model/ModelClippingPlanesPipelineStageSpec.js` (Apache-2.0).

use cesium_math::{Cartesian2, Cartesian3, Color, Matrix4};
use cesium_renderer::clipping_plane::ClippingPlane;
use cesium_renderer::clipping_plane_collection::{
    ClippingPlaneCollection, ClippingPlaneCollectionOptions,
};
use cesium_renderer::model_clipping_planes_pipeline_stage::{
    ModelClippingPlanesContext, ModelClippingPlanesFrameState, ModelClippingPlanesModel,
    ModelClippingPlanesPipelineStage, ModelClippingPlanesRenderResources,
    ModelClippingPlanesUniformState,
};

fn create_test_context() -> (ClippingPlaneCollection, ModelClippingPlanesModel) {
    let plane = ClippingPlane::new(Cartesian3::UNIT_X, 0.0);
    let mut clipping_planes = ClippingPlaneCollection::new(Some(ClippingPlaneCollectionOptions {
        planes: Some(vec![plane]),
        texture_resolution: Some(Cartesian2::new(1.0, 1.0)),
        ..Default::default()
    }));
    clipping_planes.texture_resolution = Some(Cartesian2::new(1.0, 1.0));

    let model = ModelClippingPlanesModel::new(
        clipping_planes.clone(),
        Matrix4::IDENTITY,
        Matrix4::IDENTITY,
    );

    (clipping_planes, model)
}

#[test]
fn test_configures_render_resources_for_default_clipping_planes() {
    let (clipping_planes, model) = create_test_context();
    let frame_state = ModelClippingPlanesFrameState {
        context: ModelClippingPlanesContext {
            floating_point_texture: false,
            uniform_state: ModelClippingPlanesUniformState {
                inverse_view_3d: Matrix4::IDENTITY,
            },
        },
    };

    let mut render_resources = ModelClippingPlanesRenderResources::new(model.clone());
    ModelClippingPlanesPipelineStage::process(&mut render_resources, &model, &frame_state);

    let defines = render_resources.shader_builder.fragment_define_lines();
    assert!(defines.contains(&"HAS_CLIPPING_PLANES".to_string()));
    assert!(defines.contains(&"CLIPPING_PLANES_LENGTH 1".to_string()));
    assert!(defines.contains(&"CLIPPING_PLANES_TEXTURE_WIDTH 1".to_string()));
    assert!(defines.contains(&"CLIPPING_PLANES_TEXTURE_HEIGHT 1".to_string()));

    let uniforms = render_resources.shader_builder.fragment_uniform_lines();
    assert!(uniforms.contains(&"uniform sampler2D model_clippingPlanes;".to_string()));
    assert!(uniforms.contains(&"uniform vec4 model_clippingPlanesEdgeStyle;".to_string()));
    assert!(uniforms.contains(&"uniform mat4 model_clippingPlanesMatrix;".to_string()));

    assert!(render_resources.uniform_map.model_clipping_planes().is_some());

    let edge_color = clipping_planes.edge_color;
    let expected_style = Color::new(
        edge_color.red,
        edge_color.green,
        edge_color.blue,
        clipping_planes.edge_width as f32,
    );
    assert_eq!(
        render_resources
            .uniform_map
            .model_clipping_planes_edge_style()
            .unwrap(),
        expected_style
    );

    assert_eq!(
        render_resources
            .uniform_map
            .model_clipping_planes_matrix()
            .unwrap(),
        Matrix4::IDENTITY
    );

    let shader_source = render_resources.shader_builder.create_fragment_shader_source();
    assert!(shader_source.sources.iter().any(|s| s.contains("modelClippingPlanesStage")));
}

#[test]
fn test_model_clipping_planes_matrix_equals_unfactored_inverse_transpose() {
    let (_clipping_planes, mut model) = create_test_context();

    let view = Matrix4::from_translation(Cartesian3::new(2.0, -3.0, 5.0));
    let reference_matrix = Matrix4::from_translation(Cartesian3::new(7.0, 1.0, -4.0));
    let clipping_matrix = Matrix4::from_translation(Cartesian3::new(-1.0, 6.0, 2.0));

    let inverse_view_3d = view.inverse_transformation().unwrap();
    let reference_times_clipping = reference_matrix.multiply(&clipping_matrix);
    let clipping_planes_matrix = reference_times_clipping.inverse_transpose().unwrap();

    let frame_state = ModelClippingPlanesFrameState {
        context: ModelClippingPlanesContext {
            floating_point_texture: false,
            uniform_state: ModelClippingPlanesUniformState { inverse_view_3d },
        },
    };

    model.model_matrix = reference_matrix;
    model.clipping_planes_matrix = clipping_planes_matrix;

    let mut render_resources = ModelClippingPlanesRenderResources::new(model.clone());
    ModelClippingPlanesPipelineStage::process(&mut render_resources, &model, &frame_state);

    let expected = view
        .multiply(&reference_times_clipping)
        .inverse_transpose()
        .unwrap();

    let actual = render_resources
        .uniform_map
        .model_clipping_planes_matrix()
        .unwrap();

    assert!(actual.equals_epsilon(&expected, 1e-10));
}

#[test]
fn test_configures_render_resources_for_unioned_clipping_planes() {
    let (mut clipping_planes, mut model) = create_test_context();
    clipping_planes.union_clipping_regions = true;
    model.clipping_planes = clipping_planes;

    let frame_state = ModelClippingPlanesFrameState::default();
    let mut render_resources = ModelClippingPlanesRenderResources::new(model.clone());
    ModelClippingPlanesPipelineStage::process(&mut render_resources, &model, &frame_state);

    let defines = render_resources.shader_builder.fragment_define_lines();
    assert!(defines.contains(&"HAS_CLIPPING_PLANES".to_string()));
    assert!(defines.contains(&"CLIPPING_PLANES_LENGTH 1".to_string()));
    assert!(defines.contains(&"UNION_CLIPPING_REGIONS".to_string()));
    assert!(defines.contains(&"CLIPPING_PLANES_TEXTURE_WIDTH 1".to_string()));
    assert!(defines.contains(&"CLIPPING_PLANES_TEXTURE_HEIGHT 1".to_string()));
}

#[test]
fn test_configures_render_resources_for_float_texture_clipping_planes() {
    let (_clipping_planes, model) = create_test_context();
    let frame_state = ModelClippingPlanesFrameState {
        context: ModelClippingPlanesContext {
            floating_point_texture: true,
            ..Default::default()
        },
    };

    let mut render_resources = ModelClippingPlanesRenderResources::new(model.clone());
    ModelClippingPlanesPipelineStage::process(&mut render_resources, &model, &frame_state);

    let defines = render_resources.shader_builder.fragment_define_lines();
    assert!(defines.contains(&"HAS_CLIPPING_PLANES".to_string()));
    assert!(defines.contains(&"CLIPPING_PLANES_LENGTH 1".to_string()));
    assert!(defines.contains(&"USE_CLIPPING_PLANES_FLOAT_TEXTURE".to_string()));
    assert!(defines.contains(&"CLIPPING_PLANES_TEXTURE_WIDTH 1".to_string()));
    assert!(defines.contains(&"CLIPPING_PLANES_TEXTURE_HEIGHT 1".to_string()));
}
