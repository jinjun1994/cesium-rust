use cesium_renderer::{
    Renderbuffer, RenderbufferFormat, RenderbufferOptions, MAXIMUM_RENDERBUFFER_SIZE,
};

#[test]
fn test_creates() {
    let rb = Renderbuffer::create(RenderbufferOptions {
        format: RenderbufferFormat::DepthComponent16,
        width: 64,
        height: 32,
        num_samples: Some(1),
    })
    .unwrap();

    assert_eq!(rb.format, RenderbufferFormat::DepthComponent16);
    assert_eq!(rb.width, 64);
    assert_eq!(rb.height, 32);
}

#[test]
fn test_destroys() {
    let mut rb = Renderbuffer::create(RenderbufferOptions {
        format: RenderbufferFormat::Rgba4,
        width: 64,
        height: 64,
        num_samples: None,
    })
    .unwrap();

    assert!(!rb.is_destroyed());
    rb.destroy();
    assert!(rb.is_destroyed());
}

#[test]
fn test_throws_when_created_with_small_or_large_dimensions() {
    assert!(Renderbuffer::create(RenderbufferOptions {
        format: RenderbufferFormat::Rgba4,
        width: 0,
        height: 64,
        num_samples: None,
    })
    .is_err());

    assert!(Renderbuffer::create(RenderbufferOptions {
        format: RenderbufferFormat::Rgba4,
        width: MAXIMUM_RENDERBUFFER_SIZE + 1,
        height: 64,
        num_samples: None,
    })
    .is_err());

    assert!(Renderbuffer::create(RenderbufferOptions {
        format: RenderbufferFormat::Rgba4,
        width: 64,
        height: 0,
        num_samples: None,
    })
    .is_err());

    assert!(Renderbuffer::create(RenderbufferOptions {
        format: RenderbufferFormat::Rgba4,
        width: 64,
        height: MAXIMUM_RENDERBUFFER_SIZE + 1,
        num_samples: None,
    })
    .is_err());
}

#[test]
fn test_throws_when_fails_to_destroy_twice() {
    let mut rb = Renderbuffer::create(RenderbufferOptions {
        format: RenderbufferFormat::Rgba4,
        width: 64,
        height: 64,
        num_samples: None,
    })
    .unwrap();

    assert!(rb.destroy_checked().is_ok());
    assert!(rb.destroy_checked().is_err());
}
