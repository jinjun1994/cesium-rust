use cesium_math::{Cartesian3, LinearSpline};

#[test]
fn test_linear_spline_throws_invalid_params() {
    assert!(LinearSpline::<f64>::new(vec![], vec![]).is_err());
    assert!(LinearSpline::<f64>::new(vec![0.0], vec![1.0]).is_err());
    assert!(LinearSpline::<f64>::new(vec![0.0, 1.0], vec![1.0, 2.0, 3.0]).is_err());
}

#[test]
fn test_linear_spline_evaluate_number() {
    let times = vec![0.0, 1.0, 2.0, 3.0];
    let points = vec![3.0, 5.0, 1.0, 10.0];

    let ls = LinearSpline::new(times.clone(), points.clone()).unwrap();

    assert!(ls.evaluate(times[0] - 1.0).is_err());
    assert_eq!(ls.evaluate(times[0]).unwrap(), points[0]);

    let time = (times[0] + times[1]) / 2.0;
    let t = (time - times[0]) / (times[1] - times[0]);
    let expected = (1.0 - t) * points[0] + t * points[1];
    assert!((ls.evaluate(time).unwrap() - expected).abs() < 1e-10);
}

#[test]
fn test_linear_spline_evaluate_cartesian3() {
    let times = vec![0.0, 1.0, 2.0, 3.0];
    let points = vec![
        Cartesian3::new(-1.0, -1.0, 0.0),
        Cartesian3::new(-0.5, -0.125, 0.0),
        Cartesian3::new(0.5, 0.125, 0.0),
        Cartesian3::new(1.0, 1.0, 0.0),
    ];

    let ls = LinearSpline::new(times.clone(), points.clone()).unwrap();

    assert_eq!(ls.evaluate(times[0]).unwrap(), points[0]);

    let time = (times[0] + times[1]) / 2.0;
    let t = (time - times[0]) / (times[1] - times[0]);
    let expected = Cartesian3::lerp(points[0], points[1], t);
    let val = ls.evaluate(time).unwrap();
    assert!((val.x - expected.x).abs() < 1e-10);
    assert!((val.y - expected.y).abs() < 1e-10);
    assert!((val.z - expected.z).abs() < 1e-10);
}

#[test]
fn test_linear_spline_wrap_and_clamp_time() {
    let times = vec![0.0, 1.0, 2.0, 3.0];
    let points = vec![3.0, 5.0, 1.0, 10.0];
    let ls = LinearSpline::new(times, points).unwrap();

    assert_eq!(ls.clamp_time(-1.0), 0.0);
    assert_eq!(ls.clamp_time(4.0), 3.0);
    assert_eq!(ls.clamp_time(1.5), 1.5);

    assert_eq!(ls.wrap_time(0.0), 0.0);
    assert_eq!(ls.wrap_time(3.0), 3.0);
    assert_eq!(ls.wrap_time(3.5), 0.5);
    assert_eq!(ls.wrap_time(-0.5), 2.5);
}
