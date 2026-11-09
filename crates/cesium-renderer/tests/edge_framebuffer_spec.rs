use cesium_math::DeveloperError;
use cesium_renderer::EdgeFramebuffer;

#[test]
fn test_constructs() {
    let edge_fb = EdgeFramebuffer::new().expect("should construct EdgeFramebuffer");
    assert!(!edge_fb.is_destroyed());
}

#[test]
fn test_creates_framebuffer_with_correct_dimensions() {
    let mut edge_fb = EdgeFramebuffer::new().expect("should construct");
    edge_fb.update(256, 256, false).expect("should update");

    assert!(edge_fb.framebuffer().is_some());
    let color_tex = edge_fb.color_texture().expect("should have color texture");
    assert_eq!(color_tex.width, 256);
    assert_eq!(color_tex.height, 256);

    edge_fb.destroy();
    assert!(edge_fb.is_destroyed());
}

#[test]
fn test_creates_multiple_render_targets() {
    let mut edge_fb = EdgeFramebuffer::new().expect("should construct");
    edge_fb.update(256, 256, false).expect("should update");

    let fb = edge_fb.framebuffer().expect("should have framebuffer");
    assert!(fb.color_textures.len() > 1);

    assert!(edge_fb.color_texture().is_some());
    assert!(edge_fb.id_texture().is_some());
    assert!(edge_fb.depth_texture().is_some());

    edge_fb.destroy();
}

#[test]
fn test_updates_framebuffer_when_dimensions_change() {
    let mut edge_fb = EdgeFramebuffer::new().expect("should construct");
    let changed1 = edge_fb.update(256, 256, false).expect("should update");
    assert!(changed1);

    let changed2 = edge_fb.update(512, 512, false).expect("should update");
    assert!(changed2);

    let color_tex = edge_fb.color_texture().expect("should have color texture");
    assert_eq!(color_tex.width, 512);
    assert_eq!(color_tex.height, 512);

    edge_fb.destroy();
}

#[test]
fn test_does_not_update_framebuffer_when_dimensions_are_the_same() {
    let mut edge_fb = EdgeFramebuffer::new().expect("should construct");
    let changed1 = edge_fb.update(256, 256, false).expect("should update");
    assert!(changed1);

    let changed2 = edge_fb.update(256, 256, false).expect("should update");
    assert!(!changed2);

    edge_fb.destroy();
}

#[test]
fn test_provides_access_to_edge_color_texture() {
    let mut edge_fb = EdgeFramebuffer::new().expect("should construct");
    edge_fb.update(256, 256, false).expect("should update");

    let color_tex = edge_fb.color_texture().expect("should have color texture");
    assert_eq!(color_tex.width, 256);
    assert_eq!(color_tex.height, 256);

    edge_fb.destroy();
}

#[test]
fn test_provides_access_to_edge_id_texture() {
    let mut edge_fb = EdgeFramebuffer::new().expect("should construct");
    edge_fb.update(256, 256, false).expect("should update");

    let id_tex = edge_fb.id_texture().expect("should have id texture");
    assert_eq!(id_tex.width, 256);
    assert_eq!(id_tex.height, 256);

    edge_fb.destroy();
}

#[test]
fn test_clears_the_framebuffer() {
    let mut edge_fb = EdgeFramebuffer::new().expect("should construct");
    edge_fb.update(256, 256, false).expect("should update");

    let clear_cmd = edge_fb.get_clear_command(None);
    assert!(clear_cmd.is_ok());

    edge_fb.destroy();
}

#[test]
fn test_destroys_properly() {
    let mut edge_fb = EdgeFramebuffer::new().expect("should construct");
    edge_fb.update(256, 256, false).expect("should update");
    assert!(!edge_fb.is_destroyed());

    edge_fb.destroy();
    assert!(edge_fb.is_destroyed());
}

#[test]
fn test_can_be_destroyed_multiple_times_without_errors() {
    let mut edge_fb = EdgeFramebuffer::new().expect("should construct");
    assert!(!edge_fb.is_destroyed());

    edge_fb.destroy();
    assert!(edge_fb.is_destroyed());

    edge_fb.destroy();
    assert!(edge_fb.is_destroyed());
}

#[test]
fn test_handles_invalid_dimensions_gracefully() {
    let mut edge_fb = EdgeFramebuffer::new().expect("should construct");

    let err1 = edge_fb.update(0, 256, false).expect_err("should reject 0 width");
    assert_eq!(err1.name(), DeveloperError::NAME);

    let err2 = edge_fb.update(256, 0, false).expect_err("should reject 0 height");
    assert_eq!(err2.name(), DeveloperError::NAME);

    edge_fb.destroy();
}
