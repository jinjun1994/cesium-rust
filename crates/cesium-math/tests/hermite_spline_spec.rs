use cesium_math::{Cartesian3, HermiteSpline};

#[test]
fn test_hermite_spline_throws_invalid() {
    assert!(HermiteSpline::new(vec![], vec![], vec![], vec![]).is_err());
    assert!(HermiteSpline::new(
        vec![0.0],
        vec![Cartesian3::ZERO],
        vec![],
        vec![]
    )
    .is_err());
    assert!(HermiteSpline::new(
        vec![0.0, 1.0],
        vec![Cartesian3::ZERO, Cartesian3::UNIT_X],
        vec![],
        vec![]
    )
    .is_err());
}

#[test]
fn test_hermite_spline_create_and_evaluate() {
    let times = vec![0.0, 1.0, 3.0, 4.5, 6.0];
    let points = vec![
        Cartesian3::new(1235398.0, -4810983.0, 4146266.0),
        Cartesian3::new(1372574.0, -5345182.0, 4606657.0),
        Cartesian3::new(-757983.0, -5542796.0, 4514323.0),
        Cartesian3::new(-2821260.0, -5248423.0, 4021290.0),
        Cartesian3::new(-2539788.0, -4724797.0, 3620093.0),
    ];

    let out_tangents = vec![
        Cartesian3::new(1125196.0, -161816.0, 270551.0),
        Cartesian3::new(-1051415.0, -891086.0, 88497.0),
        Cartesian3::new(-1359149.0, 11676.0, -520121.0),
        Cartesian3::new(432429.0, 693106.0, -264007.0),
    ];
    let in_tangents = vec![
        Cartesian3::new(-1968.0, -13560.0, -6526.0),
        Cartesian3::new(-20341.0, 777.0, -9785.0),
        Cartesian3::new(-19400.0, -432.0, -10309.0),
        Cartesian3::new(-2973.0, 7575.0, -5156.0),
    ];

    let hs = HermiteSpline::new(
        times.clone(),
        points.clone(),
        in_tangents.clone(),
        out_tangents.clone(),
    )
    .unwrap();

    let p0 = hs.evaluate(times[0]).unwrap();
    assert!((p0.x - points[0].x).abs() < 1e-4);
    assert!((p0.y - points[0].y).abs() < 1e-4);
    assert!((p0.z - points[0].z).abs() < 1e-4);

    let p_mid = hs.evaluate(0.5).unwrap();
    assert!(p_mid.x.is_finite());
    assert!(p_mid.y.is_finite());
    assert!(p_mid.z.is_finite());
}

#[test]
fn test_hermite_spline_clamped_and_natural() {
    let times = vec![0.0, 1.0, 2.0, 3.0];
    let points = vec![
        Cartesian3::new(-1.0, -1.0, 0.0),
        Cartesian3::new(-0.5, -0.125, 0.0),
        Cartesian3::new(0.5, 0.125, 0.0),
        Cartesian3::new(1.0, 1.0, 0.0),
    ];

    let first_tangent = Cartesian3::new(0.5, 0.5, 0.0);
    let last_tangent = Cartesian3::new(0.5, 0.5, 0.0);

    let clamped = HermiteSpline::create_clamped_spline(
        times.clone(),
        points.clone(),
        first_tangent,
        last_tangent,
    )
    .unwrap();
    assert_eq!(clamped.points.len(), 4);
    assert_eq!(clamped.in_tangents.len(), 3);
    assert_eq!(clamped.out_tangents.len(), 3);

    let natural = HermiteSpline::create_natural_spline(times, points).unwrap();
    assert_eq!(natural.points.len(), 4);
    assert_eq!(natural.in_tangents.len(), 3);
    assert_eq!(natural.out_tangents.len(), 3);
}
