//! ModelColorPipelineStage matching CesiumJS `Scene/Model/ModelColorPipelineStage.js` (Apache-2.0).

use std::sync::Arc;

use cesium_math::{Color, ColorBlendMode};
use crate::alpha_pipeline_stage::ModelAlphaOptions;
use crate::commands::Pass;
use crate::render_state::{ColorMaskState, RenderState};
use crate::shader_builder::{ShaderBuilder, ShaderDestination};

pub const SHADERS_MODEL_COLOR_STAGE_FS: &str = r#"
void modelColorStage(inout czm_modelMaterial material)
{
    material.diffuse = mix(material.diffuse, model_color.rgb, model_colorBlend);
    float highlight = ceil(model_colorBlend);
    material.diffuse *= mix(model_color.rgb, vec3(1.0), highlight);
    material.alpha *= model_color.a;
}
"#;

pub type ColorGetter = Arc<dyn Fn() -> Color + Send + Sync>;
pub type ColorBlendGetter = Arc<dyn Fn() -> f64 + Send + Sync>;

/// Uniform map populated by `ModelColorPipelineStage`.
#[derive(Clone, Default)]
pub struct ModelColorUniformMap {
    pub model_color: Option<ColorGetter>,
    pub model_color_blend: Option<ColorBlendGetter>,
}

impl ModelColorUniformMap {
    pub fn model_color(&self) -> Option<Color> {
        self.model_color.as_ref().map(|cb| cb())
    }

    pub fn model_color_blend(&self) -> Option<f64> {
        self.model_color_blend.as_ref().map(|cb| cb())
    }
}

/// Render resources configured by `ModelColorPipelineStage`.
pub struct ModelColorRenderResources<'a> {
    pub shader_builder: &'a mut ShaderBuilder,
    pub uniform_map: &'a mut ModelColorUniformMap,
    pub alpha_options: &'a mut ModelAlphaOptions,
    pub render_state: &'a mut RenderState,
}

/// Model descriptor for `ModelColorPipelineStage`.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelColorModel {
    pub color: Color,
    pub color_blend_mode: ColorBlendMode,
    pub color_blend_amount: f64,
    pub has_silhouette: bool,
}

impl Default for ModelColorModel {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            color_blend_mode: ColorBlendMode::Highlight,
            color_blend_amount: 0.5,
            has_silhouette: false,
        }
    }
}

impl ModelColorModel {
    pub fn new(color: Color, color_blend_mode: ColorBlendMode, color_blend_amount: f64) -> Self {
        Self {
            color,
            color_blend_mode,
            color_blend_amount,
            has_silhouette: false,
        }
    }

    pub fn with_silhouette(mut self, has_silhouette: bool) -> Self {
        self.has_silhouette = has_silhouette;
        self
    }
}

/// Pipeline stage for applying a solid or blending color to a 3D model.
pub struct ModelColorPipelineStage;

impl ModelColorPipelineStage {
    pub const NAME: &'static str = "ModelColorPipelineStage";
    pub const COLOR_UNIFORM_NAME: &'static str = "model_color";
    pub const COLOR_BLEND_UNIFORM_NAME: &'static str = "model_colorBlend";

    /// Configures shader defines, uniforms, alpha options, and render state for model color.
    pub fn process(
        render_resources: &mut ModelColorRenderResources<'_>,
        model: &ModelColorModel,
    ) {
        let shader_builder = &mut render_resources.shader_builder;

        shader_builder.add_define(
            "HAS_MODEL_COLOR",
            None,
            Some(ShaderDestination::FRAGMENT),
        );
        shader_builder.add_fragment_lines(&[SHADERS_MODEL_COLOR_STAGE_FS]);

        let color = model.color;

        if color.alpha == 0.0 && !model.has_silhouette {
            render_resources.render_state.color_mask = ColorMaskState {
                red: false,
                green: false,
                blue: false,
                alpha: false,
            };
        }

        if color.alpha < 1.0 {
            render_resources.alpha_options.pass = Some(Pass::Translucent);
        }

        shader_builder.add_uniform(
            "vec4",
            Self::COLOR_UNIFORM_NAME,
            Some(ShaderDestination::FRAGMENT),
        );
        render_resources.uniform_map.model_color = Some(Arc::new(move || color));

        let blend = model.color_blend_mode.get_color_blend(model.color_blend_amount);

        shader_builder.add_uniform(
            "float",
            Self::COLOR_BLEND_UNIFORM_NAME,
            Some(ShaderDestination::FRAGMENT),
        );
        render_resources.uniform_map.model_color_blend = Some(Arc::new(move || blend));
    }
}
