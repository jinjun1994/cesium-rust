use cesium_math::{Color, NearFarScalar, SceneMode};
use cesium_renderer::commands::{DrawCommand, DrawCommandOptions, Pass};
use cesium_renderer::globe_translucency_state::{
    DerivedCommandType, GlobeTranslucencyConfig, GlobeTranslucencyInput, GlobeTranslucencyState,
};

fn raw_types(types: &[DerivedCommandType]) -> Vec<u8> {
    types.iter().map(|t| *t as u8).collect()
}

fn check_types(
    state: &GlobeTranslucencyState,
    expected_main: &[u8],
    expected_blend: &[u8],
    expected_pick: &[u8],
    expected_update: &[u8],
) {
    assert_eq!(raw_types(state.derived_command_types()), expected_main);
    assert_eq!(
        raw_types(state.derived_blend_command_types()),
        expected_blend
    );
    assert_eq!(raw_types(state.derived_pick_command_types()), expected_pick);
    assert_eq!(
        raw_types(state.derived_command_types_to_update()),
        expected_update
    );
}

#[test]
fn gets_front_and_back_face_alpha_by_distance() {
    let mut state = GlobeTranslucencyState::new();

    // Opaque
    state.update(&GlobeTranslucencyInput::default());
    assert_eq!(state.front_face_alpha_by_distance().near_value, 1.0);
    assert_eq!(state.front_face_alpha_by_distance().far_value, 1.0);
    assert_eq!(state.back_face_alpha_by_distance().near_value, 1.0);
    assert_eq!(state.back_face_alpha_by_distance().far_value, 1.0);

    // Front and back translucent
    let input = GlobeTranslucencyInput {
        translucency: GlobeTranslucencyConfig {
            enabled: true,
            front_face_alpha: 0.5,
            back_face_alpha: 0.25,
            ..Default::default()
        },
        ..Default::default()
    };
    state.update(&input);
    assert_eq!(state.front_face_alpha_by_distance().near_value, 0.5);
    assert_eq!(state.front_face_alpha_by_distance().far_value, 0.5);
    assert_eq!(state.back_face_alpha_by_distance().near_value, 0.25);
    assert_eq!(state.back_face_alpha_by_distance().far_value, 0.25);

    // Front and back translucent with alpha by distance
    let input = GlobeTranslucencyInput {
        translucency: GlobeTranslucencyConfig {
            enabled: true,
            front_face_alpha: 0.5,
            back_face_alpha: 0.25,
            front_face_alpha_by_distance: Some(NearFarScalar::new(0.0, 0.5, 1.0, 0.75)),
            ..Default::default()
        },
        ..Default::default()
    };
    state.update(&input);
    assert_eq!(state.front_face_alpha_by_distance().near_value, 0.25);
    assert_eq!(state.front_face_alpha_by_distance().far_value, 0.375);
    assert_eq!(state.back_face_alpha_by_distance().near_value, 0.25);
    assert_eq!(state.back_face_alpha_by_distance().far_value, 0.25);
}

#[test]
fn detects_translucent_sun_visible_environment_visible_and_depth_plane() {
    let mut state = GlobeTranslucencyState::new();

    // Globe absent
    state.update(&GlobeTranslucencyInput {
        globe_present: false,
        ..Default::default()
    });
    assert!(!state.translucent());
    assert!(state.sun_visible_through_globe());
    assert!(state.environment_visible());
    assert!(!state.use_depth_plane());

    // Globe hidden
    state.update(&GlobeTranslucencyInput {
        globe_show: false,
        ..Default::default()
    });
    assert!(!state.translucent());
    assert!(state.sun_visible_through_globe());
    assert!(state.environment_visible());
    assert!(!state.use_depth_plane());

    // Default opaque globe
    state.update(&GlobeTranslucencyInput::default());
    assert!(!state.translucent());
    assert!(!state.sun_visible_through_globe());
    assert!(state.environment_visible());
    assert!(state.use_depth_plane());

    // Base color transparent
    state.update(&GlobeTranslucencyInput {
        base_color: Color::TRANSPARENT,
        translucency: GlobeTranslucencyConfig {
            enabled: true,
            ..Default::default()
        },
        ..Default::default()
    });
    assert!(state.translucent());

    // Front face translucent, back opaque, above ground
    state.update(&GlobeTranslucencyInput {
        translucency: GlobeTranslucencyConfig {
            enabled: true,
            front_face_alpha: 0.5,
            ..Default::default()
        },
        ..Default::default()
    });
    assert!(state.translucent());
    assert!(!state.sun_visible_through_globe());
    assert!(state.environment_visible());
    assert!(!state.use_depth_plane());

    // Front face translucent, back translucent, above ground
    state.update(&GlobeTranslucencyInput {
        translucency: GlobeTranslucencyConfig {
            enabled: true,
            front_face_alpha: 0.5,
            back_face_alpha: 0.5,
            ..Default::default()
        },
        ..Default::default()
    });
    assert!(state.sun_visible_through_globe());

    // Opaque underground
    state.update(&GlobeTranslucencyInput {
        camera_underground: true,
        ..Default::default()
    });
    assert!(!state.environment_visible());
    assert!(!state.use_depth_plane());

    // Front face translucent underground
    state.update(&GlobeTranslucencyInput {
        camera_underground: true,
        translucency: GlobeTranslucencyConfig {
            enabled: true,
            front_face_alpha: 0.5,
            ..Default::default()
        },
        ..Default::default()
    });
    assert!(state.sun_visible_through_globe());
    assert!(state.environment_visible());
}

#[test]
fn gets_number_of_texture_uniforms_required() {
    let mut state = GlobeTranslucencyState::new();

    // Opaque -> 0
    state.update(&GlobeTranslucencyInput::default());
    assert_eq!(state.number_of_texture_uniforms(), 0);

    // Translucent + manual depth test -> 2
    state.update(&GlobeTranslucencyInput {
        translucency: GlobeTranslucencyConfig {
            enabled: true,
            front_face_alpha: 0.5,
            ..Default::default()
        },
        depth_texture_supported: true,
        ..Default::default()
    });
    assert_eq!(state.number_of_texture_uniforms(), 2);

    // Translucent + depthTestAgainstTerrain -> 1
    state.update(&GlobeTranslucencyInput {
        depth_test_against_terrain: true,
        translucency: GlobeTranslucencyConfig {
            enabled: true,
            front_face_alpha: 0.5,
            ..Default::default()
        },
        ..Default::default()
    });
    assert_eq!(state.number_of_texture_uniforms(), 1);
}

#[test]
fn gets_derived_commands_to_update_and_derives_passes() {
    let mut state = GlobeTranslucencyState::new();

    // Front opaque
    state.update(&GlobeTranslucencyInput::default());
    check_types(&state, &[], &[], &[], &[]);

    // Front translucent, back opaque, depthTestAgainstTerrain = true
    state.update(&GlobeTranslucencyInput {
        depth_test_against_terrain: true,
        translucency: GlobeTranslucencyConfig {
            enabled: true,
            front_face_alpha: 0.5,
            ..Default::default()
        },
        ..Default::default()
    });
    check_types(&state, &[2, 1, 5], &[1, 5], &[2, 1, 9], &[1, 2, 5, 9]);

    // Front translucent, back opaque, manual depth test
    state.update(&GlobeTranslucencyInput {
        depth_texture_supported: true,
        translucency: GlobeTranslucencyConfig {
            enabled: true,
            front_face_alpha: 0.5,
            ..Default::default()
        },
        ..Default::default()
    });
    check_types(&state, &[2, 1, 7], &[1, 7], &[2, 1, 9], &[1, 2, 7, 9]);
    assert!(state.derived_commands_dirty());

    // Front translucent, back opaque, manual depth test, camera underground
    state.update(&GlobeTranslucencyInput {
        camera_underground: true,
        depth_texture_supported: true,
        translucency: GlobeTranslucencyConfig {
            enabled: true,
            front_face_alpha: 0.5,
            ..Default::default()
        },
        ..Default::default()
    });
    check_types(&state, &[3, 0, 8], &[0, 8], &[3, 0, 10], &[0, 3, 8, 10]);

    // Front translucent, back translucent
    state.update(&GlobeTranslucencyInput {
        translucency: GlobeTranslucencyConfig {
            enabled: true,
            front_face_alpha: 0.5,
            back_face_alpha: 0.5,
            ..Default::default()
        },
        ..Default::default()
    });
    check_types(&state, &[4, 6, 5], &[6, 5], &[4, 10, 9], &[4, 5, 6, 9, 10]);

    // Front translucent, back translucent, camera underground
    state.update(&GlobeTranslucencyInput {
        camera_underground: true,
        translucency: GlobeTranslucencyConfig {
            enabled: true,
            front_face_alpha: 0.5,
            back_face_alpha: 0.5,
            ..Default::default()
        },
        ..Default::default()
    });
    check_types(&state, &[4, 5, 6], &[5, 6], &[4, 9, 10], &[4, 5, 6, 9, 10]);

    // Translucent, 2D
    state.update(&GlobeTranslucencyInput {
        mode: SceneMode::Scene2D,
        translucency: GlobeTranslucencyConfig {
            enabled: true,
            front_face_alpha: 0.5,
            ..Default::default()
        },
        ..Default::default()
    });
    check_types(&state, &[2, 5], &[2, 5], &[2, 9], &[2, 5, 9]);

    let base = DrawCommand::new(DrawCommandOptions::default());
    let derived = state.derive_commands(&base, false, false);
    assert_eq!(derived.len(), 2);
    assert_eq!(derived[0].0, DerivedCommandType::DepthOnlyFrontFace);
    assert_eq!(derived[0].1.pass, Some(Pass::Globe));
    assert_eq!(derived[1].0, DerivedCommandType::TranslucentFrontFace);
    assert_eq!(derived[1].1.pass, Some(Pass::Translucent));
}
