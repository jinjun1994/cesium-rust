use cesium_math::{Cartesian3, SceneMode};
use cesium_renderer::sun::{Sun, SunFramePasses, SunFrameState};

#[test]
fn test_sun_draws_in_3d() {
    let mut sun = Sun::new();
    sun.set_glow_factor(100.0);

    let frame_state = SunFrameState {
        mode: SceneMode::Scene3D,
        sun_position_wc: Cartesian3::new(1.496e11, 0.0, 0.0),
        ..Default::default()
    };

    let commands = sun.update(&frame_state, None, false);
    assert!(commands.is_some());
    let cmd = commands.unwrap();
    assert!(cmd.draw_command.is_some());
    let draw_cmd = cmd.draw_command.as_ref().unwrap();
    assert!(draw_cmd.bounding_volume.is_some());
    assert_eq!(
        draw_cmd.bounding_volume.unwrap().center,
        frame_state.sun_position_wc.into()
    );
}

#[test]
fn test_sun_draws_in_columbus_view() {
    let mut sun = Sun::new();

    let frame_state = SunFrameState {
        mode: SceneMode::ColumbusView,
        sun_position_columbus_view: Cartesian3::new(10.0, 20.0, 30.0),
        ..Default::default()
    };

    let commands = sun.update(&frame_state, None, false);
    assert!(commands.is_some());
    let cmd = commands.unwrap();
    assert!(cmd.draw_command.is_some());
    let draw_cmd = cmd.draw_command.as_ref().unwrap();
    assert!(draw_cmd.bounding_volume.is_some());
    // Columbus view center maps (z, x, y)
    assert_eq!(
        draw_cmd.bounding_volume.unwrap().center,
        Cartesian3::new(30.0, 10.0, 20.0).into()
    );
}

#[test]
fn test_sun_does_not_render_when_show_is_false() {
    let mut sun = Sun::new();
    sun.show = false;

    let frame_state = SunFrameState::default();
    assert!(sun.update(&frame_state, None, false).is_none());
}

#[test]
fn test_sun_does_not_render_in_2d() {
    let mut sun = Sun::new();
    let frame_state = SunFrameState {
        mode: SceneMode::Scene2D,
        ..Default::default()
    };

    assert!(sun.update(&frame_state, None, false).is_none());
}

#[test]
fn test_sun_does_not_render_without_a_render_pass() {
    let mut sun = Sun::new();
    let frame_state = SunFrameState {
        passes: SunFramePasses {
            render: false,
            pick: false,
        },
        ..Default::default()
    };

    assert!(sun.update(&frame_state, None, false).is_none());
}

#[test]
fn test_sun_can_set_glow_factor() {
    let mut sun = Sun::new();
    sun.set_glow_factor(0.0);
    assert_eq!(sun.glow_factor(), 0.0);
    assert_eq!(sun.radius_ts, 0.5);

    sun.set_glow_factor(2.0);
    assert_eq!(sun.glow_factor(), 2.0);

    // Negative clamp to 0.0
    sun.set_glow_factor(-5.0);
    assert_eq!(sun.glow_factor(), 0.0);
}

#[test]
fn test_sun_draws_without_lens_flare() {
    let mut sun = Sun::new();
    sun.set_glow_factor(0.0);

    let frame_state = SunFrameState::default();
    let commands = sun.update(&frame_state, None, false);
    assert!(commands.is_some());
    assert!(commands.unwrap().draw_command.is_some());
}

#[test]
fn test_sun_is_destroyed() {
    let mut sun = Sun::new();
    assert!(!sun.is_destroyed());

    sun.destroy();
    assert!(sun.is_destroyed());

    let frame_state = SunFrameState::default();
    assert!(sun.update(&frame_state, None, false).is_none());
}
