use cesium_math::{
    Color, ColorBlendMode, HeightReference, Matrix4, SceneMode, ShadowMode, SplitDirection,
};
use cesium_renderer::{Model, ModelGltfOptions};
use serde_json::json;

#[test]
fn from_gltf_async_throws_with_undefined_options() {
    assert!(Model::from_gltf_async(None).is_err());
}

#[test]
fn from_gltf_async_throws_with_undefined_url_and_gltf() {
    assert!(Model::from_gltf_async(Some(ModelGltfOptions::default())).is_err());
}

#[test]
fn sets_default_properties() {
    let model = Model::new();

    assert!(model.show);
    assert_eq!(model.model_matrix, Matrix4::IDENTITY);
    assert_eq!(model.scale, 1.0);
    assert_eq!(model.minimum_pixel_size, 0.0);
    assert!(model.maximum_scale.is_none());

    assert!(model.id.is_none());
    assert!(model.allow_picking);
    assert!(model.enable_vertical_exaggeration);

    assert!(model.clamp_animations);
    assert_eq!(model.shadows, ShadowMode::Enabled);
    assert!(!model.debug_show_bounding_volume);
    assert!(!model.debug_wireframe);

    assert!(model.cull);
    assert_eq!(model.back_face_culling, Some(true));
    assert_eq!(model.height_reference, HeightReference::None);

    assert!(model.color.is_none());
    assert_eq!(model.color_blend_mode, ColorBlendMode::Highlight);
    assert_eq!(model.color_blend_amount, 0.5);
    assert_eq!(model.silhouette_color, Color::RED);
    assert_eq!(model.silhouette_size, 0.0);

    assert!(model.show_outline);
    assert_eq!(model.outline_color, Color::BLACK);
    assert_eq!(model.split_direction, SplitDirection::None);
    assert!(model.ready);
}

#[test]
fn initializes_from_json_object() {
    let gltf_json = json!({
        "asset": {
            "version": "2.0",
            "generator": "COLLADA2GLTF"
        },
        "scene": 0,
        "scenes": [{
            "nodes": [0]
        }],
        "nodes": [{
            "name": "RootNode"
        }]
    });

    let model = Model::from_gltf_async(Some(ModelGltfOptions {
        gltf: Some(gltf_json),
        show: Some(true),
        scale: Some(2.0),
        color: Some(Color::BLUE),
        color_blend_mode: Some(ColorBlendMode::Mix),
        ..Default::default()
    }))
    .expect("model from gltf json should succeed");

    assert!(model.ready);
    assert!(model.scene_graph.is_some());
    assert_eq!(model.scale, 2.0);
    assert_eq!(model.color, Some(Color::BLUE));
    assert_eq!(model.color_blend_mode, ColorBlendMode::Mix);
}

#[test]
fn initializes_from_bytes() {
    let dummy_glb_bytes = [0u8; 64];
    let model = Model::from_gltf_bytes(&dummy_glb_bytes)
        .expect("model from gltf bytes should succeed");

    assert!(model.ready);
    assert!(model.resources_loaded);
    assert!(model.scene_graph.is_some());
}

#[test]
fn show_works() {
    let mut model = Model::new();
    assert!(model.show);

    // Initial update produces commands
    let commands = model.update(SceneMode::Scene3D);
    assert!(!commands.is_empty());

    // When show is set to false, update clears draw commands
    model.show = false;
    let commands_hidden = model.update(SceneMode::Scene3D);
    assert!(commands_hidden.is_empty());

    // Restore show = true, commands regenerate
    model.show = true;
    let commands_shown = model.update(SceneMode::Scene3D);
    assert!(!commands_shown.is_empty());
}

#[test]
fn destroy_and_lifecycle() {
    let mut model = Model::new();
    assert!(!model.is_destroyed());

    model.update(SceneMode::Scene3D);
    assert!(!model.draw_commands.is_empty());

    model.destroy();
    assert!(model.is_destroyed());
    assert!(model.draw_commands.is_empty());
    assert!(model.scene_graph.is_none());
}
