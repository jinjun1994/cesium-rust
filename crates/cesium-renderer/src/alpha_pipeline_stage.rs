//! AlphaPipelineStage matching CesiumJS `Scene/Model/AlphaPipelineStage.js` (Apache-2.0).

use crate::commands::Pass;
use crate::render_state::{BlendingState, RenderState};
use crate::shader_builder::{ShaderBuilder, ShaderDestination};
use std::collections::HashMap;

/// Alpha options for model rendering.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelAlphaOptions {
    pub pass: Option<Pass>,
    pub alpha_cutoff: Option<f64>,
}

/// Dynamic uniform function type.
pub type UniformCallback = Box<dyn Fn() -> f64 + Send + Sync>;

/// Render resources configured by model pipeline stages.
pub struct AlphaRenderResources<'a> {
    pub opaque_pass: Pass,
    pub shader_builder: &'a mut ShaderBuilder,
    pub alpha_options: &'a mut ModelAlphaOptions,
    pub uniform_map: &'a mut HashMap<String, UniformCallback>,
    pub render_state: &'a mut RenderState,
}

/// Backward compatibility alias.
pub type ModelRenderResources<'a> = AlphaRenderResources<'a>;

/// Pipeline stage for configuring the alpha options for handling translucency.
pub struct AlphaPipelineStage;

impl AlphaPipelineStage {
    pub const NAME: &'static str = "AlphaPipelineStage";

    /// Configures render resources according to alpha options.
    pub fn process(resources: &mut AlphaRenderResources<'_>) {
        if resources.alpha_options.pass.is_none() {
            resources.alpha_options.pass = Some(resources.opaque_pass);
        }

        if resources.alpha_options.pass == Some(Pass::Translucent) {
            resources.render_state.cull.enabled = false;
            resources.render_state.depth_mask = false;
            resources.render_state.blending = BlendingState::ALPHA_BLEND;
        }

        if let Some(cutoff) = resources.alpha_options.alpha_cutoff {
            resources.shader_builder.add_define(
                "ALPHA_MODE_MASK",
                None,
                Some(ShaderDestination::FRAGMENT),
            );
            resources.shader_builder.add_uniform(
                "float",
                "u_alphaCutoff",
                Some(ShaderDestination::FRAGMENT),
            );
            resources.uniform_map.insert(
                "u_alphaCutoff".to_string(),
                Box::new(move || cutoff),
            );
        }
    }
}
