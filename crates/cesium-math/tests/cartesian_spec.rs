use cesium_math::{Cartesian2, Cartesian3, Cartesian4, Color, Ellipsoid};

#[test]
fn test_cartesian2_upstream() {
    let a = Cartesian2::new(1.0, 2.0);
    let b = Cartesian2::new(3.0, 4.0);

    let mut buf = [0.0; 2];
    a.pack(&mut buf, 0).unwrap();
    assert_eq!(buf, [1.0, 2.0]);
    assert_eq!(Cartesian2::unpack(&[0.0, 1.0, 2.0, 3.0], 1).unwrap(), a);
    assert_eq!(Cartesian2::from_array(&[1.0, 2.0], 0).unwrap(), a);

    assert_eq!(a + b, Cartesian2::new(4.0, 6.0));
    assert_eq!(b - a, Cartesian2::new(2.0, 2.0));
    assert_eq!(a * 2.0, Cartesian2::new(2.0, 4.0));
    assert_eq!(b / 2.0, Cartesian2::new(1.5, 2.0));
    assert_eq!(-a, Cartesian2::new(-1.0, -2.0));

    assert_eq!(a.dot(&b), 11.0);
    assert_eq!(a.cross(&b), -2.0);
    assert_eq!(Cartesian2::new(3.0, 4.0).magnitude(), 5.0);
    assert_eq!(Cartesian2::new(3.0, 4.0).magnitude_squared(), 25.0);
    assert_eq!(Cartesian2::distance(&a, &b), (4.0 + 4.0_f64).sqrt());

    let norm = Cartesian2::new(3.0, 4.0).normalize();
    assert!((norm.x - 0.6).abs() < 1e-12);
    assert!((norm.y - 0.8).abs() < 1e-12);

    let angle = Cartesian2::UNIT_X.angle_between(&Cartesian2::UNIT_Y);
    assert!((angle - std::f64::consts::FRAC_PI_2).abs() < 1e-12);

    let lerped = Cartesian2::lerp(Cartesian2::ZERO, Cartesian2::new(10.0, 20.0), 0.5);
    assert_eq!(lerped, Cartesian2::new(5.0, 10.0));
}

#[test]
fn test_cartesian3_upstream() {
    let a = Cartesian3::new(1.0, 2.0, 3.0);
    let b = Cartesian3::new(4.0, 5.0, 6.0);

    let mut buf = [0.0; 3];
    a.pack(&mut buf, 0).unwrap();
    assert_eq!(buf, [1.0, 2.0, 3.0]);
    assert_eq!(Cartesian3::unpack(&[0.0, 1.0, 2.0, 3.0], 1).unwrap(), a);
    assert_eq!(Cartesian3::from_array(&[1.0, 2.0, 3.0], 0).unwrap(), a);

    assert_eq!(a + b, Cartesian3::new(5.0, 7.0, 9.0));
    assert_eq!(b - a, Cartesian3::new(3.0, 3.0, 3.0));
    assert_eq!(a * 2.0, Cartesian3::new(2.0, 4.0, 6.0));
    assert_eq!(b / 2.0, Cartesian3::new(2.0, 2.5, 3.0));
    assert_eq!(-a, Cartesian3::new(-1.0, -2.0, -3.0));

    assert_eq!(a.dot(&b), 32.0);
    assert_eq!(Cartesian3::UNIT_X.cross(&Cartesian3::UNIT_Y), Cartesian3::UNIT_Z);
    assert_eq!(Cartesian3::new(1.0, 2.0, 2.0).magnitude(), 3.0);

    let angle = Cartesian3::UNIT_X.angle_between(&Cartesian3::UNIT_Z);
    assert!((angle - std::f64::consts::FRAC_PI_2).abs() < 1e-12);

    let most_ortho = Cartesian3::UNIT_X.most_orthogonal_axis();
    assert_eq!(most_ortho, Cartesian3::UNIT_Y);

    let proj = Cartesian3::new(2.0, 3.0, 4.0).project_vector(&Cartesian3::UNIT_X);
    assert_eq!(proj, Cartesian3::new(2.0, 0.0, 0.0));

    // from_radians and from_degrees with WGS84
    let surface_pt = Cartesian3::from_degrees(0.0, 0.0, Some(0.0), Some(&Ellipsoid::WGS84)).unwrap();
    assert!((surface_pt.x - 6378137.0).abs() < 1e-3);
    assert!(surface_pt.y.abs() < 1e-3);
    assert!(surface_pt.z.abs() < 1e-3);
}

#[test]
fn test_cartesian4_upstream() {
    let a = Cartesian4::new(1.0, 2.0, 3.0, 4.0);
    let b = Cartesian4::new(5.0, 6.0, 7.0, 8.0);

    let mut buf = [0.0; 4];
    a.pack(&mut buf, 0).unwrap();
    assert_eq!(buf, [1.0, 2.0, 3.0, 4.0]);
    assert_eq!(Cartesian4::unpack(&[0.0, 1.0, 2.0, 3.0, 4.0], 1).unwrap(), a);
    assert_eq!(Cartesian4::from_array(&[1.0, 2.0, 3.0, 4.0], 0).unwrap(), a);

    assert_eq!(a + b, Cartesian4::new(6.0, 8.0, 10.0, 12.0));
    assert_eq!(b - a, Cartesian4::new(4.0, 4.0, 4.0, 4.0));
    assert_eq!(a * 2.0, Cartesian4::new(2.0, 4.0, 6.0, 8.0));
    assert_eq!(b / 2.0, Cartesian4::new(2.5, 3.0, 3.5, 4.0));
    assert_eq!(-a, Cartesian4::new(-1.0, -2.0, -3.0, -4.0));

    assert_eq!(Cartesian4::new(1.0, 1.0, 1.0, 1.0).magnitude(), 2.0);

    let c = Color::new(0.1, 0.2, 0.3, 0.4);
    let from_c = Cartesian4::from_color(c);
    assert!((from_c.x - 0.1).abs() < 1e-6);
    assert!((from_c.y - 0.2).abs() < 1e-6);
    assert!((from_c.z - 0.3).abs() < 1e-6);
    assert!((from_c.w - 0.4).abs() < 1e-6);
}
