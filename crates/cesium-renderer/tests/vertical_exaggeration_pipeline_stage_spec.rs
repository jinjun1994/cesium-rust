use cesium_math::Cartesian2;
use cesium_renderer::vertical_exaggeration_pipeline_stage::{
    MockVerticalExaggerationRenderResources, VerticalExaggerationFrameState,
    VerticalExaggerationPipelineStage, VERTICAL_EXAGGERATION_STAGE_VS,
};

#[test]
fn test_vertical_exaggeration_pipeline_stage_name() {
    assert_eq!(
        VerticalExaggerationPipelineStage::NAME,
        "VerticalExaggerationPipelineStage"
    );
}

#[test]
fn test_adds_shader_lines_defines_and_uniforms() {
    let mut render_resources = MockVerticalExaggerationRenderResources::default();
    let frame_state = VerticalExaggerationFrameState {
        vertical_exaggeration: 2.0,
        vertical_exaggeration_relative_height: 100.0,
    };

    VerticalExaggerationPipelineStage::process(&mut render_resources, &frame_state);

    let sb = &render_resources.shader_builder;

    assert!(sb
        .vertex_lines()
        .contains(&VERTICAL_EXAGGERATION_STAGE_VS.to_string()));

    assert!(sb
        .vertex_define_lines()
        .contains(&"HAS_VERTICAL_EXAGGERATION".to_string()));

    assert!(sb.vertex_uniform_lines().contains(
        &"uniform vec2 u_verticalExaggerationAndRelativeHeight;".to_string()
    ));

    let expected_uniform = Cartesian2::new(2.0, 100.0);
    assert_eq!(render_resources.bound_exaggeration, Some(expected_uniform));
}
