use cesium_math::Color;
use cesium_renderer::{
    ModelSilhouetteModel, ModelSilhouettePipelineStage, ModelSilhouetteRenderResources,
    ShaderBuilder, MODEL_SILHOUETTE_STAGE_FS, MODEL_SILHOUETTE_STAGE_VS,
};

#[test]
fn test_configures_render_resources_for_silhouette() {
    ModelSilhouettePipelineStage::reset_silhouettes_length();
    assert_eq!(ModelSilhouettePipelineStage::silhouettes_length(), 0);

    let mut mock_model = ModelSilhouetteModel {
        silhouette_color: Color::RED,
        silhouette_size: 1.0,
        silhouette_id: None,
    };

    let mut shader_builder = ShaderBuilder::new();
    let mut render_resources = ModelSilhouetteRenderResources {
        shader_builder: &mut shader_builder,
        has_silhouette: false,
    };

    ModelSilhouettePipelineStage::process(&mut render_resources, &mut mock_model);

    assert_eq!(ModelSilhouettePipelineStage::silhouettes_length(), 1);
    assert_eq!(mock_model.silhouette_id, Some(1));

    // Check defines
    let vs_defines = render_resources.shader_builder.vertex_define_lines();
    assert!(vs_defines.iter().any(|d| d.contains("HAS_SILHOUETTE")));
    let fs_defines = render_resources.shader_builder.fragment_define_lines();
    assert!(fs_defines.iter().any(|d| d.contains("HAS_SILHOUETTE")));

    // Check uniforms
    let vs_uniforms = render_resources.shader_builder.vertex_uniform_lines();
    assert!(vs_uniforms.iter().any(|u| u.contains("model_silhouetteSize")));
    assert!(vs_uniforms.iter().any(|u| u.contains("model_silhouettePass")));

    let fs_uniforms = render_resources.shader_builder.fragment_uniform_lines();
    assert!(fs_uniforms.iter().any(|u| u.contains("model_silhouetteColor")));
    assert!(fs_uniforms.iter().any(|u| u.contains("model_silhouettePass")));

    // Check shader lines
    let vs_lines = render_resources.shader_builder.vertex_lines();
    assert!(vs_lines.iter().any(|line| line == MODEL_SILHOUETTE_STAGE_VS));

    let fs_lines = render_resources.shader_builder.fragment_lines();
    assert!(fs_lines.iter().any(|line| line == MODEL_SILHOUETTE_STAGE_FS));

    assert!(render_resources.has_silhouette);
}
