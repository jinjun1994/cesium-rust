use cesium_math::{Credit, GeoRectangle};
use cesium_renderer::imagery_layer::ImageryProvider;
use cesium_renderer::mapbox_imagery_provider::{
    MapboxImageryProvider, MapboxImageryProviderOptions,
};

#[test]
fn requires_map_id_and_access_token() {
    assert!(MapboxImageryProvider::new(MapboxImageryProviderOptions {
        access_token: "test-token".to_string(),
        map_id: String::new(),
        ..Default::default()
    })
    .is_err());

    assert!(MapboxImageryProvider::new(MapboxImageryProviderOptions {
        access_token: String::new(),
        map_id: "test-id".to_string(),
        ..Default::default()
    })
    .is_err());
}

#[test]
fn returns_valid_value_for_has_alpha_channel() {
    let provider = MapboxImageryProvider::new(MapboxImageryProviderOptions {
        access_token: "test-token".to_string(),
        url: Some("made/up/mapbox/server/".to_string()),
        map_id: "test-id".to_string(),
        ..Default::default()
    })
    .unwrap();

    assert!(!provider.has_alpha_channel());
}

#[test]
fn supports_slash_and_no_slash_at_end_of_url() {
    let with_slash = MapboxImageryProvider::new(MapboxImageryProviderOptions {
        access_token: "test-token".to_string(),
        url: Some("made/up/mapbox/server/".to_string()),
        map_id: "test-id".to_string(),
        ..Default::default()
    })
    .unwrap();

    let without_slash = MapboxImageryProvider::new(MapboxImageryProviderOptions {
        access_token: "test-token".to_string(),
        url: Some("made/up/mapbox/server".to_string()),
        map_id: "test-id".to_string(),
        ..Default::default()
    })
    .unwrap();

    let expected_template = "made/up/mapbox/server/test-id/{z}/{x}/{y}.png?access_token=test-token";
    assert_eq!(with_slash.url, expected_template);
    assert_eq!(without_slash.url, expected_template);
}

#[test]
fn request_image_formats_url_correctly() {
    let provider = MapboxImageryProvider::new(MapboxImageryProviderOptions {
        access_token: "test-token".to_string(),
        url: Some("made/up/mapbox/server/".to_string()),
        map_id: "test-id".to_string(),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(provider.tile_width(), 256);
    assert_eq!(provider.tile_height(), 256);
    assert_eq!(provider.minimum_level(), 0);
    assert!(provider.maximum_level().is_none());

    let tile_url = provider.request_image_url(2, 3, 1);
    assert_eq!(
        tile_url,
        "made/up/mapbox/server/test-id/1/2/3.png?access_token=test-token"
    );
}

#[test]
fn respects_custom_rectangle_and_levels() {
    let custom_rect = GeoRectangle::from_degrees(10.0, 20.0, 30.0, 40.0);
    let provider = MapboxImageryProvider::new(MapboxImageryProviderOptions {
        access_token: "test-token".to_string(),
        map_id: "test-id".to_string(),
        rectangle: Some(custom_rect),
        minimum_level: Some(2),
        maximum_level: Some(8),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(provider.rectangle(), custom_rect);
    assert_eq!(provider.minimum_level(), 2);
    assert_eq!(provider.maximum_level(), Some(8));
}

#[test]
fn credit_handling_default_and_custom() {
    let default_provider = MapboxImageryProvider::new(MapboxImageryProviderOptions {
        access_token: "test-token".to_string(),
        map_id: "test-id".to_string(),
        ..Default::default()
    })
    .unwrap();

    assert!(default_provider.credit.html().contains("Mapbox"));

    let custom_credit = Credit::new("Thanks to custom imagery provider", Some(true));
    let custom_provider = MapboxImageryProvider::new(MapboxImageryProviderOptions {
        access_token: "test-token".to_string(),
        map_id: "test-id".to_string(),
        credit: Some(custom_credit.clone()),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(custom_provider.credit, custom_credit);
}
