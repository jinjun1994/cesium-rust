use cesium_math::Cartesian2;
use cesium_renderer::classification_model_draw_command::StencilConstants;
use cesium_renderer::tileset_pipeline_stage::{
    MockTilesetModel, MockTilesetRenderResources, TilesetPipelineStage,
};

#[test]
fn test_tileset_pipeline_stage_name() {
    assert_eq!(TilesetPipelineStage::NAME, "TilesetPipelineStage");
}

#[test]
fn test_configures_the_render_resources_for_skipping_level_of_detail() {
    let mut render_resources = MockTilesetRenderResources {
        model: MockTilesetModel {
            has_skip_level_of_detail: true,
        },
        ..Default::default()
    };

    let model = render_resources.model.clone();
    TilesetPipelineStage::process(&mut render_resources, &model);

    let sb = &render_resources.shader_builder;
    assert!(sb.vertex_define_lines().is_empty());
    assert!(sb.fragment_define_lines().contains(&"POLYGON_OFFSET".to_string()));

    // No uniform declarations should have been added
    assert!(sb.vertex_uniform_lines().is_empty());
    assert!(sb.fragment_uniform_lines().is_empty());

    assert_eq!(render_resources.u_polygon_offset, Some(Cartesian2::ZERO));
    assert!(render_resources.has_skip_level_of_detail);
}

#[test]
fn test_sets_stencil_test_and_mask() {
    let mut render_resources = MockTilesetRenderResources::default();
    let model = render_resources.model.clone();

    TilesetPipelineStage::process(&mut render_resources, &model);

    let render_state = &render_resources.render_state_options;
    assert_eq!(
        render_state.stencil_test,
        StencilConstants::set_cesium_3d_tile_bit()
    );
    assert_eq!(
        render_state.stencil_mask,
        StencilConstants::CESIUM_3D_TILE_MASK
    );
}
