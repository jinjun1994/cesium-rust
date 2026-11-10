//! ModelSilhouettePipelineStage matching CesiumJS `Scene/Model/ModelSilhouettePipelineStage.js` (Apache-2.0).

use std::sync::atomic::{AtomicUsize, Ordering};
use cesium_math::Color;

use crate::shader_builder::{ShaderBuilder, ShaderDestination};

/// Vertex shader code for silhouette extrusion.
pub const MODEL_SILHOUETTE_STAGE_VS: &str = r#"void silhouetteStage(in ProcessedAttributes attributes, inout vec4 positionClip) {
     #ifdef HAS_NORMALS
     if(model_silhouettePass) {
          vec3 normal = normalize(czm_normal3D * attributes.normalMC);
          normal.x *= czm_projection[0][0];
          normal.y *= czm_projection[1][1];
          positionClip.xy += normal.xy * positionClip.w * model_silhouetteSize * czm_pixelRatio / czm_viewport.z;
    }
    #endif
}"#;

/// Fragment shader code for silhouette color output.
pub const MODEL_SILHOUETTE_STAGE_FS: &str = r#"void silhouetteStage(inout vec4 color) {
    if(model_silhouettePass) {
        color = czm_gammaCorrect(model_silhouetteColor);
    }
}"#;

static SILHOUETTES_LENGTH: AtomicUsize = AtomicUsize::new(0);

/// Model inputs for silhouette stage.
#[derive(Clone, Debug)]
pub struct ModelSilhouetteModel {
    pub silhouette_color: Color,
    pub silhouette_size: f32,
    pub silhouette_id: Option<usize>,
}

impl Default for ModelSilhouetteModel {
    fn default() -> Self {
        Self {
            silhouette_color: Color::RED,
            silhouette_size: 1.0,
            silhouette_id: None,
        }
    }
}

/// Render resources for silhouette pipeline stage.
pub struct ModelSilhouetteRenderResources<'a> {
    pub shader_builder: &'a mut ShaderBuilder,
    pub has_silhouette: bool,
}

/// The model silhouette pipeline stage is responsible for applying silhouettes to the model.
pub struct ModelSilhouettePipelineStage;

impl ModelSilhouettePipelineStage {
    pub const NAME: &'static str = "ModelSilhouettePipelineStage";

    /// Resets the global silhouettes counter (primarily for specs/tests).
    pub fn reset_silhouettes_length() {
        SILHOUETTES_LENGTH.store(0, Ordering::SeqCst);
    }

    /// Gets current silhouettes counter.
    pub fn silhouettes_length() -> usize {
        SILHOUETTES_LENGTH.load(Ordering::SeqCst)
    }

    /// Process a model and inject silhouette shader stages and uniforms.
    pub fn process(
        render_resources: &mut ModelSilhouetteRenderResources<'_>,
        model: &mut ModelSilhouetteModel,
    ) {
        if model.silhouette_id.is_none() {
            let next_id = SILHOUETTES_LENGTH.fetch_add(1, Ordering::SeqCst) + 1;
            model.silhouette_id = Some(next_id);
        }

        let shader_builder = &mut render_resources.shader_builder;
        shader_builder.add_define("HAS_SILHOUETTE", None, Some(ShaderDestination::BOTH));

        shader_builder.add_vertex_lines(&[MODEL_SILHOUETTE_STAGE_VS]);
        shader_builder.add_fragment_lines(&[MODEL_SILHOUETTE_STAGE_FS]);

        shader_builder.add_uniform(
            "vec4",
            "model_silhouetteColor",
            Some(ShaderDestination::FRAGMENT),
        );

        shader_builder.add_uniform(
            "float",
            "model_silhouetteSize",
            Some(ShaderDestination::VERTEX),
        );

        shader_builder.add_uniform(
            "bool",
            "model_silhouettePass",
            Some(ShaderDestination::BOTH),
        );

        render_resources.has_silhouette = true;
    }
}
