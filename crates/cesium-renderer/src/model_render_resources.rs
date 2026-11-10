//! ModelRenderResources matching CesiumJS `Scene/Model/ModelRenderResources.js` (Apache-2.0).

use std::collections::HashMap;
use cesium_math::DeveloperError;

use crate::alpha_pipeline_stage::ModelAlphaOptions;
use crate::render_state::{DepthFunction, DepthTestState, RenderState};
use crate::shader_builder::ShaderBuilder;

/// Dynamic uniform callback function type.
pub type ModelUniformCallback = Box<dyn Fn() -> f64 + Send + Sync>;

/// Model render resources are for setting details that are consistent across
/// the entire model.
pub struct ModelRenderResources<M> {
    /// A reference to the model or model options.
    pub model: M,
    /// An object used to build a shader incrementally.
    pub shader_builder: ShaderBuilder,
    /// A dictionary mapping uniform name to callback functions.
    pub uniform_map: HashMap<String, ModelUniformCallback>,
    /// Options for configuring the alpha stage.
    pub alpha_options: ModelAlphaOptions,
    /// Render state options for draw command creation.
    pub render_state_options: RenderState,
    /// Whether the model has a silhouette.
    pub has_silhouette: bool,
    /// Whether the model is part of a tileset that uses skipLevelOfDetail.
    pub has_skip_level_of_detail: bool,
}

impl<M: std::fmt::Debug> std::fmt::Debug for ModelRenderResources<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelRenderResources")
            .field("model", &self.model)
            .field("render_state_options", &self.render_state_options)
            .field("alpha_options", &self.alpha_options)
            .field("has_silhouette", &self.has_silhouette)
            .field("has_skip_level_of_detail", &self.has_skip_level_of_detail)
            .finish()
    }
}

impl<M> ModelRenderResources<M> {
    /// Returns the default render state for model rendering with depth test enabled and LESS_OR_EQUAL.
    pub fn default_render_state() -> RenderState {
        RenderState {
            depth_test: DepthTestState {
                enabled: true,
                func: DepthFunction::LEqual,
            },
            ..Default::default()
        }
    }

    /// Attempts to construct a new `ModelRenderResources` instance, checking that `model` is provided.
    pub fn try_new(model: Option<M>) -> Result<Self, DeveloperError> {
        let model = model.ok_or_else(|| DeveloperError::new("model is required."))?;
        Ok(Self {
            model,
            shader_builder: ShaderBuilder::new(),
            uniform_map: HashMap::new(),
            alpha_options: ModelAlphaOptions::default(),
            render_state_options: Self::default_render_state(),
            has_silhouette: false,
            has_skip_level_of_detail: false,
        })
    }

    /// Constructs a new `ModelRenderResources` instance for the given model.
    pub fn new(model: M) -> Self {
        Self::try_new(Some(model)).expect("model cannot be None")
    }
}
