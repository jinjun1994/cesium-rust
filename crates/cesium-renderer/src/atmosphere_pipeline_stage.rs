//! AtmospherePipelineStage matching CesiumJS `Scene/Model/AtmospherePipelineStage.js` (Apache-2.0).

use crate::shader_builder::{ShaderBuilder, ShaderDestination};
use cesium_math::Cartesian3;
use std::collections::HashMap;

pub const ATMOSPHERE_STAGE_VS: &str = r#"
void atmosphereStage(ProcessedAttributes attributes) {
    vec3 lightDirection = czm_getDynamicAtmosphereLightDirection(v_positionWC, czm_atmosphereDynamicLighting);

    czm_computeGroundAtmosphereScattering(
        v_positionWC,
        lightDirection,
        v_atmosphereRayleighColor,
        v_atmosphereMieColor,
        v_atmosphereOpacity
    );
}
"#;

pub const ATMOSPHERE_STAGE_FS: &str = r#"
vec2 nearestPointOnEllipseFast(vec2 pos, vec2 radii) {
    vec2 p = abs(pos);
    vec2 inverseRadii = 1.0 / radii;
    vec2 evoluteScale = (radii.x * radii.x - radii.y * radii.y) * vec2(1.0, -1.0) * inverseRadii;
    vec2 tTrigs = vec2(0.70710678118);
    vec2 v = radii * tTrigs;
    vec2 evolute = evoluteScale * tTrigs * tTrigs * tTrigs;
    vec2 q = normalize(p - evolute) * length(v - evolute);
    tTrigs = (q + evolute) * inverseRadii;
    tTrigs = normalize(clamp(tTrigs, 0.0, 1.0));
    v = radii * tTrigs;
    return v * sign(pos);
}

vec3 computeEllipsoidPositionWC(vec3 positionMC) {
    vec3 positionWC = (czm_model * vec4(positionMC, 1.0)).xyz;
    vec2 positionEllipse = vec2(length(positionWC.xy), positionWC.z);
    vec2 nearestPoint = nearestPointOnEllipseFast(positionEllipse, czm_ellipsoidRadii.xz);
    return vec3(nearestPoint.x * normalize(positionWC.xy), nearestPoint.y);
}

void applyFog(inout vec4 color, vec4 groundAtmosphereColor, vec3 lightDirection, float distanceToCamera) {
    vec3 fogColor = groundAtmosphereColor.rgb;
    const float NONE = 0.0;
    if (czm_atmosphereDynamicLighting != NONE) {
        float darken = clamp(dot(normalize(czm_viewerPositionWC), lightDirection), czm_fogMinimumBrightness, 1.0);
        fogColor *= darken;
    }
    #ifndef HDR
        fogColor.rgb = czm_pbrNeutralTonemapping(fogColor.rgb);
        fogColor.rgb = czm_inverseGamma(fogColor.rgb);
    #endif
    vec3 withFog = czm_fog(distanceToCamera, color.rgb, fogColor, czm_fogVisualDensityScalar);
    color = vec4(withFog, color.a);
}

void atmosphereStage(inout vec4 color, in ProcessedAttributes attributes) {
    vec3 rayleighColor;
    vec3 mieColor;
    float opacity;
    vec3 positionWC = attributes.positionWC;
    vec3 lightDirection = czm_getDynamicAtmosphereLightDirection(positionWC, czm_atmosphereDynamicLighting);
    rayleighColor = v_atmosphereRayleighColor;
    mieColor = v_atmosphereMieColor;
    opacity = v_atmosphereOpacity;
    const bool ignoreBlackPixels = true;
    rayleighColor = czm_applyHSBShift(rayleighColor, czm_atmosphereHsbShift, ignoreBlackPixels);
    mieColor = czm_applyHSBShift(mieColor, czm_atmosphereHsbShift, ignoreBlackPixels);
    vec4 groundAtmosphereColor = czm_computeAtmosphereColor(positionWC, lightDirection, rayleighColor, mieColor, opacity);
    if (u_isInFog) {
        float distanceToCamera = length(attributes.positionEC);
        applyFog(color, groundAtmosphereColor, lightDirection, distanceToCamera);
    }
}
"#;

/// Minimal model context for atmosphere processing.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AtmosphereModelInfo {
    pub bounding_sphere_center: Cartesian3,
}

/// Minimal frame state for atmosphere calculation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AtmosphereFrameState {
    pub camera_position_wc: Cartesian3,
    pub fog_density: f64,
    pub camera_in_space: bool,
}

/// Dynamic boolean callback uniform.
pub type BoolUniformCallback = Box<dyn Fn() -> bool + Send + Sync>;

/// Render resources for atmosphere pipeline stage.
pub struct AtmosphereRenderResources<'a> {
    pub shader_builder: &'a mut ShaderBuilder,
    pub uniform_map: &'a mut HashMap<String, BoolUniformCallback>,
}

/// Pipeline stage configuring atmospheric scattering and fog for models.
pub struct AtmospherePipelineStage;

impl AtmospherePipelineStage {
    pub const NAME: &'static str = "AtmospherePipelineStage";

    pub fn process(
        resources: &mut AtmosphereRenderResources<'_>,
        model: &AtmosphereModelInfo,
        frame_state: &AtmosphereFrameState,
    ) {
        let sb = &mut resources.shader_builder;

        sb.add_define("HAS_ATMOSPHERE", None, Some(ShaderDestination::BOTH));
        sb.add_define(
            "COMPUTE_POSITION_WC_ATMOSPHERE",
            None,
            Some(ShaderDestination::BOTH),
        );

        sb.add_varying("vec3", "v_atmosphereRayleighColor", None);
        sb.add_varying("vec3", "v_atmosphereMieColor", None);
        sb.add_varying("float", "v_atmosphereOpacity", None);

        sb.add_vertex_lines(&[ATMOSPHERE_STAGE_VS]);
        sb.add_fragment_lines(&[ATMOSPHERE_STAGE_FS]);

        sb.add_uniform("bool", "u_isInFog", Some(ShaderDestination::FRAGMENT));

        let camera_pos = frame_state.camera_position_wc;
        let center = model.bounding_sphere_center;
        let density = frame_state.fog_density;
        let in_space = frame_state.camera_in_space;

        resources.uniform_map.insert(
            "u_isInFog".to_string(),
            Box::new(move || {
                if in_space {
                    return false;
                }
                let distance = camera_pos.distance(&center);
                let scalar = distance * density;
                let fog = 1.0 - (-(scalar * scalar)).exp();
                fog > 1e-3
            }),
        );
    }
}
