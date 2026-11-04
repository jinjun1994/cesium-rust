use cesium_math::defined;

#[test]
fn test_defined_works_for_defined_value() {
    let val = 0;
    assert!(defined(Some(&val)));
}

#[test]
fn test_defined_works_for_none() {
    let none_val: Option<i32> = None;
    assert!(!defined(none_val.as_ref()));
}
