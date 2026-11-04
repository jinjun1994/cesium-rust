use cesium_math::{one_time_warning, reset_warnings};

#[test]
fn test_one_time_warning_logs_once_per_identifier() {
    reset_warnings();

    let first = one_time_warning("oneTime-identifier", Some("message")).unwrap();
    assert!(first); // logged

    let second = one_time_warning("oneTime-identifier", None).unwrap();
    assert!(!second); // skipped

    let another = one_time_warning("another oneTime-identifier", None).unwrap();
    assert!(another); // logged
}

#[test]
fn test_one_time_warning_throws_without_identifier() {
    assert!(one_time_warning("", None).is_err());
}
