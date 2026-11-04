use cesium_math::{
    build_module_url, build_module_url_from_base_url, clear_base_url, match_cesium_script_url,
    set_base_url,
};

#[test]
fn test_matches_the_expected_forms_of_urls_to_cesium_js() {
    assert_eq!(match_cesium_script_url("Cesium.js"), Some(""));
    assert_eq!(match_cesium_script_url("Cesium.js?v=1.7"), Some(""));
    assert_eq!(
        match_cesium_script_url("assets/foo/Cesium.js"),
        Some("assets/foo/")
    );
    assert_eq!(
        match_cesium_script_url("assets/foo/Cesium.js?v=1.7"),
        Some("assets/foo/")
    );
    assert_eq!(
        match_cesium_script_url("http://example.invalid/Cesium/assets/foo/Cesium.js"),
        Some("http://example.invalid/Cesium/assets/foo/")
    );
    assert_eq!(
        match_cesium_script_url("http://example.invalid/Cesium/assets/foo/Cesium.js?v=1.7"),
        Some("http://example.invalid/Cesium/assets/foo/")
    );

    assert_eq!(match_cesium_script_url("cesium.js"), None);
    assert_eq!(match_cesium_script_url("Cesium.js%20"), None);
    assert_eq!(match_cesium_script_url("Cesium.min.js"), None);
    assert_eq!(match_cesium_script_url("CesiumSomething.js"), None);
    assert_eq!(match_cesium_script_url("CesiumSomething.js?v=1.7"), None);
    assert_eq!(match_cesium_script_url("assets/foo/bar.cesium.js"), None);
    assert_eq!(
        match_cesium_script_url("assets/foo/bar.cesium.js?v=1.7"),
        None
    );
    assert_eq!(
        match_cesium_script_url("assets/foo/CesiumSomething.js"),
        None
    );
    assert_eq!(
        match_cesium_script_url("assets/foo/CesiumSomething.js?v=1.7"),
        None
    );
}

#[test]
fn test_cesium_base_url_works_with_trailing_slash() {
    clear_base_url();
    let url = build_module_url_from_base_url("http://test.com/source/", "Core/Cartesian3.js");
    assert_eq!(url, "http://test.com/source/Core/Cartesian3.js");
}

#[test]
fn test_cesium_base_url_works_without_trailing_slash() {
    clear_base_url();
    let url = build_module_url_from_base_url("http://test.com/source", "Core/Cartesian3.js");
    assert_eq!(url, "http://test.com/source/Core/Cartesian3.js");
}

#[test]
fn test_produces_absolute_url_for_module() {
    set_base_url("https://assets.cesium.com/cesium/1.100.0/");
    let url = build_module_url("Workers/transferTypedArrayTest.js").expect("resolved module url");
    assert_eq!(
        url,
        "https://assets.cesium.com/cesium/1.100.0/Workers/transferTypedArrayTest.js"
    );
    clear_base_url();
}
