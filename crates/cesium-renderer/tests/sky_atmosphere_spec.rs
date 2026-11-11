use cesium_math::{Ellipsoid, Matrix4, SceneMode};
use cesium_renderer::commands::Pass;
use cesium_renderer::dynamic_atmosphere_lighting_type::DynamicAtmosphereLightingType;
use cesium_renderer::sky_atmosphere::{
    SkyAtmosphere, SkyAtmosphereFramePasses, SkyAtmosphereFrameState,
};

#[test]
fn test_draws_sky_in_3d() {
    let mut sky = SkyAtmosphere::default();
    let frame_state = SkyAtmosphereFrameState {
        mode: SceneMode::Scene3D,
        passes: SkyAtmosphereFramePasses { render: true },
        inverse_view_rotation: Some(Matrix4::IDENTITY),
        globe_translucent: false,
    };

    let command = sky.update(&frame_state, None);
    assert!(command.is_some());
    let cmd = command.unwrap();
    assert_eq!(cmd.pass, Some(Pass::Environment));
    assert!(cmd.model_matrix.is_some());
}

#[test]
fn test_dynamic_lighting_scene_light() {
    let mut sky = SkyAtmosphere::default();
    sky.set_dynamic_lighting(DynamicAtmosphereLightingType::SceneLight);
    assert_eq!(sky.radii_and_dynamic_atmosphere_color.z, 1.0);
}

#[test]
fn test_dynamic_lighting_sunlight() {
    let mut sky = SkyAtmosphere::default();
    sky.set_dynamic_lighting(DynamicAtmosphereLightingType::Sunlight);
    assert_eq!(sky.radii_and_dynamic_atmosphere_color.z, 2.0);
}

#[test]
fn test_dynamic_lighting_off() {
    let mut sky = SkyAtmosphere::default();
    sky.set_dynamic_lighting(DynamicAtmosphereLightingType::None);
    assert_eq!(sky.radii_and_dynamic_atmosphere_color.z, 0.0);
}

#[test]
fn test_color_correction_active() {
    let mut sky = SkyAtmosphere::default();
    assert!(!sky.has_color_correction());

    sky.hue_shift = 0.5;
    assert!(sky.has_color_correction());
}

#[test]
fn test_does_not_render_when_show_is_false() {
    let mut sky = SkyAtmosphere::default();
    sky.show = false;

    let frame_state = SkyAtmosphereFrameState {
        mode: SceneMode::Scene3D,
        passes: SkyAtmosphereFramePasses { render: true },
        inverse_view_rotation: None,
        globe_translucent: false,
    };

    assert!(sky.update(&frame_state, None).is_none());
}

#[test]
fn test_does_not_render_in_2d() {
    let mut sky = SkyAtmosphere::default();
    let frame_state = SkyAtmosphereFrameState {
        mode: SceneMode::Scene2D,
        passes: SkyAtmosphereFramePasses { render: true },
        inverse_view_rotation: None,
        globe_translucent: false,
    };

    assert!(sky.update(&frame_state, None).is_none());
}

#[test]
fn test_does_not_render_without_render_pass() {
    let mut sky = SkyAtmosphere::default();
    let frame_state = SkyAtmosphereFrameState {
        mode: SceneMode::Scene3D,
        passes: SkyAtmosphereFramePasses { render: false },
        inverse_view_rotation: None,
        globe_translucent: false,
    };

    assert!(sky.update(&frame_state, None).is_none());
}

#[test]
fn test_gets_ellipsoid() {
    let sky = SkyAtmosphere::new(Some(Ellipsoid::UNIT_SPHERE));
    assert_eq!(sky.ellipsoid(), Ellipsoid::UNIT_SPHERE);
}

#[test]
fn test_is_destroyed() {
    let mut sky = SkyAtmosphere::default();
    assert!(!sky.is_destroyed());
    sky.destroy();
    assert!(sky.is_destroyed());
}
