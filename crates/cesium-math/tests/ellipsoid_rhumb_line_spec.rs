use std::f64::consts::PI;
use cesium_math::{Cartographic, Ellipsoid, EllipsoidRhumbLine, EPSILON10};

#[test]
fn test_ellipsoid_rhumb_line_throws_without_endpoints() {
    let empty = EllipsoidRhumbLine::default();
    assert!(empty.surface_distance().is_err());
    assert!(empty.heading().is_err());
    assert!(empty.interpolate_using_surface_distance(0.0).is_err());
}

#[test]
fn test_ellipsoid_rhumb_line_from_start_heading_distance() {
    let ellipsoid = Ellipsoid::WGS84;
    let fifteen_deg = PI / 12.0;
    let start = Cartographic::new(fifteen_deg, fifteen_deg, 0.0);
    let heading = fifteen_deg;
    let distance = fifteen_deg * ellipsoid.maximum_radius();

    let rhumb = EllipsoidRhumbLine::from_start_heading_distance(
        start,
        heading,
        distance,
        Some(ellipsoid),
    );

    assert_eq!(rhumb.start(), Some(start));
    let surf_dist = rhumb.surface_distance().unwrap();
    assert!((surf_dist - distance).abs() < 1e-4);
    let h = rhumb.heading().unwrap();
    assert!((h - heading).abs() < 1e-10);
}

#[test]
fn test_ellipsoid_rhumb_line_two_points() {
    let fifteen_deg = PI / 12.0;
    let start = Cartographic::new(fifteen_deg, fifteen_deg, 0.0);
    let thirty_deg = PI / 6.0;
    let end = Cartographic::new(thirty_deg, thirty_deg, 0.0);

    let rhumb = EllipsoidRhumbLine::new(Some(start), Some(end), None);
    assert_eq!(rhumb.start(), Some(start));
    assert_eq!(rhumb.end(), Some(end));

    let dist = rhumb.surface_distance().unwrap();
    assert!(dist > 0.0);

    let p0 = rhumb.interpolate_using_fraction(0.0).unwrap();
    assert!((p0.longitude - start.longitude).abs() < EPSILON10);
    assert!((p0.latitude - start.latitude).abs() < EPSILON10);

    let p1 = rhumb.interpolate_using_fraction(1.0).unwrap();
    assert!((p1.longitude - end.longitude).abs() < EPSILON10);
    assert!((p1.latitude - end.latitude).abs() < EPSILON10);
}
