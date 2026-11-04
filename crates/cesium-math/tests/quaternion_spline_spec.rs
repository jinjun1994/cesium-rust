use std::f64::consts::FRAC_PI_4;

use cesium_math::{Cartesian3, Quaternion, QuaternionSpline, EPSILON6};

fn quat_approx_eq(a: &Quaternion, b: &Quaternion, eps: f64) -> bool {
    (a.x - b.x).abs() <= eps
        && (a.y - b.y).abs() <= eps
        && (a.z - b.z).abs() <= eps
        && (a.w - b.w).abs() <= eps
}

#[test]
fn test_quaternion_spline_constructor_throws() {
    assert!(QuaternionSpline::new(vec![0.0], vec![Quaternion::IDENTITY]).is_err());
    assert!(QuaternionSpline::new(
        vec![0.0, 1.0],
        vec![
            Quaternion::IDENTITY,
            Quaternion::IDENTITY,
            Quaternion::IDENTITY
        ]
    )
    .is_err());
}

#[test]
fn test_quaternion_spline_evaluate_throws_out_of_range() {
    let times = vec![0.0, 1.0];
    let points = vec![
        Quaternion::from_axis_angle(Cartesian3::UNIT_X, FRAC_PI_4),
        Quaternion::from_axis_angle(Cartesian3::UNIT_Z, FRAC_PI_4),
    ];
    let qs = QuaternionSpline::new(times, points).unwrap();

    assert!(qs.evaluate(-1.0).is_err());
    assert!(qs.evaluate(2.0).is_err());
}

#[test]
fn test_quaternion_spline_evaluate() {
    let points = vec![
        Quaternion::from_axis_angle(Cartesian3::UNIT_X, FRAC_PI_4),
        Quaternion::from_axis_angle(Cartesian3::UNIT_Z, FRAC_PI_4),
        Quaternion::from_axis_angle(Cartesian3::UNIT_X, -FRAC_PI_4),
        Quaternion::from_axis_angle(Cartesian3::UNIT_Y, FRAC_PI_4),
    ];
    let times = vec![0.0, 1.0, 2.0, 3.0];
    let qs = QuaternionSpline::new(times.clone(), points.clone()).unwrap();

    assert_eq!(qs.evaluate(times[0]).unwrap(), points[0]);

    let time = (times[2] + times[1]) * 0.5;
    let t = (time - times[1]) / (times[2] - times[1]);

    let actual = qs.evaluate(time).unwrap();
    let expected = Quaternion::slerp(&points[1], &points[2], t);
    assert!(quat_approx_eq(&actual, &expected, EPSILON6));
}

#[test]
fn test_quaternion_spline_with_two_control_points() {
    let points = vec![
        Quaternion::from_axis_angle(Cartesian3::UNIT_X, FRAC_PI_4),
        Quaternion::from_axis_angle(Cartesian3::UNIT_Z, FRAC_PI_4),
    ];
    let times = vec![0.0, 1.0];
    let qs = QuaternionSpline::new(times.clone(), points.clone()).unwrap();

    let t = (times[0] + times[1]) * 0.5;
    let actual = qs.evaluate(t).unwrap();
    let expected = Quaternion::slerp(&points[0], &points[1], t);
    assert!(quat_approx_eq(&actual, &expected, EPSILON6));
}
