use cesium_math::{GeoRectangle, WebMercatorTilingScheme};
use cesium_renderer::{
    Azure2DImageryProvider, Azure2DImageryProviderOptions, ImageryProvider,
};

#[test]
fn test_requires_subscription_key() {
    let res = Azure2DImageryProvider::new(Azure2DImageryProviderOptions {
        tileset_id: Some("a-tileset-id".to_string()),
        ..Default::default()
    });
    assert!(res.is_err());
}

#[test]
fn test_default_properties_and_url() {
    let provider = Azure2DImageryProvider::new(Azure2DImageryProviderOptions {
        subscription_key: "test-subscriptionKey".to_string(),
        tileset_id: Some("a-tileset-id".to_string()),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(
        provider.url(),
        "https://atlas.microsoft.com/map/tile?api-version=2024-04-01&tilesetId=a-tileset-id&subscription-key=test-subscriptionKey&zoom={z}&x={x}&y={y}"
    );
    assert_eq!(provider.tile_width(), 256);
    assert_eq!(provider.tile_height(), 256);
    assert_eq!(provider.maximum_level(), Some(22));
    assert_eq!(provider.minimum_level(), 0);
    assert!(provider.is_ready());
    assert!(provider.has_alpha_channel());
    assert_eq!(
        provider.rectangle(),
        WebMercatorTilingScheme::default().rectangle()
    );
    assert_eq!(
        provider.build_tile_url(1, 2, 3),
        "https://atlas.microsoft.com/map/tile?api-version=2024-04-01&tilesetId=a-tileset-id&subscription-key=test-subscriptionKey&zoom=3&x=1&y=2"
    );
}

#[test]
fn test_rectangle_min_max_level_and_credit() {
    let rect = GeoRectangle::new(0.1, 0.2, 0.3, 0.4);
    let provider = Azure2DImageryProvider::new(Azure2DImageryProviderOptions {
        subscription_key: "key".to_string(),
        tileset_id: Some("a-tileset-id".to_string()),
        rectangle: Some(rect),
        minimum_level: Some(2),
        maximum_level: Some(15),
        credit: Some("Bing/Azure Credit".to_string()),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(provider.rectangle(), rect);
    assert_eq!(provider.minimum_level(), 2);
    assert_eq!(provider.maximum_level(), Some(15));
    assert_eq!(provider.credit(), Some("Bing/Azure Credit"));
    assert!(provider.build_tile_url(0, 0, 0).contains("zoom=0&x=0&y=0"));
}
