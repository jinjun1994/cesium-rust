use cesium_math::Check;

#[test]
fn test_check_defined() {
    let some_val = Some(42);
    assert!(Check::defined("test", some_val.as_ref()).is_ok());

    let none_val: Option<i32> = None;
    assert!(Check::defined("test", none_val.as_ref()).is_err());
}

#[test]
fn test_check_type_of_number() {
    assert!(Check::type_of_number("num", 42.5).is_ok());
    assert!(Check::type_of_number("num", f64::NAN).is_err());
}

#[test]
fn test_check_type_of_bool() {
    assert!(Check::type_of_bool("flag", true).is_ok());
    assert!(Check::type_of_bool("flag", false).is_ok());
}
