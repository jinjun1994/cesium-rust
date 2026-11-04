//! Cases adapted from CesiumJS EllipsoidTangentPlaneSpec (Apache-2.0).
use cesium_math::{Cartographic, Ellipsoid, EllipsoidTangentPlane};
use glam::{DVec2, DVec3};

#[test]
fn upstream_construction_and_axes() {
    let t = EllipsoidTangentPlane::new(DVec3::X * 2.0, Ellipsoid::UNIT_SPHERE).unwrap();
    assert_eq!(t.origin, DVec3::X);
    assert_eq!(
        (t.x_axis, t.y_axis, t.z_axis()),
        (DVec3::Y, DVec3::Z, DVec3::X)
    );
    let from =
        EllipsoidTangentPlane::from_points(&[DVec3::X * 2.0, DVec3::ZERO], Ellipsoid::UNIT_SPHERE)
            .unwrap();
    assert_eq!(from.origin, DVec3::X);
    assert!(EllipsoidTangentPlane::new(DVec3::ZERO, Ellipsoid::WGS84).is_err());
    assert!(EllipsoidTangentPlane::from_points(&[], Ellipsoid::WGS84).is_err());
}

#[test]
fn upstream_radial_and_nearest_projection() {
    let t = EllipsoidTangentPlane::new(DVec3::X, Ellipsoid::UNIT_SPHERE).unwrap();
    let points = [
        DVec3::new(1.0, 0.0, 1.0),
        DVec3::X,
        DVec3::Z,
        DVec3::new(1.0, 1.0, 0.0),
        DVec3::Y,
    ];
    assert_eq!(t.project_point_onto_plane(DVec3::Z), None);
    assert_eq!(t.project_point_onto_plane(DVec3::ZERO), None);
    assert_eq!(
        t.project_points_onto_plane(&points),
        vec![DVec2::Y, DVec2::ZERO, DVec2::X]
    );
    assert_eq!(t.project_point_onto_plane(-DVec3::X), Some(DVec2::ZERO));
    for x in [-1.0, 0.0, 1.0, 2.0] {
        assert_eq!(
            t.project_point_to_nearest_on_plane(DVec3::X * x),
            Some(DVec2::ZERO)
        );
    }
    assert_eq!(
        t.project_points_to_nearest_on_plane(&points[..2]).unwrap(),
        vec![DVec2::Y, DVec2::ZERO]
    );
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
fn upstream_wgs84_positions_roundtrip() {
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
