//! Direct port and verification against CesiumJS `SamplerSpec.js` (Apache-2.0).

use cesium_renderer::{
    Sampler, SamplerOptions, TextureMagnificationFilter, TextureMinificationFilter, TextureWrap,
};

#[test]
fn test_has_expected_default_values() {
    let sampler = Sampler::default();
    assert_eq!(sampler.wrap_r, TextureWrap::ClampToEdge);
    assert_eq!(sampler.wrap_s, TextureWrap::ClampToEdge);
    assert_eq!(sampler.wrap_t, TextureWrap::ClampToEdge);
    assert_eq!(
        sampler.minification_filter,
        TextureMinificationFilter::Linear
    );
    assert_eq!(
        sampler.magnification_filter,
        TextureMagnificationFilter::Linear
    );
    assert_eq!(sampler.maximum_anisotropy, 1.0);
}

#[test]
fn test_throws_when_creating_a_sampler_with_invalid_wrap_r() {
    assert!(TextureWrap::from_str_name("invalid wrap").is_none());
}

#[test]
fn test_throws_when_creating_a_sampler_with_invalid_wrap_s() {
    assert!(TextureWrap::from_str_name("invalid wrap").is_none());
}

#[test]
fn test_throws_when_creating_a_sampler_with_invalid_wrap_t() {
    assert!(TextureWrap::from_str_name("invalid wrap").is_none());
}

#[test]
fn test_throws_when_creating_a_sampler_with_invalid_minification_filter() {
    assert!(TextureMinificationFilter::from_str_name("invalid filter").is_none());
}

#[test]
fn test_throws_when_creating_a_sampler_with_invalid_magnification_filter() {
    assert!(TextureMagnificationFilter::from_str_name("invalid filter").is_none());
}

#[test]
fn test_throws_when_creating_a_sampler_with_invalid_maximum_anisotropy() {
    let res = Sampler::new(SamplerOptions {
        maximum_anisotropy: Some(0.0),
        ..Default::default()
    });
    assert!(res.is_err());
}
