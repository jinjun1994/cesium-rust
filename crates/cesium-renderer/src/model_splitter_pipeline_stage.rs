//! ModelSplitterPipelineStage matching CesiumJS `Scene/Model/ModelSplitterPipelineStage.js` (Apache-2.0).

use std::collections::HashMap;
use cesium_math::SplitDirection;

use crate::model_render_resources::ModelUniformCallback;
use crate::shader_builder::{ShaderBuilder, ShaderDestination};

/// Fragment shader code for model splitting.
pub const MODEL_SPLITTER_STAGE_FS: &str = r#"void modelSplitterStage()
{
    // Don't split when rendering the shadow map, because it is rendered from
    // the perspective of a totally different camera.
#ifndef SHADOW_MAP
    if (model_splitDirection < 0.0 && gl_FragCoord.x > czm_splitPosition) discard;
    if (model_splitDirection > 0.0 && gl_FragCoord.x < czm_splitPosition) discard;
#endif
}"#;

/// Model configuration needed by ModelSplitterPipelineStage.
#[derive(Clone, Debug, Default)]
pub struct ModelSplitterModel {
    pub split_direction: SplitDirection,
}

/// Render resources for ModelSplitterPipelineStage.
pub struct ModelSplitterRenderResources<'a> {
    pub shader_builder: &'a mut ShaderBuilder,
    pub uniform_map: &'a mut HashMap<String, ModelUniformCallback>,
}

/// The model splitting pipeline stage is responsible for discarding fragments on the wrong side of the splitter.
pub struct ModelSplitterPipelineStage;

impl ModelSplitterPipelineStage {
    pub const NAME: &'static str = "ModelSplitterPipelineStage";
    pub const SPLIT_DIRECTION_UNIFORM_NAME: &'static str = "model_splitDirection";

    /// Process a model and inject splitter shader stages and uniforms.
    pub fn process(
        render_resources: &mut ModelSplitterRenderResources<'_>,
        model: &ModelSplitterModel,
    ) {
        let shader_builder = &mut render_resources.shader_builder;

        shader_builder.add_define(
            "HAS_MODEL_SPLITTER",
            None,
            Some(ShaderDestination::FRAGMENT),
        );

        shader_builder.add_fragment_lines(&[MODEL_SPLITTER_STAGE_FS]);

        shader_builder.add_uniform(
            "float",
            Self::SPLIT_DIRECTION_UNIFORM_NAME,
            Some(ShaderDestination::FRAGMENT),
        );

        let split_dir = model.split_direction as i32 as f64;
        render_resources.uniform_map.insert(
            Self::SPLIT_DIRECTION_UNIFORM_NAME.to_string(),
            Box::new(move || split_dir),
        );
    }
}
