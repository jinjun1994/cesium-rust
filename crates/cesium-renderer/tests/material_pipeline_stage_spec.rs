//! Tests for MaterialPipelineStage matching CesiumJS `Scene/Model/MaterialPipelineStageSpec.js` (Apache-2.0).

use std::collections::HashMap;

use cesium_math::{
    BoundingSphere, Cartesian2, Cartesian3, Cartesian4, ClassificationType, Matrix3,
};
use cesium_renderer::material_pipeline_stage::{
    AlphaMode, Anisotropy, Clearcoat, ConstantLod, FeatureId,
    LightingModel, LineStyle, Material, MaterialFrameState, MaterialFrameStateCamera,
    MaterialFrameStateContext, MaterialFrustum, MaterialModelOptions, MaterialPipelineStage,
    MaterialRenderResources, MaterialUniformCallback, MetallicRoughness,
    ModelAlphaOptions, ModelLightingOptions, ModelTextureReader, Pass, PlanarFill,
    PointCloudShading, RenderState, ShaderBuilder, ShaderDestination, Specular,
    SpecularGlossiness, SHADERS_MATERIAL_STAGE_FS,
};
use cesium_renderer::model_components::{Attribute, FeatureIdAttribute, Primitive};

fn mock_render_resources<'a>(
    shader_builder: &'a mut ShaderBuilder,
    uniform_map: &'a mut HashMap<String, MaterialUniformCallback>,
    lighting_options: &'a mut ModelLightingOptions,
    alpha_options: &'a mut ModelAlphaOptions,
    render_state_options: &'a mut RenderState,
    model: &'a mut MaterialModelOptions,
) -> MaterialRenderResources<'a> {
    MaterialRenderResources {
        shader_builder,
        uniform_map,
        lighting_options,
        alpha_options,
        render_state_options,
        model,
        planar_fill_id_pass: None,
    }
}

fn mock_frame_state() -> MaterialFrameState {
    MaterialFrameState {
        context: MaterialFrameStateContext {
            default_texture: None,
            default_normal_texture: None,
            default_emissive_texture: None,
            drawing_buffer_width: 1024.0,
        },
        pixel_ratio: 1.0,
        camera: Some(MaterialFrameStateCamera {
            position_wc: Cartesian3::new(100.0, 0.0, 0.0),
            frustum: MaterialFrustum {
                left: Some(-10.0),
                right: Some(10.0),
                width: Some(20.0),
            },
        }),
        planar_fill_requested: None,
    }
}

#[test]
fn processes_default_material() {
    let mut sb = ShaderBuilder::default();
    let mut um = HashMap::new();
    let mut lo = ModelLightingOptions::default();
    let mut ao = ModelAlphaOptions::default();
    let mut rs = RenderState::default();
    let mut mo = MaterialModelOptions::default();

    let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);
    let mut fs = mock_frame_state();

    let primitive = Primitive {
        material: Some(Material::default()),
        attributes: vec![Attribute {
            semantic: Some("NORMAL".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };

    MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);

    let frag_src = sb.create_fragment_shader_source();
    assert!(frag_src.defines.iter().any(|d| d == "USE_METALLIC_ROUGHNESS"));
    assert_eq!(lo.lighting_model, LightingModel::Pbr);
}

#[test]
fn adds_material_and_metallic_roughness_uniforms() {
    let mut sb = ShaderBuilder::default();
    let mut um = HashMap::new();
    let mut lo = ModelLightingOptions::default();
    let mut ao = ModelAlphaOptions::default();
    let mut rs = RenderState::default();
    let mut mo = MaterialModelOptions::default();

    let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);
    let mut fs = mock_frame_state();

    let primitive = Primitive {
        material: Some(Material {
            emissive_factor: Cartesian3::new(0.5, 0.5, 0.5),
            emissive_texture: Some(ModelTextureReader {
                index: 0,
                texture: None,
                tex_coord: 0,
                scale: None,
                transform: None,
                channels: None,
                constant_lod: None,
            }),
            normal_texture: Some(ModelTextureReader {
                index: 1,
                texture: None,
                tex_coord: 0,
                scale: None,
                transform: None,
                channels: None,
                constant_lod: None,
            }),
            occlusion_texture: Some(ModelTextureReader {
                index: 2,
                texture: None,
                tex_coord: 0,
                scale: None,
                transform: None,
                channels: None,
                constant_lod: None,
            }),
            metallic_roughness: Some(MetallicRoughness {
                base_color_texture: Some(ModelTextureReader {
                    index: 3,
                    texture: None,
                    tex_coord: 0,
                    scale: None,
                    transform: None,
                    channels: None,
                    constant_lod: None,
                }),
                metallic_roughness_texture: Some(ModelTextureReader {
                    index: 4,
                    texture: None,
                    tex_coord: 0,
                    scale: None,
                    transform: None,
                    channels: None,
                    constant_lod: None,
                }),
                ..Default::default()
            }),
            ..Default::default()
        }),
        attributes: vec![Attribute {
            semantic: Some("NORMAL".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };

    MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);

    let frag_src = sb.create_fragment_shader_source();
    let frag_defines = &frag_src.defines;
    assert!(frag_defines.iter().any(|l| l.contains("HAS_BASE_COLOR_TEXTURE")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_EMISSIVE_FACTOR")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_EMISSIVE_TEXTURE")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_METALLIC_ROUGHNESS_TEXTURE")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_NORMAL_TEXTURE")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_OCCLUSION_TEXTURE")));
    assert!(frag_defines.iter().any(|l| l.contains("TEXCOORD_BASE_COLOR v_texCoord_0")));
    assert!(frag_defines.iter().any(|l| l.contains("USE_METALLIC_ROUGHNESS")));

    let uniform_lines = sb.fragment_uniform_lines();
    assert!(uniform_lines.iter().any(|l| l.contains("u_baseColorTexture")));
    assert!(uniform_lines.iter().any(|l| l.contains("u_emissiveTexture")));
    assert!(uniform_lines.iter().any(|l| l.contains("u_metallicRoughnessTexture")));
    assert!(uniform_lines.iter().any(|l| l.contains("u_normalTexture")));
    assert!(uniform_lines.iter().any(|l| l.contains("u_occlusionTexture")));
    assert!(uniform_lines.iter().any(|l| l.contains("u_emissiveFactor")));

    assert_eq!(
        um.get("u_emissiveFactor").unwrap()().as_vec3(),
        Some(Cartesian3::new(0.5, 0.5, 0.5))
    );
}

#[test]
fn adds_metallic_roughness_uniforms_without_defaults() {
    let mut sb = ShaderBuilder::default();
    let mut um = HashMap::new();
    let mut lo = ModelLightingOptions::default();
    let mut ao = ModelAlphaOptions::default();
    let mut rs = RenderState::default();
    let mut mo = MaterialModelOptions::default();

    let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);
    let mut fs = mock_frame_state();

    let primitive = Primitive {
        material: Some(Material {
            metallic_roughness: Some(MetallicRoughness {
                base_color_factor: Cartesian4::new(0.5, 0.5, 0.5, 0.5),
                metallic_factor: 0.5,
                roughness_factor: 0.5,
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);

    let frag_src = sb.create_fragment_shader_source();
    let frag_defines = &frag_src.defines;
    assert!(frag_defines.iter().any(|l| l.contains("HAS_BASE_COLOR_FACTOR")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_METALLIC_FACTOR")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_ROUGHNESS_FACTOR")));

    assert_eq!(
        um.get("u_baseColorFactor").unwrap()().as_vec4(),
        Some(Cartesian4::new(0.5, 0.5, 0.5, 0.5))
    );
    assert_eq!(um.get("u_metallicFactor").unwrap()().as_f64(), Some(0.5));
    assert_eq!(um.get("u_roughnessFactor").unwrap()().as_f64(), Some(0.5));
}

#[test]
fn does_not_add_emissive_uniforms_when_emissive_factor_is_default() {
    let mut sb = ShaderBuilder::default();
    let mut um = HashMap::new();
    let mut lo = ModelLightingOptions::default();
    let mut ao = ModelAlphaOptions::default();
    let mut rs = RenderState::default();
    let mut mo = MaterialModelOptions::default();

    let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);
    let mut fs = mock_frame_state();

    let primitive = Primitive {
        material: Some(Material {
            emissive_factor: Cartesian3::ZERO,
            ..Default::default()
        }),
        ..Default::default()
    };

    MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);

    assert!(!um.contains_key("u_emissiveFactor"));
    let frag_src = sb.create_fragment_shader_source();
    assert!(!frag_src
        .defines
        .iter()
        .any(|l| l.contains("HAS_EMISSIVE_FACTOR")));
}

#[test]
fn adds_specular_glossiness_uniforms() {
    let mut sb = ShaderBuilder::default();
    let mut um = HashMap::new();
    let mut lo = ModelLightingOptions::default();
    let mut ao = ModelAlphaOptions::default();
    let mut rs = RenderState::default();
    let mut mo = MaterialModelOptions::default();

    let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);
    let mut fs = mock_frame_state();

    let primitive = Primitive {
        material: Some(Material {
            specular_glossiness: Some(SpecularGlossiness {
                diffuse_texture: Some(ModelTextureReader {
                    index: 0,
                    texture: None,
                    tex_coord: 0,
                    scale: None,
                    transform: None,
                    channels: None,
                    constant_lod: None,
                }),
                specular_glossiness_texture: Some(ModelTextureReader {
                    index: 1,
                    texture: None,
                    tex_coord: 0,
                    scale: None,
                    transform: None,
                    channels: None,
                    constant_lod: None,
                }),
                glossiness_factor: 0.8,
                ..Default::default()
            }),
            ..Default::default()
        }),
        attributes: vec![Attribute {
            semantic: Some("NORMAL".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };

    MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);

    let frag_src = sb.create_fragment_shader_source();
    let frag_defines = &frag_src.defines;
    assert!(frag_defines.iter().any(|l| l.contains("USE_SPECULAR_GLOSSINESS")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_DIFFUSE_TEXTURE")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_SPECULAR_GLOSSINESS_TEXTURE")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_GLOSSINESS_FACTOR")));

    let gloss = um.get("u_glossinessFactor").unwrap()().as_f64().unwrap();
    assert!((gloss - 0.8).abs() < 1e-5);
}

#[test]
fn adds_specular_glossiness_uniforms_without_defaults() {
    let mut sb = ShaderBuilder::default();
    let mut um = HashMap::new();
    let mut lo = ModelLightingOptions::default();
    let mut ao = ModelAlphaOptions::default();
    let mut rs = RenderState::default();
    let mut mo = MaterialModelOptions::default();

    let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);
    let mut fs = mock_frame_state();

    let primitive = Primitive {
        material: Some(Material {
            specular_glossiness: Some(SpecularGlossiness {
                diffuse_factor: Cartesian4::new(0.5, 0.5, 0.5, 0.5),
                specular_factor: Cartesian3::new(0.5, 0.5, 0.5),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);

    let frag_src = sb.create_fragment_shader_source();
    let frag_defines = &frag_src.defines;
    assert!(frag_defines.iter().any(|l| l.contains("HAS_DIFFUSE_FACTOR")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_LEGACY_SPECULAR_FACTOR")));

    assert_eq!(
        um.get("u_diffuseFactor").unwrap()().as_vec4(),
        Some(Cartesian4::new(0.5, 0.5, 0.5, 0.5))
    );
    assert_eq!(
        um.get("u_legacySpecularFactor").unwrap()().as_vec3(),
        Some(Cartesian3::new(0.5, 0.5, 0.5))
    );
}

#[test]
fn adds_uniforms_and_defines_for_khr_materials_specular() {
    let mut sb = ShaderBuilder::default();
    let mut um = HashMap::new();
    let mut lo = ModelLightingOptions::default();
    let mut ao = ModelAlphaOptions::default();
    let mut rs = RenderState::default();
    let mut mo = MaterialModelOptions::default();

    let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);
    let mut fs = mock_frame_state();

    let primitive = Primitive {
        material: Some(Material {
            specular: Some(Specular {
                specular_factor: 0.5,
                specular_texture: Some(ModelTextureReader {
                    index: 0,
                    texture: None,
                    tex_coord: 0,
                    scale: None,
                    transform: None,
                    channels: None,
                    constant_lod: None,
                }),
                specular_color_factor: Cartesian3::new(0.2, 0.3, 0.4),
                specular_color_texture: Some(ModelTextureReader {
                    index: 1,
                    texture: None,
                    tex_coord: 0,
                    scale: None,
                    transform: None,
                    channels: None,
                    constant_lod: None,
                }),
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);

    let frag_src = sb.create_fragment_shader_source();
    let frag_defines = &frag_src.defines;
    assert!(frag_defines.iter().any(|l| l.contains("USE_SPECULAR")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_SPECULAR_FACTOR")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_SPECULAR_TEXTURE")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_SPECULAR_COLOR_FACTOR")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_SPECULAR_COLOR_TEXTURE")));

    let sf = um.get("u_specularFactor").unwrap()().as_f64().unwrap();
    assert!((sf - 0.5).abs() < 1e-5);
    assert_eq!(
        um.get("u_specularColorFactor").unwrap()().as_vec3(),
        Some(Cartesian3::new(0.2, 0.3, 0.4))
    );
}

#[test]
fn adds_uniforms_and_defines_for_khr_materials_anisotropy() {
    let mut sb = ShaderBuilder::default();
    let mut um = HashMap::new();
    let mut lo = ModelLightingOptions::default();
    let mut ao = ModelAlphaOptions::default();
    let mut rs = RenderState::default();
    let mut mo = MaterialModelOptions::default();

    let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);
    let mut fs = mock_frame_state();

    let rotation = std::f32::consts::FRAC_PI_4;
    let strength = 0.75_f32;

    let primitive = Primitive {
        material: Some(Material {
            anisotropy: Some(Anisotropy {
                anisotropy_strength: strength,
                anisotropy_rotation: rotation,
                anisotropy_texture: Some(ModelTextureReader {
                    index: 0,
                    texture: None,
                    tex_coord: 0,
                    scale: None,
                    transform: None,
                    channels: None,
                    constant_lod: None,
                }),
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);

    let frag_src = sb.create_fragment_shader_source();
    let frag_defines = &frag_src.defines;
    assert!(frag_defines.iter().any(|l| l.contains("USE_ANISOTROPY")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_ANISOTROPY_TEXTURE")));

    let ani_vec = um.get("u_anisotropy").unwrap()().as_vec3().unwrap();
    assert!((ani_vec.x - (rotation as f64).cos()).abs() < 1e-5);
    assert!((ani_vec.y - (rotation as f64).sin()).abs() < 1e-5);
    assert!((ani_vec.z - strength as f64).abs() < 1e-5);
}

#[test]
fn adds_uniforms_and_defines_for_khr_materials_clearcoat() {
    let mut sb = ShaderBuilder::default();
    let mut um = HashMap::new();
    let mut lo = ModelLightingOptions::default();
    let mut ao = ModelAlphaOptions::default();
    let mut rs = RenderState::default();
    let mut mo = MaterialModelOptions::default();

    let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);
    let mut fs = mock_frame_state();

    let primitive = Primitive {
        material: Some(Material {
            clearcoat: Some(Clearcoat {
                clearcoat_factor: 0.9,
                clearcoat_roughness_factor: 0.3,
                clearcoat_texture: Some(ModelTextureReader {
                    index: 0,
                    texture: None,
                    tex_coord: 0,
                    scale: None,
                    transform: None,
                    channels: None,
                    constant_lod: None,
                }),
                clearcoat_roughness_texture: Some(ModelTextureReader {
                    index: 1,
                    texture: None,
                    tex_coord: 0,
                    scale: None,
                    transform: None,
                    channels: None,
                    constant_lod: None,
                }),
                clearcoat_normal_texture: Some(ModelTextureReader {
                    index: 2,
                    texture: None,
                    tex_coord: 0,
                    scale: Some(0.5),
                    transform: None,
                    channels: None,
                    constant_lod: None,
                }),
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);

    let frag_src = sb.create_fragment_shader_source();
    let frag_defines = &frag_src.defines;
    assert!(frag_defines.iter().any(|l| l.contains("USE_CLEARCOAT")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_CLEARCOAT_FACTOR")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_CLEARCOAT_ROUGHNESS_FACTOR")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_CLEARCOAT_TEXTURE")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_CLEARCOAT_ROUGHNESS_TEXTURE")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_CLEARCOAT_NORMAL_TEXTURE")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_CLEARCOAT_NORMAL_TEXTURE_SCALE")));

    let cf = um.get("u_clearcoatFactor").unwrap()().as_f64().unwrap();
    assert!((cf - 0.9).abs() < 1e-5);
    let crf = um.get("u_clearcoatRoughnessFactor").unwrap()().as_f64().unwrap();
    assert!((crf - 0.3).abs() < 1e-5);
    let scale = um.get("u_clearcoatNormalTextureScale").unwrap()().as_f64().unwrap();
    assert!((scale - 0.5).abs() < 1e-5);
}

#[test]
fn adds_uniforms_and_defines_for_a_normal_texture_scalar() {
    let mut sb = ShaderBuilder::default();
    let mut um = HashMap::new();
    let mut lo = ModelLightingOptions::default();
    let mut ao = ModelAlphaOptions::default();
    let mut rs = RenderState::default();
    let mut mo = MaterialModelOptions::default();

    let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);
    let mut fs = mock_frame_state();

    let primitive = Primitive {
        material: Some(Material {
            normal_texture: Some(ModelTextureReader {
                index: 0,
                texture: None,
                tex_coord: 0,
                scale: Some(0.8),
                transform: None,
                channels: None,
                constant_lod: None,
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);

    let frag_src = sb.create_fragment_shader_source();
    assert!(frag_src
        .defines
        .iter()
        .any(|l| l.contains("HAS_NORMAL_TEXTURE_SCALE")));
    let scale = um.get("u_normalTextureScale").unwrap()().as_f64().unwrap();
    assert!((scale - 0.8).abs() < 1e-5);
}

#[test]
fn does_not_add_texture_uniforms_for_classification_models() {
    let mut sb = ShaderBuilder::default();
    let mut um = HashMap::new();
    let mut lo = ModelLightingOptions::default();
    let mut ao = ModelAlphaOptions::default();
    let mut rs = RenderState::default();
    let mut mo = MaterialModelOptions {
        classification_type: Some(ClassificationType::Both),
        ..Default::default()
    };

    let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);
    let mut fs = mock_frame_state();

    let primitive = Primitive {
        material: Some(Material {
            emissive_factor: Cartesian3::new(1.0, 0.0, 0.0),
            emissive_texture: Some(ModelTextureReader {
                index: 0,
                texture: None,
                tex_coord: 0,
                scale: None,
                transform: None,
                channels: None,
                constant_lod: None,
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);

    let frag_src = sb.create_fragment_shader_source();
    assert!(frag_src
        .defines
        .iter()
        .any(|l| l.contains("HAS_EMISSIVE_FACTOR")));
    assert!(!frag_src
        .defines
        .iter()
        .any(|l| l.contains("HAS_EMISSIVE_TEXTURE")));
    assert_eq!(lo.lighting_model, LightingModel::Unlit);
}

#[test]
fn enables_pbr_lighting_for_metallic_roughness_and_specular_glossiness() {
    let mut fs = mock_frame_state();

    // Metallic roughness with normal
    {
        let mut sb = ShaderBuilder::default();
        let mut um = HashMap::new();
        let mut lo = ModelLightingOptions::default();
        let mut ao = ModelAlphaOptions::default();
        let mut rs = RenderState::default();
        let mut mo = MaterialModelOptions::default();
        let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);

        let primitive = Primitive {
            material: Some(Material::default()),
            attributes: vec![Attribute {
                semantic: Some("NORMAL".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };

        MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);
        assert_eq!(lo.lighting_model, LightingModel::Pbr);
    }

    // Specular glossiness with normal
    {
        let mut sb = ShaderBuilder::default();
        let mut um = HashMap::new();
        let mut lo = ModelLightingOptions::default();
        let mut ao = ModelAlphaOptions::default();
        let mut rs = RenderState::default();
        let mut mo = MaterialModelOptions::default();
        let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);

        let primitive = Primitive {
            material: Some(Material {
                specular_glossiness: Some(SpecularGlossiness::default()),
                ..Default::default()
            }),
            attributes: vec![Attribute {
                semantic: Some("NORMAL".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };

        MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);
        assert_eq!(lo.lighting_model, LightingModel::Pbr);
    }
}

#[test]
fn enables_unlit_lighting_under_special_conditions() {
    let mut fs = mock_frame_state();

    // 1. Explicit unlit material
    {
        let mut sb = ShaderBuilder::default();
        let mut um = HashMap::new();
        let mut lo = ModelLightingOptions::default();
        let mut ao = ModelAlphaOptions::default();
        let mut rs = RenderState::default();
        let mut mo = MaterialModelOptions::default();
        let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);

        let primitive = Primitive {
            material: Some(Material {
                unlit: true,
                ..Default::default()
            }),
            attributes: vec![Attribute {
                semantic: Some("NORMAL".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };

        MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);
        assert_eq!(lo.lighting_model, LightingModel::Unlit);
    }

    // 2. Missing normals
    {
        let mut sb = ShaderBuilder::default();
        let mut um = HashMap::new();
        let mut lo = ModelLightingOptions::default();
        let mut ao = ModelAlphaOptions::default();
        let mut rs = RenderState::default();
        let mut mo = MaterialModelOptions::default();
        let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);

        let primitive = Primitive {
            material: Some(Material::default()),
            attributes: vec![],
            ..Default::default()
        };

        MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);
        assert_eq!(lo.lighting_model, LightingModel::Unlit);
    }

    // 3. Point cloud shading disabled normals
    {
        let mut sb = ShaderBuilder::default();
        let mut um = HashMap::new();
        let mut lo = ModelLightingOptions::default();
        let mut ao = ModelAlphaOptions::default();
        let mut rs = RenderState::default();
        let mut mo = MaterialModelOptions {
            point_cloud_shading: Some(PointCloudShading {
                normal_shading: false,
                back_face_culling: true,
            }),
            ..Default::default()
        };
        let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);

        let primitive = Primitive {
            material: Some(Material::default()),
            attributes: vec![Attribute {
                semantic: Some("NORMAL".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };

        MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);
        assert_eq!(lo.lighting_model, LightingModel::Unlit);
    }
}

#[test]
fn handles_alpha_modes() {
    let mut fs = mock_frame_state();

    // Opaque
    {
        let mut sb = ShaderBuilder::default();
        let mut um = HashMap::new();
        let mut lo = ModelLightingOptions::default();
        let mut ao = ModelAlphaOptions::default();
        let mut rs = RenderState::default();
        let mut mo = MaterialModelOptions::default();
        let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);

        let primitive = Primitive {
            material: Some(Material {
                alpha_mode: AlphaMode::Opaque,
                ..Default::default()
            }),
            ..Default::default()
        };

        MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);
        assert_eq!(ao.pass, None);
        assert_eq!(ao.alpha_cutoff, None);
    }

    // Mask
    {
        let mut sb = ShaderBuilder::default();
        let mut um = HashMap::new();
        let mut lo = ModelLightingOptions::default();
        let mut ao = ModelAlphaOptions::default();
        let mut rs = RenderState::default();
        let mut mo = MaterialModelOptions::default();
        let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);

        let primitive = Primitive {
            material: Some(Material {
                alpha_mode: AlphaMode::Mask,
                alpha_cutoff: 0.6,
                ..Default::default()
            }),
            ..Default::default()
        };

        MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);
        assert_eq!(ao.pass, None);
        let cutoff = ao.alpha_cutoff.unwrap();
        assert!((cutoff - 0.6).abs() < 1e-5);
    }

    // Blend
    {
        let mut sb = ShaderBuilder::default();
        let mut um = HashMap::new();
        let mut lo = ModelLightingOptions::default();
        let mut ao = ModelAlphaOptions::default();
        let mut rs = RenderState::default();
        let mut mo = MaterialModelOptions::default();
        let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);

        let primitive = Primitive {
            material: Some(Material {
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            }),
            ..Default::default()
        };

        MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);
        assert_eq!(ao.pass, Some(Pass::Translucent));
    }
}

#[test]
fn handles_back_face_culling() {
    let mut fs = mock_frame_state();

    // Disabled when model back_face_culling is false
    {
        let mut sb = ShaderBuilder::default();
        let mut um = HashMap::new();
        let mut lo = ModelLightingOptions::default();
        let mut ao = ModelAlphaOptions::default();
        let mut rs = RenderState::default();
        let mut mo = MaterialModelOptions {
            back_face_culling: false,
            ..Default::default()
        };
        let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);

        let primitive = Primitive {
            material: Some(Material {
                double_sided: false,
                ..Default::default()
            }),
            ..Default::default()
        };

        MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);
        assert!(!rs.cull.enabled);
    }

    // Enabled when model back_face_culling is true and material is not double_sided
    {
        let mut sb = ShaderBuilder::default();
        let mut um = HashMap::new();
        let mut lo = ModelLightingOptions::default();
        let mut ao = ModelAlphaOptions::default();
        let mut rs = RenderState::default();
        let mut mo = MaterialModelOptions {
            back_face_culling: true,
            ..Default::default()
        };
        let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);

        let primitive = Primitive {
            material: Some(Material {
                double_sided: false,
                ..Default::default()
            }),
            ..Default::default()
        };

        MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);
        assert!(rs.cull.enabled);
    }

    // Disabled when material is double_sided
    {
        let mut sb = ShaderBuilder::default();
        let mut um = HashMap::new();
        let mut lo = ModelLightingOptions::default();
        let mut ao = ModelAlphaOptions::default();
        let mut rs = RenderState::default();
        let mut mo = MaterialModelOptions {
            back_face_culling: true,
            ..Default::default()
        };
        let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);

        let primitive = Primitive {
            material: Some(Material {
                double_sided: true,
                ..Default::default()
            }),
            ..Default::default()
        };

        MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);
        assert!(!rs.cull.enabled);

        let vert_src = sb.create_vertex_shader_source();
        let frag_src = sb.create_fragment_shader_source();
        assert!(vert_src
            .defines
            .iter()
            .any(|l| l.contains("HAS_DOUBLE_SIDED_MATERIAL")));
        assert!(frag_src
            .defines
            .iter()
            .any(|l| l.contains("HAS_DOUBLE_SIDED_MATERIAL")));
    }
}

#[test]
fn processes_texture_transforms_and_scales() {
    let mut sb = ShaderBuilder::default();
    let mut um = HashMap::new();
    let matrix = Matrix3::new(0.5, 0.0, 0.5, 0.0, 0.5, 0.0, 0.0, 0.0, 1.0);
    let texture_reader = ModelTextureReader {
        index: 0,
        texture: None,
        tex_coord: 1,
        scale: Some(2.0),
        transform: Some(matrix),
        channels: None,
        constant_lod: None,
    };

    MaterialPipelineStage::process_texture(
        &mut sb,
        &mut um,
        &texture_reader,
        "u_testTexture",
        "TEST",
        None,
        None,
    );

    let frag_src = sb.create_fragment_shader_source();
    let frag_defines = &frag_src.defines;
    assert!(frag_defines.iter().any(|l| l.contains("HAS_TEST_TEXTURE")));
    assert!(frag_defines.iter().any(|l| l.contains("TEXCOORD_TEST v_texCoord_1")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_TEST_TEXTURE_TRANSFORM")));
    assert!(frag_defines.iter().any(|l| l.contains("HAS_TEST_TEXTURE_SCALE")));

    let uniform_lines = sb.fragment_uniform_lines();
    assert!(uniform_lines.iter().any(|l| l.contains("u_testTextureTransform")));
    assert!(uniform_lines.iter().any(|l| l.contains("u_testTextureScale")));

    assert_eq!(
        um.get("u_testTextureTransform").unwrap()().as_mat3(),
        Some(matrix)
    );
    assert_eq!(
        um.get("u_testTextureScale").unwrap()().as_f64(),
        Some(2.0)
    );
}

#[test]
fn processes_bentley_materials_line_style() {
    let mut sb = ShaderBuilder::default();
    let mut um = HashMap::new();
    let mut lo = ModelLightingOptions::default();
    let mut ao = ModelAlphaOptions::default();
    let mut rs = RenderState::default();
    let mut mo = MaterialModelOptions::default();
    let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);

    let mut fs = mock_frame_state();
    fs.pixel_ratio = 2.0;

    let primitive = Primitive {
        material: Some(Material {
            line_style: Some(LineStyle {
                width: Some(3.0),
                pattern: Some(0xaaaa),
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);

    let frag_src = sb.create_fragment_shader_source();
    assert!(frag_src
        .defines
        .iter()
        .any(|l| l.contains("HAS_LINE_PATTERN")));
    assert_eq!(um.get("u_lineWidth").unwrap()().as_f64(), Some(6.0));
    assert_eq!(um.get("u_linePattern").unwrap()().as_f64(), Some(0xaaaa as f64));
}

#[test]
fn processes_bentley_materials_point_style() {
    let mut sb = ShaderBuilder::default();
    let mut um = HashMap::new();
    let mut lo = ModelLightingOptions::default();
    let mut ao = ModelAlphaOptions::default();
    let mut rs = RenderState::default();
    let mut mo = MaterialModelOptions::default();
    let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);

    let mut fs = mock_frame_state();
    fs.pixel_ratio = 1.5;

    let primitive = Primitive {
        material: Some(Material {
            point_diameter: Some(5.0),
            ..Default::default()
        }),
        ..Default::default()
    };

    MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);

    let vert_src = sb.create_vertex_shader_source();
    let frag_src = sb.create_fragment_shader_source();
    assert!(vert_src
        .defines
        .iter()
        .any(|l| l.contains("HAS_POINT_DIAMETER")));
    assert!(frag_src
        .defines
        .iter()
        .any(|l| l.contains("HAS_POINT_DIAMETER")));
    assert_eq!(um.get("u_pointDiameter").unwrap()().as_f64(), Some(7.5));
}

#[test]
fn processes_bentley_materials_planar_fill() {
    let mut fs = mock_frame_state();

    // 1. Non-behind planar fill with feature ID
    {
        let mut sb = ShaderBuilder::default();
        let mut um = HashMap::new();
        let mut lo = ModelLightingOptions::default();
        let mut ao = ModelAlphaOptions::default();
        let mut rs = RenderState::default();
        let mut mo = MaterialModelOptions::default();
        let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);

        let primitive = Primitive {
            material: Some(Material {
                planar_fill: Some(PlanarFill {
                    behind: false,
                    background_fill: false,
                }),
                ..Default::default()
            }),
            feature_ids: vec![FeatureId::Attribute(FeatureIdAttribute {
                positional_label: Some("featureId_0".to_string()),
                ..Default::default()
            })],
            ..Default::default()
        };

        MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);
        let id_pass = rr.planar_fill_id_pass;

        let frag_src = sb.create_fragment_shader_source();
        let frag_defines = &frag_src.defines;
        assert!(frag_defines.iter().any(|l| l.contains("HAS_PLANAR_FILL_DEPTH")));
        assert!(frag_defines.iter().any(|l| l.contains("HAS_PLANAR_FILL_ID_PASS")));
        assert!(frag_defines.iter().any(|l| l.contains("PLANAR_FILL_FEATURE_ID featureId_0")));
        assert_eq!(fs.planar_fill_requested, Some(true));
        assert_eq!(id_pass, Some(true));
        assert_eq!(um.get("u_isPlanarFillIdPass").unwrap()().as_bool(), Some(false));
    }

    // 2. Behind planar fill
    {
        let mut sb = ShaderBuilder::default();
        let mut um = HashMap::new();
        let mut lo = ModelLightingOptions::default();
        let mut ao = ModelAlphaOptions::default();
        let mut rs = RenderState::default();
        let mut mo = MaterialModelOptions::default();
        let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);

        let primitive = Primitive {
            material: Some(Material {
                planar_fill: Some(PlanarFill {
                    behind: true,
                    background_fill: false,
                }),
                ..Default::default()
            }),
            feature_ids: vec![FeatureId::Attribute(FeatureIdAttribute {
                positional_label: Some("featureId_0".to_string()),
                ..Default::default()
            })],
            ..Default::default()
        };

        MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);

        let frag_src = sb.create_fragment_shader_source();
        let frag_defines = &frag_src.defines;
        assert!(frag_defines.iter().any(|l| l.contains("HAS_PLANAR_FILL_DEPTH")));
        assert!(frag_defines.iter().any(|l| l.contains("HAS_PLANAR_FILL_BEHIND")));
    }

    // 3. Background fill
    {
        let mut sb = ShaderBuilder::default();
        let mut um = HashMap::new();
        let mut lo = ModelLightingOptions::default();
        let mut ao = ModelAlphaOptions::default();
        let mut rs = RenderState::default();
        let mut mo = MaterialModelOptions::default();
        let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);

        let primitive = Primitive {
            material: Some(Material {
                planar_fill: Some(PlanarFill {
                    behind: false,
                    background_fill: true,
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);

        let frag_src = sb.create_fragment_shader_source();
        assert!(frag_src
            .defines
            .iter()
            .any(|l| l.contains("HAS_BACKGROUND_FILL")));
        assert_eq!(lo.lighting_model, LightingModel::Unlit);
    }
}

#[test]
fn processes_ext_texture_info_constant_lod() {
    let mut sb = ShaderBuilder::default();
    sb.add_function(
        "setDynamicVaryingsVS",
        "void setDynamicVaryingsVS()\n{\n}",
        ShaderDestination::VERTEX,
    );

    let mut um = HashMap::new();
    let mut lo = ModelLightingOptions::default();
    let mut ao = ModelAlphaOptions::default();
    let mut rs = RenderState::default();
    let mut mo = MaterialModelOptions {
        bounding_sphere: Some(BoundingSphere::new(glam::DVec3::new(10.0, 20.0, 30.0), 50.0)),
        ..Default::default()
    };
    let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);
    let mut fs = mock_frame_state();

    let primitive = Primitive {
        material: Some(Material {
            metallic_roughness: Some(MetallicRoughness {
                base_color_texture: Some(ModelTextureReader {
                    index: 0,
                    texture: None,
                    tex_coord: 0,
                    scale: None,
                    transform: None,
                    channels: None,
                    constant_lod: Some(ConstantLod {
                        min_clamp_distance: 10.0,
                        max_clamp_distance: 500.0,
                        repetitions: 4.0,
                        offset: Cartesian2::new(1.0, 2.0),
                    }),
                }),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);

    let vert_src = sb.create_vertex_shader_source();
    let frag_src = sb.create_fragment_shader_source();

    assert!(vert_src.defines.iter().any(|l| l.contains("HAS_CONSTANT_LOD")));
    assert!(vert_src.defines.iter().any(|l| l.contains("HAS_BASE_COLOR_CONSTANT_LOD")));
    assert!(frag_src.defines.iter().any(|l| l.contains("HAS_BASE_COLOR_CONSTANT_LOD")));
    assert!(frag_src.defines.iter().any(|l| l.contains("HAS_CONSTANT_LOD")));

    let vert_uniforms = sb.vertex_uniform_lines();
    assert!(vert_uniforms.iter().any(|l| l.contains("u_constantLodOffset")));
    assert!(vert_uniforms.iter().any(|l| l.contains("u_constantLodDistance")));
    assert!(vert_uniforms.iter().any(|l| l.contains("u_constantLodWorldToEnu")));

    let frag_uniforms = sb.fragment_uniform_lines();
    assert!(frag_uniforms.iter().any(|l| l.contains("u_baseColorTextureConstantLodParams")));

    assert_eq!(
        um.get("u_baseColorTextureConstantLodParams").unwrap()().as_vec3(),
        Some(Cartesian3::new(10.0, 500.0, 4.0))
    );
}

#[test]
fn adds_material_stage_functions_to_fragment_shader() {
    let mut sb = ShaderBuilder::default();
    let mut um = HashMap::new();
    let mut lo = ModelLightingOptions::default();
    let mut ao = ModelAlphaOptions::default();
    let mut rs = RenderState::default();
    let mut mo = MaterialModelOptions::default();
    let mut rr = mock_render_resources(&mut sb, &mut um, &mut lo, &mut ao, &mut rs, &mut mo);
    let mut fs = mock_frame_state();

    let primitive = Primitive::default();
    MaterialPipelineStage::process(&mut rr, &primitive, &mut fs);

    let frag_src = sb.create_fragment_shader_source();
    let fs_text = frag_src.sources.join("\n");
    assert!(fs_text.contains(SHADERS_MATERIAL_STAGE_FS));
}
