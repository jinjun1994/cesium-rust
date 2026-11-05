//! Cases adapted from CesiumJS EllipsoidalOccluderSpec (Apache-2.0).
use cesium_math::{Ellipsoid, EllipsoidalOccluder, GeoRectangle, Sphere};
use glam::DVec3;

#[test]
fn constructor_uses_ellipsoid_and_equality() {
    let ellipsoid = Ellipsoid::new(DVec3::new(2.0, 3.0, 4.0)).unwrap();
    let occluder = EllipsoidalOccluder::from_ellipsoid(ellipsoid).unwrap();
    assert_eq!(occluder.ellipsoid(), ellipsoid);
    assert_eq!(occluder.camera_position(), DVec3::ZERO);

    let occ2 = EllipsoidalOccluder::new(ellipsoid, DVec3::new(1.0, 2.0, 3.0)).unwrap();
    assert_eq!(occ2.ellipsoid(), ellipsoid);
    assert_eq!(occ2.camera_position(), DVec3::new(1.0, 2.0, 3.0));

    assert!(occ2.equals(Some(&occ2)));
    assert!(!occ2.equals(Some(&occluder)));
    assert!(!occ2.equals(None));
}

#[test]
fn visibility_above_and_below_surface() {
    let ellipsoid = Ellipsoid::new(DVec3::new(1.0, 1.1, 0.9)).unwrap();
    let occ = EllipsoidalOccluder::new(ellipsoid, DVec3::new(0.0, 0.0, 2.5)).unwrap();
    let point = DVec3::new(0.0, -3.0, -3.0);
    assert!(occ.is_point_visible(point));
    assert!(occ.is_scaled_space_point_visible(ellipsoid.to_scaled_space(point)));
    let mut occ =
        EllipsoidalOccluder::new(Ellipsoid::WGS84, DVec3::new(7_000_000.0, 0.0, 0.0)).unwrap();
    assert!(!occ.is_point_visible(DVec3::new(-7_000_000.0, 0.0, 0.0)));
    assert!(occ.is_point_visible(DVec3::new(6_900_000.0, 0.0, 0.0)));
    assert!(occ.is_point_visible(DVec3::new(7_100_000.0, 0.0, 0.0)));
    assert!(!occ.is_point_visible(DVec3::new(4_510_635.0, 4_510_635.0, 0.0)));
    occ.set_camera_position(DVec3::new(
        Ellipsoid::WGS84.minimum_radius() - 100.0,
        0.0,
        0.0,
    ))
    .unwrap();
    assert!(!occ.is_point_visible(DVec3::new(-7_000_000.0, 0.0, 0.0)));
    let previous = occ.camera_position();
    assert!(occ.set_camera_position(DVec3::splat(f64::NAN)).is_err());
    assert_eq!(occ.camera_position(), previous);
}

#[test]
fn culling_point_degenerate_and_direction_cases() {
    let ellipsoid = Ellipsoid::new(DVec3::new(12345.0, 4567.0, 8910.0)).unwrap();
    let occ = EllipsoidalOccluder::new(ellipsoid, DVec3::ZERO).unwrap();
    assert_eq!(
        occ.compute_horizon_culling_point(DVec3::X, &[DVec3::new(12345.0, 0.0, 0.0)]),
        Some(DVec3::X)
    );
    assert_eq!(
        occ.compute_horizon_culling_point(DVec3::X, &[DVec3::new(0.0, 4567.0, 0.0)]),
        None
    );
    assert_eq!(
        occ.compute_horizon_culling_point(DVec3::X, &[DVec3::new(-14000.0, -1000.0, 0.0)]),
        None
    );
    assert_eq!(
        occ.compute_horizon_culling_point(DVec3::ZERO, &[DVec3::X]),
        None
    );
    assert_eq!(occ.compute_horizon_culling_point(DVec3::X, &[]), None);
}

#[test]
fn under_surface_culling_and_vertex_equivalence() {
    let occ = EllipsoidalOccluder::new(Ellipsoid::UNIT_SPHERE, DVec3::Z).unwrap();
    for (direction, expected) in [(DVec3::ONE.normalize(), true), (DVec3::Y, false)] {
        let point = direction * 0.5;
        let original = occ.compute_horizon_culling_point(point, &[point]).unwrap();
        let shrunk = occ
            .compute_horizon_culling_point_under_ellipsoid(point, &[point], -0.5)
            .unwrap();
        assert!(!occ.is_scaled_space_point_visible(original));
        assert_eq!(
            occ.is_scaled_space_point_visible_under_ellipsoid(shrunk, -0.5),
            expected
        );
        let center = DVec3::splat(0.25);
        let v = point - center;
        let from_vertices = occ
            .compute_horizon_culling_point_from_vertices(
                point,
                &[v.x, v.y, v.z, 99.0],
                4,
                center,
                -0.5,
            )
            .unwrap();
        assert!((from_vertices - shrunk).length() < 1e-14);
    }
    assert!(occ
        .compute_horizon_culling_point_from_vertices(DVec3::X, &[1.0, 2.0], 3, DVec3::ZERO, 0.0)
        .is_none());
    assert!(occ
        .compute_horizon_culling_point_from_rectangle(GeoRectangle::MAXIMUM, Ellipsoid::UNIT_SPHERE)
        .is_none());
}

// For a sphere, the closest distance of a forward ray from the origin can
// be computed independently of the grazing-altitude implementation.
fn assert_grazing(origin: DVec3, points: &[DVec3], radius: f64) {
    let mut tangent = false;
    for point in points {
        let direction = (*point - origin).normalize();
        let t = -origin.dot(direction);
        assert!(t >= 0.0);
        let altitude = (origin + direction * t).length() - radius;
        assert!(altitude <= 1e-5, "altitude {altitude}");
        tangent |= altitude.abs() < 1e-5;
    }
    assert!(tangent);
}

#[test]
fn upstream_grazing_points_and_interleaved_vertices() {
    let radius = 12345.0;
    let ellipsoid = Ellipsoid::new(DVec3::splat(radius)).unwrap();
    let occ = EllipsoidalOccluder::new(ellipsoid, DVec3::ZERO).unwrap();
    let points = [
        DVec3::new(-12345.0, 12345.0, 12345.0),
        DVec3::new(-12346.0, 12345.0, 12345.0),
        DVec3::new(-12446.0, 12445.0, 12445.0),
    ];
    let direction = Sphere::from_points(&points).unwrap().center;
    let scaled = occ
        .compute_horizon_culling_point(direction, &points)
        .unwrap();
    assert_grazing(ellipsoid.from_scaled_space(scaled), &points, radius);
    let center = DVec3::new(-12000.0, 12000.0, 12000.0);
    let mut vertices = Vec::new();
    for &point in &points {
        let local = point - center;
        vertices.extend_from_slice(&[local.x, local.y, local.z, 1.0, 2.0, 3.0, 4.0]);
    }
    let from_vertices = occ
        .compute_horizon_culling_point_from_vertices(direction, &vertices, 7, center, 0.0)
        .unwrap();
    assert!((from_vertices - scaled).length() < 1e-14);
}

#[test]
fn upstream_rectangle_culling_point_is_conservative() {
    let radius = 12345.0;
    let ellipsoid = Ellipsoid::new(DVec3::splat(radius)).unwrap();
    let occ = EllipsoidalOccluder::new(ellipsoid, DVec3::ZERO).unwrap();
    let rectangle = GeoRectangle::radians(0.1, 0.2, 0.3, 0.4).unwrap();
    let scaled = occ
        .compute_horizon_culling_point_from_rectangle(rectangle, ellipsoid)
        .unwrap();
    let corners = [
        rectangle.southwest(),
        rectangle.southeast(),
        rectangle.northwest(),
        rectangle.northeast(),
    ]
    .map(|p| ellipsoid.cartographic_to_cartesian(p).unwrap());
    assert_grazing(ellipsoid.from_scaled_space(scaled), &corners, radius);
}
