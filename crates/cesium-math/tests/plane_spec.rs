//! Cases adapted from CesiumJS PlaneSpec (Apache-2.0).
use cesium_math::{AxisAlignedBoundingBox, Plane, PlaneIntersection, Sphere};
use glam::{DMat4, DVec3, DVec4};

#[test]
fn construction_equality_and_invalid_normals() {
    let normal = DVec3::new(1.0, 2.0, 3.0).normalize();
    let point = DVec3::new(4.0, 5.0, 6.0);
    let p = Plane::from_point_normal(point, normal).unwrap();
    assert_eq!(p.distance, -normal.dot(point));
    assert_eq!(
        p,
        Plane::from_coefficients(normal.extend(p.distance)).unwrap()
    );
    assert_ne!(p, Plane::new(normal, 1.0).unwrap());
    assert_eq!(
        Plane::from_coefficients(DVec4::X).unwrap(),
        Plane::ORIGIN_YZ
    );
    assert!(Plane::new(DVec3::ZERO, 0.0).is_err());
    assert!(Plane::new(DVec3::new(1.0, 2.0, 3.0), 0.0).is_err());
    assert!(Plane::new(DVec3::X, f64::NAN).is_err());
    assert!(Plane::from_point_normal(DVec3::splat(f64::NAN), normal).is_err());
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
fn upstream_uniform_and_nonuniform_transform() {
    let normal = DVec3::new(1.0, 2.0, 3.0).normalize();
    let plane = Plane::new(normal, 12.34).unwrap();
    let transform =
        DMat4::from_scale(DVec3::splat(2.0)) * DMat4::from_rotation_y(std::f64::consts::PI);
    let transformed = plane.transform(transform).unwrap();
    assert!((transformed.distance - 24.68).abs() < 1e-12);
    assert!((transformed.normal - DVec3::new(-normal.x, normal.y, -normal.z)).length() < 1e-14);
    let p = Plane::new(DVec3::new(1.0, 0.0, 1.0).normalize(), 0.0).unwrap();
    let scale = DMat4::from_scale(DVec3::new(4.0, 1.0, 10.0));
    let result = p.transform(scale).unwrap();
    assert!(
        result
            .point_distance(scale.transform_point3(DVec3::new(1.0, 0.0, -1.0)))
            .abs()
            < 1e-15
    );
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
