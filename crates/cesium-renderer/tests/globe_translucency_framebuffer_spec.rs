use cesium_math::BoundingRectangle;
use cesium_renderer::commands::FramebufferHandle;
use cesium_renderer::framebuffer::PixelDatatype;
use cesium_renderer::globe_translucency_framebuffer::GlobeTranslucencyFramebuffer;
use cesium_renderer::pass_state::PassState;

fn packed_fb_id(cmd: &cesium_renderer::commands::DrawCommand) -> Option<u64> {
    match &cmd.framebuffer {
        Some(FramebufferHandle::Framebuffer(fb)) => Some(fb.id),
        _ => None,
    }
}

#[test]
fn creates_resources() {
    let mut globe_translucency = GlobeTranslucencyFramebuffer::new();
    let viewport = BoundingRectangle::new(0.0, 0.0, 100.0, 100.0);
    let pass_state = PassState::new("test_context");
    globe_translucency
        .update_and_clear(false, &viewport, true, &pass_state)
        .unwrap();

    assert!(globe_translucency.classification_texture().is_some());
    assert!(globe_translucency.classification_framebuffer().is_some());
    assert!(globe_translucency.packed_depth_texture().is_some());
    assert!(globe_translucency.packed_depth_framebuffer().is_some());
    assert!(
        globe_translucency.depth_stencil_texture().is_some()
            || globe_translucency.depth_stencil_renderbuffer().is_some()
    );
    assert!(globe_translucency.packed_depth_command().is_some());
    assert!(globe_translucency.clear_command().is_some());
}

#[test]
fn recreates_resources_when_viewport_changes() {
    let mut globe_translucency = GlobeTranslucencyFramebuffer::new();
    let mut viewport = BoundingRectangle::new(0.0, 0.0, 100.0, 100.0);
    let pass_state = PassState::new("test_context");
    globe_translucency
        .update_and_clear(false, &viewport, true, &pass_state)
        .unwrap();

    let first_fb_id = globe_translucency.classification_framebuffer().unwrap().id;
    let first_packed_fb_id = globe_translucency.packed_depth_framebuffer().unwrap().id;
    assert_eq!(
        globe_translucency
            .clear_command()
            .unwrap()
            .framebuffer
            .as_ref()
            .unwrap()
            .id,
        first_fb_id
    );
    assert_eq!(
        packed_fb_id(globe_translucency.packed_depth_command().unwrap()),
        Some(first_packed_fb_id)
    );

    viewport.width = 50.0;
    globe_translucency
        .update_and_clear(false, &viewport, true, &pass_state)
        .unwrap();

    assert_eq!(
        globe_translucency.classification_texture().unwrap().width,
        50
    );
    assert_ne!(
        globe_translucency
            .clear_command()
            .unwrap()
            .framebuffer
            .as_ref()
            .unwrap()
            .id,
        first_fb_id
    );
    assert_ne!(
        packed_fb_id(globe_translucency.packed_depth_command().unwrap()),
        Some(first_packed_fb_id)
    );
}

#[test]
fn recreates_resources_when_hdr_changes() {
    let mut globe_translucency = GlobeTranslucencyFramebuffer::new();
    let viewport = BoundingRectangle::new(0.0, 0.0, 100.0, 100.0);
    let pass_state = PassState::new("test_context");

    globe_translucency
        .update_and_clear(false, &viewport, true, &pass_state)
        .unwrap();
    assert_eq!(
        globe_translucency
            .classification_texture()
            .unwrap()
            .pixel_datatype,
        PixelDatatype::UnsignedByte
    );

    globe_translucency
        .update_and_clear(true, &viewport, true, &pass_state)
        .unwrap();
    assert_eq!(
        globe_translucency
            .classification_texture()
            .unwrap()
            .pixel_datatype,
        PixelDatatype::HalfFloat
    );

    globe_translucency
        .update_and_clear(true, &viewport, false, &pass_state)
        .unwrap();
    assert_eq!(
        globe_translucency
            .classification_texture()
            .unwrap()
            .pixel_datatype,
        PixelDatatype::Float
    );
}

#[test]
fn destroys() {
    let mut globe_translucency = GlobeTranslucencyFramebuffer::new();
    let viewport = BoundingRectangle::new(0.0, 0.0, 100.0, 100.0);
    let pass_state = PassState::new("test_context");
    globe_translucency
        .update_and_clear(false, &viewport, true, &pass_state)
        .unwrap();

    assert!(!globe_translucency.is_destroyed());
    globe_translucency.destroy();
    assert!(globe_translucency.is_destroyed());
    assert!(globe_translucency.classification_texture().is_none());
    assert!(globe_translucency.classification_framebuffer().is_none());
    assert!(globe_translucency.packed_depth_framebuffer().is_none());
}
