use cesium_math::Color;
use cesium_renderer::planar_fill_id_framebuffer::PlanarFillIdFramebuffer;

#[test]
fn test_constructs() {
    let framebuffer = PlanarFillIdFramebuffer::new().expect("should construct");
    assert!(!framebuffer.is_destroyed());
    assert!(framebuffer.framebuffer().is_none());
}

#[test]
fn test_creates_framebuffer_with_correct_dimensions() {
    let mut framebuffer = PlanarFillIdFramebuffer::new().unwrap();
    let width = 256;
    let height = 256;
    let hdr = false;

    let updated = framebuffer.update(width, height, hdr).unwrap();
    assert!(updated);

    assert!(framebuffer.framebuffer().is_some());
    let id_texture = framebuffer.id_texture().expect("id texture should exist");
    assert_eq!(id_texture.width, width);
    assert_eq!(id_texture.height, height);

    framebuffer.destroy();
    assert!(framebuffer.is_destroyed());
}

#[test]
fn test_updates_framebuffer_when_dimensions_change() {
    let mut framebuffer = PlanarFillIdFramebuffer::new().unwrap();
    let mut width = 256;
    let mut height = 256;
    let hdr = false;

    framebuffer.update(width, height, hdr).unwrap();
    let original_id = framebuffer.framebuffer().unwrap().id;

    width = 512;
    height = 512;
    let changed = framebuffer.update(width, height, hdr).unwrap();
    assert!(changed);

    let new_id = framebuffer.framebuffer().unwrap().id;
    assert_ne!(original_id, new_id);

    let id_texture = framebuffer.id_texture().expect("id texture exists");
    assert_eq!(id_texture.width, width);
    assert_eq!(id_texture.height, height);

    framebuffer.destroy();
}

#[test]
fn test_does_not_update_framebuffer_when_dimensions_are_the_same() {
    let mut framebuffer = PlanarFillIdFramebuffer::new().unwrap();
    let width = 256;
    let height = 256;
    let hdr = false;

    let updated1 = framebuffer.update(width, height, hdr).unwrap();
    assert!(updated1);
    let original_id = framebuffer.framebuffer().unwrap().id;

    let updated2 = framebuffer.update(width, height, hdr).unwrap();
    assert!(!updated2);
    let same_id = framebuffer.framebuffer().unwrap().id;
    assert_eq!(original_id, same_id);

    framebuffer.destroy();
}

#[test]
fn test_provides_access_to_id_texture() {
    let mut framebuffer = PlanarFillIdFramebuffer::new().unwrap();
    let width = 256;
    let height = 256;
    let hdr = false;

    framebuffer.update(width, height, hdr).unwrap();

    let id_texture = framebuffer.id_texture();
    assert!(id_texture.is_some());
    let tex = id_texture.unwrap();
    assert_eq!(tex.width, width);
    assert_eq!(tex.height, height);

    framebuffer.destroy();
}

#[test]
fn test_provides_access_to_depth_stencil_texture_when_supported() {
    let mut framebuffer = PlanarFillIdFramebuffer::new().unwrap();
    let width = 256;
    let height = 256;
    let hdr = false;

    framebuffer.update(width, height, hdr).unwrap();

    let depth_stencil = framebuffer.depth_stencil_texture();
    assert!(depth_stencil.is_some());
    let ds_tex = depth_stencil.unwrap();
    assert_eq!(ds_tex.width, width);
    assert_eq!(ds_tex.height, height);

    framebuffer.destroy();
}

#[test]
fn test_returns_a_clear_command() {
    let mut framebuffer = PlanarFillIdFramebuffer::new().unwrap();
    let width = 256;
    let height = 256;
    let hdr = false;

    framebuffer.update(width, height, hdr).unwrap();

    let clear_command = framebuffer.get_clear_command(None).unwrap();
    assert_eq!(
        clear_command.framebuffer.as_ref().map(|fb| fb.id),
        Some(framebuffer.framebuffer().unwrap().id)
    );
    assert_eq!(clear_command.color, Some(Color::new(0.0, 0.0, 0.0, 0.0)));
    assert_eq!(clear_command.depth, Some(1.0));
    assert_eq!(clear_command.stencil, Some(0));

    // Custom clear color
    let custom_color = Color::new(1.0, 0.5, 0.25, 1.0);
    let custom_command = framebuffer.get_clear_command(Some(custom_color)).unwrap();
    assert_eq!(custom_command.color, Some(custom_color));

    framebuffer.destroy();
}

#[test]
fn test_destroys_properly() {
    let mut framebuffer = PlanarFillIdFramebuffer::new().unwrap();
    let width = 256;
    let height = 256;
    let hdr = false;

    framebuffer.update(width, height, hdr).unwrap();
    assert!(!framebuffer.is_destroyed());

    framebuffer.destroy();
    assert!(framebuffer.is_destroyed());

    // Calling operations on destroyed object fails
    assert!(framebuffer.update(width, height, hdr).is_err());
    assert!(framebuffer.get_clear_command(None).is_err());
}

#[test]
fn test_can_be_destroyed_without_calling_update() {
    let mut framebuffer = PlanarFillIdFramebuffer::new().unwrap();
    assert!(!framebuffer.is_destroyed());

    framebuffer.destroy();
    assert!(framebuffer.is_destroyed());
}

#[test]
fn test_release_resources() {
    let mut framebuffer = PlanarFillIdFramebuffer::new().unwrap();
    framebuffer.update(128, 128, false).unwrap();
    assert!(framebuffer.framebuffer().is_some());

    framebuffer.release_resources();
    assert!(!framebuffer.is_destroyed());

    // Can update again after release
    let re_updated = framebuffer.update(256, 256, false).unwrap();
    assert!(re_updated);
    assert!(framebuffer.framebuffer().is_some());
    assert_eq!(framebuffer.id_texture().unwrap().width, 256);

    framebuffer.destroy();
}
