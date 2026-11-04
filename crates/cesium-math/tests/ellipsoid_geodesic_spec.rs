use std::f64::consts::PI;
use cesium_math::{
    Cartographic, Ellipsoid, EllipsoidGeodesic, EPSILON11, EPSILON12, PI_OVER_TWO,
};
use glam::DVec3;

#[test]
fn test_ellipsoid_geodesic_throws_without_endpoints() {
    let empty = EllipsoidGeodesic::default();
    assert!(empty.surface_distance().is_err());
    assert!(empty.start_heading().is_err());
    assert!(empty.end_heading().is_err());
    assert!(empty.interpolate_using_surface_distance(0.0).is_err());
}

#[test]
fn test_ellipsoid_geodesic_two_points() {
    let fifteen_deg = PI / 12.0;
    let start = Cartographic::new(fifteen_deg, fifteen_deg, 0.0);
    let thirty_deg = PI / 6.0;
    let end = Cartographic::new(thirty_deg, thirty_deg, 0.0);

    let geodesic = EllipsoidGeodesic::new(Some(start), Some(end), None);
    assert_eq!(geodesic.start(), Some(start));
    assert_eq!(geodesic.end(), Some(end));
}

#[test]
fn test_ellipsoid_geodesic_headings_and_equator_distance() {
    let ellipsoid = Ellipsoid::new(DVec3::new(6.0, 6.0, 3.0)).unwrap();
    let start = Cartographic::new(PI_OVER_TWO, 0.0, 0.0);
    let end = Cartographic::new(PI, 0.0, 0.0);

    let geodesic = EllipsoidGeodesic::new(Some(start), Some(end), Some(ellipsoid));
    let sh = geodesic.start_heading().unwrap();
    let eh = geodesic.end_heading().unwrap();
    assert!((sh - PI_OVER_TWO).abs() < EPSILON11);
    assert!((eh - PI_OVER_TWO).abs() < EPSILON11);

    let dist = geodesic.surface_distance().unwrap();
    assert!((dist - PI_OVER_TWO * 6.0).abs() < EPSILON11);
}

#[test]
fn test_ellipsoid_geodesic_meridian_distance() {
    let ellipsoid = Ellipsoid::new(DVec3::new(6.0, 6.0, 6.0)).unwrap();
    let fifteen_deg = PI / 12.0;
    let start = Cartographic::new(PI_OVER_TWO, fifteen_deg, 0.0);
    let fortyfive_deg = PI / 4.0;
    let end = Cartographic::new(PI_OVER_TWO, fortyfive_deg, 0.0);

    let geodesic = EllipsoidGeodesic::new(Some(start), Some(end), Some(ellipsoid));
    let thirty_deg = PI / 6.0;
    assert!((geodesic.surface_distance().unwrap() - thirty_deg * 6.0).abs() < EPSILON11);
}

#[test]
fn test_ellipsoid_geodesic_interpolation() {
    let fifteen_deg = PI / 12.0;
    let start = Cartographic::new(fifteen_deg, 0.0, 0.0);
    let fortyfive_deg = PI / 4.0;
    let end = Cartographic::new(fortyfive_deg, 0.0, 0.0);
    let thirty_deg = PI / 6.0;

    let geodesic = EllipsoidGeodesic::new(Some(start), Some(end), None);

    let mid = geodesic.interpolate_using_fraction(0.5).unwrap();
    assert!((mid.longitude - thirty_deg).abs() < EPSILON12);
    assert!(mid.latitude.abs() < EPSILON12);

    let p0 = geodesic.interpolate_using_fraction(0.0).unwrap();
    let p1 = geodesic.interpolate_using_fraction(1.0).unwrap();
    assert!((p0.longitude - start.longitude).abs() < EPSILON12);
    assert!((p1.longitude - end.longitude).abs() < EPSILON12);
}
