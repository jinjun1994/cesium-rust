use cesium_math::{
    Cartesian2, Cartesian3, Ellipsoid, Stereographic, EPSILON7, NORTH_POLE_TANGENT_PLANE,
    SOUTH_POLE_TANGENT_PLANE,
};

#[test]
fn test_stereographic_construct_default() {
    let s = Stereographic::default();
    assert_eq!(s.x(), 0.0);
    assert_eq!(s.y(), 0.0);
    assert_eq!(s.tangent_plane, *NORTH_POLE_TANGENT_PLANE);
}

#[test]
fn test_stereographic_construct_with_values() {
    let s = Stereographic::new(
        Some(Cartesian2::new(1.0, 2.0)),
        Some(*SOUTH_POLE_TANGENT_PLANE),
    );
    assert_eq!(s.x(), 1.0);
    assert_eq!(s.y(), 2.0);
    assert_eq!(s.tangent_plane, *SOUTH_POLE_TANGENT_PLANE);
}

#[test]
fn test_stereographic_from_cartesian_northern_hemisphere() {
    let pt = Cartesian3::from_degrees(30.0, 60.0, None, None).unwrap();
    let s = Stereographic::from_cartesian(pt).unwrap();
    assert!((s.x() - 0.1347555369).abs() < EPSILON7);
    assert!((s.y() - (-0.2334034365)).abs() < EPSILON7);
    assert_eq!(s.tangent_plane, *NORTH_POLE_TANGENT_PLANE);
}

#[test]
fn test_stereographic_from_cartesian_at_origin() {
    let pt = Cartesian3::from_degrees(0.0, 0.0, None, None).unwrap();
    let s = Stereographic::from_cartesian(pt).unwrap();
    assert!((s.x() - 0.0).abs() < EPSILON7);
    assert!((s.y() - (-1.0)).abs() < EPSILON7);
    assert_eq!(s.tangent_plane, *NORTH_POLE_TANGENT_PLANE);
}

#[test]
fn test_stereographic_from_cartesian_southern_hemisphere() {
    let pt = Cartesian3::from_degrees(30.0, -60.0, None, None).unwrap();
    let s = Stereographic::from_cartesian(pt).unwrap();
    assert!((s.x() - 0.1347555369).abs() < EPSILON7);
    assert!((s.y() - (-0.2334034365)).abs() < EPSILON7);
    assert_eq!(s.tangent_plane, *SOUTH_POLE_TANGENT_PLANE);
}

#[test]
fn test_stereographic_longitude_northern_hemisphere() {
    let cases = [
        (30.0, 60.0),
        (60.0, 30.0),
        (-60.0, 30.0),
        (-135.0, 60.0),
        (135.0, 60.0),
    ];
    for (lon, lat) in cases {
        let pt = Cartesian3::from_degrees(lon, lat, None, None).unwrap();
        let s = Stereographic::from_cartesian(pt).unwrap();
        assert!(
            (s.longitude() - lon.to_radians()).abs() < EPSILON7,
            "Failed for lon {}, lat {}",
            lon,
            lat
        );
    }
}

#[test]
fn test_stereographic_longitude_southern_hemisphere() {
    let cases = [
        (30.0, -60.0),
        (60.0, -30.0),
        (-60.0, -30.0),
        (-135.0, -60.0),
        (135.0, -60.0),
    ];
    for (lon, lat) in cases {
        let pt = Cartesian3::from_degrees(lon, lat, None, None).unwrap();
        let s = Stereographic::from_cartesian(pt).unwrap();
        assert!(
            (s.longitude() - lon.to_radians()).abs() < EPSILON7,
            "Failed for lon {}, lat {}",
            lon,
            lat
        );
    }
}

#[test]
fn test_stereographic_conformal_latitude_northern_hemisphere() {
    let pt1 = Cartesian3::from_degrees(30.0, 60.0, None, None).unwrap();
    let s1 = Stereographic::from_cartesian(pt1).unwrap();
    assert!((s1.conformal_latitude() - 1.04428418).abs() < EPSILON7);

    let pt2 = Cartesian3::from_degrees(60.0, 30.0, None, None).unwrap();
    let s2 = Stereographic::from_cartesian(pt2).unwrap();
    assert!((s2.conformal_latitude() - 0.52069517).abs() < EPSILON7);
}

#[test]
fn test_stereographic_conformal_latitude_southern_hemisphere() {
    let pt1 = Cartesian3::from_degrees(30.0, -60.0, None, None).unwrap();
    let s1 = Stereographic::from_cartesian(pt1).unwrap();
    assert!((s1.conformal_latitude() - (-1.04428418)).abs() < EPSILON7);

    let pt2 = Cartesian3::from_degrees(60.0, -30.0, None, None).unwrap();
    let s2 = Stereographic::from_cartesian(pt2).unwrap();
    assert!((s2.conformal_latitude() - (-0.52069517)).abs() < EPSILON7);
}

#[test]
fn test_stereographic_get_latitude_northern_hemisphere() {
    let pt1 = Cartesian3::from_degrees(30.0, 60.0, None, None).unwrap();
    let s1 = Stereographic::from_cartesian(pt1).unwrap();
    assert!((s1.get_latitude(Some(&Ellipsoid::WGS84)) - 60.0_f64.to_radians()).abs() < EPSILON7);

    let pt2 = Cartesian3::from_degrees(60.0, 30.0, None, None).unwrap();
    let s2 = Stereographic::from_cartesian(pt2).unwrap();
    assert!((s2.get_latitude(Some(&Ellipsoid::WGS84)) - 30.0_f64.to_radians()).abs() < EPSILON7);
}

#[test]
fn test_stereographic_get_latitude_southern_hemisphere() {
    let pt1 = Cartesian3::from_degrees(30.0, -60.0, None, None).unwrap();
    let s1 = Stereographic::from_cartesian(pt1).unwrap();
    assert!((s1.get_latitude(Some(&Ellipsoid::WGS84)) - (-60.0_f64.to_radians())).abs() < EPSILON7);

    let pt2 = Cartesian3::from_degrees(60.0, -30.0, None, None).unwrap();
    let s2 = Stereographic::from_cartesian(pt2).unwrap();
    assert!((s2.get_latitude(Some(&Ellipsoid::WGS84)) - (-30.0_f64.to_radians())).abs() < EPSILON7);
}

#[test]
fn test_stereographic_from_cartesian_array() {
    let pts = [
        Cartesian3::from_degrees(30.0, 60.0, None, None).unwrap(),
        Cartesian3::from_degrees(30.0, -60.0, None, None).unwrap(),
    ];
    let stereos = Stereographic::from_cartesian_array(&pts).unwrap();
    assert_eq!(stereos.len(), 2);

    assert!((stereos[0].x() - 0.1347555369).abs() < EPSILON7);
    assert!((stereos[0].y() - (-0.2334034365)).abs() < EPSILON7);
    assert_eq!(stereos[0].tangent_plane, *NORTH_POLE_TANGENT_PLANE);

    assert!((stereos[1].x() - 0.1347555369).abs() < EPSILON7);
    assert!((stereos[1].y() - (-0.2334034365)).abs() < EPSILON7);
    assert_eq!(stereos[1].tangent_plane, *SOUTH_POLE_TANGENT_PLANE);
}
