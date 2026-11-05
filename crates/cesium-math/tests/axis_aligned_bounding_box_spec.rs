//! CesiumJS AxisAlignedBoundingBoxSpec and intersection boundary cases.
use cesium_math::{AxisAlignedBoundingBox as Aabb, Intersect, Plane};
use glam::DVec3;

#[test]
fn constructor_sets_expected_default_values() {
    let box_val = Aabb::default();
    assert_eq!(box_val.minimum, DVec3::ZERO);
    assert_eq!(box_val.maximum, DVec3::ZERO);
    assert_eq!(box_val.center, DVec3::ZERO);
}

#[test]
fn constructor_sets_expected_parameter_values() {
    let minimum = DVec3::new(1.0, 2.0, 3.0);
    let maximum = DVec3::new(4.0, 5.0, 6.0);
    let center = DVec3::new(2.5, 3.5, 4.5);
    let box_val = Aabb::new(minimum, maximum, Some(center));
    assert_eq!(box_val.minimum, minimum);
    assert_eq!(box_val.maximum, maximum);
    assert_eq!(box_val.center, center);
}

#[test]
fn constructor_computes_center_if_not_supplied() {
    let minimum = DVec3::new(1.0, 2.0, 3.0);
    let maximum = DVec3::new(4.0, 5.0, 6.0);
    let expected_center = DVec3::new(2.5, 3.5, 4.5);
    let box_val = Aabb::from_corners(minimum, maximum);
    assert_eq!(box_val.minimum, minimum);
    assert_eq!(box_val.maximum, maximum);
    assert_eq!(box_val.center, expected_center);
}

#[test]
fn from_corners_sets_expected_values() {
    let minimum = DVec3::new(0.0, 0.0, 0.0);
    let maximum = DVec3::new(1.0, 1.0, 1.0);
    let expected_center = DVec3::new(0.5, 0.5, 0.5);

    let box_val = Aabb::from_corners(minimum, maximum);
    assert_eq!(box_val.minimum, minimum);
    assert_eq!(box_val.maximum, maximum);
    assert_eq!(box_val.center, expected_center);
}

#[test]
fn from_points_empty_and_multiple() {
    assert_eq!(Aabb::from_points(&[]).unwrap(), Aabb::default());
    let positions = [
        DVec3::new(3.0, -1.0, -3.0),
        DVec3::new(2.0, -2.0, -2.0),
        DVec3::new(1.0, -3.0, -1.0),
        DVec3::ZERO,
        DVec3::new(-1.0, 1.0, 1.0),
        DVec3::new(-2.0, 2.0, 2.0),
        DVec3::new(-3.0, 3.0, 3.0),
    ];
    let expected_min = DVec3::new(-3.0, -3.0, -3.0);
    let expected_max = DVec3::new(3.0, 3.0, 3.0);
    let expected_center = DVec3::ZERO;

    let box_val = Aabb::from_points(&positions).unwrap();
    assert_eq!(box_val.minimum, expected_min);
    assert_eq!(box_val.maximum, expected_max);
    assert_eq!(box_val.center, expected_center);

    // Single position
    let single = Aabb::from_points(&positions[..1]).unwrap();
    assert_eq!(single.minimum, positions[0]);
    assert_eq!(single.maximum, positions[0]);
    assert_eq!(single.center, positions[0]);

    // Reject non-finite points
    assert!(Aabb::from_points(&[DVec3::ZERO, DVec3::splat(f64::NAN)]).is_err());
}

#[test]
fn clone_and_equality() {
    let box_val = Aabb::new(DVec3::X, DVec3::Y, Some(DVec3::Z));
    assert_eq!(box_val.clone(), box_val);
    assert!(box_val.equals(Some(&Aabb::new(DVec3::X, DVec3::Y, Some(DVec3::Z)))));
    assert!(!box_val.equals(Some(&Aabb::new(DVec3::new(2.0, 3.0, 4.0), DVec3::Y, Some(DVec3::Y)))));
    assert!(!box_val.equals(Some(&Aabb::new(DVec3::X, DVec3::new(2.0, 3.0, 4.0), Some(DVec3::Z)))));
    assert!(!box_val.equals(Some(&Aabb::new(DVec3::X, DVec3::Y, Some(DVec3::new(2.0, 3.0, 4.0))))));
    assert!(!box_val.equals(None));
}

#[test]
fn intersect_plane_works_with_box_on_positive_side_of_plane() {
    let box_val = Aabb::from_corners(-DVec3::X, DVec3::ZERO);
    let normal = -DVec3::X;
    let position = DVec3::X;
    let plane = Plane::new(normal, -normal.dot(position)).unwrap();
    assert_eq!(box_val.intersect_plane(&plane), Intersect::INSIDE);
}

#[test]
fn intersect_plane_works_with_box_on_negative_side_of_plane() {
    let box_val = Aabb::from_corners(-DVec3::X, DVec3::ZERO);
    let normal = DVec3::X;
    let position = DVec3::X;
    let plane = Plane::new(normal, -normal.dot(position)).unwrap();
    assert_eq!(box_val.intersect_plane(&plane), Intersect::OUTSIDE);
}

#[test]
fn intersect_plane_works_with_box_intersecting_a_plane() {
    let box_val = Aabb::from_corners(DVec3::ZERO, 2.0 * DVec3::X);
    let normal = DVec3::X;
    let position = DVec3::X;
    let plane = Plane::new(normal, -normal.dot(position)).unwrap();
    assert_eq!(box_val.intersect_plane(&plane), Intersect::INTERSECTING);
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
