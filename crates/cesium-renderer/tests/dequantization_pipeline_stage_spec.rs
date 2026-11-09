use cesium_math::ClassificationType;
use cesium_renderer::{
    DequantizationAttribute, DequantizationModelOptions, DequantizationPipelineStage,
    DequantizationPrimitive, DequantizationRenderResources, Quantization, ShaderBuilder,
};
use std::collections::HashMap;

#[test]
fn test_adds_a_dequantization_function() {
    let mut shader_builder = ShaderBuilder::new();
    let mut uniform_map = HashMap::new();
    let model = DequantizationModelOptions::default();

    let mut resources = DequantizationRenderResources {
        shader_builder: &mut shader_builder,
        uniform_map: &mut uniform_map,
        model: &model,
    };

    let primitive = DequantizationPrimitive {
        attributes: vec![
            DequantizationAttribute {
                variable_name: "normalMC".to_string(),
                semantic: Some("NORMAL".to_string()),
                glsl_type: "vec3".to_string(),
                quantization: Some(Quantization {
                    oct_encoded: true,
                    oct_encoded_zxy: true,
                    normalization_range: Some(vec![1023.0]),
                    ..Default::default()
                }),
            },
            DequantizationAttribute {
                variable_name: "positionMC".to_string(),
                semantic: Some("POSITION".to_string()),
                glsl_type: "vec3".to_string(),
                quantization: Some(Quantization {
                    oct_encoded: false,
                    quantized_volume_offset: Some(vec![-2.43, 0.26, -1.39]),
                    quantized_volume_step_size: Some(vec![0.00029, 0.00029, 0.00029]),
                    ..Default::default()
                }),
            },
        ],
    };

    DequantizationPipelineStage::process(&mut resources, &primitive);

    let vs = resources.shader_builder.create_vertex_shader_source();
    let fs = resources.shader_builder.create_fragment_shader_source();

    assert!(vs.defines.iter().any(|d| d == "USE_DEQUANTIZATION"));
    assert!(fs.defines.is_empty());

    let vs_text = vs.sources.join("\n");
    assert!(vs_text.contains("attributes.normalMC = czm_octDecode(a_quantized_normalMC, model_normalizationRange_normalMC).zxy;"));
    assert!(vs_text.contains("attributes.positionMC = model_quantizedVolumeOffset_positionMC + a_quantized_positionMC * model_quantizedVolumeStepSize_positionMC;"));
}

#[test]
fn test_adds_dequantization_uniforms() {
    let mut shader_builder = ShaderBuilder::new();
    let mut uniform_map = HashMap::new();
    let model = DequantizationModelOptions::default();

    let mut resources = DequantizationRenderResources {
        shader_builder: &mut shader_builder,
        uniform_map: &mut uniform_map,
        model: &model,
    };

    let primitive = DequantizationPrimitive {
        attributes: vec![
            DequantizationAttribute {
                variable_name: "normalMC".to_string(),
                semantic: Some("NORMAL".to_string()),
                glsl_type: "vec3".to_string(),
                quantization: Some(Quantization {
                    oct_encoded: true,
                    oct_encoded_zxy: true,
                    normalization_range: Some(vec![1023.0]),
                    ..Default::default()
                }),
            },
            DequantizationAttribute {
                variable_name: "texCoord_0".to_string(),
                semantic: Some("TEXCOORD".to_string()),
                glsl_type: "vec2".to_string(),
                quantization: Some(Quantization {
                    oct_encoded: false,
                    quantized_volume_offset: Some(vec![0.0029, 0.0156]),
                    quantized_volume_step_size: Some(vec![0.00023, 0.00023]),
                    ..Default::default()
                }),
            },
            DequantizationAttribute {
                variable_name: "positionMC".to_string(),
                semantic: Some("POSITION".to_string()),
                glsl_type: "vec3".to_string(),
                quantization: Some(Quantization {
                    oct_encoded: false,
                    quantized_volume_offset: Some(vec![-2.43, 0.26, -1.39]),
                    quantized_volume_step_size: Some(vec![0.00029, 0.00029, 0.00029]),
                    ..Default::default()
                }),
            },
        ],
    };

    DequantizationPipelineStage::process(&mut resources, &primitive);

    let vs = resources.shader_builder.create_vertex_shader_source();
    let vs_text = vs.sources.join("\n");

    assert!(vs_text.contains("uniform float model_normalizationRange_normalMC;"));
    assert!(vs_text.contains("uniform vec2 model_quantizedVolumeOffset_texCoord_0;"));
    assert!(vs_text.contains("uniform vec2 model_quantizedVolumeStepSize_texCoord_0;"));
    assert!(vs_text.contains("uniform vec3 model_quantizedVolumeOffset_positionMC;"));
    assert!(vs_text.contains("uniform vec3 model_quantizedVolumeStepSize_positionMC;"));

    assert_eq!(
        resources.uniform_map["model_normalizationRange_normalMC"]().as_f64().unwrap(),
        1023.0
    );
}

#[test]
fn test_promotes_vertex_color_dequantization_uniforms_to_vec4() {
    let mut shader_builder = ShaderBuilder::new();
    let mut uniform_map = HashMap::new();
    let model = DequantizationModelOptions::default();

    let mut resources = DequantizationRenderResources {
        shader_builder: &mut shader_builder,
        uniform_map: &mut uniform_map,
        model: &model,
    };

    let primitive = DequantizationPrimitive {
        attributes: vec![DequantizationAttribute {
            variable_name: "color_0".to_string(),
            semantic: Some("COLOR".to_string()),
            glsl_type: "vec3".to_string(),
            quantization: Some(Quantization {
                oct_encoded: false,
                quantized_volume_offset: Some(vec![0.1, 0.2, 0.3]),
                quantized_volume_step_size: Some(vec![0.01, 0.01, 0.01]),
                ..Default::default()
            }),
        }],
    };

    DequantizationPipelineStage::process(&mut resources, &primitive);

    let vs = resources.shader_builder.create_vertex_shader_source();
    let vs_text = vs.sources.join("\n");

    assert!(vs_text.contains("uniform vec4 model_quantizedVolumeOffset_color_0;"));
    assert!(vs_text.contains("uniform vec4 model_quantizedVolumeStepSize_color_0;"));

    let offset_val = resources.uniform_map["model_quantizedVolumeOffset_color_0"]();
    let offset_arr = offset_val.as_array().unwrap();
    assert_eq!(offset_arr.len(), 4);
    assert_eq!(offset_arr[0].as_f64().unwrap(), 0.1);
    assert_eq!(offset_arr[1].as_f64().unwrap(), 0.2);
    assert_eq!(offset_arr[2].as_f64().unwrap(), 0.3);
    assert_eq!(offset_arr[3].as_f64().unwrap(), 0.0); // alpha defaults to 0.0

    let step_val = resources.uniform_map["model_quantizedVolumeStepSize_color_0"]();
    let step_arr = step_val.as_array().unwrap();
    assert_eq!(step_arr.len(), 4);
    assert_eq!(step_arr[0].as_f64().unwrap(), 0.01);
    assert_eq!(step_arr[1].as_f64().unwrap(), 0.01);
    assert_eq!(step_arr[2].as_f64().unwrap(), 0.01);
    assert_eq!(step_arr[3].as_f64().unwrap(), 1.0); // alpha defaults to 1.0
}

#[test]
fn test_only_dequantizes_position_and_texcoords_for_classification_models() {
    let mut shader_builder = ShaderBuilder::new();
    let mut uniform_map = HashMap::new();
    let model = DequantizationModelOptions {
        classification_type: Some(ClassificationType::Both),
    };

    let mut resources = DequantizationRenderResources {
        shader_builder: &mut shader_builder,
        uniform_map: &mut uniform_map,
        model: &model,
    };

    let primitive = DequantizationPrimitive {
        attributes: vec![
            DequantizationAttribute {
                variable_name: "normalMC".to_string(),
                semantic: Some("NORMAL".to_string()),
                glsl_type: "vec3".to_string(),
                quantization: Some(Quantization {
                    oct_encoded: true,
                    oct_encoded_zxy: true,
                    normalization_range: Some(vec![1023.0]),
                    ..Default::default()
                }),
            },
            DequantizationAttribute {
                variable_name: "positionMC".to_string(),
                semantic: Some("POSITION".to_string()),
                glsl_type: "vec3".to_string(),
                quantization: Some(Quantization {
                    oct_encoded: false,
                    quantized_volume_offset: Some(vec![-2.43, 0.26, -1.39]),
                    quantized_volume_step_size: Some(vec![0.00029, 0.00029, 0.00029]),
                    ..Default::default()
                }),
            },
            DequantizationAttribute {
                variable_name: "color_0".to_string(),
                semantic: Some("COLOR".to_string()),
                glsl_type: "vec3".to_string(),
                quantization: Some(Quantization {
                    oct_encoded: false,
                    quantized_volume_offset: Some(vec![0.1, 0.2, 0.3]),
                    quantized_volume_step_size: Some(vec![0.01, 0.01, 0.01]),
                    ..Default::default()
                }),
            },
        ],
    };

    DequantizationPipelineStage::process(&mut resources, &primitive);

    let vs = resources.shader_builder.create_vertex_shader_source();
    let vs_text = vs.sources.join("\n");

    // Only positionMC should be dequantized
    assert!(vs_text.contains("uniform vec3 model_quantizedVolumeOffset_positionMC;"));
    assert!(!vs_text.contains("model_normalizationRange_normalMC"));
    assert!(!vs_text.contains("model_quantizedVolumeOffset_color_0"));
}

#[test]
fn test_skips_non_quantized_attributes() {
    let mut shader_builder = ShaderBuilder::new();
    let mut uniform_map = HashMap::new();
    let model = DequantizationModelOptions::default();

    let mut resources = DequantizationRenderResources {
        shader_builder: &mut shader_builder,
        uniform_map: &mut uniform_map,
        model: &model,
    };

    let primitive = DequantizationPrimitive {
        attributes: vec![DequantizationAttribute {
            variable_name: "positionMC".to_string(),
            semantic: Some("POSITION".to_string()),
            glsl_type: "vec3".to_string(),
            quantization: None,
        }],
    };

    DequantizationPipelineStage::process(&mut resources, &primitive);

    let vs = resources.shader_builder.create_vertex_shader_source();
    assert!(vs.defines.iter().any(|d| d == "USE_DEQUANTIZATION"));
    assert!(resources.uniform_map.is_empty());
}
