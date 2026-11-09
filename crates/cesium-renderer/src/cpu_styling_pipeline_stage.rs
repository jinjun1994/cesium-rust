//! CPUStylingPipelineStage matching CesiumJS `Scene/Model/CPUStylingPipelineStage.js` (Apache-2.0).

use crate::alpha_pipeline_stage::ModelAlphaOptions;
use crate::commands::Pass;
use crate::shader_builder::{ShaderBuilder, ShaderDestination};
use cesium_math::{Color, ColorBlendMode};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub const CPU_STYLING_STAGE_VS: &str = r#"
void filterByPassType(inout vec3 positionMC, vec4 featureColor)
{
    bool styleTranslucent = (featureColor.a != 1.0);
    if (czm_pass == czm_passTranslucent && !styleTranslucent && !model_commandTranslucent)
    {
        #ifdef HAS_SILHOUETTE
        positionMC *= float(model_silhouettePass);
        #else
        positionMC *= 0.0;
        #endif
    }
    else if (czm_pass != czm_passTranslucent && styleTranslucent)
    {
        positionMC *= 0.0;
    }
}

void cpuStylingStage(inout vec3 positionMC, inout SelectedFeature feature)
{
    float show = ceil(feature.color.a);
    positionMC *= show;

    #if defined(HAS_SELECTED_FEATURE_ID_ATTRIBUTE) && !defined(HAS_CLASSIFICATION)
    filterByPassType(positionMC, feature.color);
    #endif
}
"#;

pub const CPU_STYLING_STAGE_FS: &str = r#"
void filterByPassType(vec4 featureColor)
{
    bool styleTranslucent = (featureColor.a != 1.0);
    if (czm_pass == czm_passTranslucent && !styleTranslucent && !model_commandTranslucent)
    {   
        #ifdef HAS_SILHOUETTE
        if(!model_silhouettePass) {
            discard;
        }
        #else
        discard;
        #endif
    }
    else if (czm_pass != czm_passTranslucent && styleTranslucent)
    {
        discard;
    }
}

void cpuStylingStage(inout czm_modelMaterial material, SelectedFeature feature)
{
    vec4 featureColor = feature.color;
    if (featureColor.a == 0.0)
    {
        discard;
    }

    #if defined(HAS_SELECTED_FEATURE_ID_TEXTURE) && !defined(HAS_CLASSIFICATION)
    filterByPassType(featureColor);
    #endif

    featureColor = czm_gammaCorrect(featureColor);

    #ifdef HAS_CLASSIFICATION
    material.diffuse = featureColor.rgb * featureColor.a;
    #else
    float highlight = ceil(model_colorBlend);
    material.diffuse *= mix(featureColor.rgb, vec3(1.0), highlight);
    #endif
    
    material.alpha *= featureColor.a;
}
"#;

/// CPU styling model configuration.
#[derive(Clone, Debug, PartialEq)]
pub struct CPUStylingModelOptions {
    pub color: Option<Color>,
    pub color_blend_mode: ColorBlendMode,
    pub color_blend_amount: f64,
}

impl Default for CPUStylingModelOptions {
    fn default() -> Self {
        Self {
            color: None,
            color_blend_mode: ColorBlendMode::Highlight,
            color_blend_amount: 0.5,
        }
    }
}

/// Dynamic uniform callback function for CPU styling stage.
pub type CPUStylingUniformCallback = Box<dyn Fn() -> serde_json::Value + Send + Sync>;

/// Render resources for CPU styling stage.
pub struct CPUStylingRenderResources<'a> {
    pub alpha_options: Arc<Mutex<ModelAlphaOptions>>,
    pub model: &'a CPUStylingModelOptions,
    pub shader_builder: &'a mut ShaderBuilder,
    pub uniform_map: &'a mut HashMap<String, CPUStylingUniformCallback>,
}

/// Pipeline stage configuring runtime styling for feature colors.
pub struct CPUStylingPipelineStage;

impl CPUStylingPipelineStage {
    pub const NAME: &'static str = "CPUStylingPipelineStage";
    pub const COLOR_BLEND_UNIFORM_NAME: &'static str = "model_colorBlend";

    /// Processes render resources to inject styling shaders and uniforms.
    pub fn process(resources: &mut CPUStylingRenderResources<'_>) {
        let sb = &mut resources.shader_builder;

        sb.add_vertex_lines(&[CPU_STYLING_STAGE_VS]);
        sb.add_fragment_lines(&[CPU_STYLING_STAGE_FS]);
        sb.add_define("USE_CPU_STYLING", None, Some(ShaderDestination::BOTH));

        // If no static model.color is present, inject model_colorBlend
        if resources.model.color.is_none() {
            sb.add_uniform("float", Self::COLOR_BLEND_UNIFORM_NAME, Some(ShaderDestination::FRAGMENT));
            let mode = resources.model.color_blend_mode;
            let amount = resources.model.color_blend_amount;
            resources.uniform_map.insert(
                Self::COLOR_BLEND_UNIFORM_NAME.to_string(),
                Box::new(move || {
                    let blend = mode.get_color_blend(amount);
                    serde_json::json!(blend)
                }),
            );
        }

        sb.add_uniform("bool", "model_commandTranslucent", Some(ShaderDestination::BOTH));
        let alpha_opts = Arc::clone(&resources.alpha_options);
        resources.uniform_map.insert(
            "model_commandTranslucent".to_string(),
            Box::new(move || {
                let is_translucent = alpha_opts.lock().unwrap().pass == Some(Pass::Translucent);
                serde_json::json!(is_translucent)
            }),
        );
    }
}
