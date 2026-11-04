use cesium_math::DeveloperError;

#[test]
fn test_developer_error() {
    let test_message = "Testing";
    let e = DeveloperError::new(test_message);

    assert_eq!(e.name(), "DeveloperError");
    assert_eq!(e.message, test_message);

    let s = e.to_string();
    assert!(s.contains("DeveloperError: Testing"));
}
