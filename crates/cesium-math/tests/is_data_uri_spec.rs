use cesium_math::is_data_uri;

#[test]
fn test_determines_that_a_uri_is_not_a_data_uri() {
    assert!(!is_data_uri("http://cesiumjs.org/"));
    assert!(!is_data_uri("https://example.com/data"));
}

#[test]
fn test_determines_that_a_uri_is_a_data_uri() {
    let uri = "data:text/plain;base64,YSBkYXRhIHVyaQ==";
    assert!(is_data_uri(uri));
    assert!(is_data_uri("DATA:image/png;base64,iVBORw0KGgoAAAANSUhEUg=="));
}
