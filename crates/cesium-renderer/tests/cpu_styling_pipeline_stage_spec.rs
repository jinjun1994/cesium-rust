use cesium_math::{Color, ColorBlendMode};
use cesium_renderer::{
    CPUStylingModelOptions, CPUStylingPipelineStage, CPUStylingRenderResources, ModelAlphaOptions,
    Pass, ShaderBuilder, CPU_STYLING_STAGE_FS, CPU_STYLING_STAGE_VS,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[test]
fn test_adds_shader_functions() {
    let mut shader_builder = ShaderBuilder::new();
    let alpha_options = Arc::new(Mutex::new(ModelAlphaOptions::default()));
    let model = CPUStylingModelOptions::default();
    let mut uniform_map = HashMap::new();

    let mut resources = CPUStylingRenderResources {
        alpha_options,
        model: &model,
        shader_builder: &mut shader_builder,
        uniform_map: &mut uniform_map,
    };

    CPUStylingPipelineStage::process(&mut resources);

    let vs = resources.shader_builder.create_vertex_shader_source();
    let fs = resources.shader_builder.create_fragment_shader_source();

    let vs_text = vs.sources.join("\n");
    let fs_text = fs.sources.join("\n");

    assert!(vs_text.contains(CPU_STYLING_STAGE_VS));
    assert!(fs_text.contains(CPU_STYLING_STAGE_FS));
}

#[test]
fn test_adds_color_blend_uniform() {
    let mut shader_builder = ShaderBuilder::new();
    let alpha_options = Arc::new(Mutex::new(ModelAlphaOptions::default()));
    let model = CPUStylingModelOptions {
        color: None,
        color_blend_mode: ColorBlendMode::Mix,
        color_blend_amount: 0.75,
    };
    let mut uniform_map = HashMap::new();

    let expected_color_blend = model
        .color_blend_mode
        .get_color_blend(model.color_blend_amount);

    let mut resources = CPUStylingRenderResources {
        alpha_options,
        model: &model,
        shader_builder: &mut shader_builder,
        uniform_map: &mut uniform_map,
    };

    CPUStylingPipelineStage::process(&mut resources);

    let fs = resources.shader_builder.create_fragment_shader_source();
    let fs_text = fs.sources.join("\n");

    assert!(fs_text.contains("uniform bool model_commandTranslucent;"));
    assert!(fs_text.contains("uniform float model_colorBlend;"));

    let color_blend_cb = resources
        .uniform_map
        .get("model_colorBlend")
        .expect("model_colorBlend uniform should exist");
    assert_eq!(
        color_blend_cb().as_f64().unwrap(),
        expected_color_blend
    );

    let translucent_cb = resources
        .uniform_map
        .get("model_commandTranslucent")
        .expect("model_commandTranslucent uniform should exist");
    assert!(!translucent_cb().as_bool().unwrap());
}

#[test]
fn test_does_not_add_color_blend_uniform_if_model_color_is_present() {
    let mut shader_builder = ShaderBuilder::new();
    let alpha_options = Arc::new(Mutex::new(ModelAlphaOptions::default()));
    let model = CPUStylingModelOptions {
        color: Some(Color::RED),
        color_blend_mode: ColorBlendMode::Mix,
        color_blend_amount: 0.75,
    };
    let mut uniform_map = HashMap::new();

    let mut resources = CPUStylingRenderResources {
        alpha_options,
        model: &model,
        shader_builder: &mut shader_builder,
        uniform_map: &mut uniform_map,
    };

    CPUStylingPipelineStage::process(&mut resources);

    let fs = resources.shader_builder.create_fragment_shader_source();
    let fs_text = fs.sources.join("\n");

    assert!(fs_text.contains("uniform bool model_commandTranslucent;"));
    assert!(!fs_text.contains("uniform float model_colorBlend;"));

    assert!(!resources.uniform_map.contains_key("model_colorBlend"));
    let translucent_cb = resources
        .uniform_map
        .get("model_commandTranslucent")
        .expect("model_commandTranslucent uniform should exist");
    assert!(!translucent_cb().as_bool().unwrap());
}

#[test]
fn test_adds_command_translucent_uniform() {
    let mut shader_builder = ShaderBuilder::new();
    let alpha_options = Arc::new(Mutex::new(ModelAlphaOptions {
        pass: Some(Pass::Translucent),
        alpha_cutoff: None,
    }));
    let model = CPUStylingModelOptions::default();
    let mut uniform_map = HashMap::new();

    let mut resources = CPUStylingRenderResources {
        alpha_options,
        model: &model,
        shader_builder: &mut shader_builder,
        uniform_map: &mut uniform_map,
    };

    CPUStylingPipelineStage::process(&mut resources);

    let fs = resources.shader_builder.create_fragment_shader_source();
    let fs_text = fs.sources.join("\n");

    assert!(fs_text.contains("uniform bool model_commandTranslucent;"));
    assert!(fs_text.contains("uniform float model_colorBlend;"));

    let translucent_cb = resources
        .uniform_map
        .get("model_commandTranslucent")
        .expect("model_commandTranslucent uniform should exist");
    assert!(translucent_cb().as_bool().unwrap());
}

#[test]
fn test_model_command_translucent_accounts_for_changes_from_later_pipeline_stages() {
    let mut shader_builder = ShaderBuilder::new();
    let alpha_options = Arc::new(Mutex::new(ModelAlphaOptions::default()));
    let model = CPUStylingModelOptions {
        color: Some(Color::RED),
        color_blend_mode: ColorBlendMode::Mix,
        color_blend_amount: 0.75,
    };
    let mut uniform_map = HashMap::new();

    let mut resources = CPUStylingRenderResources {
        alpha_options: Arc::clone(&alpha_options),
        model: &model,
        shader_builder: &mut shader_builder,
        uniform_map: &mut uniform_map,
    };

    CPUStylingPipelineStage::process(&mut resources);

    let translucent_cb = resources
        .uniform_map
        .get("model_commandTranslucent")
        .expect("model_commandTranslucent uniform should exist");
    assert!(!translucent_cb().as_bool().unwrap());

    // Simulate applying a custom shader with isTranslucent = true
    alpha_options.lock().unwrap().pass = Some(Pass::Translucent);

    assert!(translucent_cb().as_bool().unwrap());
}
