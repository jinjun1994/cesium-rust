use cesium_math::{deprecation_warning, reset_warnings};

#[test]
fn test_deprecation_warning_logs_once_per_identifier() {
    reset_warnings();

    let first = deprecation_warning("deprecation-identifier", "message").unwrap();
    assert!(first); // logged

    let second = deprecation_warning("deprecation-identifier", "message").unwrap();
    assert!(!second); // skipped

    let another = deprecation_warning("another deprecation-identifier", "another message").unwrap();
    assert!(another); // logged
}

#[test]
fn test_deprecation_warning_throws_without_identifier_or_message() {
    assert!(deprecation_warning("", "message").is_err());
    assert!(deprecation_warning("id", "").is_err());
}
