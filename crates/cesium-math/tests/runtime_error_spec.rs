use cesium_math::RuntimeError;

#[test]
fn test_runtime_error() {
    let test_message = "Testing";
    let e = RuntimeError::new(test_message);

    assert_eq!(e.name(), "RuntimeError");
    assert_eq!(e.message, test_message);

    let s = e.to_string();
    assert!(s.contains("RuntimeError: Testing"));
}
