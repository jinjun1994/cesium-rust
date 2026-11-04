//! CesiumJS AxisAlignedBoundingBoxSpec and intersection boundary cases.
use cesium_math::{AxisAlignedBoundingBox as Aabb, PlaneIntersection};
use glam::DVec3;

#[test]
fn constructor_corners_clone_and_equality() {
    let zero = Aabb::default();
    assert_eq!(
        (zero.minimum, zero.maximum, zero.center),
        (DVec3::ZERO, DVec3::ZERO, DVec3::ZERO)
    );
    let min = DVec3::new(1.0, 2.0, 3.0);
    let max = DVec3::new(4.0, 5.0, 6.0);
    let expected = DVec3::new(2.5, 3.5, 4.5);
    assert_eq!(
        Aabb::from_corners(min, max),
        Aabb::new(min, max, Some(expected))
    );
    let offset = Aabb::new(DVec3::Y, DVec3::X, Some(DVec3::Z));
    assert_eq!(offset.clone(), offset);
    assert_ne!(offset, Aabb::new(DVec3::Y, DVec3::X, Some(DVec3::Y)));
    assert_ne!(offset, Aabb::new(DVec3::ZERO, DVec3::X, Some(DVec3::Z)));
    assert_ne!(offset, Aabb::new(DVec3::Y, DVec3::ZERO, Some(DVec3::Z)));
}

#[test]
fn upstream_from_points_empty_single_and_multiple() {
    assert_eq!(Aabb::from_points(&[]).unwrap(), Aabb::default());
    let points = [
        DVec3::new(3.0, -1.0, -3.0),
        DVec3::new(2.0, -2.0, -2.0),
        DVec3::new(1.0, -3.0, -1.0),
        DVec3::ZERO,
        DVec3::ONE,
        DVec3::new(-2.0, 2.0, 2.0),
        DVec3::new(-3.0, 3.0, 3.0),
    ];
    assert_eq!(
        Aabb::from_points(&points[..1]).unwrap(),
        Aabb::from_corners(points[0], points[0])
    );
    assert_eq!(
        Aabb::from_points(&points).unwrap(),
        Aabb::from_corners(DVec3::splat(-3.0), DVec3::splat(3.0))
    );
    assert!(Aabb::from_points(&[DVec3::ZERO, DVec3::splat(f64::NAN)]).is_err());
}

#[test]
fn upstream_plane_classification_and_tangent() {
    let b = Aabb::from_corners(-DVec3::X, DVec3::ZERO);
    assert_eq!(b.intersect_plane(-DVec3::X, 1.0), PlaneIntersection::Inside);
    assert_eq!(
        b.intersect_plane(DVec3::X, -1.0),
        PlaneIntersection::Outside
    );
    let b = Aabb::from_corners(DVec3::ZERO, 2.0 * DVec3::X);
    assert_eq!(
        b.intersect_plane(DVec3::X, -1.0),
        PlaneIntersection::Intersecting
    );
    assert_eq!(
        b.intersect_plane(DVec3::X, -2.0),
        PlaneIntersection::Intersecting
    );
    let offset = Aabb::new(DVec3::ZERO, DVec3::ONE, Some(DVec3::splat(10.0)));
    assert_eq!(
        offset.intersect_plane(DVec3::X, -2.0),
        PlaneIntersection::Inside
    );
}

#[test]
fn box_intersection_separation_and_contact() {
    let b = Aabb::from_corners(DVec3::ZERO, DVec3::ONE);
    for p in [
        DVec3::new(1.0, 0.0, 0.0),
        DVec3::new(1.0, 1.0, 0.0),
        DVec3::ONE,
    ] {
        let other = Aabb::from_corners(p, p + DVec3::ONE);
        assert!(b.intersects(other) && other.intersects(b));
    }
    for axis in [DVec3::X, DVec3::Y, DVec3::Z] {
        assert!(!b.intersects(Aabb::from_corners(axis * 1.01, axis * 1.01 + DVec3::ONE)));
    }
}
