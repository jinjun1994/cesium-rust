use cesium_renderer::{
    CustomShader, CustomShaderAttributeInfo, CustomShaderOptions,
    CustomShaderPipelineStage, CustomShaderPrimitive, CustomShaderRenderResources,
    CustomShaderTranslucencyMode, CustomShaderUniformType, CustomShaderValue, LightingModel,
    ModelAlphaOptions, Pass, ShaderBuilder, UniformSpecifier, VaryingType,
};
use std::collections::{BTreeSet, HashMap};

const EMPTY_VERTEX_SHADER: &str =
    "void vertexMain(VertexInput vsInput, inout czm_modelVertexOutput vsOutput) {}";
const EMPTY_FRAGMENT_SHADER: &str =
    "void fragmentMain(FragmentInput fsInput, inout czm_modelMaterial material) {}";

fn default_primitive() -> CustomShaderPrimitive {
    CustomShaderPrimitive {
        attributes: vec![
            CustomShaderAttributeInfo {
                variable_name: "positionMC".to_string(),
                glsl_type: "vec3".to_string(),
            },
            CustomShaderAttributeInfo {
                variable_name: "normalMC".to_string(),
                glsl_type: "vec3".to_string(),
            },
            CustomShaderAttributeInfo {
                variable_name: "texCoord_0".to_string(),
                glsl_type: "vec2".to_string(),
            },
        ],
        property_ids: BTreeSet::new(),
    }
}

#[test]
fn test_sets_defines_in_the_shader() {
    let custom_shader = CustomShader::new(CustomShaderOptions {
        vertex_shader_text: Some(EMPTY_VERTEX_SHADER.to_string()),
        fragment_shader_text: Some(EMPTY_FRAGMENT_SHADER.to_string()),
        ..Default::default()
    })
    .unwrap();

    let mut shader_builder = ShaderBuilder::new();
    let mut lighting_model = None;
    let mut alpha_options = ModelAlphaOptions::default();
    let mut uniform_map = HashMap::new();

    let mut render_resources = CustomShaderRenderResources {
        shader_builder: &mut shader_builder,
        lighting_model: &mut lighting_model,
        alpha_options: &mut alpha_options,
        custom_shader: &custom_shader,
        uniform_map: &mut uniform_map,
    };

    let primitive = default_primitive();
    CustomShaderPipelineStage::process(&mut render_resources, &primitive);

    let vs = render_resources.shader_builder.create_vertex_shader_source();
    let fs = render_resources.shader_builder.create_fragment_shader_source();

    assert!(vs.defines.iter().any(|d| d == "HAS_CUSTOM_VERTEX_SHADER"));
    assert!(fs.defines.iter().any(|d| d == "HAS_CUSTOM_FRAGMENT_SHADER"));
    assert!(fs.defines.iter().any(|d| d == "CUSTOM_SHADER_MODIFY_MATERIAL"));
}

#[test]
fn test_adds_uniforms_from_the_custom_shader() {
    let mut uniforms = HashMap::new();
    uniforms.insert(
        "u_time".to_string(),
        UniformSpecifier {
            uniform_type: CustomShaderUniformType::Float,
            value: CustomShaderValue::Float(0.0),
        },
    );
    uniforms.insert(
        "u_enableAnimation".to_string(),
        UniformSpecifier {
            uniform_type: CustomShaderUniformType::Bool,
            value: CustomShaderValue::Bool(true),
        },
    );

    let custom_shader = CustomShader::new(CustomShaderOptions {
        uniforms,
        vertex_shader_text: Some(EMPTY_VERTEX_SHADER.to_string()),
        fragment_shader_text: Some(EMPTY_FRAGMENT_SHADER.to_string()),
        ..Default::default()
    })
    .unwrap();

    let mut shader_builder = ShaderBuilder::new();
    let mut lighting_model = None;
    let mut alpha_options = ModelAlphaOptions::default();
    let mut uniform_map = HashMap::new();

    let mut render_resources = CustomShaderRenderResources {
        shader_builder: &mut shader_builder,
        lighting_model: &mut lighting_model,
        alpha_options: &mut alpha_options,
        custom_shader: &custom_shader,
        uniform_map: &mut uniform_map,
    };

    let primitive = default_primitive();
    CustomShaderPipelineStage::process(&mut render_resources, &primitive);

    let vs = render_resources.shader_builder.create_vertex_shader_source();
    let fs = render_resources.shader_builder.create_fragment_shader_source();

    let vs_text = vs.sources.join("\n");
    let fs_text = fs.sources.join("\n");

    assert!(vs_text.contains("uniform bool u_enableAnimation;"));
    assert!(vs_text.contains("uniform float u_time;"));
    assert!(fs_text.contains("uniform bool u_enableAnimation;"));
    assert!(fs_text.contains("uniform float u_time;"));

    assert!(render_resources.uniform_map.contains_key("u_time"));
    assert!(render_resources.uniform_map.contains_key("u_enableAnimation"));
}

#[test]
fn test_adds_varying_declarations_from_the_custom_shader() {
    let mut varyings = HashMap::new();
    varyings.insert("v_float".to_string(), VaryingType::Float);
    varyings.insert("v_vec2".to_string(), VaryingType::Vec2);
    varyings.insert("v_vec3".to_string(), VaryingType::Vec3);
    varyings.insert("v_vec4".to_string(), VaryingType::Vec4);
    varyings.insert("v_mat2x2".to_string(), VaryingType::Mat2);
    varyings.insert("v_mat3x3".to_string(), VaryingType::Mat3);
    varyings.insert("v_mat4x4".to_string(), VaryingType::Mat4);

    let custom_shader = CustomShader::new(CustomShaderOptions {
        vertex_shader_text: Some(EMPTY_VERTEX_SHADER.to_string()),
        fragment_shader_text: Some(EMPTY_FRAGMENT_SHADER.to_string()),
        varyings,
        ..Default::default()
    })
    .unwrap();

    let mut shader_builder = ShaderBuilder::new();
    let mut lighting_model = None;
    let mut alpha_options = ModelAlphaOptions::default();
    let mut uniform_map = HashMap::new();

    let mut render_resources = CustomShaderRenderResources {
        shader_builder: &mut shader_builder,
        lighting_model: &mut lighting_model,
        alpha_options: &mut alpha_options,
        custom_shader: &custom_shader,
        uniform_map: &mut uniform_map,
    };

    let primitive = default_primitive();
    CustomShaderPipelineStage::process(&mut render_resources, &primitive);

    let vs = render_resources.shader_builder.create_vertex_shader_source();
    let fs = render_resources.shader_builder.create_fragment_shader_source();

    let vs_text = vs.sources.join("\n");
    let fs_text = fs.sources.join("\n");

    assert!(vs_text.contains("out float v_float;"));
    assert!(vs_text.contains("out vec2 v_vec2;"));
    assert!(vs_text.contains("out vec3 v_vec3;"));
    assert!(vs_text.contains("out vec4 v_vec4;"));
    assert!(vs_text.contains("out mat2 v_mat2x2;"));
    assert!(vs_text.contains("out mat3 v_mat3x3;"));
    assert!(vs_text.contains("out mat4 v_mat4x4;"));

    assert!(fs_text.contains("in float v_float;"));
    assert!(fs_text.contains("in vec2 v_vec2;"));
    assert!(fs_text.contains("in vec3 v_vec3;"));
    assert!(fs_text.contains("in vec4 v_vec4;"));
    assert!(fs_text.contains("in mat2 v_mat2x2;"));
    assert!(fs_text.contains("in mat3 v_mat3x3;"));
    assert!(fs_text.contains("in mat4 v_mat4x4;"));
}

#[test]
fn test_computes_position_wc_define_when_used_in_fs() {
    let fs = r#"
        void fragmentMain(FragmentInput fsInput, inout czm_modelMaterial material) {
            material.diffuse = fsInput.attributes.positionWC;
        }
    "#;

    let custom_shader = CustomShader::new(CustomShaderOptions {
        fragment_shader_text: Some(fs.to_string()),
        ..Default::default()
    })
    .unwrap();

    let mut shader_builder = ShaderBuilder::new();
    let mut lighting_model = None;
    let mut alpha_options = ModelAlphaOptions::default();
    let mut uniform_map = HashMap::new();

    let mut render_resources = CustomShaderRenderResources {
        shader_builder: &mut shader_builder,
        lighting_model: &mut lighting_model,
        alpha_options: &mut alpha_options,
        custom_shader: &custom_shader,
        uniform_map: &mut uniform_map,
    };

    let primitive = default_primitive();
    CustomShaderPipelineStage::process(&mut render_resources, &primitive);

    let vs = render_resources.shader_builder.create_vertex_shader_source();
    let fs = render_resources.shader_builder.create_fragment_shader_source();

    assert!(vs.defines.iter().any(|d| d == "COMPUTE_POSITION_WC_CUSTOM_SHADER"));
    assert!(fs.defines.iter().any(|d| d == "COMPUTE_POSITION_WC_CUSTOM_SHADER"));
}

#[test]
fn test_configures_lighting_and_translucency_mode() {
    let custom_shader = CustomShader::new(CustomShaderOptions {
        lighting_model: Some(LightingModel::Unlit),
        translucency_mode: Some(CustomShaderTranslucencyMode::Translucent),
        vertex_shader_text: Some(EMPTY_VERTEX_SHADER.to_string()),
        ..Default::default()
    })
    .unwrap();

    let mut shader_builder = ShaderBuilder::new();
    let mut lighting_model = None;
    let mut alpha_options = ModelAlphaOptions::default();
    let mut uniform_map = HashMap::new();

    let mut render_resources = CustomShaderRenderResources {
        shader_builder: &mut shader_builder,
        lighting_model: &mut lighting_model,
        alpha_options: &mut alpha_options,
        custom_shader: &custom_shader,
        uniform_map: &mut uniform_map,
    };

    let primitive = default_primitive();
    CustomShaderPipelineStage::process(&mut render_resources, &primitive);

    assert_eq!(*render_resources.lighting_model, Some(LightingModel::Unlit));
    assert_eq!(render_resources.alpha_options.pass, Some(Pass::Translucent));
}
