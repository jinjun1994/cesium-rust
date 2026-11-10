use cesium_math::{PixelDatatype, PixelFormat};
use cesium_renderer::sampler::{
    Sampler, TextureMagnificationFilter, TextureMinificationFilter, TextureWrap,
};
use cesium_renderer::texture_uniform::{TextureUniform, TextureUniformOptions};

#[test]
fn test_throws_if_no_texture_is_specified() {
    let err = TextureUniform::new(TextureUniformOptions::default()).unwrap_err();
    assert_eq!(
        err.message,
        "exactly one of options.typedArray, options.url must be defined"
    );
}

#[test]
fn test_throws_if_both_url_and_typed_array_are_specified() {
    let err = TextureUniform::new(TextureUniformOptions {
        url: Some("https://example.com/texture.png".to_string()),
        typed_array: Some(vec![0, 255, 0, 255]),
        width: Some(1),
        height: Some(1),
        ..Default::default()
    })
    .unwrap_err();
    assert_eq!(
        err.message,
        "exactly one of options.typedArray, options.url must be defined"
    );
}

#[test]
fn test_constructs_with_a_url() {
    let example_url = "https://example.com/url";
    let texture_uniform = TextureUniform::new(TextureUniformOptions {
        url: Some(example_url.to_string()),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(texture_uniform.url.as_deref(), Some(example_url));
    assert!(texture_uniform.typed_array.is_none());
    assert!(texture_uniform.width.is_none());
    assert!(texture_uniform.height.is_none());
    assert_eq!(texture_uniform.pixel_format, PixelFormat::Rgba);
    assert_eq!(texture_uniform.pixel_datatype, PixelDatatype::UnsignedByte);

    let default_sampler = Sampler {
        wrap_s: TextureWrap::Repeat,
        wrap_t: TextureWrap::Repeat,
        wrap_r: TextureWrap::ClampToEdge,
        minification_filter: TextureMinificationFilter::Linear,
        magnification_filter: TextureMagnificationFilter::Linear,
        maximum_anisotropy: 1.0,
    };
    assert_eq!(texture_uniform.sampler, default_sampler);
}

#[test]
fn test_throws_if_width_and_height_are_not_provided_for_a_typed_array() {
    let err = TextureUniform::new(TextureUniformOptions {
        typed_array: Some(vec![0, 255, 0, 255]),
        ..Default::default()
    })
    .unwrap_err();
    assert_eq!(
        err.message,
        "options.width and options.height are required when options.typedArray is defined"
    );
}

#[test]
fn test_constructs_with_a_typed_array() {
    let green_texture = vec![0, 255, 0, 255];
    let texture_uniform = TextureUniform::new(TextureUniformOptions {
        typed_array: Some(green_texture.clone()),
        width: Some(1),
        height: Some(1),
        ..Default::default()
    })
    .unwrap();

    assert!(texture_uniform.url.is_none());
    assert_eq!(texture_uniform.typed_array, Some(green_texture));
    assert_eq!(texture_uniform.width, Some(1));
    assert_eq!(texture_uniform.height, Some(1));
    assert_eq!(texture_uniform.pixel_format, PixelFormat::Rgba);
    assert_eq!(texture_uniform.pixel_datatype, PixelDatatype::UnsignedByte);

    assert_eq!(texture_uniform.sampler.wrap_s, TextureWrap::Repeat);
    assert_eq!(texture_uniform.sampler.wrap_t, TextureWrap::Repeat);
}

#[test]
fn test_sets_sampler_settings() {
    let texture_uniform = TextureUniform::new(TextureUniformOptions {
        url: Some("https://example.com/url".to_string()),
        repeat: Some(false),
        minification_filter: Some(TextureMinificationFilter::Nearest),
        magnification_filter: Some(TextureMagnificationFilter::Nearest),
        maximum_anisotropy: Some(4.0),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(texture_uniform.sampler.wrap_s, TextureWrap::ClampToEdge);
    assert_eq!(texture_uniform.sampler.wrap_t, TextureWrap::ClampToEdge);
    assert_eq!(
        texture_uniform.sampler.minification_filter,
        TextureMinificationFilter::Nearest
    );
    assert_eq!(
        texture_uniform.sampler.magnification_filter,
        TextureMagnificationFilter::Nearest
    );
    assert_eq!(texture_uniform.sampler.maximum_anisotropy, 4.0);
}
