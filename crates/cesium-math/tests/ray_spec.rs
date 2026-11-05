//! Cases adapted from CesiumJS RaySpec and IntervalSpec (Apache-2.0).
use cesium_math::{AxisAlignedBoundingBox, Ellipsoid, Interval, Plane, Ray, Sphere};
use glam::DVec3;

#[test]
fn default_constructor_creates_zero_valued_ray() {
    let ray = Ray::default();
    assert_eq!(ray.origin, DVec3::ZERO);
    assert_eq!(ray.direction, DVec3::ZERO);
}

#[test]
fn constructor_sets_expected_properties() {
    let origin = DVec3::Y;
    let direction = DVec3::X;
    let ray = Ray::new(origin, direction).unwrap();
    assert_eq!(ray.origin, origin);
    assert_eq!(ray.direction, direction);
}

#[test]
fn constructor_normalizes_direction() {
    let origin = DVec3::Y;
    let direction = DVec3::X * 18.0;
    let ray = Ray::new(origin, direction).unwrap();
    assert_eq!(ray.origin, origin);
    assert_eq!(ray.direction, DVec3::X);
}

#[test]
fn clone_and_equality() {
    let direction = DVec3::new(1.0, 2.0, 3.0).normalize();
    let ray = Ray::new(DVec3::X, direction).unwrap();
    let cloned = ray;
    assert_eq!(ray, cloned);
    assert!(ray.equals(Some(&cloned)));
    assert!(!ray.equals(Some(&Ray::new(DVec3::Y, direction).unwrap())));
    assert!(!ray.equals(Some(&Ray::new(DVec3::X, DVec3::Y).unwrap())));
    assert!(!ray.equals(None));
}

#[test]
fn get_point_along_ray_works() {
    let direction = DVec3::new(1.0, 2.0, 3.0).normalize();
    let ray = Ray::new(DVec3::X, direction).unwrap();
    for i in -10..=10 {
        let expected = DVec3::X + direction * (i as f64);
        assert_eq!(ray.point_at(i as f64), expected);
        assert_eq!(ray.get_point(i as f64), expected);
    }
}

#[test]
fn rejects_invalid_inputs() {
    assert!(Ray::new(DVec3::splat(f64::NAN), DVec3::X).is_err());
    assert!(Ray::new(DVec3::ZERO, DVec3::splat(f64::INFINITY)).is_err());
}

#[test]
fn upstream_interval_construction() {
    assert_eq!(Interval::default(), Interval::new(0.0, 0.0));
    let interval = Interval::new(1.0, 2.0);
    assert_eq!((interval.start, interval.stop), (1.0, 2.0));
}

#[test]
fn normalized_ray_connects_existing_intersection_functions() {
    let ray = Ray::new(DVec3::new(3.0, 0.0, 0.0), -DVec3::X * 18.0).unwrap();
    let expected = Some(Interval::new(2.0, 4.0));
    assert_eq!(ray.intersect_ellipsoid(Ellipsoid::UNIT_SPHERE), expected);
    assert_eq!(
        ray.intersect_sphere(Sphere {
            center: DVec3::ZERO,
            radius: 1.0
        }),
        expected
    );
    assert_eq!(
        ray.intersect_box(AxisAlignedBoundingBox::from_corners(
            -DVec3::ONE,
            DVec3::ONE
        )),
        expected
    );
    assert_eq!(ray.intersect_plane(Plane::ORIGIN_YZ), Some(DVec3::ZERO));
    let triangle = [
        DVec3::new(0.0, -1.0, -1.0),
        DVec3::new(0.0, 1.0, -1.0),
        DVec3::new(0.0, 0.0, 1.0),
    ];
    assert_eq!(ray.triangle_parameter(triangle, false), Some(3.0));
    assert_eq!(ray.intersect_triangle(triangle, false), Some(DVec3::ZERO));
    assert_eq!(
        Ray::default().intersect_sphere(Sphere {
            center: DVec3::ZERO,
            radius: 1.0
        }),
        None
    );
}
