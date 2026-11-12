use cesium_math::GeoRectangle;
use cesium_renderer::{
    AnyTilingScheme, ArcGisBaseMapType, ArcGisMapServerImageryProvider,
    ArcGisMapServerImageryProviderOptions, ImageryProvider,
};
use serde_json::json;

#[test]
fn test_from_url_and_basemap_type() {
    assert!(ArcGisMapServerImageryProvider::from_url("", None).is_err());

    let p = ArcGisMapServerImageryProvider::from_url("https://example.com/MapServer/", None)
        .unwrap();
    assert_eq!(p.url(), "https://example.com/MapServer");
    assert!(p.is_ready());
    assert!(p.has_alpha_channel());
    assert_eq!(p.tile_width(), 256);
    assert_eq!(p.tile_height(), 256);
    assert_eq!(p.minimum_level(), 0);

    let sat = ArcGisMapServerImageryProvider::from_basemap_type(
        ArcGisBaseMapType::Satellite,
        None,
        None,
    )
    .unwrap();
    assert!(sat.url().contains("World_Imagery/MapServer"));
    assert!(sat.credit().is_some());

    let hills = ArcGisMapServerImageryProvider::from_basemap_type(
        ArcGisBaseMapType::Hillshade,
        Some("my-token"),
        None,
    )
    .unwrap();
    assert!(hills.url().contains("World_Hillshade/MapServer"));
    assert_eq!(hills.token(), Some("my-token"));
    assert!(hills.credit().is_none());
}

#[test]
fn test_tiled_web_mercator_and_unsupported_wkid() {
    let bad_meta = json!({
        "tileInfo": {
            "rows": 256,
            "cols": 256,
            "spatialReference": { "wkid": 12345 },
            "lods": []
        }
    });
    assert!(ArcGisMapServerImageryProvider::from_url_with_metadata(
        "https://example.com/MapServer",
        &bad_meta,
        None
    )
    .is_err());

    let wm_meta = json!({
        "copyrightText": "Test copyright text",
        "tileInfo": {
            "rows": 128,
            "cols": 256,
            "spatialReference": { "wkid": 102100 },
            "lods": [
                { "level": 0 },
                { "level": 1 },
                { "level": 2 }
            ]
        },
        "fullExtent": {
            "xmin": -20037508.342787,
            "ymin": -20037508.342787,
            "xmax": 20037508.342787,
            "ymax": 20037508.342787,
            "spatialReference": { "wkid": 102100 }
        }
    });

    let p = ArcGisMapServerImageryProvider::from_url_with_metadata(
        "https://example.com/MapServer",
        &wm_meta,
        Some(ArcGisMapServerImageryProviderOptions {
            token: Some("abc".to_string()),
            ..Default::default()
        }),
    )
    .unwrap();

    assert!(matches!(p.tiling_scheme(), AnyTilingScheme::WebMercator(_)));
    assert_eq!(p.tile_width(), 256);
    assert_eq!(p.tile_height(), 128);
    assert_eq!(p.maximum_level(), Some(2));
    assert_eq!(p.credit(), Some("Test copyright text"));
    assert!(p.using_precached_tiles());
    assert_eq!(
        p.build_tile_url(1, 2, 3),
        "https://example.com/MapServer/tile/3/2/1?token=abc"
    );
}

#[test]
fn test_tiled_geographic_and_full_extent_clamping() {
    let geo_meta = json!({
        "tileInfo": {
            "rows": 256,
            "cols": 256,
            "spatialReference": { "wkid": 4326 },
            "lods": [{ "level": 0 }, { "level": 1 }]
        },
        "fullExtent": {
            "xmin": -200.0,
            "ymin": -100.0,
            "xmax": 200.0,
            "ymax": 100.0,
            "spatialReference": { "wkid": 4326 }
        }
    });

    let p = ArcGisMapServerImageryProvider::from_url_with_metadata(
        "https://example.com/MapServer",
        &geo_meta,
        None,
    )
    .unwrap();
    assert!(matches!(p.tiling_scheme(), AnyTilingScheme::Geographic(_)));
    assert_eq!(p.rectangle(), GeoRectangle::MAXIMUM);
}

#[test]
fn test_non_tiled_and_pick_features() {
    let meta = json!({});
    let mut p = ArcGisMapServerImageryProvider::from_url_with_metadata(
        "https://example.com/MapServer",
        &meta,
        Some(ArcGisMapServerImageryProviderOptions {
            layers: Some("0,1".to_string()),
            token: Some("tok".to_string()),
            ..Default::default()
        }),
    )
    .unwrap();
    assert!(!p.using_precached_tiles());
    let tile_url = p.build_tile_url(0, 0, 0);
    assert!(tile_url.contains("/export?bbox="));
    assert!(tile_url.contains("layers=show%3A0,1"));
    assert!(tile_url.contains("&token=tok"));

    let pick_url = p.build_pick_features_url(0, 0, 0, 0.1, 0.2).unwrap();
    assert!(pick_url.contains("/identify?"));
    assert!(pick_url.contains("layers=visible:0,1"));
    assert!(pick_url.contains("&token=tok"));

    p.set_enable_pick_features(false);
    assert!(p.build_pick_features_url(0, 0, 0, 0.1, 0.2).is_none());
}
