use cesium_renderer::{
    Framebuffer, FramebufferOptions, PixelDatatype, PixelFormat, Renderbuffer, RenderbufferFormat,
    RenderbufferOptions, Texture, TextureOptions,
};

#[test]
fn test_has_a_color_texture_attachment() {
    let texture = Texture::new(TextureOptions {
        width: 1,
        height: 1,
        pixel_format: Some(PixelFormat::Rgba),
        pixel_datatype: Some(PixelDatatype::UnsignedByte),
    });

    let fb = Framebuffer::new(FramebufferOptions {
        color_textures: vec![Some(texture)],
        ..Default::default()
    })
    .unwrap();

    assert!(fb.get_color_texture(0).is_some());
    assert_eq!(fb.number_of_color_attachments(), 1);
}

#[test]
fn test_has_a_color_renderbuffer_attachment() {
    let rb = Renderbuffer::new(RenderbufferOptions {
        width: 1,
        height: 1,
        format: RenderbufferFormat::Rgba4,
        num_samples: Some(1),
    });

    let fb = Framebuffer::new(FramebufferOptions {
        color_renderbuffers: vec![Some(rb)],
        ..Default::default()
    })
    .unwrap();

    assert!(fb.get_color_renderbuffer(0).is_some());
    assert_eq!(fb.number_of_color_attachments(), 1);
}

#[test]
fn test_has_a_depth_texture_attachment() {
    let depth_tex = Texture::new(TextureOptions {
        width: 1,
        height: 1,
        pixel_format: Some(PixelFormat::DepthComponent),
        pixel_datatype: Some(PixelDatatype::UnsignedShort),
    });

    let fb = Framebuffer::new(FramebufferOptions {
        depth_texture: Some(depth_tex),
        ..Default::default()
    })
    .unwrap();

    assert!(fb.depth_texture.is_some());
}

#[test]
fn test_throws_when_conflicting_attachments() {
    let tex = Texture::new(TextureOptions {
        width: 1,
        height: 1,
        pixel_format: Some(PixelFormat::Rgba),
        pixel_datatype: Some(PixelDatatype::UnsignedByte),
    });
    let rb = Renderbuffer::new(RenderbufferOptions {
        width: 1,
        height: 1,
        format: RenderbufferFormat::Rgba4,
        num_samples: Some(1),
    });

    // Cannot have both color texture and color renderbuffer
    let res = Framebuffer::new(FramebufferOptions {
        color_textures: vec![Some(tex)],
        color_renderbuffers: vec![Some(rb)],
        ..Default::default()
    });
    assert!(res.is_err());
}

#[test]
fn test_destroys_attachments_when_destroy_is_called() {
    let tex = Texture::new(TextureOptions {
        width: 1,
        height: 1,
        pixel_format: Some(PixelFormat::Rgba),
        pixel_datatype: Some(PixelDatatype::UnsignedByte),
    });

    let mut fb = Framebuffer::new(FramebufferOptions {
        color_textures: vec![Some(tex)],
        destroy_attachments: Some(true),
        ..Default::default()
    })
    .unwrap();

    assert!(!fb.is_destroyed());
    assert!(!fb.get_color_texture(0).unwrap().is_destroyed());

    fb.destroy();
    assert!(fb.is_destroyed());
    assert!(fb.get_color_texture(0).unwrap().is_destroyed());
}
