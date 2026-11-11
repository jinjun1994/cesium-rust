use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use cesium_math::{Credit, GeoRectangle, WebMercatorTilingScheme};
use cesium_renderer::imagery_layer::ImageryProvider;
use cesium_renderer::url_template_imagery_provider::{
    ImageryProviderError, UrlTemplateImageryProvider, UrlTemplateImageryProviderOptions,
};

#[test]
fn test_requires_the_url_to_be_specified() {
    let result = UrlTemplateImageryProvider::new(UrlTemplateImageryProviderOptions {
        url: None,
        ..Default::default()
    });
    assert!(result.is_err());
}

#[test]
fn test_returns_valid_value_for_has_alpha_channel() {
    let provider = UrlTemplateImageryProvider::new(UrlTemplateImageryProviderOptions {
        url: Some("made/up/tms/server/".to_string()),
        ..Default::default()
    })
    .unwrap();

    assert!(provider.has_alpha_channel());
}

#[test]
fn test_request_image_defaults_and_url_formatting() {
    let provider = UrlTemplateImageryProvider::new(UrlTemplateImageryProviderOptions {
        url: Some("made/up/tms/server/{Z}/{X}/{reverseY}".to_string()),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(provider.url, "made/up/tms/server/{Z}/{X}/{reverseY}");
    assert_eq!(provider.tile_width(), 256);
    assert_eq!(provider.tile_height(), 256);
    assert_eq!(provider.minimum_level(), 0);
    assert_eq!(provider.maximum_level(), None);
    assert_eq!(
        provider.rectangle(),
        WebMercatorTilingScheme::default().rectangle()
    );

    let url = provider.request_image_url(0, 0, 0);
    assert_eq!(url, "made/up/tms/server/0/0/0");

    let img = provider.request_image(0, 0, 0).unwrap();
    assert!(!img.is_empty());
}

#[test]
fn test_credits() {
    let provider_no_credit = UrlTemplateImageryProvider::new(UrlTemplateImageryProviderOptions {
        url: Some("made/up/tms/server".to_string()),
        credit: None,
        ..Default::default()
    })
    .unwrap();
    assert!(provider_no_credit.credit.is_none());

    let provider_with_credit =
        UrlTemplateImageryProvider::new(UrlTemplateImageryProviderOptions {
            url: Some("made/up/gms/server".to_string()),
            credit: Some(Credit::new(
                "Thanks to our awesome made up source of this imagery!",
                None,
            )),
            ..Default::default()
        })
        .unwrap();
    assert!(provider_with_credit.credit.is_some());
}

#[test]
fn test_rectangle_does_not_affect_tile_numbering() {
    let custom_rect = GeoRectangle::from_degrees(0.1, 0.2, 0.3, 0.4);
    let provider = UrlTemplateImageryProvider::new(UrlTemplateImageryProviderOptions {
        url: Some("made/up/tms/server/{z}/{x}/{reverseY}".to_string()),
        rectangle: Some(custom_rect),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(provider.rectangle(), custom_rect);
    let url = provider.request_image_url(0, 0, 0);
    assert!(url.contains("/0/0/0"));
}

#[test]
fn test_uses_minimum_and_maximum_level() {
    let provider = UrlTemplateImageryProvider::new(UrlTemplateImageryProviderOptions {
        url: Some("made/up/tms/server".to_string()),
        minimum_level: Some(1),
        maximum_level: Some(5),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(provider.minimum_level(), 1);
    assert_eq!(provider.maximum_level(), Some(5));
}

#[test]
fn test_zero_padding_and_tags() {
    let mut zero_pad = HashMap::new();
    zero_pad.insert("{x}".to_string(), "0000".to_string());
    zero_pad.insert("{y}".to_string(), "000".to_string());

    let provider = UrlTemplateImageryProvider::new(UrlTemplateImageryProviderOptions {
        url: Some("server/{z}/{x}/{y}?w={width}&h={height}&sub={s}".to_string()),
        url_scheme_zero_padding: Some(zero_pad),
        subdomains: Some(vec!["sub1".to_string()]),
        ..Default::default()
    })
    .unwrap();

    let url = provider.request_image_url(12, 5, 2);
    assert_eq!(url, "server/2/0012/005?w=256&h=256&sub=sub1");
}

#[test]
fn test_raises_error_event_when_image_fails() {
    let provider = UrlTemplateImageryProvider::new(UrlTemplateImageryProviderOptions {
        url: Some("made/up/tms/server".to_string()),
        ..Default::default()
    })
    .unwrap();

    let tries = Arc::new(Mutex::new(0));
    let t_clone = tries.clone();

    provider.error_event.add_event_listener(move |err| {
        let mut count = t_clone.lock().unwrap();
        assert_eq!(err.times_retried, *count);
        *count += 1;
        if *count < 3 {
            err.retry = true;
        }
    });

    let mut err = ImageryProviderError {
        message: "Failed to load tile".to_string(),
        x: 0,
        y: 0,
        level: 0,
        times_retried: 0,
        retry: false,
    };

    provider.error_event.raise_event(&mut err);
    assert_eq!(*tries.lock().unwrap(), 1);
    assert!(err.retry);
}
