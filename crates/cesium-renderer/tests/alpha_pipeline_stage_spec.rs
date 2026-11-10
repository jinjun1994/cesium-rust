use cesium_renderer::{
    AlphaPipelineStage, AlphaRenderResources, BlendingState, ModelAlphaOptions, Pass, RenderState,
    ShaderBuilder,
};
use std::collections::HashMap;

#[test]
fn test_defaults_to_the_models_pass_if_not_specified() {
    let mut shader_builder = ShaderBuilder::new();
    let mut alpha_options = ModelAlphaOptions::default();
    let mut uniform_map = HashMap::new();
    let mut render_state = RenderState::default();

    assert_eq!(alpha_options.pass, None);

    let mut resources = AlphaRenderResources {
        opaque_pass: Pass::Cesium3dTile,
        shader_builder: &mut shader_builder,
        alpha_options: &mut alpha_options,
        uniform_map: &mut uniform_map,
        render_state: &mut render_state,
    };

    AlphaPipelineStage::process(&mut resources);

    assert_eq!(resources.alpha_options.pass, Some(Pass::Cesium3dTile));
}

#[test]
fn test_sets_render_state_options_when_given_translucent_pass() {
    let mut shader_builder = ShaderBuilder::new();
    let mut alpha_options = ModelAlphaOptions {
        pass: Some(Pass::Translucent),
        alpha_cutoff: None,
    };
    let mut uniform_map = HashMap::new();
    let mut render_state = RenderState::default();

    let mut resources = AlphaRenderResources {
        opaque_pass: Pass::Cesium3dTile,
        shader_builder: &mut shader_builder,
        alpha_options: &mut alpha_options,
        uniform_map: &mut uniform_map,
        render_state: &mut render_state,
    };

    AlphaPipelineStage::process(&mut resources);

    assert!(!resources.render_state.cull.enabled);
    assert!(!resources.render_state.depth_mask);
    assert_eq!(resources.render_state.blending, BlendingState::ALPHA_BLEND);
}

#[test]
fn test_handles_alpha_cutoff() {
    let cutoff = 0.6;
    let mut shader_builder = ShaderBuilder::new();
    let mut alpha_options = ModelAlphaOptions {
        pass: Some(Pass::Translucent),
        alpha_cutoff: Some(cutoff),
    };
    let mut uniform_map = HashMap::new();
    let mut render_state = RenderState::default();

    let mut resources = AlphaRenderResources {
        opaque_pass: Pass::Cesium3dTile,
        shader_builder: &mut shader_builder,
        alpha_options: &mut alpha_options,
        uniform_map: &mut uniform_map,
        render_state: &mut render_state,
    };

    AlphaPipelineStage::process(&mut resources);

    // Verify fragment defines
    let fs = resources.shader_builder.create_fragment_shader_source();
    assert!(fs.defines.iter().any(|d| d == "ALPHA_MODE_MASK"));

    // Verify fragment uniforms in source
    let source_text = fs.sources.join("\n");
    assert!(source_text.contains("uniform float u_alphaCutoff;"));

    // Verify uniform map function
    let callback = resources
        .uniform_map
        .get("u_alphaCutoff")
        .expect("u_alphaCutoff callback should exist");
    assert_eq!(callback(), cutoff);
}
