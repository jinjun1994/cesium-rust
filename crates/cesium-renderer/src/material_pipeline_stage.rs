//! MaterialPipelineStage matching CesiumJS `Scene/Model/MaterialPipelineStage.js` (Apache-2.0).

use std::collections::HashMap;

use cesium_math::{
    BoundingSphere, Cartesian3, Cartesian4, ClassificationType, FixedFrameTransforms, Matrix3,
    Matrix4, PrimitiveType,
};

pub use crate::alpha_pipeline_stage::ModelAlphaOptions;
pub use crate::commands::Pass;
pub use crate::custom_shader::LightingModel;
pub use crate::lighting_pipeline_stage::ModelLightingOptions;
pub use crate::model_components::{
    AlphaMode, Anisotropy, Clearcoat, ConstantLod, FeatureId, LineStyle, Material,
    MetallicRoughness, ModelTextureReader, PlanarFill, Primitive, Specular, SpecularGlossiness,
};
pub use crate::render_state::RenderState;
pub use crate::shader_builder::{ShaderBuilder, ShaderDestination};
pub use crate::texture::Texture;

pub const SHADERS_MATERIAL_STAGE_FS: &str = r#"
// If the style color is white, it implies the feature has not been styled.
bool isDefaultStyleColor(vec3 color)
{
    return all(greaterThan(color, vec3(1.0 - czm_epsilon3)));
}

vec3 blend(vec3 sourceColor, vec3 styleColor, float styleColorBlend)
{
    vec3 blendColor = mix(sourceColor, styleColor, styleColorBlend);
    vec3 color = isDefaultStyleColor(styleColor.rgb) ? sourceColor : blendColor;
    return color;
}

#ifdef HAS_NORMAL_TEXTURE
vec2 getNormalTexCoords()
{
    vec2 texCoord = TEXCOORD_NORMAL;
    #ifdef HAS_NORMAL_TEXTURE_TRANSFORM
        texCoord = czm_computeTextureTransform(texCoord, u_normalTextureTransform);
    #endif
    return texCoord;
}
#endif

#if defined(HAS_NORMAL_TEXTURE) || defined(HAS_CLEARCOAT_NORMAL_TEXTURE)
vec3 computeTangent(in vec3 position, in vec2 normalTexCoords)
{
    vec2 tex_dx = dFdx(normalTexCoords);
    vec2 tex_dy = dFdy(normalTexCoords);
    float determinant = tex_dx.x * tex_dy.y - tex_dy.x * tex_dx.y;
    vec3 tangent = tex_dy.t * dFdx(position) - tex_dx.t * dFdy(position);
    return tangent / determinant;
}
#endif

#ifdef USE_ANISOTROPY
struct NormalInfo {
    vec3 tangent;
    vec3 bitangent;
    vec3 normal;
    vec3 geometryNormal;
};

NormalInfo getNormalInfo(ProcessedAttributes attributes)
{
    vec3 geometryNormal = attributes.normalEC;
    #ifdef HAS_NORMAL_TEXTURE
        vec2 normalTexCoords = getNormalTexCoords();
    #endif

    #ifdef HAS_BITANGENTS
        vec3 tangent = attributes.tangentEC;
        vec3 bitangent = attributes.bitangentEC;
    #else // Assume HAS_NORMAL_TEXTURE
        vec3 tangent = computeTangent(attributes.positionEC, normalTexCoords);
        tangent = normalize(tangent - geometryNormal * dot(geometryNormal, tangent));
        vec3 bitangent = normalize(cross(geometryNormal, tangent));
    #endif

    #ifdef HAS_NORMAL_TEXTURE
        mat3 tbn = mat3(tangent, bitangent, geometryNormal);
        
        vec3 normalSample;
        #if defined(HAS_NORMAL_CONSTANT_LOD) && defined(HAS_CONSTANT_LOD)
            #ifdef HAS_NORMAL_TEXTURE_TRANSFORM
                normalSample = constantLodTextureLookup(u_normalTexture, u_normalTextureConstantLodParams, u_normalTextureTransform).rgb;
            #else
                normalSample = constantLodTextureLookup(u_normalTexture, u_normalTextureConstantLodParams).rgb;
            #endif
        #else
            normalSample = texture(u_normalTexture, normalTexCoords).rgb;
        #endif
        
        normalSample = 2.0 * normalSample - 1.0;
        #ifdef HAS_NORMAL_TEXTURE_SCALE
            normalSample.xy *= u_normalTextureScale;
        #endif
        vec3 normal = normalize(tbn * normalSample);
    #else
        vec3 normal = geometryNormal;
    #endif

    #ifdef HAS_DOUBLE_SIDED_MATERIAL
        if (czm_backFacing()) {
            tangent *= -1.0;
            bitangent *= -1.0;
            normal *= -1.0;
            geometryNormal *= -1.0;
        }
    #endif

    NormalInfo normalInfo;
    normalInfo.tangent = tangent;
    normalInfo.bitangent = bitangent;
    normalInfo.normal = normal;
    normalInfo.geometryNormal = geometryNormal;

    return normalInfo;
}
#endif

#if defined(HAS_NORMAL_TEXTURE) && !defined(HAS_WIREFRAME)
vec3 getNormalFromTexture(ProcessedAttributes attributes, vec3 geometryNormal)
{
    vec2 normalTexCoords = getNormalTexCoords();

    // If HAS_BITANGENTS is set, then HAS_TANGENTS is also set
    #ifdef HAS_BITANGENTS
        vec3 t = attributes.tangentEC;
        vec3 b = attributes.bitangentEC;
    #else
        vec3 t = computeTangent(attributes.positionEC, normalTexCoords);
        t = normalize(t - geometryNormal * dot(geometryNormal, t));
        vec3 b = normalize(cross(geometryNormal, t));
    #endif

    mat3 tbn = mat3(t, b, geometryNormal);
    
    vec3 normalSample;
    #if defined(HAS_NORMAL_CONSTANT_LOD) && defined(HAS_CONSTANT_LOD)
        #ifdef HAS_NORMAL_TEXTURE_TRANSFORM
            normalSample = constantLodTextureLookup(u_normalTexture, u_normalTextureConstantLodParams, u_normalTextureTransform).rgb;
        #else
            normalSample = constantLodTextureLookup(u_normalTexture, u_normalTextureConstantLodParams).rgb;
        #endif
    #else
        normalSample = texture(u_normalTexture, normalTexCoords).rgb;
    #endif
    
    normalSample = 2.0 * normalSample - 1.0;
    #ifdef HAS_NORMAL_TEXTURE_SCALE
        normalSample.xy *= u_normalTextureScale;
    #endif
    return normalize(tbn * normalSample);
}
#endif

#ifdef HAS_CLEARCOAT_NORMAL_TEXTURE
vec3 getClearcoatNormalFromTexture(ProcessedAttributes attributes, vec3 geometryNormal)
{
    vec2 normalTexCoords = TEXCOORD_CLEARCOAT_NORMAL;
    #ifdef HAS_CLEARCOAT_NORMAL_TEXTURE_TRANSFORM
        normalTexCoords = vec2(u_clearcoatNormalTextureTransform * vec3(normalTexCoords, 1.0));
    #endif

    // If HAS_BITANGENTS is set, then HAS_TANGENTS is also set
    #ifdef HAS_BITANGENTS
        vec3 t = attributes.tangentEC;
        vec3 b = attributes.bitangentEC;
    #else
        vec3 t = computeTangent(attributes.positionEC, normalTexCoords);
        t = normalize(t - geometryNormal * dot(geometryNormal, t));
        vec3 b = normalize(cross(geometryNormal, t));
    #endif

    mat3 tbn = mat3(t, b, geometryNormal);
    vec3 normalSample = texture(u_clearcoatNormalTexture, normalTexCoords).rgb;
    normalSample = 2.0 * normalSample - 1.0;
    #ifdef HAS_CLEARCOAT_NORMAL_TEXTURE_SCALE
        normalSample.xy *= u_clearcoatNormalTextureScale;
    #endif
    return normalize(tbn * normalSample);
}
#endif

#ifdef HAS_NORMALS
vec3 computeNormal(ProcessedAttributes attributes)
{
    // Geometry normal. This is already normalized 
    vec3 normal = attributes.normalEC;

    #if defined(HAS_NORMAL_TEXTURE) && !defined(HAS_WIREFRAME)
        normal = getNormalFromTexture(attributes, normal);
    #endif

    #ifdef HAS_DOUBLE_SIDED_MATERIAL
        if (czm_backFacing()) {
            normal = -normal;
        }
    #endif

    return normal;
}
#endif

#ifdef HAS_BASE_COLOR_TEXTURE
vec4 getBaseColorFromTexture()
{
    vec2 baseColorTexCoords = TEXCOORD_BASE_COLOR;
    #ifdef HAS_BASE_COLOR_TEXTURE_TRANSFORM
        baseColorTexCoords = czm_computeTextureTransform(baseColorTexCoords, u_baseColorTextureTransform);
    #endif

    vec4 baseColorWithAlpha;
    #if defined(HAS_BASE_COLOR_CONSTANT_LOD) && defined(HAS_CONSTANT_LOD)
        #ifdef HAS_BASE_COLOR_TEXTURE_TRANSFORM
            baseColorWithAlpha = czm_srgbToLinear(constantLodTextureLookup(u_baseColorTexture, u_baseColorTextureConstantLodParams, u_baseColorTextureTransform));
        #else
            baseColorWithAlpha = czm_srgbToLinear(constantLodTextureLookup(u_baseColorTexture, u_baseColorTextureConstantLodParams));
        #endif
    #else
        baseColorWithAlpha = czm_srgbToLinear(texture(u_baseColorTexture, baseColorTexCoords));
    #endif

    #ifdef HAS_BASE_COLOR_FACTOR
        baseColorWithAlpha *= u_baseColorFactor;
    #endif

    return baseColorWithAlpha;
}
#endif

#ifdef HAS_EMISSIVE_TEXTURE
vec3 getEmissiveFromTexture()
{
    vec2 emissiveTexCoords = TEXCOORD_EMISSIVE;
    #ifdef HAS_EMISSIVE_TEXTURE_TRANSFORM
        emissiveTexCoords = czm_computeTextureTransform(emissiveTexCoords, u_emissiveTextureTransform);
    #endif

    vec3 emissive = czm_srgbToLinear(texture(u_emissiveTexture, emissiveTexCoords).rgb);
    #ifdef HAS_EMISSIVE_FACTOR
        emissive *= u_emissiveFactor;
    #endif

    return emissive;
}
#endif

#if defined(LIGHTING_PBR) && defined(USE_SPECULAR_GLOSSINESS)
void setSpecularGlossiness(inout czm_modelMaterial material)
{
    #ifdef HAS_SPECULAR_GLOSSINESS_TEXTURE
        vec2 specularGlossinessTexCoords = TEXCOORD_SPECULAR_GLOSSINESS;
        #ifdef HAS_SPECULAR_GLOSSINESS_TEXTURE_TRANSFORM
            specularGlossinessTexCoords = czm_computeTextureTransform(specularGlossinessTexCoords, u_specularGlossinessTextureTransform);
        #endif

        vec4 specularGlossiness = czm_srgbToLinear(texture(u_specularGlossinessTexture, specularGlossinessTexCoords));
        vec3 specular = specularGlossiness.rgb;
        float glossiness = specularGlossiness.a;
        #ifdef HAS_LEGACY_SPECULAR_FACTOR
            specular *= u_legacySpecularFactor;
        #endif

        #ifdef HAS_GLOSSINESS_FACTOR
            glossiness *= u_glossinessFactor;
        #endif
    #else
        #ifdef HAS_LEGACY_SPECULAR_FACTOR
            vec3 specular = clamp(u_legacySpecularFactor, vec3(0.0), vec3(1.0));
        #else
            vec3 specular = vec3(1.0);
        #endif

        #ifdef HAS_GLOSSINESS_FACTOR
            float glossiness = clamp(u_glossinessFactor, 0.0, 1.0);
        #else
            float glossiness = 1.0;
        #endif
    #endif

    #ifdef HAS_DIFFUSE_TEXTURE
        vec2 diffuseTexCoords = TEXCOORD_DIFFUSE;
        #ifdef HAS_DIFFUSE_TEXTURE_TRANSFORM
            diffuseTexCoords = czm_computeTextureTransform(diffuseTexCoords, u_diffuseTextureTransform);
        #endif

        vec4 diffuse = czm_srgbToLinear(texture(u_diffuseTexture, diffuseTexCoords));
        #ifdef HAS_DIFFUSE_FACTOR
            diffuse *= u_diffuseFactor;
        #endif
    #elif defined(HAS_DIFFUSE_FACTOR)
        vec4 diffuse = clamp(u_diffuseFactor, vec4(0.0), vec4(1.0));
    #else
        vec4 diffuse = vec4(1.0);
    #endif

    material.diffuse = diffuse.rgb * (1.0 - czm_maximumComponent(specular));
    // the specular glossiness extension's alpha overrides anything set
    // by the base material.
    material.alpha = diffuse.a;

    material.specular = specular;

    // glossiness is the opposite of roughness, but easier for artists to use.
    material.roughness = 1.0 - glossiness;
}
#elif defined(LIGHTING_PBR)
float setMetallicRoughness(inout czm_modelMaterial material)
{
    #ifdef HAS_METALLIC_ROUGHNESS_TEXTURE
        vec2 metallicRoughnessTexCoords = TEXCOORD_METALLIC_ROUGHNESS;
        #ifdef HAS_METALLIC_ROUGHNESS_TEXTURE_TRANSFORM
            metallicRoughnessTexCoords = czm_computeTextureTransform(metallicRoughnessTexCoords, u_metallicRoughnessTextureTransform);
        #endif

        vec3 metallicRoughness = texture(u_metallicRoughnessTexture, metallicRoughnessTexCoords).rgb;
        float metalness = clamp(metallicRoughness.b, 0.0, 1.0);
        float roughness = clamp(metallicRoughness.g, 0.0, 1.0);
        #ifdef HAS_METALLIC_FACTOR
            metalness = clamp(metalness * u_metallicFactor, 0.0, 1.0);
        #endif

        #ifdef HAS_ROUGHNESS_FACTOR
            roughness = clamp(roughness * u_roughnessFactor, 0.0, 1.0);
        #endif
    #else
        #ifdef HAS_METALLIC_FACTOR
            float metalness = clamp(u_metallicFactor, 0.0, 1.0);
        #else
            float metalness = 1.0;
        #endif

        #ifdef HAS_ROUGHNESS_FACTOR
            float roughness = clamp(u_roughnessFactor, 0.0, 1.0);
        #else
            float roughness = 1.0;
        #endif
    #endif

    // dielectrics use f0 = 0.04, metals use albedo as f0
    const vec3 REFLECTANCE_DIELECTRIC = vec3(0.04);
    vec3 f0 = mix(REFLECTANCE_DIELECTRIC, material.baseColor.rgb, metalness);

    material.specular = f0;

    // diffuse only applies to dielectrics.
    material.diffuse = mix(material.baseColor.rgb, vec3(0.0), metalness);

    // This is perceptual roughness. The square of this value is used for direct lighting
    material.roughness = roughness;

    return metalness;
}
#ifdef USE_SPECULAR
void setSpecular(inout czm_modelMaterial material, in float metalness)
{
    #ifdef HAS_SPECULAR_TEXTURE
        vec2 specularTexCoords = TEXCOORD_SPECULAR;
        #ifdef HAS_SPECULAR_TEXTURE_TRANSFORM
            specularTexCoords = czm_computeTextureTransform(specularTexCoords, u_specularTextureTransform);
        #endif
        float specularWeight = texture(u_specularTexture, specularTexCoords).a;
        #ifdef HAS_SPECULAR_FACTOR
            specularWeight *= u_specularFactor;
        #endif
    #else
        #ifdef HAS_SPECULAR_FACTOR
            float specularWeight = u_specularFactor;
        #else
            float specularWeight = 1.0;
        #endif
    #endif

    #ifdef HAS_SPECULAR_COLOR_TEXTURE
        vec2 specularColorTexCoords = TEXCOORD_SPECULAR_COLOR;
        #ifdef HAS_SPECULAR_COLOR_TEXTURE_TRANSFORM
            specularColorTexCoords = czm_computeTextureTransform(specularColorTexCoords, u_specularColorTextureTransform);
        #endif
        vec3 specularColorSample = texture(u_specularColorTexture, specularColorTexCoords).rgb;
        vec3 specularColorFactor = czm_srgbToLinear(specularColorSample);
        #ifdef HAS_SPECULAR_COLOR_FACTOR
            specularColorFactor *= u_specularColorFactor;
        #endif
    #else
        #ifdef HAS_SPECULAR_COLOR_FACTOR
            vec3 specularColorFactor = u_specularColorFactor;
        #else
            vec3 specularColorFactor = vec3(1.0);
        #endif
    #endif
    material.specularWeight = specularWeight;
    vec3 f0 = material.specular;
    vec3 dielectricSpecularF0 = min(f0 * specularColorFactor, vec3(1.0));
    material.specular = mix(dielectricSpecularF0, material.baseColor.rgb, metalness);
}
#endif
#ifdef USE_ANISOTROPY
void setAnisotropy(inout czm_modelMaterial material, in NormalInfo normalInfo)
{
    mat2 rotation = mat2(u_anisotropy.xy, -u_anisotropy.y, u_anisotropy.x);
    float anisotropyStrength = u_anisotropy.z;

    vec2 direction = vec2(1.0, 0.0);
    #ifdef HAS_ANISOTROPY_TEXTURE
        vec2 anisotropyTexCoords = TEXCOORD_ANISOTROPY;
        #ifdef HAS_ANISOTROPY_TEXTURE_TRANSFORM
            anisotropyTexCoords = czm_computeTextureTransform(anisotropyTexCoords, u_anisotropyTextureTransform);
        #endif
        vec3 anisotropySample = texture(u_anisotropyTexture, anisotropyTexCoords).rgb;
        direction = anisotropySample.rg * 2.0 - vec2(1.0);
        anisotropyStrength *= anisotropySample.b;
    #endif

    direction = rotation * direction;
    mat3 tbn = mat3(normalInfo.tangent, normalInfo.bitangent, normalInfo.normal);
    vec3 anisotropicT = tbn * normalize(vec3(direction, 0.0));
    vec3 anisotropicB = cross(normalInfo.geometryNormal, anisotropicT);

    material.anisotropicT = anisotropicT;
    material.anisotropicB = anisotropicB;
    material.anisotropyStrength = anisotropyStrength;
}
#endif
#ifdef USE_CLEARCOAT
void setClearcoat(inout czm_modelMaterial material, in ProcessedAttributes attributes)
{
    #ifdef HAS_CLEARCOAT_TEXTURE
        vec2 clearcoatTexCoords = TEXCOORD_CLEARCOAT;
        #ifdef HAS_CLEARCOAT_TEXTURE_TRANSFORM
            clearcoatTexCoords = czm_computeTextureTransform(clearcoatTexCoords, u_clearcoatTextureTransform);
        #endif
        float clearcoatFactor = texture(u_clearcoatTexture, clearcoatTexCoords).r;
        #ifdef HAS_CLEARCOAT_FACTOR
            clearcoatFactor *= u_clearcoatFactor;
        #endif
    #else
        #ifdef HAS_CLEARCOAT_FACTOR
            float clearcoatFactor = u_clearcoatFactor;
        #else
            // PERFORMANCE_IDEA: this case should turn the whole extension off
            float clearcoatFactor = 0.0;
        #endif
    #endif

    #ifdef HAS_CLEARCOAT_ROUGHNESS_TEXTURE
        vec2 clearcoatRoughnessTexCoords = TEXCOORD_CLEARCOAT_ROUGHNESS;
        #ifdef HAS_CLEARCOAT_ROUGHNESS_TEXTURE_TRANSFORM
            clearcoatRoughnessTexCoords = czm_computeTextureTransform(clearcoatRoughnessTexCoords, u_clearcoatRoughnessTextureTransform);
        #endif
        float clearcoatRoughness = texture(u_clearcoatRoughnessTexture, clearcoatRoughnessTexCoords).g;
        #ifdef HAS_CLEARCOAT_ROUGHNESS_FACTOR
            clearcoatRoughness *= u_clearcoatRoughnessFactor;
        #endif
    #else
        #ifdef HAS_CLEARCOAT_ROUGHNESS_FACTOR
            float clearcoatRoughness = u_clearcoatRoughnessFactor;
        #else
            float clearcoatRoughness = 0.0;
        #endif
    #endif

    material.clearcoatFactor = clearcoatFactor;
    // This is perceptual roughness. The square of this value is used for direct lighting
    material.clearcoatRoughness = clearcoatRoughness;
    #ifdef HAS_CLEARCOAT_NORMAL_TEXTURE
        material.clearcoatNormal = getClearcoatNormalFromTexture(attributes, attributes.normalEC);
    #else
        material.clearcoatNormal = attributes.normalEC;
    #endif
}
#endif
#endif

void materialStage(inout czm_modelMaterial material, ProcessedAttributes attributes, SelectedFeature feature)
{
    #ifdef USE_ANISOTROPY
        NormalInfo normalInfo = getNormalInfo(attributes);
        material.normalEC = normalInfo.normal;
    #elif defined(HAS_NORMALS)
        material.normalEC = computeNormal(attributes);
    #endif

    vec4 baseColorWithAlpha = vec4(1.0);
    // Regardless of whether we use PBR, set a base color.
    // HAS_BACKGROUND_FILL (from BENTLEY_materials_planar_fill) overrides with
    // the view's background color to create an invisible masking polygon.
    // The background color is in sRGB, so convert to linear to match the
    // material pipeline's expected color space.
    #ifdef HAS_BACKGROUND_FILL
        baseColorWithAlpha = czm_srgbToLinear(czm_backgroundColor);
    #elif defined(HAS_BASE_COLOR_TEXTURE)
        baseColorWithAlpha = getBaseColorFromTexture();
    #elif defined(HAS_BASE_COLOR_FACTOR)
        baseColorWithAlpha = u_baseColorFactor;
    #endif

    #ifdef HAS_IMAGERY
    baseColorWithAlpha = blendBaseColorWithImagery(baseColorWithAlpha);
    #endif // HAS_IMAGERY

    #ifdef HAS_POINT_CLOUD_COLOR_STYLE
        baseColorWithAlpha = v_pointCloudColor;
    #elif defined(HAS_COLOR_0)
        vec4 color = attributes.color_0;
        // .pnts files store colors in the sRGB color space
        #ifdef HAS_SRGB_COLOR
            color = czm_srgbToLinear(color);
        #endif
        baseColorWithAlpha *= color;
    #endif

    #ifdef USE_CPU_STYLING
        baseColorWithAlpha.rgb = blend(baseColorWithAlpha.rgb, feature.color.rgb, model_colorBlend);
    #endif
    material.baseColor = baseColorWithAlpha;
    material.diffuse = baseColorWithAlpha.rgb;
    material.alpha = baseColorWithAlpha.a;

    #ifdef HAS_OCCLUSION_TEXTURE
        vec2 occlusionTexCoords = TEXCOORD_OCCLUSION;
        #ifdef HAS_OCCLUSION_TEXTURE_TRANSFORM
            occlusionTexCoords = czm_computeTextureTransform(occlusionTexCoords, u_occlusionTextureTransform);
        #endif
        material.occlusion = texture(u_occlusionTexture, occlusionTexCoords).r;
    #endif

    #ifdef HAS_EMISSIVE_TEXTURE
        material.emissive = getEmissiveFromTexture();
    #elif defined(HAS_EMISSIVE_FACTOR)
        material.emissive = u_emissiveFactor;
    #endif

    #if defined(LIGHTING_PBR) && defined(USE_SPECULAR_GLOSSINESS)
        setSpecularGlossiness(material);
    #elif defined(LIGHTING_PBR)
        float metalness = setMetallicRoughness(material);
        #ifdef USE_SPECULAR
            setSpecular(material, metalness);
        #endif
        #ifdef USE_ANISOTROPY
            setAnisotropy(material, normalInfo);
        #endif
        #ifdef USE_CLEARCOAT
            setClearcoat(material, attributes);
        #endif
    #endif
}
"#;

pub const SHADERS_CONSTANT_LOD_STAGE_VS: &str = r#"#ifdef HAS_CONSTANT_LOD
// Extract model scale to compensate for minimumPixelSize scaling
float modelScaleX = length(czm_model[0].xyz);
float modelScaleY = length(czm_model[1].xyz);
float modelScaleZ = length(czm_model[2].xyz);
float modelScale = (modelScaleX + modelScaleY + modelScaleZ) / 3.0;

// Transform model position through ENU but as direction only (w=0) to avoid position-dependent rotation
vec3 enuDir = (u_constantLodWorldToEnu * czm_model * vec4(v_positionMC, 0.0)).xyz;
v_constantLodUvCustom.xy = (enuDir.yx + u_constantLodOffset) / modelScale;
v_constantLodUvCustom.z = u_constantLodDistance / modelScale;
#endif"#;

pub const SHADERS_CONSTANT_LOD_STAGE_FS: &str = r#"#ifdef HAS_CONSTANT_LOD

vec4 constantLodTextureLookup(sampler2D textureSampler, vec3 constantLodParams) {
    bool atMaxClamp = v_constantLodUvCustom.z >= constantLodParams.y;
    bool atMinClamp = v_constantLodUvCustom.z <= constantLodParams.x;
    bool atClampBoundary = atMaxClamp || atMinClamp;
    
    float effectiveDistance = atMaxClamp ? constantLodParams.y : 
                              (atMinClamp ? constantLodParams.x : v_constantLodUvCustom.z);
    
    float logDepth = log2(effectiveDistance);
    logDepth = clamp(logDepth, -10.0, 20.0);
    
    float f = fract(logDepth);
    float p = floor(logDepth);
    
    if (atClampBoundary) {
        float clampedP = ceil(logDepth);
        vec2 tc = v_constantLodUvCustom.xy / pow(2.0, clampedP) * constantLodParams.z;
        return texture(textureSampler, tc);
    }

    vec2 tc1 = v_constantLodUvCustom.xy / pow(2.0, p) * constantLodParams.z;
    vec2 tc2 = v_constantLodUvCustom.xy / pow(2.0, p + 1.0) * constantLodParams.z;
    return mix(texture(textureSampler, tc1), texture(textureSampler, tc2), f);
}

vec4 constantLodTextureLookup(sampler2D textureSampler, vec3 constantLodParams, mat3 textureTransform) {
    bool atMaxClamp = v_constantLodUvCustom.z >= constantLodParams.y;
    bool atMinClamp = v_constantLodUvCustom.z <= constantLodParams.x;
    bool atClampBoundary = atMaxClamp || atMinClamp;
    
    float effectiveDistance = atMaxClamp ? constantLodParams.y : 
                              (atMinClamp ? constantLodParams.x : v_constantLodUvCustom.z);
    
    float logDepth = log2(effectiveDistance);
    logDepth = clamp(logDepth, -10.0, 20.0);
    
    float f = fract(logDepth);
    float p = floor(logDepth);
    
    if (atClampBoundary) {
        float clampedP = ceil(logDepth);
        vec2 tc = v_constantLodUvCustom.xy / pow(2.0, clampedP) * constantLodParams.z;
        // Apply texture transform to the final texture coordinates
        tc = czm_computeTextureTransform(tc, textureTransform);
        return texture(textureSampler, tc);
    }

    vec2 tc1 = v_constantLodUvCustom.xy / pow(2.0, p) * constantLodParams.z;
    vec2 tc2 = v_constantLodUvCustom.xy / pow(2.0, p + 1.0) * constantLodParams.z;
    
    // Apply texture transform to both LOD texture coordinates before mixing
    tc1 = czm_computeTextureTransform(tc1, textureTransform);
    tc2 = czm_computeTextureTransform(tc2, textureTransform);
    
    return mix(texture(textureSampler, tc1), texture(textureSampler, tc2), f);
}

#endif"#;

/// Point cloud shading options matching CesiumJS `PointCloudShading`.
#[derive(Debug, Clone, PartialEq)]
pub struct PointCloudShading {
    pub normal_shading: bool,
    pub back_face_culling: bool,
    pub attenuation: bool,
    pub geometric_error_scale: f64,
    pub maximum_attenuation: f64,
    pub base_resolution: f64,
}

impl Default for PointCloudShading {
    fn default() -> Self {
        Self {
            normal_shading: false,
            back_face_culling: false,
            attenuation: false,
            geometric_error_scale: 1.0,
            maximum_attenuation: 0.0,
            base_resolution: 0.0,
        }
    }
}

/// Dynamic uniform values for material pipeline stage.
#[derive(Debug, Clone, PartialEq)]
pub enum MaterialUniformValue {
    Float(f64),
    Int(i64),
    Bool(bool),
    Vec2([f64; 2]),
    Vec3([f64; 3]),
    Vec4([f64; 4]),
    Mat3([f64; 9]),
    Mat4([f64; 16]),
    Texture(Option<Texture>),
    String(String),
}

impl MaterialUniformValue {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float(v) => Some(*v),
            Self::Int(v) => Some(*v as f64),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_vec3(&self) -> Option<Cartesian3> {
        match self {
            Self::Vec3(arr) => Some(Cartesian3::new(arr[0], arr[1], arr[2])),
            _ => None,
        }
    }

    pub fn as_vec4(&self) -> Option<Cartesian4> {
        match self {
            Self::Vec4(arr) => Some(Cartesian4::new(arr[0], arr[1], arr[2], arr[3])),
            _ => None,
        }
    }

    pub fn as_mat3(&self) -> Option<Matrix3> {
        match self {
            Self::Mat3(arr) => Some(Matrix3::from_column_major_array(*arr)),
            _ => None,
        }
    }

    pub fn as_texture(&self) -> Option<&Option<Texture>> {
        match self {
            Self::Texture(tex) => Some(tex),
            _ => None,
        }
    }
}

pub type MaterialUniformCallback = Box<dyn Fn() -> MaterialUniformValue>;

/// Camera frustum options.
#[derive(Debug, Clone, Default)]
pub struct MaterialFrustum {
    pub left: Option<f64>,
    pub right: Option<f64>,
    pub width: Option<f64>,
}

/// Frame state camera.
#[derive(Debug, Clone, Default)]
pub struct MaterialFrameStateCamera {
    pub position_wc: Cartesian3,
    pub frustum: MaterialFrustum,
}

/// Frame state context.
#[derive(Debug, Clone, Default)]
pub struct MaterialFrameStateContext {
    pub default_texture: Option<Texture>,
    pub default_normal_texture: Option<Texture>,
    pub default_emissive_texture: Option<Texture>,
    pub drawing_buffer_width: f64,
}

/// Frame state passed to the material pipeline stage.
#[derive(Debug, Clone, Default)]
pub struct MaterialFrameState {
    pub context: MaterialFrameStateContext,
    pub pixel_ratio: f64,
    pub camera: Option<MaterialFrameStateCamera>,
    pub planar_fill_requested: Option<bool>,
}

/// Model options relevant to material pipeline stage.
#[derive(Debug, Clone, Default)]
pub struct MaterialModelOptions {
    pub classification_type: Option<ClassificationType>,
    pub point_cloud_shading: Option<PointCloudShading>,
    pub back_face_culling: bool,
    pub bounding_sphere: Option<BoundingSphere>,
}

/// Render resources for primitive material configuration.
pub struct MaterialRenderResources<'a> {
    pub shader_builder: &'a mut ShaderBuilder,
    pub uniform_map: &'a mut HashMap<String, MaterialUniformCallback>,
    pub lighting_options: &'a mut ModelLightingOptions,
    pub alpha_options: &'a mut ModelAlphaOptions,
    pub render_state_options: &'a mut RenderState,
    pub model: &'a mut MaterialModelOptions,
    pub planar_fill_id_pass: Option<bool>,
}

/// Pipeline stage for processing primitive materials.
pub struct MaterialPipelineStage;

impl MaterialPipelineStage {
    pub const NAME: &'static str = "MaterialPipelineStage";

    /// Process a primitive's material and configure the render resources.
    pub fn process(
        render_resources: &mut MaterialRenderResources<'_>,
        primitive: &Primitive,
        frame_state: &mut MaterialFrameState,
    ) {
        let material = primitive.material.clone().unwrap_or_default();

        // 1. u_constantLodDistance uniform fallback
        if !render_resources
            .uniform_map
            .contains_key("u_constantLodDistance")
        {
            let bounding_sphere = render_resources.model.bounding_sphere;
            let camera = frame_state.camera.clone();
            render_resources.uniform_map.insert(
                "u_constantLodDistance".to_string(),
                Box::new(move || {
                    if let (Some(bs), Some(cam)) = (bounding_sphere, &camera) {
                        let center = Cartesian3::new(bs.center.x, bs.center.y, bs.center.z);
                        let distance = Cartesian3::distance(&cam.position_wc, &center);
                        MaterialUniformValue::Float(distance.max(0.1))
                    } else {
                        MaterialUniformValue::Float(1.0)
                    }
                }),
            );
        }

        let has_classification = render_resources.model.classification_type.is_some();
        let disable_textures = has_classification;

        let default_texture = frame_state.context.default_texture.clone();
        let default_normal_texture = frame_state.context.default_normal_texture.clone();
        let default_emissive_texture = frame_state.context.default_emissive_texture.clone();

        Self::process_material_uniforms(
            &material,
            render_resources.uniform_map,
            render_resources.shader_builder,
            default_texture.clone(),
            default_normal_texture,
            default_emissive_texture,
            disable_textures,
            render_resources.model.bounding_sphere,
        );

        if let Some(spec_gloss) = &material.specular_glossiness {
            Self::process_specular_glossiness_uniforms(
                spec_gloss,
                render_resources.uniform_map,
                render_resources.shader_builder,
                default_texture,
                disable_textures,
                render_resources.model.bounding_sphere,
            );
        } else {
            if let Some(specular) = &material.specular {
                Self::process_specular_uniforms(
                    specular,
                    render_resources.uniform_map,
                    render_resources.shader_builder,
                    default_texture.clone(),
                    disable_textures,
                    render_resources.model.bounding_sphere,
                );
            }
            if let Some(anisotropy) = &material.anisotropy {
                Self::process_anisotropy_uniforms(
                    anisotropy,
                    render_resources.uniform_map,
                    render_resources.shader_builder,
                    default_texture.clone(),
                    disable_textures,
                    render_resources.model.bounding_sphere,
                );
            }
            if let Some(clearcoat) = &material.clearcoat {
                Self::process_clearcoat_uniforms(
                    clearcoat,
                    render_resources.uniform_map,
                    render_resources.shader_builder,
                    default_texture.clone(),
                    disable_textures,
                    render_resources.model.bounding_sphere,
                );
            }
            let metallic_roughness = material.metallic_roughness.clone().unwrap_or_default();
            Self::process_metallic_roughness_uniforms(
                &metallic_roughness,
                render_resources.uniform_map,
                render_resources.shader_builder,
                default_texture,
                disable_textures,
                render_resources.model.bounding_sphere,
            );
        }

        let has_normals = primitive
            .attributes
            .iter()
            .any(|a| a.semantic.as_deref() == Some("NORMAL"));

        let disable_point_cloud_normals = render_resources
            .model
            .point_cloud_shading
            .as_ref()
            .map(|pcs| !pcs.normal_shading)
            .unwrap_or(false);

        if let Some(planar_fill) = &material.planar_fill {
            Self::process_planar_fill(
                planar_fill,
                primitive,
                render_resources.uniform_map,
                render_resources.shader_builder,
                &mut render_resources.planar_fill_id_pass,
                frame_state,
            );
        }

        let has_background_fill = material
            .planar_fill
            .as_ref()
            .map(|pf| pf.background_fill)
            .unwrap_or(false);

        if material.unlit
            || !has_normals
            || has_classification
            || disable_point_cloud_normals
            || has_background_fill
        {
            render_resources.lighting_options.lighting_model = LightingModel::Unlit;
        } else {
            render_resources.lighting_options.lighting_model = LightingModel::Pbr;
        }

        let cull = render_resources.model.back_face_culling && !material.double_sided;
        render_resources.render_state_options.cull.enabled = cull;

        match material.alpha_mode {
            AlphaMode::Blend => {
                render_resources.alpha_options.pass = Some(Pass::Translucent);
            }
            AlphaMode::Mask => {
                render_resources.alpha_options.alpha_cutoff = Some(material.alpha_cutoff as f64);
            }
            AlphaMode::Opaque => {}
        }

        if let Some(point_diameter) = material.point_diameter {
            render_resources.shader_builder.add_define(
                "HAS_POINT_DIAMETER",
                None,
                Some(ShaderDestination::BOTH),
            );
            render_resources.shader_builder.add_uniform(
                "float",
                "u_pointDiameter",
                Some(ShaderDestination::VERTEX),
            );
            let pixel_ratio = frame_state.pixel_ratio;
            render_resources.uniform_map.insert(
                "u_pointDiameter".to_string(),
                Box::new(move || MaterialUniformValue::Float(point_diameter as f64 * pixel_ratio)),
            );
        }

        if let Some(line_style) = &material.line_style {
            Self::process_line_style_uniforms(
                line_style,
                primitive,
                frame_state,
                render_resources.uniform_map,
                render_resources.shader_builder,
            );
        }

        render_resources
            .shader_builder
            .add_fragment_lines(&[SHADERS_MATERIAL_STAGE_FS]);

        if material.double_sided {
            render_resources.shader_builder.add_define(
                "HAS_DOUBLE_SIDED_MATERIAL",
                None,
                Some(ShaderDestination::BOTH),
            );
        }
    }

    /// Process texture transform matrix.
    pub fn process_texture_transform(
        shader_builder: &mut ShaderBuilder,
        uniform_map: &mut HashMap<String, MaterialUniformCallback>,
        texture_reader: &ModelTextureReader,
        uniform_name: &str,
        define_name: &str,
    ) {
        let transform_define = format!("HAS_{define_name}_TEXTURE_TRANSFORM");
        shader_builder.add_define(
            &transform_define,
            None,
            Some(ShaderDestination::FRAGMENT),
        );

        let transform_uniform_name = format!("{uniform_name}Transform");
        shader_builder.add_uniform(
            "mat3",
            &transform_uniform_name,
            Some(ShaderDestination::FRAGMENT),
        );

        let transform = texture_reader.transform.unwrap_or(Matrix3::IDENTITY);
        uniform_map.insert(
            transform_uniform_name,
            Box::new(move || MaterialUniformValue::Mat3(transform.values)),
        );
    }

    /// Process texture coordinate scale.
    pub fn process_texture_scale(
        shader_builder: &mut ShaderBuilder,
        uniform_map: &mut HashMap<String, MaterialUniformCallback>,
        texture_reader: &ModelTextureReader,
        uniform_name: &str,
        define_name: &str,
    ) {
        let scale_define = format!("HAS_{define_name}_TEXTURE_SCALE");
        shader_builder.add_define(
            &scale_define,
            None,
            Some(ShaderDestination::FRAGMENT),
        );

        let scale_uniform_name = format!("{uniform_name}Scale");
        shader_builder.add_uniform(
            "float",
            &scale_uniform_name,
            Some(ShaderDestination::FRAGMENT),
        );

        let scale = texture_reader.scale.unwrap_or(1.0);
        uniform_map.insert(
            scale_uniform_name,
            Box::new(move || MaterialUniformValue::Float(scale)),
        );
    }

    /// Process a texture reader and add it to the shader builder and uniform map.
    pub fn process_texture(
        shader_builder: &mut ShaderBuilder,
        uniform_map: &mut HashMap<String, MaterialUniformCallback>,
        texture_reader: &ModelTextureReader,
        uniform_name: &str,
        define_name: &str,
        default_texture: Option<Texture>,
        bounding_sphere: Option<BoundingSphere>,
    ) {
        shader_builder.add_uniform(
            "sampler2D",
            uniform_name,
            Some(ShaderDestination::FRAGMENT),
        );

        let texture = texture_reader.texture.clone().or(default_texture);
        uniform_map.insert(
            uniform_name.to_string(),
            Box::new(move || MaterialUniformValue::Texture(texture.clone())),
        );

        let texture_define = format!("HAS_{define_name}_TEXTURE");
        shader_builder.add_define(
            &texture_define,
            None,
            Some(ShaderDestination::FRAGMENT),
        );

        let tex_coord_index = texture_reader.tex_coord;
        let tex_coord_varying = format!("v_texCoord_{tex_coord_index}");
        let tex_coord_define = format!("TEXCOORD_{define_name}");
        shader_builder.add_define(
            &tex_coord_define,
            Some(&tex_coord_varying),
            Some(ShaderDestination::FRAGMENT),
        );

        if let Some(transform) = texture_reader.transform {
            if transform != Matrix3::IDENTITY {
                Self::process_texture_transform(
                    shader_builder,
                    uniform_map,
                    texture_reader,
                    uniform_name,
                    define_name,
                );
            }
        }

        if let Some(scale) = texture_reader.scale {
            if (scale - 1.0).abs() > f64::EPSILON {
                Self::process_texture_scale(
                    shader_builder,
                    uniform_map,
                    texture_reader,
                    uniform_name,
                    define_name,
                );
            }
        }

        Self::process_constant_lod(
            shader_builder,
            uniform_map,
            texture_reader,
            uniform_name,
            define_name,
            bounding_sphere,
        );
    }

    fn process_constant_lod(
        shader_builder: &mut ShaderBuilder,
        uniform_map: &mut HashMap<String, MaterialUniformCallback>,
        texture_reader: &ModelTextureReader,
        uniform_name: &str,
        define_name: &str,
        bounding_sphere: Option<BoundingSphere>,
    ) {
        let constant_lod = match &texture_reader.constant_lod {
            Some(lod) => lod,
            None => return,
        };

        let constant_lod_define = format!("HAS_{define_name}_CONSTANT_LOD");
        shader_builder.add_define(
            &constant_lod_define,
            None,
            Some(ShaderDestination::BOTH),
        );

        if !uniform_map.contains_key("u_constantLodOffset") {
            shader_builder.add_define("HAS_CONSTANT_LOD", None, Some(ShaderDestination::BOTH));
            shader_builder.add_varying("vec3", "v_constantLodUvCustom", None);
            shader_builder.add_uniform(
                "vec2",
                "u_constantLodOffset",
                Some(ShaderDestination::VERTEX),
            );
            shader_builder.add_uniform(
                "float",
                "u_constantLodDistance",
                Some(ShaderDestination::VERTEX),
            );
            shader_builder.add_uniform(
                "mat4",
                "u_constantLodWorldToEnu",
                Some(ShaderDestination::VERTEX),
            );

            shader_builder.add_fragment_lines(&[SHADERS_CONSTANT_LOD_STAGE_FS]);

            let constant_lod_lines: Vec<&str> = SHADERS_CONSTANT_LOD_STAGE_VS
                .lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    !trimmed.starts_with("//") || trimmed.contains('#')
                })
                .collect();
            shader_builder.add_function_lines("setDynamicVaryingsVS", &constant_lod_lines);
        }

        let params_uniform_name = format!("{uniform_name}ConstantLodParams");
        shader_builder.add_uniform(
            "vec3",
            &params_uniform_name,
            Some(ShaderDestination::FRAGMENT),
        );

        let min_clamp = constant_lod.min_clamp_distance as f64;
        let max_clamp = constant_lod.max_clamp_distance as f64;
        let repetitions = constant_lod.repetitions as f64;
        uniform_map.insert(
            params_uniform_name,
            Box::new(move || MaterialUniformValue::Vec3([min_clamp, max_clamp, repetitions])),
        );

        if !uniform_map.contains_key("u_constantLodOffset") {
            let offset = constant_lod.offset;
            uniform_map.insert(
                "u_constantLodOffset".to_string(),
                Box::new(move || MaterialUniformValue::Vec2([offset.x, offset.y])),
            );

            let mut enu_matrix_inverse = Matrix4::IDENTITY;
            if let Some(bs) = bounding_sphere {
                let model_center = Cartesian3::new(bs.center.x, bs.center.y, bs.center.z);
                if let Ok(enu_frame) =
                    FixedFrameTransforms::east_north_up_to_fixed_frame(model_center, None)
                {
                    enu_matrix_inverse = enu_frame.inverse().unwrap_or(Matrix4::IDENTITY);
                }
            }
            uniform_map.insert(
                "u_constantLodWorldToEnu".to_string(),
                Box::new(move || MaterialUniformValue::Mat4(enu_matrix_inverse.values)),
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn process_material_uniforms(
        material: &Material,
        uniform_map: &mut HashMap<String, MaterialUniformCallback>,
        shader_builder: &mut ShaderBuilder,
        default_texture: Option<Texture>,
        default_normal_texture: Option<Texture>,
        default_emissive_texture: Option<Texture>,
        disable_textures: bool,
        bounding_sphere: Option<BoundingSphere>,
    ) {
        let emissive_factor = material.emissive_factor;
        if emissive_factor != Cartesian3::ZERO {
            shader_builder.add_uniform(
                "vec3",
                "u_emissiveFactor",
                Some(ShaderDestination::FRAGMENT),
            );
            uniform_map.insert(
                "u_emissiveFactor".to_string(),
                Box::new(move || {
                    MaterialUniformValue::Vec3([emissive_factor.x, emissive_factor.y, emissive_factor.z])
                }),
            );
            shader_builder.add_define(
                "HAS_EMISSIVE_FACTOR",
                None,
                Some(ShaderDestination::FRAGMENT),
            );

            if let Some(emissive_texture) = &material.emissive_texture {
                if !disable_textures {
                    Self::process_texture(
                        shader_builder,
                        uniform_map,
                        emissive_texture,
                        "u_emissiveTexture",
                        "EMISSIVE",
                        default_emissive_texture,
                        bounding_sphere,
                    );
                }
            }
        }

        let mut base_color_constant_lod = None;
        if let Some(mr) = &material.metallic_roughness {
            if let Some(bc_tex) = &mr.base_color_texture {
                if let Some(lod) = &bc_tex.constant_lod {
                    base_color_constant_lod = Some(lod.clone());
                }
            }
        } else if let Some(sg) = &material.specular_glossiness {
            if let Some(diff_tex) = &sg.diffuse_texture {
                if let Some(lod) = &diff_tex.constant_lod {
                    base_color_constant_lod = Some(lod.clone());
                }
            }
        }

        if let Some(normal_texture) = &material.normal_texture {
            if !disable_textures {
                let mut normal_to_process = normal_texture.clone();
                if normal_to_process.constant_lod.is_some() && base_color_constant_lod.is_some() {
                    normal_to_process.constant_lod = base_color_constant_lod;
                }
                Self::process_texture(
                    shader_builder,
                    uniform_map,
                    &normal_to_process,
                    "u_normalTexture",
                    "NORMAL",
                    default_normal_texture,
                    bounding_sphere,
                );
            }
        }

        if let Some(occlusion_texture) = &material.occlusion_texture {
            if !disable_textures {
                Self::process_texture(
                    shader_builder,
                    uniform_map,
                    occlusion_texture,
                    "u_occlusionTexture",
                    "OCCLUSION",
                    default_texture,
                    bounding_sphere,
                );
            }
        }
    }

    fn process_specular_glossiness_uniforms(
        specular_glossiness: &SpecularGlossiness,
        uniform_map: &mut HashMap<String, MaterialUniformCallback>,
        shader_builder: &mut ShaderBuilder,
        default_texture: Option<Texture>,
        disable_textures: bool,
        bounding_sphere: Option<BoundingSphere>,
    ) {
        shader_builder.add_define(
            "USE_SPECULAR_GLOSSINESS",
            None,
            Some(ShaderDestination::FRAGMENT),
        );

        if let Some(diffuse_texture) = &specular_glossiness.diffuse_texture {
            if !disable_textures {
                Self::process_texture(
                    shader_builder,
                    uniform_map,
                    diffuse_texture,
                    "u_diffuseTexture",
                    "DIFFUSE",
                    default_texture.clone(),
                    bounding_sphere,
                );
            }
        }

        let diffuse_factor = specular_glossiness.diffuse_factor;
        let default_diffuse = Cartesian4::new(1.0, 1.0, 1.0, 1.0);
        if diffuse_factor != default_diffuse {
            shader_builder.add_uniform(
                "vec4",
                "u_diffuseFactor",
                Some(ShaderDestination::FRAGMENT),
            );
            uniform_map.insert(
                "u_diffuseFactor".to_string(),
                Box::new(move || {
                    MaterialUniformValue::Vec4([
                        diffuse_factor.x,
                        diffuse_factor.y,
                        diffuse_factor.z,
                        diffuse_factor.w,
                    ])
                }),
            );
            shader_builder.add_define(
                "HAS_DIFFUSE_FACTOR",
                None,
                Some(ShaderDestination::FRAGMENT),
            );
        }

        if let Some(spec_gloss_tex) = &specular_glossiness.specular_glossiness_texture {
            if !disable_textures {
                Self::process_texture(
                    shader_builder,
                    uniform_map,
                    spec_gloss_tex,
                    "u_specularGlossinessTexture",
                    "SPECULAR_GLOSSINESS",
                    default_texture,
                    bounding_sphere,
                );
            }
        }

        let specular_factor = specular_glossiness.specular_factor;
        let default_specular = Cartesian3::new(1.0, 1.0, 1.0);
        if specular_factor != default_specular {
            shader_builder.add_uniform(
                "vec3",
                "u_legacySpecularFactor",
                Some(ShaderDestination::FRAGMENT),
            );
            uniform_map.insert(
                "u_legacySpecularFactor".to_string(),
                Box::new(move || {
                    MaterialUniformValue::Vec3([specular_factor.x, specular_factor.y, specular_factor.z])
                }),
            );
            shader_builder.add_define(
                "HAS_LEGACY_SPECULAR_FACTOR",
                None,
                Some(ShaderDestination::FRAGMENT),
            );
        }

        let glossiness_factor = specular_glossiness.glossiness_factor;
        if (glossiness_factor - 1.0).abs() > f32::EPSILON {
            shader_builder.add_uniform(
                "float",
                "u_glossinessFactor",
                Some(ShaderDestination::FRAGMENT),
            );
            uniform_map.insert(
                "u_glossinessFactor".to_string(),
                Box::new(move || MaterialUniformValue::Float(glossiness_factor as f64)),
            );
            shader_builder.add_define(
                "HAS_GLOSSINESS_FACTOR",
                None,
                Some(ShaderDestination::FRAGMENT),
            );
        }
    }

    fn process_specular_uniforms(
        specular: &Specular,
        uniform_map: &mut HashMap<String, MaterialUniformCallback>,
        shader_builder: &mut ShaderBuilder,
        default_texture: Option<Texture>,
        disable_textures: bool,
        bounding_sphere: Option<BoundingSphere>,
    ) {
        shader_builder.add_define(
            "USE_SPECULAR",
            None,
            Some(ShaderDestination::FRAGMENT),
        );

        if let Some(specular_texture) = &specular.specular_texture {
            if !disable_textures {
                Self::process_texture(
                    shader_builder,
                    uniform_map,
                    specular_texture,
                    "u_specularTexture",
                    "SPECULAR",
                    default_texture.clone(),
                    bounding_sphere,
                );
            }
        }

        let specular_factor = specular.specular_factor;
        if (specular_factor - 1.0).abs() > f32::EPSILON {
            shader_builder.add_uniform(
                "float",
                "u_specularFactor",
                Some(ShaderDestination::FRAGMENT),
            );
            uniform_map.insert(
                "u_specularFactor".to_string(),
                Box::new(move || MaterialUniformValue::Float(specular_factor as f64)),
            );
            shader_builder.add_define(
                "HAS_SPECULAR_FACTOR",
                None,
                Some(ShaderDestination::FRAGMENT),
            );
        }

        if let Some(specular_color_texture) = &specular.specular_color_texture {
            if !disable_textures {
                Self::process_texture(
                    shader_builder,
                    uniform_map,
                    specular_color_texture,
                    "u_specularColorTexture",
                    "SPECULAR_COLOR",
                    default_texture,
                    bounding_sphere,
                );
            }
        }

        let specular_color_factor = specular.specular_color_factor;
        let default_color = Cartesian3::new(1.0, 1.0, 1.0);
        if specular_color_factor != default_color {
            shader_builder.add_uniform(
                "vec3",
                "u_specularColorFactor",
                Some(ShaderDestination::FRAGMENT),
            );
            uniform_map.insert(
                "u_specularColorFactor".to_string(),
                Box::new(move || {
                    MaterialUniformValue::Vec3([
                        specular_color_factor.x,
                        specular_color_factor.y,
                        specular_color_factor.z,
                    ])
                }),
            );
            shader_builder.add_define(
                "HAS_SPECULAR_COLOR_FACTOR",
                None,
                Some(ShaderDestination::FRAGMENT),
            );
        }
    }

    fn process_anisotropy_uniforms(
        anisotropy: &Anisotropy,
        uniform_map: &mut HashMap<String, MaterialUniformCallback>,
        shader_builder: &mut ShaderBuilder,
        default_texture: Option<Texture>,
        disable_textures: bool,
        bounding_sphere: Option<BoundingSphere>,
    ) {
        shader_builder.add_define(
            "USE_ANISOTROPY",
            None,
            Some(ShaderDestination::FRAGMENT),
        );

        if let Some(anisotropy_texture) = &anisotropy.anisotropy_texture {
            if !disable_textures {
                Self::process_texture(
                    shader_builder,
                    uniform_map,
                    anisotropy_texture,
                    "u_anisotropyTexture",
                    "ANISOTROPY",
                    default_texture,
                    bounding_sphere,
                );
            }
        }

        let rotation = anisotropy.anisotropy_rotation as f64;
        let strength = anisotropy.anisotropy_strength as f64;
        let cos_rotation = rotation.cos();
        let sin_rotation = rotation.sin();

        shader_builder.add_uniform(
            "vec3",
            "u_anisotropy",
            Some(ShaderDestination::FRAGMENT),
        );
        uniform_map.insert(
            "u_anisotropy".to_string(),
            Box::new(move || MaterialUniformValue::Vec3([cos_rotation, sin_rotation, strength])),
        );
    }

    fn process_clearcoat_uniforms(
        clearcoat: &Clearcoat,
        uniform_map: &mut HashMap<String, MaterialUniformCallback>,
        shader_builder: &mut ShaderBuilder,
        default_texture: Option<Texture>,
        disable_textures: bool,
        bounding_sphere: Option<BoundingSphere>,
    ) {
        shader_builder.add_define(
            "USE_CLEARCOAT",
            None,
            Some(ShaderDestination::FRAGMENT),
        );

        let clearcoat_factor = clearcoat.clearcoat_factor;
        if clearcoat_factor.abs() > f32::EPSILON {
            shader_builder.add_uniform(
                "float",
                "u_clearcoatFactor",
                Some(ShaderDestination::FRAGMENT),
            );
            uniform_map.insert(
                "u_clearcoatFactor".to_string(),
                Box::new(move || MaterialUniformValue::Float(clearcoat_factor as f64)),
            );
            shader_builder.add_define(
                "HAS_CLEARCOAT_FACTOR",
                None,
                Some(ShaderDestination::FRAGMENT),
            );
        }

        if let Some(clearcoat_texture) = &clearcoat.clearcoat_texture {
            if !disable_textures {
                Self::process_texture(
                    shader_builder,
                    uniform_map,
                    clearcoat_texture,
                    "u_clearcoatTexture",
                    "CLEARCOAT",
                    default_texture.clone(),
                    bounding_sphere,
                );
            }
        }

        let clearcoat_roughness_factor = clearcoat.clearcoat_roughness_factor;
        if clearcoat_roughness_factor.abs() > f32::EPSILON {
            shader_builder.add_uniform(
                "float",
                "u_clearcoatRoughnessFactor",
                Some(ShaderDestination::FRAGMENT),
            );
            uniform_map.insert(
                "u_clearcoatRoughnessFactor".to_string(),
                Box::new(move || MaterialUniformValue::Float(clearcoat_roughness_factor as f64)),
            );
            shader_builder.add_define(
                "HAS_CLEARCOAT_ROUGHNESS_FACTOR",
                None,
                Some(ShaderDestination::FRAGMENT),
            );
        }

        if let Some(roughness_tex) = &clearcoat.clearcoat_roughness_texture {
            if !disable_textures {
                Self::process_texture(
                    shader_builder,
                    uniform_map,
                    roughness_tex,
                    "u_clearcoatRoughnessTexture",
                    "CLEARCOAT_ROUGHNESS",
                    default_texture.clone(),
                    bounding_sphere,
                );
            }
        }

        if let Some(normal_tex) = &clearcoat.clearcoat_normal_texture {
            if !disable_textures {
                Self::process_texture(
                    shader_builder,
                    uniform_map,
                    normal_tex,
                    "u_clearcoatNormalTexture",
                    "CLEARCOAT_NORMAL",
                    default_texture,
                    bounding_sphere,
                );
            }
        }
    }

    fn process_metallic_roughness_uniforms(
        metallic_roughness: &MetallicRoughness,
        uniform_map: &mut HashMap<String, MaterialUniformCallback>,
        shader_builder: &mut ShaderBuilder,
        default_texture: Option<Texture>,
        disable_textures: bool,
        bounding_sphere: Option<BoundingSphere>,
    ) {
        shader_builder.add_define(
            "USE_METALLIC_ROUGHNESS",
            None,
            Some(ShaderDestination::FRAGMENT),
        );

        if let Some(base_color_texture) = &metallic_roughness.base_color_texture {
            if !disable_textures {
                Self::process_texture(
                    shader_builder,
                    uniform_map,
                    base_color_texture,
                    "u_baseColorTexture",
                    "BASE_COLOR",
                    default_texture.clone(),
                    bounding_sphere,
                );
            }
        }

        let base_color_factor = metallic_roughness.base_color_factor;
        let default_base_color = Cartesian4::new(1.0, 1.0, 1.0, 1.0);
        if base_color_factor != default_base_color {
            shader_builder.add_uniform(
                "vec4",
                "u_baseColorFactor",
                Some(ShaderDestination::FRAGMENT),
            );
            uniform_map.insert(
                "u_baseColorFactor".to_string(),
                Box::new(move || {
                    MaterialUniformValue::Vec4([
                        base_color_factor.x,
                        base_color_factor.y,
                        base_color_factor.z,
                        base_color_factor.w,
                    ])
                }),
            );
            shader_builder.add_define(
                "HAS_BASE_COLOR_FACTOR",
                None,
                Some(ShaderDestination::FRAGMENT),
            );
        }

        if let Some(mr_tex) = &metallic_roughness.metallic_roughness_texture {
            if !disable_textures {
                Self::process_texture(
                    shader_builder,
                    uniform_map,
                    mr_tex,
                    "u_metallicRoughnessTexture",
                    "METALLIC_ROUGHNESS",
                    default_texture,
                    bounding_sphere,
                );
            }
        }

        let metallic_factor = metallic_roughness.metallic_factor;
        if (metallic_factor - 1.0).abs() > f32::EPSILON {
            shader_builder.add_uniform(
                "float",
                "u_metallicFactor",
                Some(ShaderDestination::FRAGMENT),
            );
            uniform_map.insert(
                "u_metallicFactor".to_string(),
                Box::new(move || MaterialUniformValue::Float(metallic_factor as f64)),
            );
            shader_builder.add_define(
                "HAS_METALLIC_FACTOR",
                None,
                Some(ShaderDestination::FRAGMENT),
            );
        }

        let roughness_factor = metallic_roughness.roughness_factor;
        if (roughness_factor - 1.0).abs() > f32::EPSILON {
            shader_builder.add_uniform(
                "float",
                "u_roughnessFactor",
                Some(ShaderDestination::FRAGMENT),
            );
            uniform_map.insert(
                "u_roughnessFactor".to_string(),
                Box::new(move || MaterialUniformValue::Float(roughness_factor as f64)),
            );
            shader_builder.add_define(
                "HAS_ROUGHNESS_FACTOR",
                None,
                Some(ShaderDestination::FRAGMENT),
            );
        }
    }

    fn process_planar_fill(
        planar_fill: &PlanarFill,
        primitive: &Primitive,
        uniform_map: &mut HashMap<String, MaterialUniformCallback>,
        shader_builder: &mut ShaderBuilder,
        planar_fill_id_pass: &mut Option<bool>,
        frame_state: &mut MaterialFrameState,
    ) {
        frame_state.planar_fill_requested = Some(true);

        shader_builder.add_define(
            "HAS_PLANAR_FILL_DEPTH",
            None,
            Some(ShaderDestination::FRAGMENT),
        );

        if planar_fill.background_fill {
            shader_builder.add_define(
                "HAS_BACKGROUND_FILL",
                None,
                Some(ShaderDestination::FRAGMENT),
            );
        }

        if primitive.feature_ids.is_empty() {
            return;
        }

        let positional_label = match &primitive.feature_ids[0] {
            FeatureId::Attribute(a) => a.positional_label.as_deref().unwrap_or("featureId_0"),
            FeatureId::Texture(t) => t.positional_label.as_deref().unwrap_or("featureId_0"),
            FeatureId::ImplicitRange(r) => r.positional_label.as_deref().unwrap_or("featureId_0"),
        };

        shader_builder.add_define(
            "PLANAR_FILL_FEATURE_ID",
            Some(positional_label),
            Some(ShaderDestination::FRAGMENT),
        );

        if planar_fill.behind {
            shader_builder.add_define(
                "HAS_PLANAR_FILL_BEHIND",
                None,
                Some(ShaderDestination::FRAGMENT),
            );
        } else {
            *planar_fill_id_pass = Some(true);
            shader_builder.add_define(
                "HAS_PLANAR_FILL_ID_PASS",
                None,
                Some(ShaderDestination::FRAGMENT),
            );
            shader_builder.add_uniform(
                "bool",
                "u_isPlanarFillIdPass",
                Some(ShaderDestination::FRAGMENT),
            );
            uniform_map.insert(
                "u_isPlanarFillIdPass".to_string(),
                Box::new(|| MaterialUniformValue::Bool(false)),
            );
        }
    }

    fn process_line_style_uniforms(
        line_style: &LineStyle,
        primitive: &Primitive,
        frame_state: &MaterialFrameState,
        uniform_map: &mut HashMap<String, MaterialUniformCallback>,
        shader_builder: &mut ShaderBuilder,
    ) {
        if let Some(width) = line_style.width {
            shader_builder.add_uniform(
                "float",
                "u_lineWidth",
                Some(ShaderDestination::VERTEX),
            );
            let pixel_ratio = frame_state.pixel_ratio;
            uniform_map.insert(
                "u_lineWidth".to_string(),
                Box::new(move || MaterialUniformValue::Float(width as f64 * pixel_ratio)),
            );
        }

        if let Some(pattern) = line_style.pattern {
            shader_builder.add_define(
                "HAS_LINE_PATTERN",
                None,
                Some(ShaderDestination::BOTH),
            );
            shader_builder.add_uniform(
                "float",
                "u_linePattern",
                Some(ShaderDestination::FRAGMENT),
            );
            shader_builder.add_varying("float", "v_lineCoord", None);
            uniform_map.insert(
                "u_linePattern".to_string(),
                Box::new(move || MaterialUniformValue::Float(pattern as f64)),
            );
        }

        let cum_dist_attribute = primitive
            .attributes
            .iter()
            .find(|a| a.semantic.as_deref() == Some("CUMULATIVE_DISTANCE"));

        let is_line_prim = matches!(
            primitive.primitive_type,
            Some(PrimitiveType::Lines) | Some(PrimitiveType::TriangleStrip)
        );

        if let (Some(_), Some(attr), true) =
            (line_style.pattern, cum_dist_attribute, is_line_prim)
        {
            shader_builder.add_define(
                "HAS_LINE_CUMULATIVE_DISTANCE",
                None,
                Some(ShaderDestination::VERTEX),
            );

            if attr.normalized {
                shader_builder.add_define(
                    "LINE_CUM_DIST_NORMALIZED",
                    None,
                    Some(ShaderDestination::VERTEX),
                );
            }

            shader_builder.add_uniform(
                "float",
                "u_cumulativeDistanceMax",
                Some(ShaderDestination::VERTEX),
            );
            shader_builder.add_uniform(
                "float",
                "u_pixelsPerWorld",
                Some(ShaderDestination::VERTEX),
            );

            let cum_dist_max = attr
                .max
                .as_ref()
                .and_then(|m| m.first().copied())
                .unwrap_or(1.0);
            uniform_map.insert(
                "u_cumulativeDistanceMax".to_string(),
                Box::new(move || MaterialUniformValue::Float(cum_dist_max)),
            );

            let camera = frame_state.camera.clone();
            let drawing_buffer_width = frame_state.context.drawing_buffer_width;
            uniform_map.insert(
                "u_pixelsPerWorld".to_string(),
                Box::new(move || {
                    let mut pixels_per_world = 1.0;
                    if let Some(cam) = &camera {
                        let frustum = &cam.frustum;
                        let world_width = if let (Some(r), Some(l)) = (frustum.right, frustum.left) {
                            r - l
                        } else {
                            frustum.width.unwrap_or(0.0)
                        };
                        if world_width > 0.0 {
                            pixels_per_world = drawing_buffer_width / world_width;
                        }
                    }
                    MaterialUniformValue::Float(pixels_per_world)
                }),
            );

            let var_name = attr.name.as_deref().unwrap_or("a_cumulativeDistance");
            let line_vs = format!(
                r#"
#ifdef HAS_LINE_CUMULATIVE_DISTANCE
void lineStyleStageVS(in ProcessedAttributes attributes)
{{
    float cumDist = attributes.{var_name};
#ifdef LINE_CUM_DIST_NORMALIZED
    cumDist *= u_cumulativeDistanceMax;
#endif
    const float textureCoordinateBase = 8192.0;
    v_lineCoord = textureCoordinateBase + cumDist * u_pixelsPerWorld;
}}
#endif
"#
            );
            shader_builder.add_vertex_lines(&[&line_vs]);
        } else if line_style.pattern.is_some() {
            let line_vs = r#"
#ifdef HAS_LINE_PATTERN
void lineStyleStageVS(in ProcessedAttributes attributes)
{
    vec4 posClip = gl_Position;
    vec2 screenPos = ((posClip.xy / posClip.w) * 0.5 + 0.5) * czm_viewport.zw;

    const float textureCoordinateBase = 8192.0;
    if (czm_viewport.z > czm_viewport.w) {
        v_lineCoord = textureCoordinateBase + screenPos.x;
    } else {
        v_lineCoord = textureCoordinateBase + screenPos.y;
    }
}
#endif
"#;
            shader_builder.add_vertex_lines(&[line_vs]);
        }
    }
}
