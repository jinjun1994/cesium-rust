use cesium_math::Interval;

#[test]
fn test_interval_default() {
    let interval = Interval::default();
    assert_eq!(interval.start, 0.0);
    assert_eq!(interval.stop, 0.0);
}

#[test]
fn test_interval_construct_with_arguments() {
    let interval = Interval::new(1.0, 2.0);
    assert_eq!(interval.start, 1.0);
    assert_eq!(interval.stop, 2.0);
}
