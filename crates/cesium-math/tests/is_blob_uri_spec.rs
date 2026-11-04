use cesium_math::is_blob_uri;

#[test]
fn test_determines_that_a_uri_is_not_a_blob_uri() {
    assert!(!is_blob_uri("http://cesiumjs.org/"));
    assert!(!is_blob_uri("https://example.com/blob"));
}

#[test]
fn test_determines_that_a_uri_is_a_blob_uri() {
    assert!(is_blob_uri("blob:http://cesiumjs.org/d97607a9-0607-4221-a3f2-895180632b70"));
    assert!(is_blob_uri("BLOB:https://example.com/12345"));
}
