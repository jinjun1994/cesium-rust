use cesium_math::{Cartesian3, CatmullRomSpline};

#[test]
fn test_catmull_rom_spline_throws_invalid() {
    assert!(CatmullRomSpline::new(vec![], vec![], None, None).is_err());
    assert!(CatmullRomSpline::new(vec![0.0], vec![Cartesian3::ZERO], None, None).is_err());
    assert!(CatmullRomSpline::new(
        vec![0.0, 1.0],
        vec![Cartesian3::ZERO, Cartesian3::UNIT_X, Cartesian3::UNIT_Y],
        None,
        None
    )
    .is_err());
}

#[test]
fn test_catmull_rom_spline_tangents() {
    let points = vec![
        Cartesian3::new(-1.0, -1.0, 0.0),
        Cartesian3::new(-0.5, -0.125, 0.0),
        Cartesian3::new(0.5, 0.125, 0.0),
        Cartesian3::new(1.0, 1.0, 0.0),
    ];
    let times = vec![0.0, 1.0, 2.0, 3.0];

    let start = points[1] - points[0];
    let end = points[3] - points[2];

    let crs = CatmullRomSpline::new(
        times.clone(),
        points.clone(),
        Some(start),
        Some(end),
    )
    .unwrap();

    assert_eq!(crs.first_tangent, start);
    assert_eq!(crs.last_tangent, end);

    // auto compute tangents
    let crs_auto = CatmullRomSpline::new(times, points.clone(), None, None).unwrap();
    let expected_start = (points[1] * 2.0 - points[2] - points[0]) * 0.5;
    let expected_end = (points[3] - points[2] * 2.0 + points[1]) * 0.5;

    assert_eq!(crs_auto.first_tangent, expected_start);
    assert_eq!(crs_auto.last_tangent, expected_end);
}

#[test]
fn test_catmull_rom_spline_evaluate() {
    let points = vec![
        Cartesian3::new(-1.0, -1.0, 0.0),
        Cartesian3::new(-0.5, -0.125, 0.0),
        Cartesian3::new(0.5, 0.125, 0.0),
        Cartesian3::new(1.0, 1.0, 0.0),
    ];
    let times = vec![0.0, 1.0, 2.0, 3.0];

    let crs = CatmullRomSpline::new(times.clone(), points.clone(), None, None).unwrap();

    let p0 = crs.evaluate(times[0]).unwrap();
    assert!((p0.x - points[0].x).abs() < 1e-10);
    assert!((p0.y - points[0].y).abs() < 1e-10);
    assert!((p0.z - points[0].z).abs() < 1e-10);

    let p1 = crs.evaluate(times[1]).unwrap();
    assert!((p1.x - points[1].x).abs() < 1e-10);
    assert!((p1.y - points[1].y).abs() < 1e-10);

    let p_mid = crs.evaluate(1.5).unwrap();
    assert!(p_mid.x.is_finite());
    assert!(p_mid.y.is_finite());
}
