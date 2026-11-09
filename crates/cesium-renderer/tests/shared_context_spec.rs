//! Direct port and verification against CesiumJS `Renderer/SharedContextSpec.js` (Apache-2.0).

use cesium_renderer::{ContextOptions, SharedContext, SharedContextOptions};

#[test]
fn test_scene_context_forwards_to_shared_context() {
    let shared = SharedContext::new(None).unwrap();
    let scene = shared.create_scene_context("canvas_1", 100, 200).unwrap();

    assert_eq!(scene.canvas().unwrap(), "canvas_1");
    assert_eq!(scene.drawing_buffer_width().unwrap(), 100);
    assert_eq!(scene.drawing_buffer_height().unwrap(), 200);

    let (shared_w, shared_h) = shared.drawing_buffer_dimensions();
    assert_ne!(scene.drawing_buffer_width().unwrap(), shared_w);
    assert_ne!(scene.drawing_buffer_height().unwrap(), shared_h);
}

#[test]
fn test_canvas_returns_scene_canvas() {
    let shared = SharedContext::new(None).unwrap();
    let scene = shared.create_scene_context("scene_canvas", 50, 50).unwrap();
    assert_eq!(scene.canvas().unwrap(), "scene_canvas");
}

#[test]
fn test_throws_if_attempting_to_create_multiple_scene_contexts_for_same_canvas() {
    let shared = SharedContext::new(None).unwrap();
    let _s1 = shared.create_scene_context("shared_canvas", 50, 50).unwrap();
    let s2_res = shared.create_scene_context("shared_canvas", 50, 50);
    assert!(s2_res.is_err());
    let err = s2_res.err().unwrap();
    assert!(err.message.contains("already associated"));
}

#[test]
fn test_throws_upon_failure_to_obtain_context() {
    let shared = SharedContext::new(None).unwrap();
    let res = shared.create_scene_context("", 50, 50);
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert!(err.message.contains("must provide a 2d context"));
}

#[test]
fn test_obtains_drawing_buffer_width_height_from_scene_canvas() {
    let shared = SharedContext::new(None).unwrap();
    let scene = shared.create_scene_context("canvas_custom", 5, 10).unwrap();

    assert_eq!(scene.drawing_buffer_width().unwrap(), 5);
    assert_eq!(scene.drawing_buffer_height().unwrap(), 10);

    let (sw, sh) = shared.drawing_buffer_dimensions();
    assert_ne!(scene.drawing_buffer_width().unwrap(), sw);
    assert_ne!(scene.drawing_buffer_height().unwrap(), sh);
}

#[test]
fn test_resizes_offscreen_canvas_to_be_at_least_as_large_as_onscreen_canvas() {
    let shared = SharedContext::new(Some(SharedContextOptions {
        context_options: Some(ContextOptions {
            width: Some(100),
            height: Some(100),
            ..Default::default()
        }),
        ..Default::default()
    }))
    .unwrap();

    let mut scene = shared.create_scene_context("canvas_resize", 5, 10).unwrap();
    let (start_w, start_h) = shared.drawing_buffer_dimensions();
    assert!(start_w > 5);
    assert!(start_h > 10);

    scene.begin_frame().unwrap();
    let (w1, h1) = shared.drawing_buffer_dimensions();
    assert_eq!(w1, start_w);
    assert_eq!(h1, start_h);

    scene.set_dimensions(start_w + 5, 10);
    assert_eq!(shared.drawing_buffer_dimensions().0, start_w);

    scene.begin_frame().unwrap();
    assert_eq!(shared.drawing_buffer_dimensions().0, start_w + 5);
    assert_eq!(shared.drawing_buffer_dimensions().1, start_h);

    scene.set_dimensions(start_w + 5, start_h + 10);
    scene.begin_frame().unwrap();
    assert_eq!(shared.drawing_buffer_dimensions().0, start_w + 5);
    assert_eq!(shared.drawing_buffer_dimensions().1, start_h + 10);

    // Shrinking scene canvas does not shrink shared canvas
    scene.set_dimensions(5, 10);
    scene.begin_frame().unwrap();
    assert_eq!(shared.drawing_buffer_dimensions().0, start_w + 5);
    assert_eq!(shared.drawing_buffer_dimensions().1, start_h + 10);
}

#[test]
fn test_destroys_shared_context_after_all_scene_contexts_are_destroyed() {
    let shared = SharedContext::new(None).unwrap();
    assert!(!shared.is_destroyed());
    assert_eq!(shared.canvases().len(), 0);

    let mut ctx0 = shared.create_scene_context("canvas_a", 10, 10).unwrap();
    let mut ctx1 = shared.create_scene_context("canvas_b", 10, 10).unwrap();
    assert_eq!(shared.canvases(), vec!["canvas_a", "canvas_b"]);

    ctx0.destroy().unwrap();
    assert!(ctx0.is_destroyed());
    assert_eq!(shared.canvases(), vec!["canvas_b"]);
    assert!(!shared.is_destroyed());
    assert!(ctx0.canvas().is_err());

    ctx1.destroy().unwrap();
    assert!(ctx1.is_destroyed());
    assert!(shared.canvases().is_empty());
    assert!(shared.is_destroyed());
}

#[test]
fn test_does_not_auto_destroy_after_all_scene_context_are_destroyed_if_so_specified() {
    let mut shared = SharedContext::new(Some(SharedContextOptions {
        auto_destroy: Some(false),
        ..Default::default()
    }))
    .unwrap();

    let mut ctx = shared.create_scene_context("canvas_manual", 10, 10).unwrap();
    ctx.destroy().unwrap();

    assert!(shared.canvases().is_empty());
    assert!(ctx.is_destroyed());
    assert!(!shared.is_destroyed());

    shared.destroy().unwrap();
    assert!(shared.is_destroyed());
}
