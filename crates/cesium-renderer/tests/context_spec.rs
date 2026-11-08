use cesium_math::Color;
use cesium_renderer::{
    ClearCommand, ClearCommandOptions, Context, ContextLimits, ContextOptions, ReadPixelsOptions,
};

#[test]
fn test_has_unique_id() {
    let c1 = Context::default();
    let c2 = Context::default();
    assert_ne!(c1.id, c2.id);
}

#[test]
fn test_stencil_bits() {
    let c = Context::default();
    assert!(c.stencil_bits >= 8);
}

#[test]
#[allow(clippy::assertions_on_constants)]
fn test_context_limits() {
    assert!(ContextLimits::MAXIMUM_COMBINED_TEXTURE_IMAGE_UNITS >= 32);
    assert!(ContextLimits::MAXIMUM_CUBE_MAP_SIZE >= 2048);
    assert!(ContextLimits::MAXIMUM_FRAGMENT_UNIFORM_VECTORS >= 224);
    assert!(ContextLimits::MAXIMUM_TEXTURE_IMAGE_UNITS >= 16);
    assert!(ContextLimits::MAXIMUM_RENDERBUFFER_SIZE >= 2048);
    assert!(ContextLimits::MAXIMUM_TEXTURE_SIZE >= 2048);
    assert!(ContextLimits::MAXIMUM_3D_TEXTURE_SIZE >= 256);
    assert!(ContextLimits::MAXIMUM_VARYING_VECTORS >= 15);
    assert!(ContextLimits::MAXIMUM_VERTEX_ATTRIBUTES >= 16);
    assert!(ContextLimits::MAXIMUM_VERTEX_TEXTURE_IMAGE_UNITS >= 16);
    assert!(ContextLimits::MAXIMUM_VERTEX_UNIFORM_VECTORS >= 256);
    assert!(ContextLimits::MINIMUM_ALIASED_LINE_WIDTH <= 1.0);
    assert!(ContextLimits::MAXIMUM_ALIASED_LINE_WIDTH >= 1.0);
    assert!(ContextLimits::MINIMUM_ALIASED_POINT_SIZE <= 1.0);
    assert!(ContextLimits::MAXIMUM_ALIASED_POINT_SIZE >= 1.0);
    assert!(ContextLimits::MAXIMUM_VIEWPORT_WIDTH > 0);
    assert!(ContextLimits::MAXIMUM_VIEWPORT_HEIGHT > 0);
    assert!(ContextLimits::MAXIMUM_TEXTURE_FILTER_ANISOTROPY >= 2.0);
    assert!(ContextLimits::MAXIMUM_DRAW_BUFFERS >= 1);
    assert!(ContextLimits::MAXIMUM_COLOR_ATTACHMENTS >= 4);
}

#[test]
fn test_antialias_option() {
    let c = Context::new(ContextOptions {
        antialias: Some(false),
        ..Default::default()
    })
    .unwrap();
    assert!(!c.antialias);
}

#[test]
fn test_standard_extensions_enabled() {
    let c = Context::default();
    assert!(c.depth_texture);
    assert!(c.floating_point_texture);
    assert!(c.texture_filter_anisotropic);
    assert!(c.vertex_array_object);
    assert!(c.fragment_depth);
    assert!(c.draw_buffers);
}

#[test]
fn test_pick_id_lifecycle() {
    let mut c = Context::default();
    let obj = "mesh_entity_123".to_string();

    let pick_id = c.create_pick_id(Some(obj.clone())).unwrap();
    assert_eq!(
        c.get_object_by_pick_color(Some(pick_id.color)).unwrap(),
        Some(obj)
    );

    // Unknown pick color returns None
    assert_eq!(
        c.get_object_by_pick_color(Some(Color::WHITE)).unwrap(),
        None
    );

    // Throws when creating pick ID without object
    assert!(c.create_pick_id(None).is_err());

    // Throws when retrieving without color
    assert!(c.get_object_by_pick_color(None).is_err());
}

#[test]
fn test_fails_to_construct_with_zero_dimensions() {
    assert!(Context::new(ContextOptions {
        width: Some(0),
        height: Some(768),
        ..Default::default()
    })
    .is_err());

    assert!(Context::new(ContextOptions {
        width: Some(1024),
        height: Some(0),
        ..Default::default()
    })
    .is_err());
}

#[test]
fn test_drawing_buffer_dimensions() {
    let c = Context::new(ContextOptions {
        width: Some(800),
        height: Some(600),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(c.drawing_buffer_width, 800);
    assert_eq!(c.drawing_buffer_height, 600);
}

#[test]
fn test_request_webgl1_mode() {
    let c1 = Context::new(ContextOptions {
        request_webgl1: Some(true),
        ..Default::default()
    })
    .unwrap();
    assert!(!c1.webgl2);

    let c2 = Context::new(ContextOptions {
        request_webgl1: Some(false),
        ..Default::default()
    })
    .unwrap();
    assert!(c2.webgl2);
}

#[test]
fn test_read_pixels_and_pbo() {
    let mut c = Context::new(ContextOptions {
        width: Some(1),
        height: Some(1),
        ..Default::default()
    })
    .unwrap();

    ClearCommand::new(ClearCommandOptions {
        color: Some(Color::WHITE),
        ..Default::default()
    })
    .execute(&mut c)
    .unwrap();

    let pixels = c.read_pixels(None).unwrap();
    assert_eq!(pixels, vec![255, 255, 255, 255]);

    // PBO read in WebGL2 mode
    let pbo = c
        .read_pixels_to_pbo(Some(ReadPixelsOptions {
            width: 1,
            height: 1,
            ..Default::default()
        }))
        .unwrap();

    let mut dest = vec![0u8; 4];
    pbo.get_buffer_data(&mut dest, None, None, None).unwrap();
    assert_eq!(dest, vec![255, 255, 255, 255]);
}

#[test]
fn test_read_pixels_to_pbo_throws_without_webgl2() {
    let c = Context::new(ContextOptions {
        request_webgl1: Some(true),
        ..Default::default()
    })
    .unwrap();

    assert!(c.read_pixels_to_pbo(None).is_err());
}

#[test]
fn test_destroys() {
    let mut c = Context::default();
    assert!(!c.is_destroyed());
    assert!(c.destroy().is_ok());
    assert!(c.is_destroyed());

    // Throws on double destroy
    assert!(c.destroy().is_err());
}
