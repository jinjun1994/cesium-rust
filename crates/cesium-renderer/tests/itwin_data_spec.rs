//! Tests adapted from CesiumJS Scene/ITwinDataSpec.js (Apache-2.0).

use std::sync::atomic::{AtomicBool, Ordering};

use cesium_renderer::itwin_data::{
    ExportStatus, ITwinData, ITwinExportRepresentation, ITwinPlatform, ITwinTilesetOptions,
    RealityDataType,
};

fn create_mock_export(id: usize, status: ExportStatus) -> ITwinExportRepresentation {
    ITwinExportRepresentation {
        id: format!("{id}"),
        display_name: format!("export {id}"),
        status,
        last_modified: "2024-11-04T12:00Z".to_string(),
        mesh_href: Some(format!("https://example.com/link/to/mesh/{id}?query=param")),
    }
}

struct TestAuthReset;
impl Drop for TestAuthReset {
    fn drop(&mut self) {
        ITwinPlatform::set_default_access_token(Some("default-access-token".to_string()));
        ITwinPlatform::set_default_share_key(None);
    }
}

#[test]
fn create_tileset_from_imodel_id_rejects_when_all_exports_invalid() {
    let _reset = TestAuthReset;
    ITwinPlatform::set_default_access_token(Some("default-access-token".to_string()));

    let exports = vec![
        create_mock_export(1, ExportStatus::Invalid),
        create_mock_export(2, ExportStatus::Invalid),
        create_mock_export(3, ExportStatus::Invalid),
        create_mock_export(4, ExportStatus::Invalid),
        create_mock_export(5, ExportStatus::Invalid),
    ];

    let result = ITwinData::create_tileset_from_exports("imodel-id-1", &exports, None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("All exports for this iModel are Invalid"));
}

#[test]
fn create_tileset_from_imodel_id_returns_none_when_empty_or_no_complete() {
    let _reset = TestAuthReset;
    ITwinPlatform::set_default_access_token(Some("default-access-token".to_string()));

    // Empty exports
    let result = ITwinData::create_tileset_from_exports("imodel-id-1", &[], None).unwrap();
    assert!(result.is_none());

    // InProgress & NotStarted only
    let exports = vec![
        create_mock_export(1, ExportStatus::InProgress),
        create_mock_export(2, ExportStatus::NotStarted),
    ];
    let result2 = ITwinData::create_tileset_from_exports("imodel-id-1", &exports, None).unwrap();
    assert!(result2.is_none());
}

#[test]
fn create_tileset_from_imodel_id_creates_tileset_for_first_complete_export() {
    let _reset = TestAuthReset;
    ITwinPlatform::set_default_access_token(Some("default-access-token".to_string()));

    let exports = vec![
        create_mock_export(1, ExportStatus::Invalid),
        create_mock_export(2, ExportStatus::Complete),
    ];

    let result = ITwinData::create_tileset_from_exports("imodel-id-1", &exports, None).unwrap();
    assert!(result.is_some());
    let (url, opts) = result.unwrap();
    assert_eq!(
        url,
        "https://example.com/link/to/mesh/2/tileset.json?query=param"
    );
    assert!(opts.is_none());
}

#[test]
fn create_tileset_from_imodel_id_passes_tileset_options() {
    let _reset = TestAuthReset;
    ITwinPlatform::set_default_access_token(Some("default-access-token".to_string()));

    let exports = vec![create_mock_export(1, ExportStatus::Complete)];
    let opts = ITwinTilesetOptions {
        show: Some(false),
        ..Default::default()
    };

    let result = ITwinData::create_tileset_from_exports("imodel-id-1", &exports, Some(opts.clone())).unwrap();
    assert!(result.is_some());
    let (_, out_opts) = result.unwrap();
    assert_eq!(out_opts, Some(opts));
}

#[test]
fn create_tileset_for_reality_data_rejects_unsupported_type() {
    let result = ITwinData::create_tileset_for_reality_data(
        Some("imodel-id-1"),
        Some("reality-data-id-1"),
        Some("DGN"),
        Some("root/path.json"),
        None,
        None,
        |_, _, _| String::new(),
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("Reality data type is not a mesh type"));
}

#[test]
fn create_tileset_for_reality_data_handles_metadata_resolution() {
    let metadata_called = AtomicBool::new(false);

    // Both defined: no metadata called
    let (url, opts) = ITwinData::create_tileset_for_reality_data(
        Some("itwin-id-1"),
        Some("reality-data-id-1"),
        Some("Cesium3DTiles"),
        Some("root/document/path.json"),
        None,
        None,
        |_, _, doc| format!("https://example.com/{doc}?auth=token"),
    )
    .unwrap();
    assert_eq!(url, "https://example.com/root/document/path.json?auth=token");
    assert_eq!(opts.maximum_screen_space_error, Some(4.0));

    // Type undefined: fetches from metadata fallback
    let fallback = ("Cesium3DTiles", "root/document/path.json");
    let (url2, _) = ITwinData::create_tileset_for_reality_data(
        Some("itwin-id-1"),
        Some("reality-data-id-1"),
        None,
        Some("root/document/path.json"),
        None,
        Some(fallback),
        |_, _, doc| {
            metadata_called.store(true, Ordering::SeqCst);
            format!("https://example.com/{doc}?auth=token")
        },
    )
    .unwrap();
    assert_eq!(url2, "https://example.com/root/document/path.json?auth=token");
    assert!(metadata_called.load(Ordering::SeqCst));

    // Root document undefined: fetches from metadata fallback
    let (url3, _) = ITwinData::create_tileset_for_reality_data(
        Some("itwin-id-1"),
        Some("reality-data-id-1"),
        Some("Cesium3DTiles"),
        None,
        None,
        Some(fallback),
        |_, _, doc| format!("https://example.com/{doc}?auth=token"),
    )
    .unwrap();
    assert_eq!(url3, "https://example.com/root/document/path.json?auth=token");
}

#[test]
fn create_tileset_for_reality_data_respects_and_overrides_options() {
    // Default 4.0 SSE with extra options
    let opts = ITwinTilesetOptions {
        cache_bytes: Some(500_000_000),
        ..Default::default()
    };
    let (_, out_opts) = ITwinData::create_tileset_for_reality_data(
        Some("itwin-id-1"),
        Some("reality-data-id-1"),
        Some("Cesium3DTiles"),
        Some("root/document/path.json"),
        Some(opts),
        None,
        |_, _, doc| format!("https://example.com/{doc}"),
    )
    .unwrap();
    assert_eq!(out_opts.maximum_screen_space_error, Some(4.0));
    assert_eq!(out_opts.cache_bytes, Some(500_000_000));

    // Explicit override of SSE
    let opts_override = ITwinTilesetOptions {
        maximum_screen_space_error: Some(32.0),
        cache_bytes: Some(500_000_000),
        ..Default::default()
    };
    let (_, out_opts2) = ITwinData::create_tileset_for_reality_data(
        Some("itwin-id-1"),
        Some("reality-data-id-1"),
        Some("Cesium3DTiles"),
        Some("root/document/path.json"),
        Some(opts_override),
        None,
        |_, _, doc| format!("https://example.com/{doc}"),
    )
    .unwrap();
    assert_eq!(out_opts2.maximum_screen_space_error, Some(32.0));
    assert_eq!(out_opts2.cache_bytes, Some(500_000_000));

    // Supports GaussianSplat3DTiles
    let (_, out_opts_gs) = ITwinData::create_tileset_for_reality_data(
        Some("itwin-id-1"),
        Some("reality-data-id-1"),
        Some("GaussianSplat3DTiles"),
        Some("root/document/path.json"),
        None,
        None,
        |_, _, doc| format!("https://example.com/{doc}"),
    )
    .unwrap();
    assert_eq!(out_opts_gs.maximum_screen_space_error, Some(4.0));
}

#[test]
fn create_data_source_for_reality_data_supports_geojson_and_kml() {
    // Rejects unsupported DGN
    let bad = ITwinData::create_data_source_for_reality_data(
        Some("imodel-id-1"),
        Some("reality-data-id-1"),
        Some("DGN"),
        Some("root/path.json"),
        None,
        |_, _, _| String::new(),
    );
    assert!(bad.is_err());
    assert!(bad.unwrap_err().message.contains("Reality data type is not a data source type"));

    // GeoJSON
    let (t_geo, url_geo) = ITwinData::create_data_source_for_reality_data(
        Some("itwin-id-1"),
        Some("reality-data-id-1"),
        Some("GeoJSON"),
        Some("root/document/path.json"),
        None,
        |_, _, doc| format!("https://example.com/{doc}?auth=token"),
    )
    .unwrap();
    assert_eq!(t_geo, RealityDataType::GeoJson);
    assert_eq!(url_geo, "https://example.com/root/document/path.json?auth=token");

    // KML
    let (t_kml, url_kml) = ITwinData::create_data_source_for_reality_data(
        Some("itwin-id-1"),
        Some("reality-data-id-1"),
        Some("KML"),
        Some("root/document/path.json"),
        None,
        |_, _, doc| format!("https://example.com/{doc}?auth=token"),
    )
    .unwrap();
    assert_eq!(t_kml, RealityDataType::Kml);
    assert_eq!(url_kml, "https://example.com/root/document/path.json?auth=token");

    // Metadata fallback when type/rootDocument are omitted
    let (t_meta, url_meta) = ITwinData::create_data_source_for_reality_data(
        Some("itwin-id-1"),
        Some("reality-data-id-1"),
        None,
        None,
        Some(("GeoJSON", "root/document/path.json")),
        |_, _, doc| format!("https://example.com/{doc}?auth=token"),
    )
    .unwrap();
    assert_eq!(t_meta, RealityDataType::GeoJson);
    assert_eq!(url_meta, "https://example.com/root/document/path.json?auth=token");
}

#[test]
fn load_geospatial_features_validations_and_resource_creation() {
    let _reset = TestAuthReset;

    ITwinPlatform::set_default_access_token(Some("test-access-token".to_string()));

    // Missing iTwinId
    let err_no_itwin = ITwinData::load_geospatial_features_resource(None, Some("col-1"), None);
    assert!(err_no_itwin.is_err());
    assert!(err_no_itwin.unwrap_err().message.contains("Expected iTwinId to be typeof string"));

    // Missing collectionId
    let err_no_col = ITwinData::load_geospatial_features_resource(Some("itwin-1"), None, None);
    assert!(err_no_col.is_err());
    assert!(err_no_col.unwrap_err().message.contains("Expected collectionId to be typeof string"));

    // Limit < 1
    let err_limit_zero = ITwinData::load_geospatial_features_resource(Some("itwin-1"), Some("col-1"), Some(0));
    assert!(err_limit_zero.is_err());
    assert!(err_limit_zero.unwrap_err().message.contains("Expected limit to be greater than or equal to 1"));

    // Limit > 10000
    let err_limit_large = ITwinData::load_geospatial_features_resource(Some("itwin-1"), Some("col-1"), Some(20000));
    assert!(err_limit_large.is_err());
    assert!(err_limit_large.unwrap_err().message.contains("Expected limit to be less than or equal to 10000"));

    // No auth token or share key
    ITwinPlatform::set_default_access_token(None);
    ITwinPlatform::set_default_share_key(None);
    let err_no_auth = ITwinData::load_geospatial_features_resource(Some("itwin-1"), Some("col-1"), None);
    assert!(err_no_auth.is_err());
    assert!(err_no_auth.unwrap_err().message.contains("Must set ITwinPlatform.defaultAccessToken"));

    // Successful resource creation with default access token
    ITwinPlatform::set_default_access_token(Some("test-access-token".to_string()));
    let res = ITwinData::load_geospatial_features_resource(Some("itwin-id-1"), Some("collection-id-1"), None).unwrap();
    assert!(res.url().contains("https://api.bentley.com/geospatial-features/itwins/itwin-id-1/ogc/collections/collection-id-1/items"));
    assert!(res.url().contains("limit=10000"));
    assert!(res.url().contains("client=CesiumJS"));
    assert_eq!(res.headers.get("Authorization").unwrap(), "Bearer test-access-token");
    assert_eq!(res.headers.get("Accept").unwrap(), "application/vnd.bentley.itwin-platform.v1+json");

    // Successful resource creation with default share key overriding access token
    ITwinPlatform::set_default_share_key(Some("my-share-key".to_string()));
    let res_share = ITwinData::load_geospatial_features_resource(Some("itwin-id-1"), Some("collection-id-1"), Some(50)).unwrap();
    assert!(res_share.url().contains("limit=50"));
    assert_eq!(res_share.headers.get("Authorization").unwrap(), "Basic my-share-key");
}
