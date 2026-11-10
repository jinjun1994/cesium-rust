//! Tests for ModelColorPipelineStage matching CesiumJS `Scene/Model/ModelColorPipelineStageSpec.js` (Apache-2.0).

use cesium_math::{Color, ColorBlendMode};
use cesium_renderer::alpha_pipeline_stage::ModelAlphaOptions;
use cesium_renderer::commands::Pass;
use cesium_renderer::model_color_pipeline_stage::{
    ModelColorModel, ModelColorPipelineStage, ModelColorRenderResources, ModelColorUniformMap,
};
use cesium_renderer::render_state::{ColorMaskState, RenderState};
use cesium_renderer::shader_builder::ShaderBuilder;

fn create_mock_render_resources<'a>(
    shader_builder: &'a mut ShaderBuilder,
    uniform_map: &'a mut ModelColorUniformMap,
    alpha_options: &'a mut ModelAlphaOptions,
    render_state: &'a mut RenderState,
) -> ModelColorRenderResources<'a> {
    alpha_options.pass = Some(Pass::Opaque);
    ModelColorRenderResources {
        shader_builder,
        uniform_map,
        alpha_options,
        render_state,
    }
}

#[test]
fn test_configures_render_resources_for_opaque_color() {
    let mock_model = ModelColorModel::new(Color::RED, ColorBlendMode::Replace, 0.25);

    let mut shader_builder = ShaderBuilder::new();
    let mut uniform_map = ModelColorUniformMap::default();
    let mut alpha_options = ModelAlphaOptions::default();
    let mut render_state = RenderState::default();

    let mut render_resources = create_mock_render_resources(
        &mut shader_builder,
        &mut uniform_map,
        &mut alpha_options,
        &mut render_state,
    );

    ModelColorPipelineStage::process(&mut render_resources, &mock_model);

    let defines = render_resources.shader_builder.fragment_define_lines();
    assert!(defines.contains(&"HAS_MODEL_COLOR".to_string()));

    let uniforms = render_resources.shader_builder.fragment_uniform_lines();
    assert!(uniforms.contains(&"uniform vec4 model_color;".to_string()));
    assert!(uniforms.contains(&"uniform float model_colorBlend;".to_string()));

    assert_eq!(render_resources.alpha_options.pass, Some(Pass::Opaque));

    assert_eq!(render_resources.uniform_map.model_color().unwrap(), Color::RED);
    assert_eq!(
        render_resources.uniform_map.model_color_blend().unwrap(),
        ColorBlendMode::Replace.get_color_blend(0.25)
    );
}

#[test]
fn test_configures_render_resources_for_translucent_color() {
    let mock_model = ModelColorModel::new(Color::RED.with_alpha(0.2), ColorBlendMode::Mix, 0.25);

    let mut shader_builder = ShaderBuilder::new();
    let mut uniform_map = ModelColorUniformMap::default();
    let mut alpha_options = ModelAlphaOptions::default();
    let mut render_state = RenderState::default();

    let mut render_resources = create_mock_render_resources(
        &mut shader_builder,
        &mut uniform_map,
        &mut alpha_options,
        &mut render_state,
    );

    ModelColorPipelineStage::process(&mut render_resources, &mock_model);

    let defines = render_resources.shader_builder.fragment_define_lines();
    assert!(defines.contains(&"HAS_MODEL_COLOR".to_string()));

    let uniforms = render_resources.shader_builder.fragment_uniform_lines();
    assert!(uniforms.contains(&"uniform vec4 model_color;".to_string()));
    assert!(uniforms.contains(&"uniform float model_colorBlend;".to_string()));

    assert_eq!(render_resources.alpha_options.pass, Some(Pass::Translucent));

    assert_eq!(
        render_resources.uniform_map.model_color().unwrap(),
        Color::RED.with_alpha(0.2)
    );
    assert_eq!(
        render_resources.uniform_map.model_color_blend().unwrap(),
        ColorBlendMode::Mix.get_color_blend(0.25)
    );
}

#[test]
fn test_configures_render_state_for_transparent_color() {
    let mock_model = ModelColorModel::new(Color::RED.with_alpha(0.0), ColorBlendMode::Mix, 0.25)
        .with_silhouette(false);

    let mut shader_builder = ShaderBuilder::new();
    let mut uniform_map = ModelColorUniformMap::default();
    let mut alpha_options = ModelAlphaOptions::default();
    let mut render_state = RenderState::default();

    let mut render_resources = create_mock_render_resources(
        &mut shader_builder,
        &mut uniform_map,
        &mut alpha_options,
        &mut render_state,
    );

    ModelColorPipelineStage::process(&mut render_resources, &mock_model);

    assert_eq!(
        render_resources.render_state.color_mask,
        ColorMaskState {
            red: false,
            green: false,
            blue: false,
            alpha: false,
        }
    );
}
