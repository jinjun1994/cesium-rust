use cesium_renderer::commands::{DrawCommand, Pass};
use cesium_renderer::pass_state::PassState;
use cesium_renderer::scene::{Scene, SceneOptions};
use cesium_renderer::texture::{Texture, TextureOptions};
use cesium_renderer::translucent_tile_classification::{
    FrustumCommandsClassification, TranslucentTileClassification,
    TranslucentTileClassificationOptions,
};

fn create_test_scene() -> Scene {
    Scene::new(Some(SceneOptions {
        canvas: Some("test_canvas".to_string()),
        ..Default::default()
    }))
    .unwrap()
}

fn expect_resources(trans: &TranslucentTileClassification, to_be_defined: bool) {
    assert_eq!(
        trans.draw_classification_fbo().framebuffer().is_some(),
        to_be_defined
    );
    assert_eq!(trans.pack_fbo().framebuffer().is_some(), to_be_defined);
    assert_eq!(
        trans.opaque_depth_stencil_texture().is_some(),
        to_be_defined
    );
    assert_eq!(
        trans
            .draw_classification_fbo()
            .get_color_texture(0)
            .ok()
            .flatten()
            .is_some(),
        to_be_defined
    );
    assert_eq!(
        trans.translucent_depth_stencil_texture().is_some(),
        to_be_defined
    );
    assert_eq!(
        trans
            .pack_fbo()
            .get_color_texture(0)
            .ok()
            .flatten()
            .is_some(),
        to_be_defined
    );
    assert_eq!(trans.pack_depth_command().is_some(), to_be_defined);
    assert_eq!(trans.accumulate_command().is_some(), to_be_defined);
    assert_eq!(trans.composite_command().is_some(), to_be_defined);
    assert_eq!(trans.copy_command().is_some(), to_be_defined);
}

fn create_depth_stencil_texture() -> Texture {
    Texture::new(TextureOptions {
        width: 100,
        height: 100,
        ..Default::default()
    })
}

#[test]
fn test_checks_for_support_on_construction() {
    let mut trans = TranslucentTileClassification::new(TranslucentTileClassificationOptions {
        depth_texture: true,
    })
    .unwrap();
    assert!(trans.is_supported());
    trans.destroy();

    let mut trans = TranslucentTileClassification::new(TranslucentTileClassificationOptions {
        depth_texture: false,
    })
    .unwrap();
    assert!(!trans.is_supported());
    trans.destroy();
}

#[test]
fn test_does_not_create_resources_if_unsupported() {
    let mut trans = TranslucentTileClassification::new(TranslucentTileClassificationOptions {
        depth_texture: false,
    })
    .unwrap();

    expect_resources(&trans, false);

    let mut scene = create_test_scene();
    let mut pass_state = PassState::new("context");
    let cmd = DrawCommand {
        pass: Some(Pass::Translucent),
        depth_for_translucent_classification: true,
        ..Default::default()
    };

    trans
        .execute_translucent_commands(&mut scene, &mut pass_state, &[cmd], None)
        .unwrap();

    expect_resources(&trans, false);
    trans.destroy();
}

#[test]
fn test_creates_resources_on_demand() {
    let mut trans = TranslucentTileClassification::new(TranslucentTileClassificationOptions {
        depth_texture: true,
    })
    .unwrap();

    expect_resources(&trans, false);

    let mut scene = create_test_scene();
    let mut pass_state = PassState::new("context");
    let depth_texture = create_depth_stencil_texture();

    // Empty commands -> should not create resources
    trans
        .execute_translucent_commands(&mut scene, &mut pass_state, &[], Some(&depth_texture))
        .unwrap();
    expect_resources(&trans, false);

    // Command with depth_for_translucent_classification -> creates resources on demand
    let cmd = DrawCommand {
        pass: Some(Pass::Translucent),
        depth_for_translucent_classification: true,
        ..Default::default()
    };

    trans
        .execute_translucent_commands(
            &mut scene,
            &mut pass_state,
            &[cmd],
            Some(&depth_texture),
        )
        .unwrap();

    expect_resources(&trans, true);
    trans.destroy();
}

#[test]
fn test_draws_translucent_commands_into_depth_buffer() {
    let mut trans = TranslucentTileClassification::new(TranslucentTileClassificationOptions {
        depth_texture: true,
    })
    .unwrap();

    let mut scene = create_test_scene();
    let mut pass_state = PassState::new("context");
    let depth_texture = create_depth_stencil_texture();

    let cmd = DrawCommand {
        pass: Some(Pass::Translucent),
        depth_for_translucent_classification: true,
        ..Default::default()
    };

    trans
        .execute_translucent_commands(
            &mut scene,
            &mut pass_state,
            &[cmd],
            Some(&depth_texture),
        )
        .unwrap();

    assert!(trans.has_translucent_depth());
    assert!(trans.pack_fbo().framebuffer().is_some());
    trans.destroy();
}

#[test]
fn test_draws_classification_commands_and_multifrustum_accumulation() {
    let mut trans = TranslucentTileClassification::new(TranslucentTileClassificationOptions {
        depth_texture: true,
    })
    .unwrap();

    let mut scene = create_test_scene();
    let mut pass_state = PassState::new("context");
    let depth_texture = create_depth_stencil_texture();

    let cmd = DrawCommand {
        pass: Some(Pass::Translucent),
        depth_for_translucent_classification: true,
        ..Default::default()
    };

    trans
        .execute_translucent_commands(
            &mut scene,
            &mut pass_state,
            std::slice::from_ref(&cmd),
            Some(&depth_texture),
        )
        .unwrap();

    let mut frustum_commands = FrustumCommandsClassification::default();
    let classification_cmd = DrawCommand {
        pass: Some(Pass::Cesium3dTileClassification),
        ..Default::default()
    };
    let pass_idx = Pass::Cesium3dTileClassification as usize;

    frustum_commands.commands.resize(pass_idx + 1, Vec::new());
    frustum_commands.indices.resize(pass_idx + 1, 0);
    frustum_commands.commands[pass_idx].push(classification_cmd);
    frustum_commands.indices[pass_idx] = 1;

    // First frustum
    trans
        .execute_classification_commands(&mut scene, &mut pass_state, &frustum_commands)
        .unwrap();
    assert!(trans.draw_classification_fbo().framebuffer().is_some());

    // Second frustum (multifrustum)
    trans
        .execute_translucent_commands(
            &mut scene,
            &mut pass_state,
            &[cmd],
            Some(&depth_texture),
        )
        .unwrap();
    trans
        .execute_classification_commands(&mut scene, &mut pass_state, &frustum_commands)
        .unwrap();

    assert!(trans.accumulation_fbo().framebuffer().is_some());
    trans.destroy();
}

#[test]
fn test_does_not_draw_classification_if_no_translucent_depth() {
    let mut trans = TranslucentTileClassification::new(TranslucentTileClassificationOptions {
        depth_texture: true,
    })
    .unwrap();

    let mut scene = create_test_scene();
    let mut pass_state = PassState::new("context");
    let depth_texture = create_depth_stencil_texture();

    // No commands executed
    trans
        .execute_translucent_commands(&mut scene, &mut pass_state, &[], Some(&depth_texture))
        .unwrap();
    assert!(!trans.has_translucent_depth());

    let frustum_commands = FrustumCommandsClassification::default();
    trans
        .execute_classification_commands(&mut scene, &mut pass_state, &frustum_commands)
        .unwrap();

    assert_eq!(pass_state.framebuffer, None);
    trans.destroy();
}

#[test]
fn test_composites_and_clears() {
    let mut trans = TranslucentTileClassification::new(TranslucentTileClassificationOptions {
        depth_texture: true,
    })
    .unwrap();

    let mut scene = create_test_scene();
    let mut pass_state = PassState::new("context");
    let depth_texture = create_depth_stencil_texture();

    let cmd = DrawCommand {
        pass: Some(Pass::Translucent),
        depth_for_translucent_classification: true,
        ..Default::default()
    };

    trans
        .execute_translucent_commands(
            &mut scene,
            &mut pass_state,
            &[cmd],
            Some(&depth_texture),
        )
        .unwrap();

    let mut frustum_commands = FrustumCommandsClassification::default();
    let classification_cmd = DrawCommand {
        pass: Some(Pass::Cesium3dTileClassification),
        ..Default::default()
    };
    let pass_idx = Pass::Cesium3dTileClassification as usize;
    frustum_commands.commands.resize(pass_idx + 1, Vec::new());
    frustum_commands.indices.resize(pass_idx + 1, 0);
    frustum_commands.commands[pass_idx].push(classification_cmd);
    frustum_commands.indices[pass_idx] = 1;

    trans
        .execute_classification_commands(&mut scene, &mut pass_state, &frustum_commands)
        .unwrap();

    // Composite & clear
    trans.execute(&mut scene, &mut pass_state).unwrap();
    assert!(trans.texture_to_composite().is_some());

    trans.destroy();
    assert!(trans.is_destroyed());
}
