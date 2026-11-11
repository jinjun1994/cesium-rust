use std::collections::HashMap;

use cesium_math::{Cartesian3, Matrix3};
use cesium_renderer::image_based_lighting::{ImageBasedLighting, ImageBasedLightingOptions};
use cesium_renderer::image_based_lighting_pipeline_stage::{
    IblContext, IblModelOptions, IblRenderResources, ImageBasedLightingPipelineStage,
    SpecularEnvironmentCubeMap,
};
use cesium_renderer::shader_builder::ShaderBuilder;

fn mock_context() -> IblContext {
    IblContext {
        floating_point_texture: true,
        color_buffer_float: true,
        half_floating_point_texture: false,
        supports_texture_lod: true,
    }
}

#[test]
fn test_configures_render_resources_for_default_image_based_lighting() {
    let ibl = ImageBasedLighting::default();
    let model = IblModelOptions {
        image_based_lighting: ibl,
        ibl_reference_frame_matrix: Matrix3::IDENTITY,
        ..Default::default()
    };

    let mut sb = ShaderBuilder::default();
    let mut uniform_map = HashMap::new();
    let mut render_resources = IblRenderResources {
        shader_builder: &mut sb,
        uniform_map: &mut uniform_map,
        model: &model,
    };

    let context = mock_context();
    ImageBasedLightingPipelineStage::process(&mut render_resources, &model, &context).unwrap();

    let fs_source = render_resources.shader_builder.create_fragment_shader_source();
    let fs = fs_source.sources.join("\n");

    assert!(fs_source.defines.contains(&"USE_IBL_LIGHTING".to_string()));
    assert!(fs.contains("uniform vec2 model_iblFactor;"));
    assert!(fs.contains("uniform mat3 model_iblReferenceFrameMatrix;"));

    // Fragment lines contain IBL stage code
    assert!(fs.contains("textureIBL"));

    let ibl_factor_val = (render_resources.uniform_map["model_iblFactor"])();
    assert_eq!(ibl_factor_val, serde_json::json!([1.0, 1.0]));

    let ref_mat_val = (render_resources.uniform_map["model_iblReferenceFrameMatrix"])();
    let mut expected_mat = [0.0; 9];
    let _ = Matrix3::IDENTITY.pack(&mut expected_mat, 0);
    assert_eq!(ref_mat_val, serde_json::json!(expected_mat));
}

#[test]
fn test_configures_render_resources_for_spherical_harmonics() {
    let test_coefficients = vec![
        Cartesian3::new(1.0, 1.0, 1.0),
        Cartesian3::new(2.0, 2.0, 2.0),
        Cartesian3::new(3.0, 3.0, 3.0),
        Cartesian3::new(4.0, 4.0, 4.0),
        Cartesian3::new(5.0, 5.0, 5.0),
        Cartesian3::new(6.0, 6.0, 6.0),
        Cartesian3::new(7.0, 7.0, 7.0),
        Cartesian3::new(8.0, 8.0, 8.0),
        Cartesian3::new(9.0, 9.0, 9.0),
    ];

    let ibl = ImageBasedLighting::new(Some(ImageBasedLightingOptions {
        spherical_harmonic_coefficients: Some(test_coefficients.clone()),
        ..Default::default()
    }))
    .unwrap();

    let model = IblModelOptions {
        image_based_lighting: ibl,
        ibl_reference_frame_matrix: Matrix3::IDENTITY,
        ..Default::default()
    };

    let mut sb = ShaderBuilder::default();
    let mut uniform_map = HashMap::new();
    let mut render_resources = IblRenderResources {
        shader_builder: &mut sb,
        uniform_map: &mut uniform_map,
        model: &model,
    };

    let context = mock_context();
    ImageBasedLightingPipelineStage::process(&mut render_resources, &model, &context).unwrap();

    let fs_source = render_resources.shader_builder.create_fragment_shader_source();
    let fs = fs_source.sources.join("\n");

    assert!(fs_source.defines.contains(&"USE_IBL_LIGHTING".to_string()));
    assert!(fs_source.defines.contains(&"DIFFUSE_IBL".to_string()));
    assert!(fs_source.defines.contains(&"CUSTOM_SPHERICAL_HARMONICS".to_string()));

    assert!(fs.contains("uniform vec2 model_iblFactor;"));
    assert!(fs.contains("uniform mat3 model_iblReferenceFrameMatrix;"));
    assert!(fs.contains("uniform vec3 model_sphericalHarmonicCoefficients[9];"));

    let ibl_factor_val = (render_resources.uniform_map["model_iblFactor"])();
    assert_eq!(ibl_factor_val, serde_json::json!([1.0, 1.0]));

    let sh_val = (render_resources.uniform_map["model_sphericalHarmonicCoefficients"])();
    let expected_sh: Vec<serde_json::Value> = test_coefficients
        .iter()
        .map(|c| serde_json::json!([c.x, c.y, c.z]))
        .collect();
    assert_eq!(sh_val, serde_json::Value::Array(expected_sh));
}

#[test]
fn test_configures_render_resources_for_specular_environment_maps() {
    let mock_cube_map = SpecularEnvironmentCubeMap {
        maximum_mipmap_level: 0.0,
        ready: true,
        texture_id: Some("example_texture".to_string()),
        ..Default::default()
    };

    let ibl = ImageBasedLighting::new(Some(ImageBasedLightingOptions {
        specular_environment_maps: Some("example.ktx2".to_string()),
        ..Default::default()
    }))
    .unwrap();

    let model = IblModelOptions {
        image_based_lighting: ibl,
        specular_environment_cube_map: Some(mock_cube_map),
        ibl_reference_frame_matrix: Matrix3::IDENTITY,
        ..Default::default()
    };

    let mut sb = ShaderBuilder::default();
    let mut uniform_map = HashMap::new();
    let mut render_resources = IblRenderResources {
        shader_builder: &mut sb,
        uniform_map: &mut uniform_map,
        model: &model,
    };

    let context = mock_context();
    ImageBasedLightingPipelineStage::process(&mut render_resources, &model, &context).unwrap();

    let fs_source = render_resources.shader_builder.create_fragment_shader_source();
    let fs = fs_source.sources.join("\n");

    assert!(fs_source.defines.contains(&"USE_IBL_LIGHTING".to_string()));
    assert!(fs_source.defines.contains(&"SPECULAR_IBL".to_string()));
    assert!(fs_source.defines.contains(&"CUSTOM_SPECULAR_IBL".to_string()));

    assert!(fs.contains("uniform vec2 model_iblFactor;"));
    assert!(fs.contains("uniform mat3 model_iblReferenceFrameMatrix;"));
    assert!(fs.contains("uniform samplerCube model_specularEnvironmentMaps;"));
    assert!(fs.contains("uniform float model_specularEnvironmentMapsMaximumLOD;"));

    let env_map_val = (render_resources.uniform_map["model_specularEnvironmentMaps"])();
    assert_eq!(env_map_val, serde_json::json!("example_texture"));

    let lod_val = (render_resources.uniform_map["model_specularEnvironmentMapsMaximumLOD"])();
    assert_eq!(lod_val, serde_json::json!(0.0));
}
