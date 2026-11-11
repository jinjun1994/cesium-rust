use std::collections::BTreeMap;

use cesium_math::{
    Ellipsoid, WebMercatorTilingScheme,
};
use cesium_renderer::quadtree_tile::AnyTilingScheme;
use cesium_renderer::{
    ImageryProvider, WebMapServiceImageryProvider, WebMapServiceImageryProviderOptions,
};

#[test]
fn test_wms_requires_url() {
    let res = WebMapServiceImageryProvider::new(WebMapServiceImageryProviderOptions {
        layers: Some("someLayer".to_string()),
        ..Default::default()
    });
    assert!(res.is_err());
}

#[test]
fn test_wms_requires_layers() {
    let res = WebMapServiceImageryProvider::new(WebMapServiceImageryProviderOptions {
        url: Some("made/up/wms/server".to_string()),
        ..Default::default()
    });
    assert!(res.is_err());
}

#[test]
fn test_wms_returns_has_alpha_channel() {
    let provider = WebMapServiceImageryProvider::new(WebMapServiceImageryProviderOptions {
        url: Some("made/up/wms/server".to_string()),
        layers: Some("someLayer".to_string()),
        ..Default::default()
    })
    .unwrap();

    assert!(provider.has_alpha_channel());
}

#[test]
fn test_wms_custom_ellipsoid() {
    let ellipsoid = Ellipsoid::new(glam::DVec3::new(1.0, 2.0, 3.0)).unwrap();
    let provider = WebMapServiceImageryProvider::new(WebMapServiceImageryProviderOptions {
        url: Some("made/up/wms/server".to_string()),
        layers: Some("someLayer".to_string()),
        ellipsoid: Some(ellipsoid),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(provider.ellipsoid(), ellipsoid);
}

#[test]
fn test_wms_includes_specified_parameters_in_url() {
    let mut params = BTreeMap::new();
    params.insert("something".to_string(), "foo".to_string());
    params.insert("another".to_string(), "false".to_string());
    params.insert("version".to_string(), "1.3.0".to_string());

    let provider = WebMapServiceImageryProvider::new(WebMapServiceImageryProviderOptions {
        url: Some("http://made/up/wms/server".to_string()),
        layers: Some("someLayer".to_string()),
        parameters: Some(params),
        ..Default::default()
    })
    .unwrap();

    let url = provider.build_image_url(0, 0, 0);
    assert!(url.contains("something=foo"));
    assert!(url.contains("another=false"));
    assert!(url.contains("version=1.3.0"));
}

#[test]
fn test_wms_includes_crs_parameters_for_v1_3_0() {
    let mut params = BTreeMap::new();
    params.insert("version".to_string(), "1.3.0".to_string());

    let provider = WebMapServiceImageryProvider::new(WebMapServiceImageryProviderOptions {
        url: Some("http://made/up/wms/server".to_string()),
        layers: Some("someLayer".to_string()),
        crs: Some("CRS:27".to_string()),
        parameters: Some(params),
        ..Default::default()
    })
    .unwrap();

    let url = provider.build_image_url(0, 0, 0);
    assert!(url.contains("crs=CRS:27"));
    assert!(url.contains("version=1.3.0"));
}

#[test]
fn test_wms_includes_bbox_parameters_for_v1_3_0_and_epsg4326() {
    let mut params = BTreeMap::new();
    params.insert("version".to_string(), "1.3.0".to_string());

    let provider = WebMapServiceImageryProvider::new(WebMapServiceImageryProviderOptions {
        url: Some("http://made/up/wms/server".to_string()),
        layers: Some("someLayer".to_string()),
        crs: Some("EPSG:4326".to_string()),
        parameters: Some(params),
        ..Default::default()
    })
    .unwrap();

    let bbox = provider.compute_tile_bbox(0, 0, 0);
    assert_eq!(bbox, "-90,-180,90,0");

    let url = provider.build_image_url(0, 0, 0);
    assert!(url.contains("bbox=-90,-180,90,0"));
}

#[test]
fn test_wms_reverses_axis_order_for_epsg_between_4000_and_5000() {
    let mut params = BTreeMap::new();
    params.insert("version".to_string(), "1.3.0".to_string());

    let provider = WebMapServiceImageryProvider::new(WebMapServiceImageryProviderOptions {
        url: Some("http://made/up/wms/server".to_string()),
        layers: Some("someLayer".to_string()),
        crs: Some("EPSG:4321".to_string()),
        parameters: Some(params),
        ..Default::default()
    })
    .unwrap();

    let bbox = provider.compute_tile_bbox(0, 0, 0);
    assert_eq!(bbox, "-90,-180,90,0");
}

#[test]
fn test_wms_reverses_axis_order_for_included_epsg_codes() {
    let mut params = BTreeMap::new();
    params.insert("version".to_string(), "1.3.0".to_string());

    let provider = WebMapServiceImageryProvider::new(WebMapServiceImageryProviderOptions {
        url: Some("http://made/up/wms/server".to_string()),
        layers: Some("someLayer".to_string()),
        crs: Some("EPSG:3035".to_string()),
        parameters: Some(params),
        ..Default::default()
    })
    .unwrap();

    assert!(provider.reverse_axis_order);
}

#[test]
fn test_wms_does_not_reverse_axis_order_for_excluded_epsg_codes() {
    let mut params = BTreeMap::new();
    params.insert("version".to_string(), "1.3.0".to_string());

    let provider = WebMapServiceImageryProvider::new(WebMapServiceImageryProviderOptions {
        url: Some("http://made/up/wms/server".to_string()),
        layers: Some("someLayer".to_string()),
        crs: Some("EPSG:4471".to_string()),
        parameters: Some(params),
        ..Default::default()
    })
    .unwrap();

    assert!(!provider.reverse_axis_order);
}

#[test]
fn test_wms_default_crs_and_srs_for_schemes() {
    // Geographic with v1.3.0 defaults to CRS:84
    let mut params_130 = BTreeMap::new();
    params_130.insert("version".to_string(), "1.3.0".to_string());
    let provider_geo_130 = WebMapServiceImageryProvider::new(WebMapServiceImageryProviderOptions {
        url: Some("http://server".to_string()),
        layers: Some("layer".to_string()),
        parameters: Some(params_130.clone()),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(provider_geo_130.crs.as_deref(), Some("CRS:84"));

    // WebMercator with v1.3.0 defaults to EPSG:3857
    let wm = WebMercatorTilingScheme::new(Ellipsoid::WGS84, None, 1, 1).unwrap();
    let provider_wm_130 = WebMapServiceImageryProvider::new(WebMapServiceImageryProviderOptions {
        url: Some("http://server".to_string()),
        layers: Some("layer".to_string()),
        tiling_scheme: Some(AnyTilingScheme::WebMercator(wm)),
        parameters: Some(params_130),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(provider_wm_130.crs.as_deref(), Some("EPSG:3857"));

    // Geographic with v1.1.1 defaults to EPSG:4326
    let provider_geo_111 = WebMapServiceImageryProvider::new(WebMapServiceImageryProviderOptions {
        url: Some("http://server".to_string()),
        layers: Some("layer".to_string()),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(provider_geo_111.srs.as_deref(), Some("EPSG:4326"));
}
