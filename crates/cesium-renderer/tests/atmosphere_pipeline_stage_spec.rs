use cesium_math::Cartesian3;
use cesium_renderer::{
    AtmosphereFrameState, AtmosphereModelInfo, AtmospherePipelineStage, AtmosphereRenderResources,
    ShaderBuilder, ATMOSPHERE_STAGE_FS, ATMOSPHERE_STAGE_VS,
};
use std::collections::HashMap;

#[test]
fn test_configures_shader() {
    let mut shader_builder = ShaderBuilder::new();
    let mut uniform_map = HashMap::new();
    let mut resources = AtmosphereRenderResources {
        shader_builder: &mut shader_builder,
        uniform_map: &mut uniform_map,
    };

    let model = AtmosphereModelInfo {
        bounding_sphere_center: Cartesian3::new(6378137.0, 0.0, 0.0),
    };
    let frame_state = AtmosphereFrameState {
        camera_position_wc: Cartesian3::new(6379137.0, 0.0, 1000.0),
        fog_density: 0.0006,
        camera_in_space: false,
    };

    AtmospherePipelineStage::process(&mut resources, &model, &frame_state);

    let vs = resources.shader_builder.create_vertex_shader_source();
    let fs = resources.shader_builder.create_fragment_shader_source();

    // Verify defines
    assert!(vs.defines.iter().any(|d| d == "HAS_ATMOSPHERE"));
    assert!(vs
        .defines
        .iter()
        .any(|d| d == "COMPUTE_POSITION_WC_ATMOSPHERE"));
    assert!(fs.defines.iter().any(|d| d == "HAS_ATMOSPHERE"));
    assert!(fs
        .defines
        .iter()
        .any(|d| d == "COMPUTE_POSITION_WC_ATMOSPHERE"));

    // Verify varyings & shader source lines
    let vs_text = vs.sources.join("\n");
    let fs_text = fs.sources.join("\n");

    assert!(vs_text.contains(ATMOSPHERE_STAGE_VS.trim()));
    assert!(fs_text.contains(ATMOSPHERE_STAGE_FS.trim()));
    assert!(fs_text.contains("uniform bool u_isInFog;"));
}

#[test]
fn test_u_is_in_fog_is_false_if_camera_at_model_center() {
    let mut shader_builder = ShaderBuilder::new();
    let mut uniform_map = HashMap::new();
    let mut resources = AtmosphereRenderResources {
        shader_builder: &mut shader_builder,
        uniform_map: &mut uniform_map,
    };

    let center = Cartesian3::new(6378137.0, 0.0, 0.0);
    let model = AtmosphereModelInfo {
        bounding_sphere_center: center,
    };
    let frame_state = AtmosphereFrameState {
        camera_position_wc: center,
        fog_density: 0.0006,
        camera_in_space: false,
    };

    AtmospherePipelineStage::process(&mut resources, &model, &frame_state);

    let callback = resources
        .uniform_map
        .get("u_isInFog")
        .expect("u_isInFog callback should exist");
    assert!(!callback());
}

#[test]
fn test_u_is_in_fog_is_false_if_camera_in_space() {
    let mut shader_builder = ShaderBuilder::new();
    let mut uniform_map = HashMap::new();
    let mut resources = AtmosphereRenderResources {
        shader_builder: &mut shader_builder,
        uniform_map: &mut uniform_map,
    };

    let center = Cartesian3::new(6378137.0, 0.0, 0.0);
    let model = AtmosphereModelInfo {
        bounding_sphere_center: center,
    };
    let frame_state = AtmosphereFrameState {
        camera_position_wc: Cartesian3::new(6378137.0 + 900000.0, 0.0, 0.0),
        fog_density: 0.0006,
        camera_in_space: true,
    };

    AtmospherePipelineStage::process(&mut resources, &model, &frame_state);

    let callback = resources
        .uniform_map
        .get("u_isInFog")
        .expect("u_isInFog callback should exist");
    assert!(!callback());
}

#[test]
fn test_u_is_in_fog_is_true_when_tile_in_fog() {
    let mut shader_builder = ShaderBuilder::new();
    let mut uniform_map = HashMap::new();
    let mut resources = AtmosphereRenderResources {
        shader_builder: &mut shader_builder,
        uniform_map: &mut uniform_map,
    };

    let center = Cartesian3::new(6378137.0, 0.0, 0.0);
    let model = AtmosphereModelInfo {
        bounding_sphere_center: center,
    };
    let frame_state = AtmosphereFrameState {
        camera_position_wc: Cartesian3::new(6378137.0 + 10000.0, 0.0, 0.0),
        fog_density: 0.0006,
        camera_in_space: false,
    };

    AtmospherePipelineStage::process(&mut resources, &model, &frame_state);

    let callback = resources
        .uniform_map
        .get("u_isInFog")
        .expect("u_isInFog callback should exist");
    assert!(callback());
}
