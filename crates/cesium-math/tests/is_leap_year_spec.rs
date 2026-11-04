use cesium_math::is_leap_year;

#[test]
fn test_check_for_valid_leap_years() {
    assert!(is_leap_year(2000));
    assert!(is_leap_year(2004));
    assert!(!is_leap_year(2003));
    assert!(!is_leap_year(2300));
    assert!(is_leap_year(2400));
    assert!(!is_leap_year(-1));
    assert!(is_leap_year(-2000));
}
