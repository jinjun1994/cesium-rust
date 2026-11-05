//! Cases adapted from CesiumJS PlaneSpec (Apache-2.0).
use cesium_math::{AxisAlignedBoundingBox, Plane, PlaneIntersection, Sphere};
use glam::{DMat4, DVec3, DVec4};

#[test]
fn constructs_sets_expected_values() {
    let normal = DVec3::X;
    let distance = 1.0;
    let plane = Plane::new(normal, distance).unwrap();
    assert_eq!(plane.normal, normal);
    assert_eq!(plane.distance, distance);
}

#[test]
fn constructs_from_point_and_normal() {
    let normal = DVec3::new(1.0, 2.0, 3.0).normalize();
    let point = DVec3::new(4.0, 5.0, 6.0);
    let plane = Plane::from_point_normal(point, normal).unwrap();
    assert_eq!(plane.normal, normal);
    assert_eq!(plane.distance, -normal.dot(point));
}

#[test]
fn constructs_from_cartesian4() {
    let result = Plane::from_cartesian4(DVec4::X).unwrap();
    assert_eq!(result.normal, DVec3::X);
    assert_eq!(result.distance, 0.0);
}

#[test]
fn gets_distance_to_point() {
    let normal = DVec3::new(1.0, 2.0, 3.0).normalize();
    let plane = Plane::new(normal, 12.34).unwrap();
    let point = DVec3::new(4.0, 5.0, 6.0);
    let expected = normal.dot(point) + plane.distance;
    assert_eq!(plane.point_distance(point), expected);
    assert_eq!(plane.get_point_distance(point), expected);
}

#[test]
fn projects_point_onto_plane() {
    let plane = Plane::new(DVec3::X, 0.0).unwrap();
    let point = DVec3::new(1.0, 1.0, 0.0);
    let result = plane.project_point_onto_plane(point);
    assert_eq!(result, DVec3::new(0.0, 1.0, 0.0));

    let plane_y = Plane::new(DVec3::Y, 0.0).unwrap();
    let result_y = plane_y.project_point_onto_plane(point);
    assert_eq!(result_y, DVec3::new(1.0, 0.0, 0.0));
}

#[test]
fn clones_and_equals() {
    let left = Plane::new(DVec3::X, 0.0).unwrap();
    let mut right = Plane::new(DVec3::Y, 1.0).unwrap();

    assert!(!left.equals(Some(&right)));
    assert!(!Plane::equals_planes(&left, &right));

    right.distance = 0.0;
    assert!(!left.equals(Some(&right)));

    right.normal = DVec3::X;
    assert!(left.equals(Some(&right)));
    assert!(Plane::equals_planes(&left, &right));

    right.distance = 1.0;
    assert!(!left.equals(Some(&right)));
    assert!(!left.equals(None));

    let cloned = left;
    assert_eq!(cloned, left);
}

#[test]
fn transforms_plane_according_to_transform() {
    let normal = DVec3::new(1.0, 2.0, 3.0).normalize();
    let plane = Plane::new(normal, 12.34).unwrap();

    let transform =
        DMat4::from_scale(DVec3::splat(2.0)) * DMat4::from_rotation_y(std::f64::consts::PI);
    let transformed = plane.transform(transform).unwrap();

    assert!((transformed.distance - plane.distance * 2.0).abs() < 1e-12);
    assert!((transformed.normal.x - (-plane.normal.x)).abs() < 1e-10);
    assert_eq!(transformed.normal.y, plane.normal.y);
    assert!((transformed.normal.z - (-plane.normal.z)).abs() < 1e-10);
}

#[test]
fn transforms_plane_according_to_nonuniform_scale_transform() {
    let normal = DVec3::new(1.0, 0.0, 1.0).normalize();
    let plane = Plane::new(normal, 0.0).unwrap();
    let plane_origin = DVec3::ZERO;
    let plane_position = DVec3::new(1.0, 0.0, -1.0);
    let plane_diff = plane_position - plane_origin;
    assert!(plane_diff.dot(plane.normal).abs() < 1e-15);

    let transform = DMat4::from_scale(DVec3::new(4.0, 1.0, 10.0));
    let transform_plane = plane.transform(transform).unwrap();
    let transform_plane_diff = transform.transform_vector3(plane_diff);
    assert!(transform_plane_diff.dot(transform_plane.normal).abs() < 1e-15);
}

#[test]
fn translated_and_singular_transform() {
    let plane = Plane::new(DVec3::X, -1.0).unwrap();
    let transform = DMat4::from_translation(DVec3::new(7.0, 2.0, -3.0));
    let transformed = plane.transform(transform).unwrap();
    assert_eq!(transformed, Plane::new(DVec3::X, -8.0).unwrap());
    assert_eq!(
        transformed.point_distance(transform.transform_point3(DVec3::X)),
        0.0
    );
    assert!(plane
        .transform(DMat4::from_scale(DVec3::new(1.0, 0.0, 1.0)))
        .is_err());
}

#[test]
fn distance_projection_and_intersection_interfaces() {
    let p = Plane::new(DVec3::X, -1.0).unwrap();
    let point = DVec3::new(4.0, 5.0, 6.0);
    assert_eq!(p.point_distance(point), 3.0);
    assert_eq!(p.project_point(point), DVec3::new(1.0, 5.0, 6.0));
    assert_eq!(
        p.intersect_ray(point, -DVec3::X),
        Some(DVec3::new(1.0, 5.0, 6.0))
    );
    assert_eq!(
        p.classify_box(AxisAlignedBoundingBox::from_corners(
            DVec3::ZERO,
            DVec3::splat(0.5)
        )),
        PlaneIntersection::Outside
    );
    assert_eq!(
        p.classify_sphere(Sphere {
            center: DVec3::X,
            radius: 0.5
        }),
        PlaneIntersection::Intersecting
    );
}

#[test]
fn rejects_invalid_inputs() {
    assert!(Plane::new(DVec3::ZERO, 0.0).is_err());
    assert!(Plane::new(DVec3::new(1.0, 2.0, 3.0), 0.0).is_err());
    assert!(Plane::new(DVec3::X, f64::NAN).is_err());
    assert!(Plane::from_point_normal(DVec3::splat(f64::NAN), DVec3::X).is_err());
    assert!(Plane::from_point_normal(DVec3::ZERO, DVec3::ZERO).is_err());
    assert!(Plane::from_cartesian4(DVec4::new(1.0, 2.0, 3.0, 4.0)).is_err());
}
