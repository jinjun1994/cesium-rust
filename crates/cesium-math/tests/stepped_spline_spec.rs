use cesium_math::{Cartesian3, Quaternion, SteppedSpline};

#[test]
fn test_stepped_spline_constructor_throws() {
    assert!(SteppedSpline::new(vec![0.0], vec![Cartesian3::ZERO]).is_err());
    assert!(SteppedSpline::new(vec![0.0, 1.0], vec![10.0, -5.0, 8.0, 3.0]).is_err());
}

#[test]
fn test_stepped_spline_evaluate_throws_out_of_range() {
    let times = vec![0.0, 1.0, 2.0, 3.0];
    let numbers = vec![10.0, -5.0, 8.0, 3.0];
    let spline = SteppedSpline::new(times.clone(), numbers).unwrap();

    assert!(spline.evaluate(times[0] - 1.0).is_err());
    assert!(spline.evaluate(times[3] + 1.0).is_err());
}

#[test]
fn test_stepped_spline_evaluate_number() {
    let times = vec![0.0, 1.0, 2.0, 3.0];
    let numbers = vec![10.0, -5.0, 8.0, 3.0];
    let spline = SteppedSpline::new(times.clone(), numbers.clone()).unwrap();

    assert_eq!(spline.evaluate(times[0]).unwrap(), numbers[0]);
    assert_eq!(spline.evaluate(times[1]).unwrap(), numbers[1]);

    let time = (times[0] + times[1]) / 2.0;
    assert_eq!(spline.evaluate(time).unwrap(), numbers[0]);
}

#[test]
fn test_stepped_spline_evaluate_cartesian3() {
    let times = vec![0.0, 1.0, 2.0, 3.0];
    let cartesian_points = vec![
        Cartesian3::new(-1.0, -1.0, 0.0),
        Cartesian3::new(-0.5, -0.125, 0.0),
        Cartesian3::new(0.5, 0.125, 0.0),
        Cartesian3::new(1.0, 1.0, 0.0),
    ];
    let spline = SteppedSpline::new(times.clone(), cartesian_points.clone()).unwrap();

    assert_eq!(spline.evaluate(times[0]).unwrap(), cartesian_points[0]);
    assert_eq!(spline.evaluate(times[1]).unwrap(), cartesian_points[1]);

    let time = (times[0] + times[1]) / 2.0;
    assert_eq!(spline.evaluate(time).unwrap(), cartesian_points[0]);

    let time2 = (times[1] + times[2]) / 2.0;
    assert_eq!(spline.evaluate(time2).unwrap(), cartesian_points[1]);
}

#[test]
fn test_stepped_spline_evaluate_quaternion() {
    let times = vec![0.0, 1.0, 2.0, 3.0];
    let quaternion_points = vec![
        Quaternion::new(0.0, 0.0, 0.0, 0.0),
        Quaternion::new(0.707, 0.0, 0.707, 0.0),
        Quaternion::new(0.5, 0.5, 0.5, 0.5),
        Quaternion::new(0.0, 0.0, 0.0, 1.0),
    ];
    let spline = SteppedSpline::new(times.clone(), quaternion_points.clone()).unwrap();

    assert_eq!(spline.evaluate(times[0]).unwrap(), quaternion_points[0]);
    assert_eq!(spline.evaluate(times[1]).unwrap(), quaternion_points[1]);

    let time = (times[0] + times[1]) / 2.0;
    assert_eq!(spline.evaluate(time).unwrap(), quaternion_points[0]);

    let time2 = (times[1] + times[2]) / 2.0;
    assert_eq!(spline.evaluate(time2).unwrap(), quaternion_points[1]);
}
