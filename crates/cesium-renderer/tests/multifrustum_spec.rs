//! MultifrustumSpec matching CesiumJS `Specs/Scene/MultifrustumSpec.js` (Apache-2.0).

use std::collections::HashMap;

use cesium_math::{BoundingSphere, Color, SceneMode, EPSILON2};
use glam::DVec3;
use cesium_renderer::{
    commands::{DrawCommand, DrawCommandOptions, Pass},
    MultiFrustum, MultiFrustumOptions,
};

fn setup_multifrustum_scene(use_log_depth: bool) -> (MultiFrustum, DVec3, DVec3, f64, f64) {
    let multi_frustum = MultiFrustum::new(MultiFrustumOptions {
        use_log_depth: Some(use_log_depth),
        far_to_near_ratio: Some(1000.0),
        logarithmic_depth_far_to_near_ratio: Some(1e9),
        near_to_far_distance_2d: Some(1.75e6),
        scene_mode: Some(SceneMode::Scene3D),
        debug_show_frustums: Some(false),
    });

    // Camera settings from MultifrustumSpec.js
    let camera_position = DVec3::ZERO;
    let camera_direction = -DVec3::Z; // Looking down -Z
    let camera_near = 1.0;
    let camera_far = 1_000_000_000.0;

    (
        multi_frustum,
        camera_position,
        camera_direction,
        camera_near,
        camera_far,
    )
}

fn create_billboard_commands() -> (Vec<DrawCommand>, HashMap<String, Color>) {
    let mut commands = Vec::new();
    let mut colors = HashMap::new();

    // billboard0: position (0, 0, -50), green
    commands.push(DrawCommand::new(DrawCommandOptions {
        bounding_volume: Some(BoundingSphere::new(DVec3::new(0.0, 0.0, -50.0), 1.0)),
        owner: Some("billboard0".to_string()),
        pass: Some(Pass::Opaque),
        ..Default::default()
    }));
    colors.insert("billboard0".to_string(), Color::new(0.0, 1.0, 0.0, 1.0));

    // billboard1: position (0, 0, -50000), blue
    commands.push(DrawCommand::new(DrawCommandOptions {
        bounding_volume: Some(BoundingSphere::new(
            DVec3::new(0.0, 0.0, -50000.0),
            10.0,
        )),
        owner: Some("billboard1".to_string()),
        pass: Some(Pass::Opaque),
        ..Default::default()
    }));
    colors.insert("billboard1".to_string(), Color::new(0.0, 0.0, 1.0, 1.0));

    // billboard2: position (0, 0, -50000000), white
    commands.push(DrawCommand::new(DrawCommandOptions {
        bounding_volume: Some(BoundingSphere::new(
            DVec3::new(0.0, 0.0, -50000000.0),
            100.0,
        )),
        owner: Some("billboard2".to_string()),
        pass: Some(Pass::Opaque),
        ..Default::default()
    }));
    colors.insert("billboard2".to_string(), Color::new(1.0, 1.0, 1.0, 1.0));

    (commands, colors)
}

#[test]
fn test_renders_primitive_in_closest_frustum() {
    let (mut mf, cam_pos, cam_dir, cam_near, cam_far) = setup_multifrustum_scene(false);
    let (commands, colors) = create_billboard_commands();

    mf.create_potentially_visible_set(cam_pos, cam_dir, cam_near, cam_far, 0.0, &commands);

    assert_eq!(mf.frustum_commands_list.len(), 3);
    let rgba = mf.simulate_rendered_rgba(&colors);
    // Green billboard in closest frustum
    assert_eq!(rgba[0], 0);
    assert_ne!(rgba[1], 0);
    assert_eq!(rgba[2], 0);
    assert_eq!(rgba[3], 255);
}

#[test]
fn test_renders_primitive_in_middle_frustum() {
    let (mut mf, cam_pos, cam_dir, cam_near, cam_far) = setup_multifrustum_scene(false);
    let (commands, mut colors) = create_billboard_commands();

    // billboard0 is transparent
    colors.insert("billboard0".to_string(), Color::new(1.0, 1.0, 1.0, 0.0));

    mf.create_potentially_visible_set(cam_pos, cam_dir, cam_near, cam_far, 0.0, &commands);

    let rgba = mf.simulate_rendered_rgba(&colors);
    // Blue billboard in middle frustum
    assert_eq!(rgba, [0, 0, 255, 255]);
}

#[test]
fn test_renders_primitive_in_last_frustum() {
    let (mut mf, cam_pos, cam_dir, cam_near, cam_far) = setup_multifrustum_scene(false);
    let (commands, mut colors) = create_billboard_commands();

    // billboard0 and billboard1 are transparent
    colors.insert("billboard0".to_string(), Color::new(1.0, 1.0, 1.0, 0.0));
    colors.insert("billboard1".to_string(), Color::new(1.0, 1.0, 1.0, 0.0));

    mf.create_potentially_visible_set(cam_pos, cam_dir, cam_near, cam_far, 0.0, &commands);

    let rgba = mf.simulate_rendered_rgba(&colors);
    // White billboard in last frustum
    assert_eq!(rgba, [255, 255, 255, 255]);
}

#[test]
fn test_renders_primitive_in_last_frustum_with_debug_show_frustums() {
    let (mut mf, cam_pos, cam_dir, cam_near, cam_far) = setup_multifrustum_scene(false);
    mf.debug_show_frustums = true;

    let (commands, _) = create_billboard_commands();
    mf.create_potentially_visible_set(cam_pos, cam_dir, cam_near, cam_far, 0.0, &commands);

    // Verify debug stats collected
    assert_eq!(mf.debug_frustum_statistics.total_commands, 3);
    assert!(!mf.debug_frustum_statistics.commands_in_frustums.is_empty());

    // Verify fragment shader transformation contains czm_Debug_main
    let sample_fs = "void main() { out_FragColor = vec4(1.0); }";
    let debug_fs = MultiFrustum::create_debug_show_frustums_fragment_shader(sample_fs);
    assert!(debug_fs.contains("czm_Debug_main"));
    assert!(debug_fs.contains("debugShowCommandsColor"));
    assert!(debug_fs.contains("debugShowFrustumsColor"));

    // Verify color mapping for debug frustums
    let frustum0_color = MultiFrustum::compute_debug_frustums_color(1 << 0);
    assert_eq!(frustum0_color, Color::new(1.0, 0.0, 0.0, 1.0));
    let frustum1_color = MultiFrustum::compute_debug_frustums_color(1 << 1);
    assert_eq!(frustum1_color, Color::new(0.0, 1.0, 0.0, 1.0));
    let frustum2_color = MultiFrustum::compute_debug_frustums_color(1 << 2);
    assert_eq!(frustum2_color, Color::new(0.0, 0.0, 1.0, 1.0));
}

#[test]
fn test_renders_primitive_with_undefined_bounding_volume() {
    let (mut mf, cam_pos, cam_dir, cam_near, cam_far) = setup_multifrustum_scene(false);

    // Primitive with undefined bounding volume (None)
    let unbounded_command = DrawCommand::new(DrawCommandOptions {
        bounding_volume: None,
        oriented_bounding_box: None,
        owner: Some("unbounded_box".to_string()),
        pass: Some(Pass::Opaque),
        ..Default::default()
    });

    let mut colors = HashMap::new();
    colors.insert(
        "unbounded_box".to_string(),
        Color::new(1.0, 1.0, 0.0, 1.0),
    );

    mf.create_potentially_visible_set(
        cam_pos,
        cam_dir,
        cam_near,
        cam_far,
        0.0,
        &[unbounded_command],
    );

    // When bounding volume is undefined, command is placed into all frustums
    for frustum in &mf.frustum_commands_list {
        assert_eq!(frustum.indices[Pass::Opaque as usize], 1);
    }

    let rgba = mf.simulate_rendered_rgba(&colors);
    // Yellow
    assert_eq!(rgba, [255, 255, 0, 255]);
}

#[test]
fn test_renders_only_in_the_closest_frustum() {
    let (mut mf, cam_pos, cam_dir, cam_near, cam_far) = setup_multifrustum_scene(false);
    let (mut commands, mut colors) = create_billboard_commands();

    // Billboards set to transparent
    colors.insert("billboard0".to_string(), Color::new(1.0, 1.0, 1.0, 0.0));
    colors.insert("billboard1".to_string(), Color::new(1.0, 1.0, 1.0, 0.0));
    colors.insert("billboard2".to_string(), Color::new(1.0, 1.0, 1.0, 0.0));

    // Primitive that spans across multiple frustums:
    // Center at -50000, radius 500000 (distance spans 0 to 550000)
    // execute_in_closest_frustum = true
    let yellow_primitive = DrawCommand::new(DrawCommandOptions {
        bounding_volume: Some(BoundingSphere::new(
            DVec3::new(0.0, 0.0, -50000.0),
            500000.0,
        )),
        owner: Some("closest_box".to_string()),
        pass: Some(Pass::Opaque),
        execute_in_closest_frustum: Some(true),
        ..Default::default()
    });
    colors.insert(
        "closest_box".to_string(),
        Color::new(1.0, 1.0, 0.0, 0.5),
    );
    commands.push(yellow_primitive);

    mf.create_potentially_visible_set(cam_pos, cam_dir, cam_near, cam_far, 0.0, &commands);

    // Because execute_in_closest_frustum is true, the box is ONLY inserted in the closest frustum (frustum 0)
    let f0_cmds = &mf.frustum_commands_list[0].commands[Pass::Opaque as usize];
    let f0_has_box = f0_cmds
        .iter()
        .any(|c| c.owner.as_deref() == Some("closest_box"));
    assert!(f0_has_box);

    for frustum in &mf.frustum_commands_list[1..] {
        let f_cmds = &frustum.commands[Pass::Opaque as usize];
        let f_has_box = f_cmds
            .iter()
            .any(|c| c.owner.as_deref() == Some("closest_box"));
        assert!(!f_has_box, "Should NOT be inserted into farther frustums");
    }

    let rgba = mf.simulate_rendered_rgba(&colors);
    // Yellowish alpha blended
    assert_ne!(rgba[0], 0);
    assert_ne!(rgba[1], 0);
    assert_eq!(rgba[2], 0);
    assert_eq!(rgba[3], 255);
}

#[test]
fn test_render_without_a_central_body_or_any_primitives() {
    let (mut mf, cam_pos, cam_dir, cam_near, cam_far) = setup_multifrustum_scene(false);

    // Empty commands list
    mf.create_potentially_visible_set(cam_pos, cam_dir, cam_near, cam_far, 0.0, &[]);

    assert!(!mf.frustum_commands_list.is_empty());
    assert!(!mf.frustum_splits.is_empty());
    assert!(mf.frustum_commands_list[0].near.is_finite());
    assert!(mf.frustum_commands_list[0].far.is_finite());
}

#[test]
fn test_does_not_crash_when_near_plane_is_greater_than_or_equal_to_far_plane() {
    let (mut mf, _, cam_dir, _, _) = setup_multifrustum_scene(false);

    // Extreme camera position and small far plane matching spec
    let extreme_cam_pos = DVec3::new(0.0, 0.0, 1e12);
    let camera_near = 1.0;
    let camera_far = 1000.0;

    let (commands, _) = create_billboard_commands();

    // Must not panic or crash
    mf.create_potentially_visible_set(
        extreme_cam_pos,
        cam_dir,
        camera_near,
        camera_far,
        1e12,
        &commands,
    );

    assert!(!mf.frustum_commands_list.is_empty());
    for f in &mf.frustum_commands_list {
        assert!(f.near.is_finite());
        assert!(f.far.is_finite());
        assert!(f.near <= f.far);
    }
}

#[test]
fn test_log_depth_uses_less_frustums() {
    let (commands, _) = create_billboard_commands();

    // Without log depth: uses 3 frustums
    let (mut mf_standard, cam_pos, cam_dir, cam_near, cam_far) = setup_multifrustum_scene(false);
    mf_standard.create_potentially_visible_set(
        cam_pos, cam_dir, cam_near, cam_far, 0.0, &commands,
    );
    assert_eq!(mf_standard.frustum_commands_list.len(), 3);

    // With log depth: uses 1 frustum
    let (mut mf_log, cam_pos, cam_dir, cam_near, cam_far) = setup_multifrustum_scene(true);
    mf_log.create_potentially_visible_set(cam_pos, cam_dir, cam_near, cam_far, 0.0, &commands);
    assert_eq!(mf_log.frustum_commands_list.len(), 1);
}

#[test]
fn test_2d_mode_frustum_distribution() {
    let mut mf = MultiFrustum::new(MultiFrustumOptions {
        scene_mode: Some(SceneMode::Scene2D),
        near_to_far_distance_2d: Some(1.75e6),
        ..Default::default()
    });

    mf.update_frustums(1.0, 1e9, 1e7, 10.0, 5e6);
    assert!(!mf.frustum_commands_list.is_empty());
    for f in &mf.frustum_commands_list {
        assert!(f.far >= f.near);
        assert!((f.far - f.near) <= 1.75e6 * (1.0 + EPSILON2));
    }
}
