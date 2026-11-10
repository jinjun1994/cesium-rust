//! ImageBasedLightingPipelineStage matching CesiumJS `Scene/Model/ImageBasedLightingPipelineStage.js` (Apache-2.0).

use std::collections::HashMap;

use cesium_math::{Cartesian2, Cartesian3, DeveloperError, Matrix3};

use crate::image_based_lighting::ImageBasedLighting;
use crate::shader_builder::{ShaderBuilder, ShaderDestination};

/// FrameState context capabilities for IBL.
#[derive(Clone, Debug, Default)]
pub struct IblContext {
    pub floating_point_texture: bool,
    pub color_buffer_float: bool,
    pub half_floating_point_texture: bool,
    pub supports_texture_lod: bool,
}

impl IblContext {
    pub fn is_supported(&self) -> bool {
        self.floating_point_texture
            && (self.color_buffer_float || self.half_floating_point_texture)
            && self.supports_texture_lod
    }
}

/// Cube map representation for IBL testing and runtime.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpecularEnvironmentCubeMap {
    pub maximum_mipmap_level: f64,
    pub ready: bool,
    pub texture_id: Option<String>,
}

/// Environment map manager options for IBL.
#[derive(Clone, Debug, Default)]
pub struct IblEnvironmentMapManager {
    pub intensity: Option<f64>,
    pub radiance_cube_map: Option<String>,
    pub spherical_harmonic_coefficients: Option<Vec<Cartesian3>>,
    pub maximum_mipmap_level: Option<f64>,
}

/// Model options for ImageBasedLightingPipelineStage.
#[derive(Clone, Debug)]
pub struct IblModelOptions {
    pub image_based_lighting: ImageBasedLighting,
    pub environment_map_manager: IblEnvironmentMapManager,
    pub ibl_reference_frame_matrix: Matrix3,
    pub specular_environment_cube_map: Option<SpecularEnvironmentCubeMap>,
    pub use_default_specular_maps: bool,
}

impl Default for IblModelOptions {
    fn default() -> Self {
        Self {
            image_based_lighting: ImageBasedLighting::default(),
            environment_map_manager: IblEnvironmentMapManager::default(),
            ibl_reference_frame_matrix: Matrix3::IDENTITY,
            specular_environment_cube_map: None,
            use_default_specular_maps: false,
        }
    }
}

/// Callback type for uniforms.
pub type IblUniformCallback = Box<dyn Fn() -> serde_json::Value + Send + Sync>;

/// Render resources for ImageBasedLightingPipelineStage.
pub struct IblRenderResources<'a> {
    pub shader_builder: &'a mut ShaderBuilder,
    pub uniform_map: &'a mut HashMap<String, IblUniformCallback>,
    pub model: &'a IblModelOptions,
}

/// Pipeline stage that adds shader code, uniforms, and defines related to image-based lighting.
pub struct ImageBasedLightingPipelineStage;

impl ImageBasedLightingPipelineStage {
    pub const NAME: &'static str = "ImageBasedLightingPipelineStage";

    pub const IMAGE_BASED_LIGHTING_STAGE_FS: &'static str = r#"#ifdef DIFFUSE_IBL
vec3 sampleDiffuseEnvironment(vec3 cubeDir)
{
    #ifdef CUSTOM_SPHERICAL_HARMONICS
        return czm_sphericalHarmonics(cubeDir, model_sphericalHarmonicCoefficients); 
    #else
        return czm_sphericalHarmonics(cubeDir, czm_sphericalHarmonicCoefficients); 
    #endif
}
#endif

#ifdef SPECULAR_IBL
vec3 sampleSpecularEnvironment(vec3 cubeDir, float roughness)
{
    #ifdef CUSTOM_SPECULAR_IBL
        float lod = roughness * model_specularEnvironmentMapsMaximumLOD;
        return czm_textureCube(model_specularEnvironmentMaps, cubeDir, lod).rgb;
    #else
        float lod = roughness * czm_specularEnvironmentMapsMaximumLOD;
        return czm_textureCube(czm_specularEnvironmentMaps, cubeDir, lod).rgb;
    #endif
}
vec3 computeSpecularIBL(vec3 cubeDir, float NdotV, vec3 f0, float roughness)
{
    // see https://bruop.github.io/ibl/ at Single Scattering Results
    // Roughness dependent fresnel, from Fdez-Aguera
    vec3 f90 = max(vec3(1.0 - roughness), f0);
    vec3 F = fresnelSchlick2(f0, f90, NdotV);

    vec2 brdfLut = texture(czm_brdfLut, vec2(NdotV, roughness)).rg;
    vec3 specularSample = sampleSpecularEnvironment(cubeDir, roughness);

    return specularSample * (F * brdfLut.x + brdfLut.y);
}
#endif

#if defined(DIFFUSE_IBL) || defined(SPECULAR_IBL)
/**
 * Compute the light contributions from environment maps and spherical harmonic coefficients.
 * See Fdez-Aguera, https://www.jcgt.org/published/0008/01/03/paper.pdf, for explanation
 * of the single- and multi-scattering terms.
 *
 * @param {vec3} viewDirectionEC Unit vector pointing from the fragment to the eye position.
 * @param {vec3} normalEC The surface normal in eye coordinates.
 * @param {czm_modelMaterial} The material properties.
 * @return {vec3} The computed HDR color.
 */
vec3 textureIBL(vec3 viewDirectionEC, vec3 normalEC, czm_modelMaterial material) {
    vec3 f0 = material.specular;
    float roughness = material.roughness;
    float specularWeight = 1.0;
    #ifdef USE_SPECULAR
        specularWeight = material.specularWeight;
    #endif
    float NdotV = clamp(dot(normalEC, viewDirectionEC), 0.0, 1.0);

    // see https://bruop.github.io/ibl/ at Single Scattering Results
    // Roughness dependent fresnel, from Fdez-Aguera
    vec3 f90 = max(vec3(1.0 - roughness), f0);
    vec3 singleScatterFresnel = fresnelSchlick2(f0, f90, NdotV);

    vec2 brdfLut = texture(czm_brdfLut, vec2(NdotV, roughness)).rg;
    vec3 FssEss = specularWeight * (singleScatterFresnel * brdfLut.x + brdfLut.y);

    #ifdef DIFFUSE_IBL
        vec3 normalMC = normalize(model_iblReferenceFrameMatrix * normalEC);
        vec3 irradiance = sampleDiffuseEnvironment(normalMC);

        vec3 averageFresnel = f0 + (1.0 - f0) / 21.0;
        float Ems = specularWeight * (1.0 - brdfLut.x - brdfLut.y);
        vec3 FmsEms = FssEss * averageFresnel * Ems / (1.0 - averageFresnel * Ems);
        vec3 dielectricScattering = (1.0 - FssEss - FmsEms) * material.diffuse;
        vec3 diffuseContribution = irradiance * (FmsEms + dielectricScattering) * model_iblFactor.x;
    #else
        vec3 diffuseContribution = vec3(0.0);
    #endif

    #ifdef USE_ANISOTROPY
        // Bend normal to account for anisotropic distortion of specular reflection
        vec3 anisotropyDirection = material.anisotropicB;
        vec3 anisotropicTangent = cross(anisotropyDirection, viewDirectionEC);
        vec3 anisotropicNormal = cross(anisotropicTangent, anisotropyDirection);
        float bendFactor = 1.0 - material.anisotropyStrength * (1.0 - roughness);
        float bendFactorPow4 = bendFactor * bendFactor * bendFactor * bendFactor;
        vec3 bentNormal = normalize(mix(anisotropicNormal, normalEC, bendFactorPow4));
        vec3 reflectEC = reflect(-viewDirectionEC, bentNormal);
    #else
        vec3 reflectEC = reflect(-viewDirectionEC, normalEC);
    #endif

    #ifdef SPECULAR_IBL
        vec3 reflectMC = normalize(model_iblReferenceFrameMatrix * reflectEC);
        vec3 radiance = sampleSpecularEnvironment(reflectMC, roughness);
        vec3 specularContribution = radiance * FssEss * model_iblFactor.y;
    #else
        vec3 specularContribution = vec3(0.0);
    #endif

    return diffuseContribution + specularContribution;
}
#endif
"#;

    /// Configures render resources with defines, uniforms, and shader code for IBL.
    pub fn process(
        render_resources: &mut IblRenderResources<'_>,
        model: &IblModelOptions,
        context: &IblContext,
    ) -> Result<(), DeveloperError> {
        let image_based_lighting = &model.image_based_lighting;
        let environment_map_manager = &model.environment_map_manager;
        let shader_builder = &mut render_resources.shader_builder;

        let specular_environment_map_atlas =
            if image_based_lighting.specular_environment_maps().is_none() {
                environment_map_manager.radiance_cube_map.clone()
            } else {
                None
            };

        let spherical_harmonic_coefficients = image_based_lighting
            .spherical_harmonic_coefficients()
            .map(|s| s.to_vec())
            .or_else(|| environment_map_manager.spherical_harmonic_coefficients.clone());

        shader_builder.add_define(
            "USE_IBL_LIGHTING",
            None,
            Some(ShaderDestination::FRAGMENT),
        );
        shader_builder.add_uniform(
            "vec2",
            "model_iblFactor",
            Some(ShaderDestination::FRAGMENT),
        );

        if context.is_supported() {
            let add_matrix = spherical_harmonic_coefficients.is_some()
                || image_based_lighting.use_specular_environment_maps()
                || image_based_lighting.enabled();
            if add_matrix {
                shader_builder.add_uniform(
                    "mat3",
                    "model_iblReferenceFrameMatrix",
                    Some(ShaderDestination::FRAGMENT),
                );
            }

            if specular_environment_map_atlas.is_some() {
                shader_builder.add_define(
                    "COMPUTE_POSITION_WC_ATMOSPHERE",
                    None,
                    Some(ShaderDestination::BOTH),
                );
            }

            if let Some(ref sh) = spherical_harmonic_coefficients {
                if !sh.is_empty() {
                    shader_builder.add_define(
                        "DIFFUSE_IBL",
                        None,
                        Some(ShaderDestination::FRAGMENT),
                    );
                    shader_builder.add_define(
                        "CUSTOM_SPHERICAL_HARMONICS",
                        None,
                        Some(ShaderDestination::FRAGMENT),
                    );
                    shader_builder.add_uniform(
                        "vec3",
                        "model_sphericalHarmonicCoefficients[9]",
                        Some(ShaderDestination::FRAGMENT),
                    );
                }
            } else if image_based_lighting.use_default_spherical_harmonics() {
                shader_builder.add_define(
                    "DIFFUSE_IBL",
                    None,
                    Some(ShaderDestination::FRAGMENT),
                );
            }

            let has_specular_cube_map = model
                .specular_environment_cube_map
                .as_ref()
                .is_some_and(|cm| cm.ready)
                || specular_environment_map_atlas.is_some();

            if has_specular_cube_map {
                shader_builder.add_define(
                    "SPECULAR_IBL",
                    None,
                    Some(ShaderDestination::FRAGMENT),
                );
                shader_builder.add_define(
                    "CUSTOM_SPECULAR_IBL",
                    None,
                    Some(ShaderDestination::FRAGMENT),
                );
                shader_builder.add_uniform(
                    "samplerCube",
                    "model_specularEnvironmentMaps",
                    Some(ShaderDestination::FRAGMENT),
                );
                shader_builder.add_uniform(
                    "float",
                    "model_specularEnvironmentMapsMaximumLOD",
                    Some(ShaderDestination::FRAGMENT),
                );
            } else if model.use_default_specular_maps {
                shader_builder.add_define(
                    "SPECULAR_IBL",
                    None,
                    Some(ShaderDestination::FRAGMENT),
                );
            }
        }

        shader_builder.add_fragment_lines(&[Self::IMAGE_BASED_LIGHTING_STAGE_FS]);

        // Register uniforms
        let ibl_factor = image_based_lighting.image_based_lighting_factor();
        let intensity = environment_map_manager.intensity.unwrap_or(1.0);
        let factor = Cartesian2::new(ibl_factor.x * intensity, ibl_factor.y * intensity);
        render_resources.uniform_map.insert(
            "model_iblFactor".to_string(),
            Box::new(move || serde_json::json!([factor.x, factor.y])),
        );

        let ref_matrix = model.ibl_reference_frame_matrix;
        render_resources.uniform_map.insert(
            "model_iblReferenceFrameMatrix".to_string(),
            Box::new(move || {
                let mut arr = [0.0; 9];
                let _ = ref_matrix.pack(&mut arr, 0);
                serde_json::json!(arr)
            }),
        );

        if let Some(sh) = spherical_harmonic_coefficients {
            render_resources.uniform_map.insert(
                "model_sphericalHarmonicCoefficients".to_string(),
                Box::new(move || {
                    let arr: Vec<serde_json::Value> = sh
                        .iter()
                        .map(|c| serde_json::json!([c.x, c.y, c.z]))
                        .collect();
                    serde_json::Value::Array(arr)
                }),
            );
        }

        if let Some(ref cm) = model.specular_environment_cube_map {
            let tex_id = cm
                .texture_id
                .clone()
                .unwrap_or_else(|| "default_cube_map".to_string());
            render_resources.uniform_map.insert(
                "model_specularEnvironmentMaps".to_string(),
                Box::new(move || serde_json::json!(tex_id)),
            );

            let max_lod = cm.maximum_mipmap_level;
            render_resources.uniform_map.insert(
                "model_specularEnvironmentMapsMaximumLOD".to_string(),
                Box::new(move || serde_json::json!(max_lod)),
            );
        } else if let Some(ref atlas) = specular_environment_map_atlas {
            let atlas_id = atlas.clone();
            render_resources.uniform_map.insert(
                "model_specularEnvironmentMaps".to_string(),
                Box::new(move || serde_json::json!(atlas_id)),
            );
            let max_lod = environment_map_manager.maximum_mipmap_level.unwrap_or(0.0);
            render_resources.uniform_map.insert(
                "model_specularEnvironmentMapsMaximumLOD".to_string(),
                Box::new(move || serde_json::json!(max_lod)),
            );
        }

        Ok(())
    }
}
