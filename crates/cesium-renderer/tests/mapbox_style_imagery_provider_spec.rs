use cesium_math::{Credit, GeoRectangle};
use cesium_renderer::imagery_layer::ImageryProvider;
use cesium_renderer::mapbox_style_imagery_provider::{
    MapboxStyleImageryProvider, MapboxStyleImageryProviderOptions,
};

#[test]
fn requires_style_id_and_access_token() {
    assert!(MapboxStyleImageryProvider::new(MapboxStyleImageryProviderOptions {
        access_token: "test-token".to_string(),
        style_id: String::new(),
        ..Default::default()
    })
    .is_err());

    assert!(MapboxStyleImageryProvider::new(MapboxStyleImageryProviderOptions {
        access_token: String::new(),
        style_id: "test-id".to_string(),
        ..Default::default()
    })
    .is_err());
}

#[test]
fn returns_valid_value_for_has_alpha_channel() {
    let provider = MapboxStyleImageryProvider::new(MapboxStyleImageryProviderOptions {
        access_token: "test-token".to_string(),
        url: Some("made/up/mapbox/server/".to_string()),
        style_id: "test-id".to_string(),
        ..Default::default()
    })
    .unwrap();

    assert!(!provider.has_alpha_channel());
}

#[test]
fn supports_slash_and_no_slash_at_end_of_url() {
    let with_slash = MapboxStyleImageryProvider::new(MapboxStyleImageryProviderOptions {
        access_token: "test-token".to_string(),
        url: Some("made/up/mapbox/server/".to_string()),
        style_id: "test-id".to_string(),
        ..Default::default()
    })
    .unwrap();

    let without_slash = MapboxStyleImageryProvider::new(MapboxStyleImageryProviderOptions {
        access_token: "test-token".to_string(),
        url: Some("made/up/mapbox/server".to_string()),
        style_id: "test-id".to_string(),
        ..Default::default()
    })
    .unwrap();

    let expected = "made/up/mapbox/server/mapbox/test-id/tiles/512/{z}/{x}/{y}?access_token=test-token";
    assert_eq!(with_slash.url, expected);
    assert_eq!(without_slash.url, expected);
}

#[test]
fn contains_specified_username_and_tilesize() {
    let provider = MapboxStyleImageryProvider::new(MapboxStyleImageryProviderOptions {
        access_token: "test-token".to_string(),
        style_id: "test-id".to_string(),
        username: Some("fakeUsername".to_string()),
        tilesize: Some(256),
        ..Default::default()
    })
    .unwrap();

    let expected = "https://api.mapbox.com/styles/v1/fakeUsername/test-id/tiles/256/{z}/{x}/{y}?access_token=test-token";
    assert_eq!(provider.url, expected);
}

#[test]
fn enables_2x_scale_factor_in_url() {
    let provider = MapboxStyleImageryProvider::new(MapboxStyleImageryProviderOptions {
        access_token: "test-token".to_string(),
        style_id: "test-id".to_string(),
        scale_factor: true,
        ..Default::default()
    })
    .unwrap();

    let tile_url = provider.request_image_url(0, 0, 0);
    assert!(tile_url.contains("/tiles/512/0/0/0@2x?access_token=test-token"));
}

#[test]
fn respects_custom_rectangle_and_credits() {
    let rect = GeoRectangle::from_degrees(10.0, 20.0, 30.0, 40.0);
    let custom_credit = Credit::new("My Mapbox Credit", Some(true));

    let provider = MapboxStyleImageryProvider::new(MapboxStyleImageryProviderOptions {
        access_token: "test-token".to_string(),
        style_id: "test-id".to_string(),
        rectangle: Some(rect),
        credit: Some(custom_credit.clone()),
        minimum_level: Some(1),
        maximum_level: Some(7),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(provider.rectangle(), rect);
    assert_eq!(provider.credit, custom_credit);
    assert_eq!(provider.minimum_level(), 1);
    assert_eq!(provider.maximum_level(), Some(7));
}
