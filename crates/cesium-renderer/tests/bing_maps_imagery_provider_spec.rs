use cesium_renderer::{
    BingMapsImageryProvider, BingMapsImageryProviderOptions, BingMapsStyle, ImageryProvider,
};
use serde_json::json;

#[test]
fn test_quadkey_conversions() {
    assert_eq!(BingMapsImageryProvider::tile_xy_to_quad_key(1, 0, 0), "1");
    assert_eq!(BingMapsImageryProvider::tile_xy_to_quad_key(1, 2, 1), "21");
    assert_eq!(BingMapsImageryProvider::tile_xy_to_quad_key(3, 5, 2), "213");
    assert_eq!(BingMapsImageryProvider::tile_xy_to_quad_key(4, 7, 2), "322");

    assert_eq!(
        BingMapsImageryProvider::quad_key_to_tile_xy("1").unwrap(),
        (1, 0, 0)
    );
    assert_eq!(
        BingMapsImageryProvider::quad_key_to_tile_xy("21").unwrap(),
        (1, 2, 1)
    );
    assert_eq!(
        BingMapsImageryProvider::quad_key_to_tile_xy("213").unwrap(),
        (3, 5, 2)
    );
    assert_eq!(
        BingMapsImageryProvider::quad_key_to_tile_xy("322").unwrap(),
        (4, 7, 2)
    );
}

#[test]
fn test_validation_and_map_layer_alpha() {
    assert!(BingMapsImageryProvider::from_url(
        "",
        BingMapsImageryProviderOptions {
            key: "k".to_string(),
            ..Default::default()
        }
    )
    .is_err());

    assert!(BingMapsImageryProvider::from_url(
        "https://dev.virtualearth.net",
        BingMapsImageryProviderOptions::default()
    )
    .is_err());

    let p_no_alpha = BingMapsImageryProvider::from_url(
        "https://dev.virtualearth.net",
        BingMapsImageryProviderOptions {
            key: "k".to_string(),
            map_style: BingMapsStyle::Aerial,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!p_no_alpha.has_alpha_channel());

    let p_alpha = BingMapsImageryProvider::from_url(
        "https://dev.virtualearth.net",
        BingMapsImageryProviderOptions {
            key: "k".to_string(),
            map_layer: Some("TrafficFlow".to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(p_alpha.has_alpha_channel());
}

#[test]
fn test_metadata_and_culture_tile_url() {
    let bad_meta = json!({ "resourceSets": [] });
    assert!(BingMapsImageryProvider::from_url_with_metadata(
        "https://dev.virtualearth.net",
        &bad_meta,
        BingMapsImageryProviderOptions {
            key: "k".to_string(),
            ..Default::default()
        }
    )
    .is_err());

    let meta = json!({
        "resourceSets": [{
            "resources": [{
                "imageUrl": "https://ecn.{subdomain}.tiles.virtualearth.net/tiles/r{quadkey}.jpeg?g=1&mkt={culture}",
                "imageUrlSubdomains": ["t0", "t1"],
                "imageWidth": 256,
                "imageHeight": 256,
                "zoomMin": 1,
                "zoomMax": 20
            }]
        }]
    });

    let p = BingMapsImageryProvider::from_url_with_metadata(
        "https://dev.virtualearth.net",
        &meta,
        BingMapsImageryProviderOptions {
            key: "k".to_string(),
            culture: Some("en-US".to_string()),
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(p.tile_width(), 256);
    assert_eq!(p.tile_height(), 256);
    assert_eq!(p.minimum_level(), 0);
    assert_eq!(p.maximum_level(), Some(19));
    let url = p.build_tile_url(3, 5, 2);
    assert!(url.contains("r213.jpeg"));
    assert!(url.contains("mkt=en-US"));
}
