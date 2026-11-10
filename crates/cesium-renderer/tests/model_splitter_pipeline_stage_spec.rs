use std::collections::HashMap;
use cesium_math::SplitDirection;
use cesium_renderer::{
    ModelSplitterModel, ModelSplitterPipelineStage, ModelSplitterRenderResources, ShaderBuilder,
    MODEL_SPLITTER_STAGE_FS,
};

#[test]
fn test_configures_shader_for_model_splitter() {
    let model = ModelSplitterModel {
        split_direction: SplitDirection::Left,
    };

    let mut shader_builder = ShaderBuilder::new();
    let mut uniform_map = HashMap::new();

    let mut render_resources = ModelSplitterRenderResources {
        shader_builder: &mut shader_builder,
        uniform_map: &mut uniform_map,
    };

    ModelSplitterPipelineStage::process(&mut render_resources, &model);

    // Vertex defines should be empty
    assert_eq!(render_resources.shader_builder.vertex_define_lines().len(), 0);

    // Fragment defines should contain HAS_MODEL_SPLITTER
    let fs_defines = render_resources.shader_builder.fragment_define_lines();
    assert!(fs_defines.iter().any(|d| d.contains("HAS_MODEL_SPLITTER")));

    // Vertex uniforms should be empty
    assert_eq!(render_resources.shader_builder.vertex_uniform_lines().len(), 0);

    // Fragment uniforms should contain model_splitDirection
    let fs_uniforms = render_resources.shader_builder.fragment_uniform_lines();
    assert!(fs_uniforms
        .iter()
        .any(|u| u.contains("model_splitDirection")));

    // Vertex lines should be empty
    assert_eq!(render_resources.shader_builder.vertex_lines().len(), 0);

    // Fragment lines should contain MODEL_SPLITTER_STAGE_FS
    let fs_lines = render_resources.shader_builder.fragment_lines();
    assert_eq!(fs_lines, &[MODEL_SPLITTER_STAGE_FS.to_string()]);

    // Check uniform callback
    let cb = render_resources
        .uniform_map
        .get("model_splitDirection")
        .expect("model_splitDirection uniform callback should exist");
    let val = cb();
    assert_eq!(val, SplitDirection::Left as i32 as f64);
}
