use cesium_math::{Cartesian2, Cartesian3, Matrix2};
use cesium_renderer::{
    CustomShader, CustomShaderMode, CustomShaderOptions, CustomShaderTranslucencyMode,
    CustomShaderValue, LightingModel, TextureUniform, UniformSpecifier, CustomShaderUniformType, VaryingType,
};
use std::collections::HashMap;

const EMPTY_VERTEX_SHADER: &str =
    "void vertexMain(VertexInput vsInput, inout czm_modelVertexOutput vsOutput) {}";
const EMPTY_FRAGMENT_SHADER: &str =
    "void fragmentMain(FragmentInput fsInput, inout czm_modelMaterial material) {}";

#[test]
fn test_constructs_with_default_values() {
    let custom_shader = CustomShader::new(CustomShaderOptions::default()).unwrap();

    assert_eq!(custom_shader.mode, CustomShaderMode::ModifyMaterial);
    assert_eq!(custom_shader.lighting_model, None);
    assert_eq!(
        custom_shader.translucency_mode,
        CustomShaderTranslucencyMode::Inherit
    );
    assert!(custom_shader.uniforms.read().unwrap().is_empty());
    assert!(custom_shader.varyings.is_empty());
    assert_eq!(custom_shader.vertex_shader_text, None);
    assert_eq!(custom_shader.fragment_shader_text, None);
    assert!(custom_shader.uniform_map.is_empty());
}

#[test]
fn test_constructs() {
    let custom_shader = CustomShader::new(CustomShaderOptions {
        mode: Some(CustomShaderMode::ReplaceMaterial),
        translucency_mode: Some(CustomShaderTranslucencyMode::Translucent),
        lighting_model: Some(LightingModel::Pbr),
        vertex_shader_text: Some(EMPTY_VERTEX_SHADER.to_string()),
        fragment_shader_text: Some(EMPTY_FRAGMENT_SHADER.to_string()),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(custom_shader.mode, CustomShaderMode::ReplaceMaterial);
    assert_eq!(custom_shader.lighting_model, Some(LightingModel::Pbr));
    assert_eq!(
        custom_shader.translucency_mode,
        CustomShaderTranslucencyMode::Translucent
    );
    assert_eq!(
        custom_shader.vertex_shader_text.as_deref(),
        Some(EMPTY_VERTEX_SHADER)
    );
    assert_eq!(
        custom_shader.fragment_shader_text.as_deref(),
        Some(EMPTY_FRAGMENT_SHADER)
    );
}

#[test]
fn test_defines_uniforms() {
    let mut uniforms = HashMap::new();
    uniforms.insert(
        "u_time".to_string(),
        UniformSpecifier {
            uniform_type: CustomShaderUniformType::Float,
            value: CustomShaderValue::Float(0.0),
        },
    );
    uniforms.insert(
        "u_offset".to_string(),
        UniformSpecifier {
            uniform_type: CustomShaderUniformType::Vec2,
            value: CustomShaderValue::Vec2(Cartesian2::new(1.0, 2.0)),
        },
    );

    let custom_shader = CustomShader::new(CustomShaderOptions {
        uniforms,
        ..Default::default()
    })
    .unwrap();

    assert_eq!(
        custom_shader.uniform_map["u_time"](),
        CustomShaderValue::Float(0.0)
    );
    assert_eq!(
        custom_shader.uniform_map["u_offset"](),
        CustomShaderValue::Vec2(Cartesian2::new(1.0, 2.0))
    );
}

#[test]
fn test_set_uniform_throws_for_undeclared_uniform() {
    let custom_shader = CustomShader::new(CustomShaderOptions::default()).unwrap();
    let res = custom_shader.set_uniform("u_time", CustomShaderValue::Float(10.0));
    assert!(res.is_err());
}

#[test]
fn test_set_uniform_updates_uniform_values() {
    let mut uniforms = HashMap::new();
    uniforms.insert(
        "u_time".to_string(),
        UniformSpecifier {
            uniform_type: CustomShaderUniformType::Float,
            value: CustomShaderValue::Float(0.0),
        },
    );

    let custom_shader = CustomShader::new(CustomShaderOptions {
        uniforms,
        ..Default::default()
    })
    .unwrap();

    assert_eq!(
        custom_shader.uniform_map["u_time"](),
        CustomShaderValue::Float(0.0)
    );
    custom_shader
        .set_uniform("u_time", CustomShaderValue::Float(10.0))
        .unwrap();
    assert_eq!(
        custom_shader.uniform_map["u_time"](),
        CustomShaderValue::Float(10.0)
    );
}

#[test]
fn test_set_uniform_clones_vectors() {
    let mut uniforms = HashMap::new();
    uniforms.insert(
        "u_vector".to_string(),
        UniformSpecifier {
            uniform_type: CustomShaderUniformType::Vec3,
            value: CustomShaderValue::Vec3(Cartesian3::ZERO),
        },
    );

    let custom_shader = CustomShader::new(CustomShaderOptions {
        uniforms,
        ..Default::default()
    })
    .unwrap();

    let val = Cartesian3::new(1.0, 0.0, 0.0);
    custom_shader
        .set_uniform("u_vector", CustomShaderValue::Vec3(val))
        .unwrap();
    assert_eq!(
        custom_shader.uniform_map["u_vector"](),
        CustomShaderValue::Vec3(val)
    );
}

#[test]
fn test_set_uniform_clones_matrices() {
    let mut uniforms = HashMap::new();
    uniforms.insert(
        "u_matrix".to_string(),
        UniformSpecifier {
            uniform_type: CustomShaderUniformType::Mat2,
            value: CustomShaderValue::Mat2(Matrix2::IDENTITY),
        },
    );

    let custom_shader = CustomShader::new(CustomShaderOptions {
        uniforms,
        ..Default::default()
    })
    .unwrap();

    let val = Matrix2::new(2.0, 0.0, 0.0, 2.0);
    custom_shader
        .set_uniform("u_matrix", CustomShaderValue::Mat2(val))
        .unwrap();
    assert_eq!(
        custom_shader.uniform_map["u_matrix"](),
        CustomShaderValue::Mat2(val)
    );
}

#[test]
fn test_declares_varyings() {
    let mut varyings = HashMap::new();
    varyings.insert("v_dist_from_center".to_string(), VaryingType::Float);
    varyings.insert("v_computedMatrix".to_string(), VaryingType::Mat4);

    let custom_shader = CustomShader::new(CustomShaderOptions {
        varyings: varyings.clone(),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(custom_shader.varyings, varyings);
}

#[test]
fn test_detects_input_variables_in_the_shader_text() {
    let vs = r#"
        void vertexMain(VertexInput vsInput, inout czm_modelVertexOutput vsOutput)
        {
            float value = vsInput.featureIds.featureId_0;
            float value2 = vsInput.metadata.temperature;
            positionMC += vsInput.attributes.expansion * vsInput.attributes.normalMC;
        }
    "#;
    let fs = r#"
        void fragmentMain(FragmentInput fsInput, inout czm_modelMaterial material)
        {
            float value = fsInput.featureIds.featureId_1 + fsInput.featureIds.instanceFeatureId_0;
            float value2 = fsInput.metadata.pressure;
            material.normalEC = normalize(fsInput.attributes.normalEC);
            material.diffuse = fsInput.attributes.color_0;
            material.specular = fsInput.attributes.positionWC / 1.0e6;
        }
    "#;

    let custom_shader = CustomShader::new(CustomShaderOptions {
        vertex_shader_text: Some(vs.to_string()),
        fragment_shader_text: Some(fs.to_string()),
        ..Default::default()
    })
    .unwrap();

    assert!(custom_shader
        .used_variables_vertex
        .attribute_set
        .contains("expansion"));
    assert!(custom_shader
        .used_variables_vertex
        .attribute_set
        .contains("normalMC"));
    assert!(custom_shader
        .used_variables_vertex
        .feature_id_set
        .contains("featureId_0"));
    assert!(custom_shader
        .used_variables_vertex
        .metadata_set
        .contains("temperature"));

    assert!(custom_shader
        .used_variables_fragment
        .attribute_set
        .contains("normalEC"));
    assert!(custom_shader
        .used_variables_fragment
        .attribute_set
        .contains("color_0"));
    assert!(custom_shader
        .used_variables_fragment
        .attribute_set
        .contains("positionWC"));
    assert!(custom_shader
        .used_variables_fragment
        .material_set
        .contains("normalEC"));
    assert!(custom_shader
        .used_variables_fragment
        .material_set
        .contains("diffuse"));
    assert!(custom_shader
        .used_variables_fragment
        .material_set
        .contains("specular"));
    assert!(custom_shader
        .used_variables_fragment
        .feature_id_set
        .contains("featureId_1"));
    assert!(custom_shader
        .used_variables_fragment
        .feature_id_set
        .contains("instanceFeatureId_0"));
    assert!(custom_shader
        .used_variables_fragment
        .metadata_set
        .contains("pressure"));
}

#[test]
fn test_detects_metadata_variables_in_the_shader_text() {
    let vs = r#"
        void vertexMain(VertexInput vsInput, inout czm_modelVertexOutput vsOutput)
        {
            float value = vsInput.metadata.temperature;
            float value2 = vsInput.metadataClass.testProperty.noData;
            float value3 = vsInput.metadataStatistics.testStatistic.mean;
        }
    "#;

    let custom_shader = CustomShader::new(CustomShaderOptions {
        vertex_shader_text: Some(vs.to_string()),
        ..Default::default()
    })
    .unwrap();

    assert!(custom_shader
        .used_variables_vertex
        .metadata_set
        .contains("temperature"));
    assert!(custom_shader
        .used_variables_vertex
        .metadata_set
        .contains("testProperty"));
    assert!(custom_shader
        .used_variables_vertex
        .metadata_set
        .contains("testStatistic"));
}

fn make_single_variable_vs(var_name: &str) -> Result<CustomShader, cesium_math::DeveloperError> {
    CustomShader::new(CustomShaderOptions {
        vertex_shader_text: Some(format!(
            "void vertexMain(VertexInput vsInput, inout czm_modelVertexOutput vsOutput) {{ vsOutput.positionMC = vsInput.attributes.{var_name}; }}"
        )),
        ..Default::default()
    })
}

fn make_single_variable_fs(var_name: &str) -> Result<CustomShader, cesium_math::DeveloperError> {
    CustomShader::new(CustomShaderOptions {
        fragment_shader_text: Some(format!(
            "void fragmentMain(FragmentInput fsInput, inout czm_modelMaterial material) {{ material.diffuse = fsInput.attributes.{var_name}; }}"
        )),
        ..Default::default()
    })
}

#[test]
fn test_validates_position() {
    assert!(make_single_variable_vs("position").is_err());
    assert!(make_single_variable_vs("positionMC").is_ok());
    assert!(make_single_variable_vs("positionWC").is_err());
    assert!(make_single_variable_vs("positionEC").is_err());

    assert!(make_single_variable_fs("position").is_err());
    assert!(make_single_variable_fs("positionMC").is_ok());
    assert!(make_single_variable_fs("positionWC").is_ok());
    assert!(make_single_variable_fs("positionEC").is_ok());
}

#[test]
fn test_validates_normal() {
    assert!(make_single_variable_vs("normal").is_err());
    assert!(make_single_variable_vs("normalMC").is_ok());
    assert!(make_single_variable_vs("normalEC").is_err());

    assert!(make_single_variable_fs("normal").is_err());
    assert!(make_single_variable_fs("normalMC").is_err());
    assert!(make_single_variable_fs("normalEC").is_ok());
}

#[test]
fn test_validates_tangent() {
    assert!(make_single_variable_vs("tangent").is_err());
    assert!(make_single_variable_vs("tangentMC").is_ok());
    assert!(make_single_variable_vs("tangentEC").is_err());

    assert!(make_single_variable_fs("tangent").is_err());
    assert!(make_single_variable_fs("tangentMC").is_err());
    assert!(make_single_variable_fs("tangentEC").is_ok());
}

#[test]
fn test_validates_bitangent() {
    assert!(make_single_variable_vs("bitangent").is_err());
    assert!(make_single_variable_vs("bitangentMC").is_ok());
    assert!(make_single_variable_vs("bitangentEC").is_err());

    assert!(make_single_variable_fs("bitangent").is_err());
    assert!(make_single_variable_fs("bitangentMC").is_err());
    assert!(make_single_variable_fs("bitangentEC").is_ok());
}

#[test]
fn test_supports_texture_uniforms() {
    let mut uniforms = HashMap::new();
    uniforms.insert(
        "u_blue".to_string(),
        UniformSpecifier {
            uniform_type: CustomShaderUniformType::Sampler2D,
            value: CustomShaderValue::Texture(TextureUniform::from_url("Data/Images/Blue2x2.png")),
        },
    );

    let custom_shader = CustomShader::new(CustomShaderOptions {
        vertex_shader_text: Some(EMPTY_VERTEX_SHADER.to_string()),
        fragment_shader_text: Some(EMPTY_FRAGMENT_SHADER.to_string()),
        uniforms,
        ..Default::default()
    })
    .unwrap();

    assert!(custom_shader.uniform_map.contains_key("u_blue"));
    let val = custom_shader.uniform_map["u_blue"]();
    match val {
        CustomShaderValue::Texture(t) => assert_eq!(t.url.as_deref(), Some("Data/Images/Blue2x2.png")),
        _ => panic!("Expected Texture value"),
    }
}

#[test]
fn test_destroys() {
    let mut custom_shader = CustomShader::new(CustomShaderOptions {
        vertex_shader_text: Some(EMPTY_VERTEX_SHADER.to_string()),
        fragment_shader_text: Some(EMPTY_FRAGMENT_SHADER.to_string()),
        ..Default::default()
    })
    .unwrap();

    assert!(!custom_shader.is_destroyed());
    custom_shader.destroy();
    assert!(custom_shader.is_destroyed());

    // set_uniform after destroy should error
    let res = custom_shader.set_uniform("u_time", CustomShaderValue::Float(1.0));
    assert!(res.is_err());
}
