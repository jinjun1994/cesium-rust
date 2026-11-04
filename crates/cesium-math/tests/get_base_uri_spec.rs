use cesium_math::get_base_uri;

#[test]
fn test_get_base_uri_works_as_expected() {
    let result = get_base_uri("http://www.mysite.com/awesome?makeitawesome=true", false);
    assert_eq!(result, "http://www.mysite.com/");

    let result = get_base_uri(
        "http://www.mysite.com/somefolder/awesome.png#makeitawesome",
        false,
    );
    assert_eq!(result, "http://www.mysite.com/somefolder/");
}

#[test]
fn test_get_base_uri_with_include_query_flag() {
    let result = get_base_uri("http://www.mysite.com/awesome?makeitawesome=true", true);
    assert_eq!(result, "http://www.mysite.com/?makeitawesome=true");

    let result = get_base_uri(
        "http://www.mysite.com/somefolder/awesome.png#makeitawesome",
        true,
    );
    assert_eq!(result, "http://www.mysite.com/somefolder/#makeitawesome");
}
