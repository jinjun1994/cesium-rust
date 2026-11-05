//! Cases adapted from CesiumJS EllipsoidTangentPlaneSpec (Apache-2.0).
use cesium_math::{Cartographic, Ellipsoid, EllipsoidTangentPlane};
use glam::{DVec2, DVec3};

#[test]
fn constructor_defaults_to_wgs84() {
    let origin = DVec3::new(Ellipsoid::WGS84.radii().x, 0.0, 0.0);
    let tangent_plane = EllipsoidTangentPlane::from_origin(origin).unwrap();
    assert_eq!(tangent_plane.ellipsoid, Ellipsoid::WGS84);
    assert_eq!(tangent_plane.origin, origin);
}

#[test]
fn constructor_sets_expected_values() {
    let tangent_plane =
        EllipsoidTangentPlane::new(DVec3::X, Ellipsoid::UNIT_SPHERE).unwrap();
    assert_eq!(tangent_plane.ellipsoid, Ellipsoid::UNIT_SPHERE);
    assert_eq!(tangent_plane.origin, DVec3::X);
}

#[test]
fn from_points_sets_expected_values() {
    let points = [DVec3::new(2.0, 0.0, 0.0), DVec3::ZERO];
    let tangent_plane =
        EllipsoidTangentPlane::from_points(&points, Ellipsoid::UNIT_SPHERE).unwrap();
    assert_eq!(tangent_plane.ellipsoid, Ellipsoid::UNIT_SPHERE);
    assert_eq!(tangent_plane.origin, DVec3::X);
}

#[test]
fn project_point_onto_plane_returns_none_for_unsolvable_projections() {
    let tangent_plane =
        EllipsoidTangentPlane::new(DVec3::X, Ellipsoid::UNIT_SPHERE).unwrap();
    assert_eq!(tangent_plane.project_point_onto_plane(DVec3::Z), None);
    assert_eq!(tangent_plane.project_point_onto_plane(DVec3::ZERO), None);
}

#[test]
fn project_point_onto_plane_works() {
    let tangent_plane =
        EllipsoidTangentPlane::new(DVec3::X, Ellipsoid::UNIT_SPHERE).unwrap();
    let position = DVec3::new(1.0, 0.0, 1.0);
    let expected = DVec2::new(0.0, 1.0);
    assert_eq!(
        tangent_plane.project_point_onto_plane(position),
        Some(expected)
    );
}

#[test]
fn project_points_onto_plane_works() {
    let tangent_plane =
        EllipsoidTangentPlane::new(DVec3::X, Ellipsoid::UNIT_SPHERE).unwrap();
    let positions = [
        DVec3::new(1.0, 0.0, 1.0),
        DVec3::new(1.0, 0.0, 0.0),
        DVec3::new(1.0, 1.0, 0.0),
    ];
    let expected = vec![
        DVec2::new(0.0, 1.0),
        DVec2::new(0.0, 0.0),
        DVec2::new(1.0, 0.0),
    ];
    assert_eq!(tangent_plane.project_points_onto_plane(&positions), expected);
}

#[test]
fn project_points_onto_plane_works_when_some_points_cannot_be_projected() {
    let tangent_plane =
        EllipsoidTangentPlane::new(DVec3::X, Ellipsoid::UNIT_SPHERE).unwrap();
    let positions = [
        DVec3::new(1.0, 0.0, 1.0),
        DVec3::new(1.0, 0.0, 0.0),
        DVec3::new(0.0, 0.0, 1.0), // unsolvable
        DVec3::new(1.0, 1.0, 0.0),
        DVec3::new(0.0, 1.0, 0.0), // unsolvable
    ];
    let expected = vec![
        DVec2::new(0.0, 1.0),
        DVec2::new(0.0, 0.0),
        DVec2::new(1.0, 0.0),
    ];
    assert_eq!(tangent_plane.project_points_onto_plane(&positions), expected);
}

#[test]
fn project_point_to_nearest_on_plane_works() {
    let tangent_plane =
        EllipsoidTangentPlane::new(DVec3::X, Ellipsoid::UNIT_SPHERE).unwrap();
    let position = DVec3::new(1.0, 0.0, 1.0);
    let expected = DVec2::new(0.0, 1.0);
    assert_eq!(
        tangent_plane.project_point_to_nearest_on_plane(position),
        Some(expected)
    );
    for x in [-1.0, 0.0, 1.0, 2.0] {
        assert_eq!(
            tangent_plane.project_point_to_nearest_on_plane(DVec3::X * x),
            Some(DVec2::ZERO)
        );
    }
}

#[test]
fn project_points_to_nearest_on_plane_works() {
    let tangent_plane =
        EllipsoidTangentPlane::new(DVec3::X, Ellipsoid::UNIT_SPHERE).unwrap();
    let positions = [
        DVec3::new(1.0, 0.0, 1.0),
        DVec3::new(1.0, 0.0, 0.0),
        DVec3::new(1.0, 1.0, 0.0),
    ];
    let expected = vec![
        DVec2::new(0.0, 1.0),
        DVec2::new(0.0, 0.0),
        DVec2::new(1.0, 0.0),
    ];
    assert_eq!(
        tangent_plane
            .project_points_to_nearest_on_plane(&positions)
            .unwrap(),
        expected
    );
}

#[test]
fn clones_and_equals() {
    let t1 = EllipsoidTangentPlane::new(DVec3::X, Ellipsoid::UNIT_SPHERE).unwrap();
    let t2 = t1;
    assert_eq!(t1, t2);
    assert!(t1.equals(Some(&t2)));
    let t3 = EllipsoidTangentPlane::new(DVec3::Y, Ellipsoid::UNIT_SPHERE).unwrap();
    assert!(!t1.equals(Some(&t3)));
    assert!(!t1.equals(None));
}

#[test]
fn upstream_geocentric_projection_and_batch() {
    let t = EllipsoidTangentPlane::new(DVec3::X, Ellipsoid::UNIT_SPHERE).unwrap();
    let input = [DVec2::new(2.0, -2.0), DVec2::new(2.0, 2.0)];
    let result = t.project_points_onto_ellipsoid(&input).unwrap();
    assert_eq!(result[0], DVec3::new(1.0 / 3.0, 2.0 / 3.0, -2.0 / 3.0));
    assert_eq!(result[1], DVec3::new(1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0));
    assert!(t
        .project_point_onto_ellipsoid(DVec2::splat(f64::NAN))
        .is_none());
}

#[test]
fn project_points_onto_ellipsoid_works_with_wgs84_positions() {
    let points: Vec<_> = [
        (-72.0, 40.0),
        (-68.0, 35.0),
        (-75.0, 30.0),
        (-70.0, 30.0),
        (-68.0, 40.0),
    ]
    .into_iter()
    .map(|(lon, lat)| {
        Ellipsoid::WGS84
            .cartographic_to_cartesian(Cartographic::degrees(lon, lat, 0.0).unwrap())
            .unwrap()
    })
    .collect();
    let t = EllipsoidTangentPlane::from_points(&points, Ellipsoid::WGS84).unwrap();
    let projected = t.project_points_onto_plane(&points);
    assert_eq!(projected.len(), points.len());
    let restored = t.project_points_onto_ellipsoid(&projected).unwrap();
    for (original, restored) in points.iter().zip(restored) {
        assert!(original.distance(restored) < 1e-7);
    }
}

#[test]
fn rejects_invalid_inputs() {
    assert!(EllipsoidTangentPlane::new(DVec3::ZERO, Ellipsoid::WGS84).is_err());
    assert!(EllipsoidTangentPlane::from_points(&[], Ellipsoid::WGS84).is_err());
    assert!(EllipsoidTangentPlane::new(DVec3::splat(f64::NAN), Ellipsoid::WGS84).is_err());
}
