use cesium_math::{Cartesian3, ConstantSpline, Quaternion};

#[test]
fn test_constant_spline_number() {
    let value = 10.0;
    let spline = ConstantSpline::new(value);
    assert_eq!(spline.value, 10.0);
    assert_eq!(spline.wrap_time(-0.5), 0.0);
    assert_eq!(spline.wrap_time(2.5), 0.0);
    assert_eq!(spline.clamp_time(-0.5), 0.0);
    assert_eq!(spline.clamp_time(2.5), 0.0);
    assert_eq!(spline.evaluate(0.0), 10.0);
    assert_eq!(spline.evaluate(5.0), 10.0);
}

#[test]
fn test_constant_spline_cartesian3() {
    let value = Cartesian3::new(1.0, 2.0, 3.0);
    let spline = ConstantSpline::new(value);
    assert_eq!(spline.value, value);
    assert_eq!(spline.evaluate(0.0), value);
    assert_eq!(spline.evaluate(10.0), value);
}

#[test]
fn test_constant_spline_quaternion() {
    let value = Quaternion::new(0.707, 0.0, 0.707, 1.0);
    let spline = ConstantSpline::new(value);
    assert_eq!(spline.value, value);
    assert_eq!(spline.evaluate(0.0), value);
}
