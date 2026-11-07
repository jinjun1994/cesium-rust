use cesium_renderer::{
    MultisampleFramebuffer, MultisampleFramebufferOptions, PixelDatatype, PixelFormat, Renderbuffer,
    RenderbufferFormat, RenderbufferOptions, Texture, TextureOptions,
};

#[test]
fn test_throws_when_missing_color_or_depth_stencil_attachment_pair() {
    let tex = Texture::new(TextureOptions {
        width: 1,
        height: 1,
        pixel_format: Some(PixelFormat::Rgba),
        pixel_datatype: None,
    });

    // Color textures without color renderbuffers
    assert!(MultisampleFramebuffer::new(MultisampleFramebufferOptions {
        width: 1,
        height: 1,
        color_textures: Some(vec![Some(tex)]),
        color_renderbuffers: None,
        ..Default::default()
    })
    .is_err());

    // Depth-stencil texture without renderbuffer
    let ds_tex = Texture::new(TextureOptions {
        width: 1,
        height: 1,
        pixel_format: Some(PixelFormat::DepthStencil),
        pixel_datatype: Some(PixelDatatype::UnsignedInt24_8),
    });
    assert!(MultisampleFramebuffer::new(MultisampleFramebufferOptions {
        width: 1,
        height: 1,
        depth_stencil_texture: Some(ds_tex),
        depth_stencil_renderbuffer: None,
        ..Default::default()
    })
    .is_err());
}

#[test]
fn test_creates_read_and_draw_framebuffers() {
    let color_tex = Texture::new(TextureOptions {
        width: 1,
        height: 1,
        pixel_format: Some(PixelFormat::Rgba),
        pixel_datatype: None,
    });
    let color_rb = Renderbuffer::new(RenderbufferOptions {
        width: 1,
        height: 1,
        format: RenderbufferFormat::Rgba4,
        num_samples: Some(2),
    });
    let ds_tex = Texture::new(TextureOptions {
        width: 1,
        height: 1,
        pixel_format: Some(PixelFormat::DepthStencil),
        pixel_datatype: Some(PixelDatatype::UnsignedInt24_8),
    });
    let ds_rb = Renderbuffer::new(RenderbufferOptions {
        width: 1,
        height: 1,
        format: RenderbufferFormat::DepthStencil,
        num_samples: Some(2),
    });

    let mut mfb = MultisampleFramebuffer::new(MultisampleFramebufferOptions {
        width: 1,
        height: 1,
        color_textures: Some(vec![Some(color_tex)]),
        color_renderbuffers: Some(vec![Some(color_rb)]),
        depth_stencil_texture: Some(ds_tex),
        depth_stencil_renderbuffer: Some(ds_rb),
        destroy_attachments: Some(true),
    })
    .unwrap();

    let read_fb = mfb.get_render_framebuffer();
    let draw_fb = mfb.get_color_framebuffer();

    assert!(read_fb.get_color_renderbuffer(0).is_some());
    assert!(read_fb.depth_stencil_renderbuffer.is_some());
    assert!(draw_fb.get_color_texture(0).is_some());
    assert!(draw_fb.depth_stencil_texture.is_some());

    mfb.destroy();
    assert!(mfb.is_destroyed());
}
