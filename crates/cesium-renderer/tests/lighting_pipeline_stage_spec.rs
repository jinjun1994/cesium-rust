//! LightingPipelineStage spec tests matching CesiumJS `Scene/Model/LightingPipelineStageSpec.js`.

use std::collections::HashMap;

use cesium_math::Cartesian3;
use cesium_renderer::custom_shader::LightingModel;
use cesium_renderer::lighting_pipeline_stage::{
    LightingModelOptions, LightingPipelineStage, LightingRenderResources, ModelLightingOptions,
    SHADERS_LIGHTING_STAGE_FS,
};
use cesium_renderer::shader_builder::ShaderBuilder;

#[test]
fn test_supports_light_color() {
    let mock_model_with_light_color = LightingModelOptions {
        light_color: Some(Cartesian3::new(1.0, 0.0, 0.0)),
    };
    let options_unlit = ModelLightingOptions {
        lighting_model: LightingModel::Unlit,
    };

    let mut sb = ShaderBuilder::default();
    let mut uniform_map = HashMap::new();

    let mut rr = LightingRenderResources {
        model: &mock_model_with_light_color,
        lighting_options: &options_unlit,
        shader_builder: &mut sb,
        uniform_map: &mut uniform_map,
    };

    LightingPipelineStage::process(&mut rr, None);

    let vs = sb.create_vertex_shader_source();
    assert!(vs.defines.is_empty());
    assert!(sb.vertex_uniform_lines().is_empty());

    let fs = sb.create_fragment_shader_source();
    assert!(fs.defines.iter().any(|d| d == "USE_CUSTOM_LIGHT_COLOR"));
    assert!(fs.defines.iter().any(|d| d == "LIGHTING_UNLIT"));

    assert!(sb
        .fragment_uniform_lines()
        .iter()
        .any(|u| u.contains("uniform vec3 model_lightColorHdr;")));

    let fs_text = fs.sources.join("\n");
    assert!(fs_text.contains(SHADERS_LIGHTING_STAGE_FS));

    assert!(uniform_map.contains_key("model_lightColorHdr"));
    let color_val = uniform_map["model_lightColorHdr"]();
    assert_eq!(color_val, serde_json::json!([1.0, 0.0, 0.0]));
}

#[test]
fn test_supports_unlit_lighting() {
    let mock_model = LightingModelOptions::default();
    let options_unlit = ModelLightingOptions {
        lighting_model: LightingModel::Unlit,
    };

    let mut sb = ShaderBuilder::default();
    let mut uniform_map = HashMap::new();

    let mut rr = LightingRenderResources {
        model: &mock_model,
        lighting_options: &options_unlit,
        shader_builder: &mut sb,
        uniform_map: &mut uniform_map,
    };

    LightingPipelineStage::process(&mut rr, None);

    let vs = sb.create_vertex_shader_source();
    assert!(vs.defines.is_empty());
    assert!(sb.vertex_uniform_lines().is_empty());

    let fs = sb.create_fragment_shader_source();
    assert_eq!(fs.defines, vec!["LIGHTING_UNLIT".to_string()]);
    assert!(sb.fragment_uniform_lines().is_empty());

    let fs_text = fs.sources.join("\n");
    assert!(fs_text.contains(SHADERS_LIGHTING_STAGE_FS));
}

#[test]
fn test_supports_pbr_lighting() {
    let mock_model = LightingModelOptions::default();
    let options_pbr = ModelLightingOptions {
        lighting_model: LightingModel::Pbr,
    };

    let mut sb = ShaderBuilder::default();
    let mut uniform_map = HashMap::new();

    let mut rr = LightingRenderResources {
        model: &mock_model,
        lighting_options: &options_pbr,
        shader_builder: &mut sb,
        uniform_map: &mut uniform_map,
    };

    LightingPipelineStage::process(&mut rr, None);

    let vs = sb.create_vertex_shader_source();
    assert!(vs.defines.is_empty());
    assert!(sb.vertex_uniform_lines().is_empty());

    let fs = sb.create_fragment_shader_source();
    assert_eq!(fs.defines, vec!["LIGHTING_PBR".to_string()]);
    assert!(sb.fragment_uniform_lines().is_empty());

    let fs_text = fs.sources.join("\n");
    assert!(fs_text.contains(SHADERS_LIGHTING_STAGE_FS));
}
