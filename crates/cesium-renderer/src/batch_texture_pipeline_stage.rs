//! BatchTexturePipelineStage matching CesiumJS `Scene/Model/BatchTexturePipelineStage.js` (Apache-2.0).

use crate::shader_builder::ShaderBuilder;
use cesium_math::{Cartesian2, Cartesian4};
use std::collections::HashMap;

/// Batch texture descriptor.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BatchTextureInfo {
    pub batch_texture: u32,
    pub texture_dimensions: Cartesian2,
    pub texture_step: Cartesian4,
}

/// Feature table metadata for batch texture rendering.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelFeatureTableInfo {
    pub features_length: i32,
    pub batch_texture: BatchTextureInfo,
}

/// Generic uniform value for batch texture stage.
#[derive(Clone, Debug, PartialEq)]
pub enum BatchTextureUniformValue {
    Int(i32),
    Sampler(u32),
    Vec2(Cartesian2),
    Vec4(Cartesian4),
}

/// Render resources for batch texture stage.
pub struct BatchTextureRenderResources<'a> {
    pub shader_builder: &'a mut ShaderBuilder,
    pub uniform_map: &'a mut HashMap<String, BatchTextureUniformValue>,
}

/// Pipeline stage for setting up batch textures and uniforms for models.
pub struct BatchTexturePipelineStage;

impl BatchTexturePipelineStage {
    pub const NAME: &'static str = "BatchTexturePipelineStage";

    /// Configures shader uniforms and defines for a primitive's batch texture.
    pub fn process(
        resources: &mut BatchTextureRenderResources<'_>,
        feature_table: &ModelFeatureTableInfo,
    ) {
        let sb = &mut resources.shader_builder;

        // Number of features in the feature table
        sb.add_uniform("int", "model_featuresLength", None);
        resources.uniform_map.insert(
            "model_featuresLength".to_string(),
            BatchTextureUniformValue::Int(feature_table.features_length),
        );

        // Batch texture sampler
        sb.add_uniform("sampler2D", "model_batchTexture", None);
        resources.uniform_map.insert(
            "model_batchTexture".to_string(),
            BatchTextureUniformValue::Sampler(feature_table.batch_texture.batch_texture),
        );

        // Batch texture step size
        sb.add_uniform("vec4", "model_textureStep", None);
        resources.uniform_map.insert(
            "model_textureStep".to_string(),
            BatchTextureUniformValue::Vec4(feature_table.batch_texture.texture_step),
        );

        // Batch texture dimensions (if multiline)
        if feature_table.batch_texture.texture_dimensions.y > 1.0 {
            sb.add_define("MULTILINE_BATCH_TEXTURE", None, None);
            sb.add_uniform("vec2", "model_textureDimensions", None);
            resources.uniform_map.insert(
                "model_textureDimensions".to_string(),
                BatchTextureUniformValue::Vec2(feature_table.batch_texture.texture_dimensions),
            );
        }
    }
}
