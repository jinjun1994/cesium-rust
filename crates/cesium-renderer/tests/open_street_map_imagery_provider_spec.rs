//! OpenStreetMapImageryProviderSpec matching CesiumJS `Specs/Scene/OpenStreetMapImageryProviderSpec.js` (Apache-2.0).

use cesium_math::{Credit, GeoRectangle, EPSILON10};
use cesium_renderer::imagery_layer::ImageryProvider;
use cesium_renderer::open_street_map_imagery_provider::{
    OpenStreetMapImageryProvider, OpenStreetMapImageryProviderOptions,
};

#[test]
fn test_default_construction_and_alpha_channel() {
    let provider =
        OpenStreetMapImageryProvider::new(OpenStreetMapImageryProviderOptions::default())
            .expect("Should create default OSM provider");

    assert!(!provider.has_alpha_channel());
    assert_eq!(provider.tile_width(), 256);
    assert_eq!(provider.tile_height(), 256);
    assert_eq!(provider.minimum_level(), 0);
    assert_eq!(provider.maximum_level(), None);
    assert_eq!(provider.tiling_scheme.rectangle(), provider.rectangle());
}

#[test]
fn test_supports_slash_at_the_end_of_url() {
    let provider = OpenStreetMapImageryProvider::new(OpenStreetMapImageryProviderOptions {
        url: Some("made/up/osm/server/".to_string()),
        ..Default::default()
    })
    .unwrap();

    let request_url = provider.request_image_url(0, 0, 0);
    assert!(!request_url.contains("//"));
    assert_eq!(request_url, "made/up/osm/server/0/0/0.png");
}

#[test]
fn test_supports_no_slash_at_the_end_of_url() {
    let provider = OpenStreetMapImageryProvider::new(OpenStreetMapImageryProviderOptions {
        url: Some("made/up/osm/server".to_string()),
        ..Default::default()
    })
    .unwrap();

    let request_url = provider.request_image_url(0, 0, 0);
    assert!(request_url.starts_with("made/up/osm/server/"));
    assert_eq!(request_url, "made/up/osm/server/0/0/0.png");
}

#[test]
fn test_credit_default_and_custom() {
    let provider_default = OpenStreetMapImageryProvider::new(OpenStreetMapImageryProviderOptions {
        url: Some("made/up/osm/server".to_string()),
        ..Default::default()
    })
    .unwrap();

    assert!(provider_default.credit().html().contains("Open Street Map"));

    let custom_credit = Credit::new("Thanks to our awesome made up source of this imagery!", None);
    let provider_custom = OpenStreetMapImageryProvider::new(OpenStreetMapImageryProviderOptions {
        url: Some("made/up/osm/server".to_string()),
        credit: Some(custom_credit.clone()),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(provider_custom.credit(), &custom_credit);
}

#[test]
fn test_rectangle_passed_to_constructor_does_not_affect_tile_numbering() {
    let custom_rect = GeoRectangle {
        west: 0.1,
        south: 0.2,
        east: 0.3,
        north: 0.4,
    };

    let provider = OpenStreetMapImageryProvider::new(OpenStreetMapImageryProviderOptions {
        url: Some("made/up/osm/server".to_string()),
        rectangle: Some(custom_rect),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(provider.tile_width(), 256);
    assert_eq!(provider.tile_height(), 256);
    assert_eq!(provider.maximum_level(), None);

    let rect = provider.rectangle();
    assert!((rect.west - custom_rect.west).abs() < EPSILON10);
    assert!((rect.south - custom_rect.south).abs() < EPSILON10);
    assert!((rect.east - custom_rect.east).abs() < EPSILON10);
    assert!((rect.north - custom_rect.north).abs() < EPSILON10);

    let url = provider.request_image_url(0, 0, 0);
    assert!(url.contains("/0/0/0.png"));
}

#[test]
fn test_minimum_and_maximum_level() {
    let provider = OpenStreetMapImageryProvider::new(OpenStreetMapImageryProviderOptions {
        url: Some("made/up/osm/server".to_string()),
        minimum_level: Some(1),
        maximum_level: Some(5),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(provider.minimum_level(), 1);
    assert_eq!(provider.maximum_level(), Some(5));
}

#[test]
fn test_retina_tiles_option() {
    let provider = OpenStreetMapImageryProvider::new(OpenStreetMapImageryProviderOptions {
        url: Some("made/up/osm/server".to_string()),
        retina_tiles: Some(true),
        ..Default::default()
    })
    .unwrap();

    let url = provider.request_image_url(0, 0, 0);
    assert!(url.contains("0/0/0@2x.png"));
}

#[test]
fn test_throws_with_more_than_four_tiles_at_the_minimum() {
    let rectangle = GeoRectangle {
        west: 0.0,
        south: 0.0,
        east: 1.0_f64.to_radians(),
        north: 1.0_f64.to_radians(),
    };

    let result = OpenStreetMapImageryProvider::new(OpenStreetMapImageryProviderOptions {
        minimum_level: Some(9),
        rectangle: Some(rectangle),
        ..Default::default()
    });

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err
        .message
        .contains("more than four tiles at the minimum level are not supported"));
}
