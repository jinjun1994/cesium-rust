use cesium_math::get_extension_from_uri;

#[test]
fn test_get_extension_from_uri_works_as_expected() {
    let result = get_extension_from_uri("http://www.mysite.com/awesome?makeitawesome=true");
    assert_eq!(result, "");

    let result =
        get_extension_from_uri("http://www.mysite.com/somefolder/awesome.png#makeitawesome");
    assert_eq!(result, "png");

    let result = get_extension_from_uri("awesome.png");
    assert_eq!(result, "png");
}
