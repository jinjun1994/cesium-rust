use cesium_math::{
    ITwinExportStatus, ITwinExportType, ITwinPlatform, ITwinRealityDataType,
};

#[test]
fn test_get_authorization_header() {
    // No tokens set
    assert!(ITwinPlatform::get_authorization_header(None, None).is_err());
    assert!(ITwinPlatform::get_authorization_header(Some(""), Some("")).is_err());

    // Access token only
    let header = ITwinPlatform::get_authorization_header(Some("access-token"), None).unwrap();
    assert_eq!(header, "Bearer access-token");

    // Share key only
    let header = ITwinPlatform::get_authorization_header(None, Some("share-key")).unwrap();
    assert_eq!(header, "Basic share-key");

    // Both set: share key takes precedence
    let header =
        ITwinPlatform::get_authorization_header(Some("access-token"), Some("share-key")).unwrap();
    assert_eq!(header, "Basic share-key");
}

#[test]
fn test_combine_reality_data_url() {
    let container_url = "https://example.com/base/path?auth=token";
    let root_doc = "root/document/path.json";
    let combined =
        ITwinPlatform::combine_reality_data_url(container_url, root_doc).unwrap();
    assert_eq!(
        combined,
        "https://example.com/base/path/root/document/path.json?auth=token"
    );

    // Without query string
    let container_url_no_q = "https://example.com/base/path/";
    let combined_no_q =
        ITwinPlatform::combine_reality_data_url(container_url_no_q, root_doc).unwrap();
    assert_eq!(
        combined_no_q,
        "https://example.com/base/path/root/document/path.json"
    );
}

#[test]
fn test_map_http_error_codes() {
    let err_401 = ITwinPlatform::map_http_error(
        401,
        r#"{"error": {"details": [{"code": "InvalidToken"}]}}"#,
        None,
    );
    assert!(err_401.message.contains("Unauthorized"));
    assert!(err_401.message.contains("InvalidToken"));

    let err_403 = ITwinPlatform::map_http_error(
        403,
        r#"{"error": {"code": "Forbidden", "message": "Access denied"}}"#,
        None,
    );
    assert!(err_403.message.contains("forbidden"));

    let err_404 = ITwinPlatform::map_http_error(
        404,
        r#"{"error": {"message": "Not found"}}"#,
        Some("itwin-1, reality-2"),
    );
    assert!(err_404.message.contains("Reality data not found: itwin-1, reality-2"));

    let err_422 = ITwinPlatform::map_http_error(
        422,
        r#"{"error": {"code": "BadEntity", "message": "failed"}}"#,
        None,
    );
    assert!(err_422.message.contains("Unprocessable Entity"));

    let err_429 = ITwinPlatform::map_http_error(
        429,
        r#"{"error": {"message": "rate limit"}}"#,
        None,
    );
    assert!(err_429.message.contains("Too many requests"));
}

#[test]
fn test_enum_types_and_constants() {
    assert_eq!(ITwinExportStatus::Complete.as_str(), "Complete");
    assert_eq!(ITwinExportType::ThreeDTiles.as_str(), "3DTILES");
    assert_eq!(ITwinRealityDataType::GaussianSplat3DTiles.as_str(), "GS_3DT");
    assert_eq!(ITwinRealityDataType::Cesium3DTiles.as_str(), "Cesium3DTiles");
}
