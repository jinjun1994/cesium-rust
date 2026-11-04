use cesium_math::get_absolute_uri;

#[test]
fn test_get_absolute_uri_works_as_expected() {
    let result = get_absolute_uri(
        "http://www.mysite.com/awesome?makeitawesome=true",
        None,
    )
    .unwrap();
    assert_eq!(result, "http://www.mysite.com/awesome?makeitawesome=true");

    let result2 = get_absolute_uri("awesome.png", Some("http://test.com/")).unwrap();
    assert_eq!(result2, "http://test.com/awesome.png");

    let result3 = get_absolute_uri("awesome.png", Some("http://test.com/dir/")).unwrap();
    assert_eq!(result3, "http://test.com/dir/awesome.png");
}
