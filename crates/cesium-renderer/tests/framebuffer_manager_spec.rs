use cesium_renderer::{
    FramebufferManager, FramebufferManagerOptions, PixelDatatype, PixelFormat, Renderbuffer,
    RenderbufferFormat, RenderbufferOptions, Texture, TextureOptions,
};

#[test]
fn test_throws_when_constructor_has_no_enabled_attachments() {
    let res = FramebufferManager::new(FramebufferManagerOptions {
        color: false,
        depth: false,
        depth_stencil: false,
        ..Default::default()
    });
    assert!(res.is_err());
}

#[test]
fn test_throws_when_constructor_enables_depth_and_depth_stencil() {
    let res = FramebufferManager::new(FramebufferManagerOptions {
        depth: true,
        depth_stencil: true,
        ..Default::default()
    });
    assert!(res.is_err());
}

#[test]
fn test_throws_if_update_is_called_without_width_or_height() {
    let mut fbm = FramebufferManager::new(FramebufferManagerOptions::default()).unwrap();
    assert!(fbm.update(None, None, None, None, None).is_err());
    assert!(fbm.update(Some(1), None, None, None, None).is_err());
    assert!(fbm.update(None, Some(1), None, None, None).is_err());
}

#[test]
fn test_throws_if_getting_color_texture_at_invalid_index() {
    let fbm = FramebufferManager::new(FramebufferManagerOptions {
        color_attachments_length: 2,
        create_color_attachments: false,
        ..Default::default()
    })
    .unwrap();

    assert!(fbm.get_color_texture(2).is_err());
}

#[test]
fn test_throws_if_setting_color_texture_when_create_color_attachments_is_true() {
    let mut fbm = FramebufferManager::new(FramebufferManagerOptions::default()).unwrap();
    assert!(fbm.set_color_texture(None, 0).is_err());
}

#[test]
fn test_throws_if_setting_color_texture_at_invalid_index() {
    let mut fbm = FramebufferManager::new(FramebufferManagerOptions {
        color_attachments_length: 2,
        create_color_attachments: false,
        ..Default::default()
    })
    .unwrap();

    assert!(fbm.set_color_texture(None, 2).is_err());
}

#[test]
fn test_sets_color_textures() {
    let mut fbm = FramebufferManager::new(FramebufferManagerOptions {
        color_attachments_length: 2,
        create_color_attachments: false,
        ..Default::default()
    })
    .unwrap();

    let tex0 = Texture::new(TextureOptions {
        width: 1,
        height: 1,
        pixel_format: Some(PixelFormat::Rgb),
        pixel_datatype: None,
    });
    let tex1 = Texture::new(TextureOptions {
        width: 1,
        height: 1,
        pixel_format: Some(PixelFormat::Rgba),
        pixel_datatype: None,
    });

    fbm.set_color_texture(Some(tex0.clone()), 0).unwrap();
    fbm.set_color_texture(Some(tex1.clone()), 1).unwrap();

    assert_eq!(fbm.get_color_texture(0).unwrap(), Some(&tex0));
    assert_eq!(fbm.get_color_texture(1).unwrap(), Some(&tex1));
}

#[test]
fn test_throws_if_setting_depth_attachments_when_create_depth_attachments_is_true() {
    let mut fbm = FramebufferManager::new(FramebufferManagerOptions::default()).unwrap();
    assert!(fbm.set_depth_texture(None).is_err());
    assert!(fbm.set_depth_renderbuffer(None).is_err());
}

#[test]
fn test_sets_depth_attachments() {
    let mut fbm = FramebufferManager::new(FramebufferManagerOptions {
        depth: true,
        create_depth_attachments: false,
        ..Default::default()
    })
    .unwrap();

    let tex = Texture::new(TextureOptions {
        width: 1,
        height: 1,
        pixel_format: Some(PixelFormat::DepthComponent),
        pixel_datatype: Some(PixelDatatype::UnsignedInt),
    });
    fbm.set_depth_texture(Some(tex.clone())).unwrap();
    assert_eq!(fbm.get_depth_texture(), Some(&tex));

    let rb = Renderbuffer::new(RenderbufferOptions {
        width: 1,
        height: 1,
        format: RenderbufferFormat::DepthComponent16,
        num_samples: Some(1),
    });
    fbm.set_depth_renderbuffer(Some(rb.clone())).unwrap();
    assert_eq!(fbm.get_depth_renderbuffer(), Some(&rb));
}

#[test]
fn test_sets_depth_stencil_attachments() {
    let mut fbm = FramebufferManager::new(FramebufferManagerOptions {
        depth_stencil: true,
        create_depth_attachments: false,
        ..Default::default()
    })
    .unwrap();

    let tex = Texture::new(TextureOptions {
        width: 1,
        height: 1,
        pixel_format: Some(PixelFormat::DepthStencil),
        pixel_datatype: Some(PixelDatatype::UnsignedInt24_8),
    });
    fbm.set_depth_stencil_texture(Some(tex.clone())).unwrap();
    assert_eq!(fbm.get_depth_stencil_texture(), Some(&tex));

    let rb = Renderbuffer::new(RenderbufferOptions {
        width: 1,
        height: 1,
        format: RenderbufferFormat::DepthStencil,
        num_samples: Some(1),
    });
    fbm.set_depth_stencil_renderbuffer(Some(rb.clone())).unwrap();
    assert_eq!(fbm.get_depth_stencil_renderbuffer(), Some(&rb));
}

#[test]
fn test_creates_and_destroys_framebuffer() {
    let mut fbm = FramebufferManager::new(FramebufferManagerOptions::default()).unwrap();
    assert!(fbm.framebuffer().is_none());

    fbm.update(Some(1), Some(1), None, None, None).unwrap();
    assert!(fbm.framebuffer().is_some());
    assert_eq!(
        fbm.framebuffer().unwrap().number_of_color_attachments(),
        1
    );

    fbm.destroy_framebuffer();
    assert!(fbm.framebuffer().is_none());

    fbm.destroy();
    assert!(fbm.is_destroyed());
}
