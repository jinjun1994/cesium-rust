use cesium_math::get_filename_from_uri;

#[test]
fn test_get_filename_from_uri_works_as_expected() {
    let result = get_filename_from_uri("http://www.mysite.com/awesome?makeitawesome=true");
    assert_eq!(result, "awesome");

    let result =
        get_filename_from_uri("http://www.mysite.com/somefolder/awesome.png#makeitawesome");
    assert_eq!(result, "awesome.png");
}
