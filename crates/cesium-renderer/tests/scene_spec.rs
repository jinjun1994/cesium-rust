//! SceneSpec matching CesiumJS `Scene/SceneSpec.js` (Apache-2.0).

use cesium_math::SceneMode;
use cesium_renderer::{
    commands::{DrawCommand, Pass},
    primitive_collection::ScenePrimitive,
    scene_primitive::PrimitiveRenderFrameState,
    Scene, SceneOptions,
};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

struct MockPrimitive {
    passes: Vec<Pass>,
}

impl MockPrimitive {
    fn new(passes: Vec<Pass>) -> Self {
        Self { passes }
    }
}

impl ScenePrimitive for MockPrimitive {
    fn id(&self) -> u64 {
        1
    }

    fn collect_draw_commands(&self, frame_state: &mut PrimitiveRenderFrameState) {
        for &pass in &self.passes {
            let cmd = DrawCommand {
                pass: Some(pass),
                ..Default::default()
            };
            frame_state.command_list.push(cmd);
        }
    }

    fn is_destroyed(&self) -> bool {
        false
    }

    fn destroy(&self) {}
}

#[test]
fn test_has_expected_defaults() {
    let scene = Scene::new(Some(SceneOptions {
        canvas: Some("test-canvas".to_string()),
        ..Default::default()
    }))
    .unwrap();

    assert_eq!(scene.canvas, "test-canvas");
    assert_eq!(scene.mode, SceneMode::Scene3D);
    assert_eq!(scene.morph_time, 1.0);
    assert!(scene.use_log_depth);
    assert!(!scene.request_render_mode);
    assert_eq!(scene.background_color, [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(scene.vertical_exaggeration, 1.0);
    assert_eq!(scene.vertical_exaggeration_relative_height, 0.0);
    assert!(!scene.debug_show_bounding_volume);
    assert!(!scene.debug_show_commands);
    assert!(!scene.debug_show_frames_per_second);
    assert_eq!(scene.primitive_count(), 0);
    assert!(!scene.is_destroyed());
}

#[test]
fn test_throws_without_options_or_canvas() {
    assert!(Scene::new(None).is_err());
    assert!(Scene::new(Some(SceneOptions {
        canvas: None,
        ..Default::default()
    }))
    .is_err());
    assert!(Scene::new(Some(SceneOptions {
        canvas: Some("".to_string()),
        ..Default::default()
    }))
    .is_err());
}

#[test]
fn test_sets_options() {
    let scene = Scene::new(Some(SceneOptions {
        canvas: Some("custom-canvas".to_string()),
        width: Some(1920),
        height: Some(1080),
        scene_mode: Some(SceneMode::ColumbusView),
        log_depth_buffer: Some(false),
        request_render_mode: Some(true),
        ..Default::default()
    }))
    .unwrap();

    assert_eq!(scene.width, 1920);
    assert_eq!(scene.height, 1080);
    assert_eq!(scene.mode, SceneMode::ColumbusView);
    assert_eq!(scene.morph_time, 0.0);
    assert!(!scene.use_log_depth);
    assert!(scene.request_render_mode);
}

#[test]
fn test_morph_functions() {
    let mut scene = Scene::new(Some(SceneOptions {
        canvas: Some("test-canvas".to_string()),
        ..Default::default()
    }))
    .unwrap();

    scene.morph_to_columbus_view(0.0);
    assert_eq!(scene.mode, SceneMode::ColumbusView);
    assert_eq!(scene.morph_time, 0.0);

    scene.morph_to_2d(0.0);
    assert_eq!(scene.mode, SceneMode::Scene2D);
    assert_eq!(scene.morph_time, 0.0);

    scene.morph_to_3d(0.0);
    assert_eq!(scene.mode, SceneMode::Scene3D);
    assert_eq!(scene.morph_time, 1.0);
}

#[test]
fn test_vertical_exaggeration() {
    let mut scene = Scene::new(Some(SceneOptions {
        canvas: Some("test-canvas".to_string()),
        ..Default::default()
    }))
    .unwrap();

    scene.set_vertical_exaggeration(2.5, 100.0);
    assert_eq!(scene.vertical_exaggeration, 2.5);
    assert_eq!(scene.vertical_exaggeration_relative_height, 100.0);
}

#[test]
fn test_primitives_management() {
    let mut scene = Scene::new(Some(SceneOptions {
        canvas: Some("test-canvas".to_string()),
        ..Default::default()
    }))
    .unwrap();

    let prim1: Arc<dyn ScenePrimitive> = Arc::new(MockPrimitive::new(vec![]));
    let prim2: Arc<dyn ScenePrimitive> = Arc::new(MockPrimitive::new(vec![]));

    scene.add_primitive(Arc::clone(&prim1));
    scene.add_primitive(Arc::clone(&prim2));
    assert_eq!(scene.primitive_count(), 2);

    assert!(scene.remove_primitive(&prim1));
    assert_eq!(scene.primitive_count(), 1);

    // Removing already removed returns false
    assert!(!scene.remove_primitive(&prim1));
}

#[test]
fn test_after_render_callbacks() {
    let mut scene = Scene::new(Some(SceneOptions {
        canvas: Some("test-canvas".to_string()),
        ..Default::default()
    }))
    .unwrap();

    let counter = Arc::new(AtomicU32::new(0));

    let counter_clone = Arc::clone(&counter);
    scene.add_after_render_callback(move |_scene| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    scene.render_for_specs().unwrap();
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    // Callbacks are drained so rendering next frame does not re-invoke unless rescheduled
    scene.render_for_specs().unwrap();
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_debug_command_filter_and_render_order() {
    let mut scene = Scene::new(Some(SceneOptions {
        canvas: Some("test-canvas".to_string()),
        ..Default::default()
    }))
    .unwrap();

    let prim: Arc<dyn ScenePrimitive> = Arc::new(MockPrimitive::new(vec![
        Pass::Translucent,
        Pass::Opaque,
        Pass::Overlay,
    ]));

    scene.add_primitive(prim);

    // 1. Without filter: sorted so Opaque comes before Translucent
    let cmds = scene.render_for_specs().unwrap();
    assert_eq!(cmds.len(), 3);
    assert_eq!(cmds[0].pass, Some(Pass::Opaque));
    assert_eq!(cmds[1].pass, Some(Pass::Overlay));
    assert_eq!(cmds[2].pass, Some(Pass::Translucent));

    // 2. With filter: only retain Opaque
    scene.set_debug_command_filter(Some(|cmd: &DrawCommand| cmd.pass == Some(Pass::Opaque)));
    let filtered_cmds = scene.render_for_specs().unwrap();
    assert_eq!(filtered_cmds.len(), 1);
    assert_eq!(filtered_cmds[0].pass, Some(Pass::Opaque));
}

#[test]
fn test_destroys() {
    let mut scene = Scene::new(Some(SceneOptions {
        canvas: Some("test-canvas".to_string()),
        ..Default::default()
    }))
    .unwrap();

    assert!(!scene.is_destroyed());
    scene.destroy();
    assert!(scene.is_destroyed());
    assert!(scene.render_for_specs().is_err());
}
