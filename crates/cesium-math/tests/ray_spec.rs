//! Cases adapted from CesiumJS RaySpec and IntervalSpec (Apache-2.0).
use cesium_math::{AxisAlignedBoundingBox, Ellipsoid, Interval, Plane, Ray, Sphere};
use glam::DVec3;

#[test]
fn upstream_construction_and_points() {
    assert_eq!(Ray::default(), Ray::new(DVec3::ZERO, DVec3::ZERO).unwrap());
    let ray = Ray::new(DVec3::Y, DVec3::X * 18.0).unwrap();
    assert_eq!((ray.origin, ray.direction), (DVec3::Y, DVec3::X));
    let ray = Ray::new(DVec3::X, DVec3::new(1.0, 2.0, 3.0)).unwrap();
    for t in -10..=10 {
        assert_eq!(ray.point_at(t as f64), DVec3::X + ray.direction * t as f64);
    }
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
