//! DequantizationPipelineStage matching CesiumJS `Scene/Model/DequantizationPipelineStage.js` (Apache-2.0).

use crate::classification_pipeline_stage::VertexAttributeSemantic;
use crate::model_components::Quantization;
use crate::shader_builder::{ShaderBuilder, ShaderDestination};
use cesium_math::ClassificationType;
use std::collections::HashMap;

/// Attribute descriptor for DequantizationPipelineStage.
#[derive(Clone, Debug, PartialEq)]
pub struct DequantizationAttribute {
    pub variable_name: String,
    pub semantic: Option<String>,
    pub glsl_type: String,
    pub quantization: Option<Quantization>,
}

/// Primitive descriptor for DequantizationPipelineStage.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DequantizationPrimitive {
    pub attributes: Vec<DequantizationAttribute>,
}

/// Model options for DequantizationPipelineStage.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DequantizationModelOptions {
    pub classification_type: Option<ClassificationType>,
}

/// Uniform callback type for dequantization stage.
pub type DequantizationUniformCallback = Box<dyn Fn() -> serde_json::Value + Send + Sync>;

/// Render resources for DequantizationPipelineStage.
pub struct DequantizationRenderResources<'a> {
    pub shader_builder: &'a mut ShaderBuilder,
    pub uniform_map: &'a mut HashMap<String, DequantizationUniformCallback>,
    pub model: &'a DequantizationModelOptions,
}

/// Pipeline stage generating shader code to dequantize attributes in the vertex shader.
pub struct DequantizationPipelineStage;

impl DequantizationPipelineStage {
    pub const NAME: &'static str = "DequantizationPipelineStage";
    pub const FUNCTION_ID_DEQUANTIZATION_STAGE_VS: &'static str = "dequantizationStage";
    pub const FUNCTION_SIGNATURE_DEQUANTIZATION_STAGE_VS: &'static str =
        "void dequantizationStage(inout ProcessedAttributes attributes)";

    /// Processes a primitive with quantized attributes.
    pub fn process(
        render_resources: &mut DequantizationRenderResources<'_>,
        primitive: &DequantizationPrimitive,
    ) {
        let has_classification = render_resources.model.classification_type.is_some();

        render_resources.shader_builder.add_define(
            "USE_DEQUANTIZATION",
            None,
            Some(ShaderDestination::VERTEX),
        );

        render_resources.shader_builder.add_function(
            Self::FUNCTION_ID_DEQUANTIZATION_STAGE_VS,
            Self::FUNCTION_SIGNATURE_DEQUANTIZATION_STAGE_VS,
            ShaderDestination::VERTEX,
        );

        let mut lines = Vec::new();

        for attribute in &primitive.attributes {
            let Some(ref quantization) = attribute.quantization else {
                continue;
            };

            let is_position = attribute.semantic.as_deref() == Some(VertexAttributeSemantic::POSITION);
            let is_texcoord = attribute.semantic.as_deref() == Some(VertexAttributeSemantic::TEXCOORD);

            if has_classification && !is_position && !is_texcoord {
                continue;
            }

            let variable_name = &attribute.variable_name;

            if quantization.oct_encoded {
                let swizzle = if quantization.oct_encoded_zxy {
                    ".zxy"
                } else {
                    ".xyz"
                };
                let norm_range_name = format!("model_normalizationRange_{variable_name}");
                lines.push(format!(
                    "    attributes.{variable_name} = czm_octDecode(a_quantized_{variable_name}, {norm_range_name}){swizzle};"
                ));

                render_resources.shader_builder.add_uniform(
                    "float",
                    &norm_range_name,
                    Some(ShaderDestination::VERTEX),
                );

                let norm_range_val = quantization
                    .normalization_range
                    .as_ref()
                    .and_then(|r| r.first().copied())
                    .unwrap_or(0.0);

                render_resources.uniform_map.insert(
                    norm_range_name,
                    Box::new(move || serde_json::json!(norm_range_val)),
                );
            } else {
                let offset_name = format!("model_quantizedVolumeOffset_{variable_name}");
                let step_size_name = format!("model_quantizedVolumeStepSize_{variable_name}");

                let is_color = is_color_attribute(variable_name);
                let glsl_type = if is_color {
                    "vec4"
                } else {
                    attribute.glsl_type.as_str()
                };

                render_resources.shader_builder.add_uniform(
                    glsl_type,
                    &offset_name,
                    Some(ShaderDestination::VERTEX),
                );
                render_resources.shader_builder.add_uniform(
                    glsl_type,
                    &step_size_name,
                    Some(ShaderDestination::VERTEX),
                );

                lines.push(format!(
                    "    attributes.{variable_name} = {offset_name} + a_quantized_{variable_name} * {step_size_name};"
                ));

                let offset_vec = quantization.quantized_volume_offset.clone().unwrap_or_default();
                let step_vec = quantization.quantized_volume_step_size.clone().unwrap_or_default();

                let offset_val = if is_color {
                    promote_to_vec4(&offset_vec, 0.0)
                } else {
                    offset_vec
                };

                let step_val = if is_color {
                    promote_to_vec4(&step_vec, 1.0)
                } else {
                    step_vec
                };

                render_resources.uniform_map.insert(
                    offset_name,
                    Box::new(move || serde_json::json!(offset_val)),
                );
                render_resources.uniform_map.insert(
                    step_size_name,
                    Box::new(move || serde_json::json!(step_val)),
                );
            }
        }

        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        render_resources.shader_builder.add_function_lines(
            Self::FUNCTION_ID_DEQUANTIZATION_STAGE_VS,
            &line_refs,
        );
    }
}

fn is_color_attribute(name: &str) -> bool {
    if let Some(rest) = name.strip_prefix("color_") {
        rest.chars().all(|c| c.is_ascii_digit())
    } else {
        false
    }
}

fn promote_to_vec4(vec: &[f64], default_alpha: f64) -> Vec<f64> {
    if vec.len() >= 4 {
        return vec[..4].to_vec();
    }
    let mut result = vec.to_vec();
    while result.len() < 3 {
        result.push(0.0);
    }
    result.push(default_alpha);
    result
}
