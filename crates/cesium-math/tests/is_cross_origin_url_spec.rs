use cesium_math::{get_absolute_uri, is_cross_origin_url, set_page_location};

#[test]
fn test_returns_false_for_relative_urls() {
    assert!(!is_cross_origin_url("/some/url.jpg"));
    assert!(!is_cross_origin_url("some/url.jpg"));
}

#[test]
fn test_returns_false_for_absolute_urls_that_are_not_cross_origin() {
    set_page_location("http://localhost:8080/index.html");

    let abs1 = get_absolute_uri("/some/url.jpg", Some("http://localhost:8080/index.html"))
        .expect("absolute uri");
    assert!(!is_cross_origin_url(&abs1));

    let abs2 = get_absolute_uri("some/url.jpg", Some("http://localhost:8080/index.html"))
        .expect("absolute uri");
    assert!(!is_cross_origin_url(&abs2));
}

#[test]
fn test_returns_true_for_absolute_urls_that_are_cross_origin() {
    set_page_location("http://localhost:8080/index.html");

    assert!(is_cross_origin_url("http://example.invalid/some/url.jpg"));

    // A different scheme counts as cross-origin (https vs http)
    assert!(is_cross_origin_url("https://localhost:8080/index.html"));

    // A different port counts as cross-origin (8081 vs 8080)
    assert!(is_cross_origin_url("http://localhost:8081/index.html"));
}
