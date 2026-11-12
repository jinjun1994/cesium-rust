use cesium_math::{
    equals_epsilon, Cartesian3, Cartographic, SceneMode, EPSILON10, EPSILON11, EPSILON12,
    EPSILON13, EPSILON14, EPSILON9,
};
use cesium_renderer::{Fog, FogCameraState, FogFrameState};

fn make_frame_state_tilted() -> FogFrameState {
    // Choose direction slightly off-nadir so 1 - |dot(dir, pos_norm)| ≈ 1.25e-6
    // matching CesiumJS default scene camera view in FogSpec.js.
    let one_minus_dot = 1.25e-6_f64;
    let cos_theta = 1.0 - one_minus_dot;
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
    FogFrameState {
        mode: SceneMode::Scene3D,
        camera: FogCameraState {
            position_wc: Cartesian3::new(0.0, -1.0e7, 0.0),
            direction_wc: Cartesian3::new(sin_theta, cos_theta, 0.0),
            position_cartographic: Some(Cartographic::new(0.0, 0.0, 10_000.0)),
        },
        ..Default::default()
    }
}

fn test_density_at_height(fog: &Fog, height: f64, expected_density: f64, epsilon: f64) {
    let mut frame_state = make_frame_state_tilted();
    frame_state.camera.position_cartographic = Some(Cartographic::new(0.0, 0.0, height));
    fog.update(&mut frame_state);
    assert!(
        equals_epsilon(frame_state.fog.density, expected_density, None, Some(epsilon)),
        "at height {height}: got {}, expected {expected_density}",
        frame_state.fog.density
    );
}

#[test]
fn disabled_when_above_max_height() {
    let mut fog = Fog::new();
    fog.max_height = 800_000.0;
    let mut frame_state = make_frame_state_tilted();

    frame_state.camera.position_cartographic = Some(Cartographic::new(0.0, 0.0, 800_001.0));
    fog.update(&mut frame_state);
    assert!(!frame_state.fog.enabled);
    assert_eq!(frame_state.fog.density, 0.0);

    frame_state.camera.position_cartographic = Some(Cartographic::new(0.0, 0.0, 5_000.0));
    fog.update(&mut frame_state);
    assert!(frame_state.fog.enabled);
    assert!(frame_state.fog.density > 0.0);

    fog.max_height = 5_000.0;
    frame_state.camera.position_cartographic = Some(Cartographic::new(0.0, 0.0, 5_001.0));
    fog.update(&mut frame_state);
    assert!(!frame_state.fog.enabled);
    assert_eq!(frame_state.fog.density, 0.0);

    frame_state.camera.position_cartographic = Some(Cartographic::new(0.0, 0.0, 4_000.0));
    fog.update(&mut frame_state);
    assert!(frame_state.fog.enabled);
    assert!(frame_state.fog.density > 0.0);
}

#[test]
fn passes_through_expected_values_unaltered() {
    let mut fog = Fog::new();
    fog.screen_space_error_factor = 11.0;
    fog.visual_density_scalar = 12.0;
    fog.minimum_brightness = 13.0;

    let mut frame_state = make_frame_state_tilted();
    frame_state.camera.position_cartographic = Some(Cartographic::new(0.0, 0.0, 10_000.0));
    fog.update(&mut frame_state);

    assert!(frame_state.fog.enabled);
    assert!(frame_state.fog.density > 0.0);
    assert_eq!(frame_state.fog.sse, 11.0);
    assert_eq!(frame_state.fog.visual_density_scalar, 12.0);
    assert_eq!(frame_state.fog.minimum_brightness, 13.0);
}

#[test]
fn density_is_0_when_not_in_3d() {
    let fog = Fog::new();
    let mut frame_state = make_frame_state_tilted();
    frame_state.mode = SceneMode::Scene2D;
    fog.update(&mut frame_state);
    assert_eq!(frame_state.fog.density, 0.0);
    assert!(!frame_state.fog.enabled);
}

#[test]
fn density_function_across_heights_and_scalars() {
    let mut fog = Fog::new();
    assert!(fog.set_height_falloff(-1.0).is_err());

    // underground
    for h in [-1.0, -10.0, -100.0, -1000.0, -10_000.0] {
        test_density_at_height(&fog, h, 1.7e-10, EPSILON11);
    }

    // close to ground
    for h in [1.0, 10.0, 20.0, 50.0] {
        test_density_at_height(&fog, h, 1.7e-10, EPSILON11);
    }

    // mid heights
    test_density_at_height(&fog, 100.0, 1.5e-10, EPSILON11);
    test_density_at_height(&fog, 1000.0, 3.9e-11, EPSILON12);
    test_density_at_height(&fog, 2000.0, 2.5e-11, EPSILON12);
    test_density_at_height(&fog, 5000.0, 1.5e-11, EPSILON12);

    // high heights
    test_density_at_height(&fog, 10_000.0, 1.0e-11, EPSILON12);
    test_density_at_height(&fog, 100_000.0, 2.5e-12, EPSILON13);
    test_density_at_height(&fog, 200_000.0, 1.7e-12, EPSILON13);
    test_density_at_height(&fog, 500_000.0, 9.9e-13, EPSILON14);

    // higher density
    fog.density = 0.005;
    for h in [1.0, 10.0, 20.0, 50.0] {
        test_density_at_height(&fog, h, 1.4e-9, EPSILON10);
    }
    test_density_at_height(&fog, 100.0, 1.2e-9, EPSILON10);
    test_density_at_height(&fog, 1000.0, 3.2e-10, EPSILON11);
    test_density_at_height(&fog, 2000.0, 2.1e-10, EPSILON11);
    test_density_at_height(&fog, 5000.0, 1.2e-10, EPSILON11);
    test_density_at_height(&fog, 10_000.0, 8.3e-11, EPSILON12);
    test_density_at_height(&fog, 100_000.0, 2.1e-11, EPSILON12);
    test_density_at_height(&fog, 200_000.0, 1.4e-11, EPSILON12);
    test_density_at_height(&fog, 500_000.0, 8.3e-12, EPSILON13);

    // heightScalar directly affects density
    let mut fog2 = Fog::new();
    fog2.height_scalar = 1.0;
    test_density_at_height(&fog2, 1000.0, 3.9e-8, EPSILON9);
    fog2.height_scalar = 2.0;
    test_density_at_height(&fog2, 1000.0, 3.9e-8 * 2.0, EPSILON9);
}
