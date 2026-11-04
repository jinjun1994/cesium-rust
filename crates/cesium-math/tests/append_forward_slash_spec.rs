use cesium_math::append_forward_slash;

#[test]
fn test_appends_to_a_url() {
    assert_eq!(
        append_forward_slash("http://cesiumjs.org"),
        "http://cesiumjs.org/"
    );
}

#[test]
fn test_does_not_append_to_a_url() {
    assert_eq!(
        append_forward_slash("http://cesiumjs.org/"),
        "http://cesiumjs.org/"
    );
}

#[test]
fn test_appends_to_an_empty_string() {
    assert_eq!(append_forward_slash(""), "/");
}
