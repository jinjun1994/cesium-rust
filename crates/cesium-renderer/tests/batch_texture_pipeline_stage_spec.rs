use cesium_math::{Cartesian2, Cartesian4};
use cesium_renderer::{
    BatchTextureInfo, BatchTexturePipelineStage, BatchTextureRenderResources,
    BatchTextureUniformValue, ModelFeatureTableInfo, ShaderBuilder,
};
use std::collections::HashMap;

#[test]
fn test_sets_up_batch_textures_from_model() {
    let mut shader_builder = ShaderBuilder::new();
    let mut uniform_map = HashMap::new();
    let mut resources = BatchTextureRenderResources {
        shader_builder: &mut shader_builder,
        uniform_map: &mut uniform_map,
    };

    let feature_table = ModelFeatureTableInfo {
        features_length: 10,
        batch_texture: BatchTextureInfo {
            batch_texture: 0,
            texture_dimensions: Cartesian2::new(10.0, 2.0),
            texture_step: Cartesian4::new(0.5, 0.5, 0.0, 0.0),
        },
    };

    BatchTexturePipelineStage::process(&mut resources, &feature_table);

    // Verify shader defines & uniforms
    let vs = resources.shader_builder.create_vertex_shader_source();
    let fs = resources.shader_builder.create_fragment_shader_source();

    assert!(vs.defines.iter().any(|d| d == "MULTILINE_BATCH_TEXTURE"));
    assert!(fs.defines.iter().any(|d| d == "MULTILINE_BATCH_TEXTURE"));

    let vs_text = vs.sources.join("\n");
    let fs_text = fs.sources.join("\n");

    for uniform_line in [
        "uniform int model_featuresLength;",
        "uniform sampler2D model_batchTexture;",
        "uniform vec4 model_textureStep;",
        "uniform vec2 model_textureDimensions;",
    ] {
        assert!(vs_text.contains(uniform_line));
        assert!(fs_text.contains(uniform_line));
    }

    // Verify uniform map values
    assert_eq!(
        resources.uniform_map.get("model_featuresLength"),
        Some(&BatchTextureUniformValue::Int(10))
    );
    assert_eq!(
        resources.uniform_map.get("model_batchTexture"),
        Some(&BatchTextureUniformValue::Sampler(0))
    );
    assert_eq!(
        resources.uniform_map.get("model_textureStep"),
        Some(&BatchTextureUniformValue::Vec4(Cartesian4::new(0.5, 0.5, 0.0, 0.0)))
    );
    assert_eq!(
        resources.uniform_map.get("model_textureDimensions"),
        Some(&BatchTextureUniformValue::Vec2(Cartesian2::new(10.0, 2.0)))
    );
}
