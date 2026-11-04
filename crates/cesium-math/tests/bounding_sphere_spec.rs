//! Cases from CesiumJS BoundingSphereSpec (Apache-2.0).
use cesium_math::{Ellipsoid, GeoRectangle, PlaneIntersection, Sphere};
use glam::{DMat3, DMat4, DQuat, DVec3};
use std::f64::consts::{FRAC_PI_2, PI};

fn sphere(center: DVec3, radius: f64) -> Sphere {
    Sphere { center, radius }
}
fn equal(a: Sphere, b: Sphere) {
    assert!(
        a.center.distance(b.center) < 1e-12 && (a.radius - b.radius).abs() < 1e-12,
        "{a:?} vs {b:?}"
    );
}

#[test]
fn upstream_points_and_interleaved_vertices() {
    let center = DVec3::new(10_000_000.0, 0.0, 0.0);
    let points = [
        DVec3::X,
        2.0 * DVec3::X,
        DVec3::ZERO,
        DVec3::X + DVec3::Y,
        DVec3::X - DVec3::Y,
        DVec3::X + DVec3::Z,
        DVec3::X - DVec3::Z,
    ]
    .map(|p| p + center);
    let expected = sphere(center + DVec3::X, 1.0);
    equal(Sphere::from_points(&points).unwrap(), expected);
    equal(Sphere::from_points(&[]).unwrap(), sphere(DVec3::ZERO, 0.0));
    equal(
        Sphere::from_points(&[points[0]]).unwrap(),
        sphere(points[0], 0.0),
    );
    let mut data = Vec::new();
    for p in points {
        data.extend([p.x - center.x, p.y - center.y, p.z - center.z, 1.23, 4.56]);
    }
    equal(Sphere::from_vertices(&data, center, 5).unwrap(), expected);
    // The final vertex may omit trailing non-position attributes.
    data.truncate(data.len() - 2);
    equal(Sphere::from_vertices(&data, center, 5).unwrap(), expected);
    assert!(Sphere::from_vertices(&data, center, 2).is_err());
    assert!(Sphere::from_vertices(&[1.0, 2.0], center, 3).is_err());
}

#[test]
fn encoded_points_and_constructed_sphere_contain_all_input() {
    let high = [10_000_000.0_f32, 0.0, 0.0, 10_000_000.0, 0.0, 0.0];
    let low = [0.125_f32, 0.0, 0.0, 1.875, 0.0, 0.0];
    equal(
        Sphere::from_encoded_vertices(&high, &low).unwrap(),
        sphere(DVec3::new(10_000_001.0, 0.0, 0.0), 0.875),
    );
    assert!(Sphere::from_encoded_vertices(&high, &low[..3]).is_err());
    let points: Vec<_> = (0..1000)
        .map(|i| {
            let t = i as f64;
            DVec3::new(
                1e7 + (t * 0.19).sin() * 200.0,
                (t * 0.31).cos() * 150.0,
                (t * 0.07).sin() * 30.0,
            )
        })
        .collect();
    let s = Sphere::from_points(&points).unwrap();
    for p in points {
        assert!(s.center.distance(p) <= s.radius + 1e-8);
    }
    assert!(Sphere::from_points(&[DVec3::splat(f64::NAN)]).is_err());
}

#[test]
fn upstream_union_expand_and_plane_classification() {
    equal(
        sphere(-DVec3::X, 1.0).union(sphere(DVec3::X, 1.0)),
        sphere(DVec3::ZERO, 2.0),
    );
    equal(
        sphere(DVec3::ZERO, 3.0).union(sphere(DVec3::X, 1.0)),
        sphere(DVec3::ZERO, 3.0),
    );
    equal(
        sphere(DVec3::X, 1.0).union(sphere(DVec3::X, 2.0)),
        sphere(DVec3::X, 2.0),
    );
    equal(
        sphere(-3.0 * DVec3::X, 3.0).union(sphere(DVec3::X, 1.0)),
        sphere(-2.0 * DVec3::X, 4.0),
    );
    equal(
        sphere(-DVec3::X, 1.0).expand(DVec3::X),
        sphere(-DVec3::X, 2.0),
    );
    assert_eq!(
        sphere(DVec3::ZERO, 0.5).intersect_plane(-DVec3::X, 1.0),
        PlaneIntersection::Inside
    );
    assert_eq!(
        sphere(DVec3::ZERO, 0.5).intersect_plane(DVec3::X, -1.0),
        PlaneIntersection::Outside
    );
    assert_eq!(
        sphere(DVec3::X, 0.5).intersect_plane(DVec3::X, -1.0),
        PlaneIntersection::Intersecting
    );
    // Cesium's strict comparisons classify opposite tangencies differently.
    assert_eq!(
        sphere(DVec3::ZERO, 1.0).intersect_plane(DVec3::X, -1.0),
        PlaneIntersection::Intersecting
    );
    assert_eq!(
        sphere(DVec3::ZERO, 1.0).intersect_plane(DVec3::X, 1.0),
        PlaneIntersection::Inside
    );
}

#[test]
fn upstream_bounding_spheres_corners_and_ellipsoid() {
    let one = sphere(DVec3::ZERO, 1.0);
    let two = sphere(3.0 * DVec3::Y, 1.0);
    let three = sphere(4.0 * DVec3::Z, 1.0);
    equal(
        Sphere::from_bounding_spheres(&[]).unwrap(),
        sphere(DVec3::ZERO, 0.0),
    );
    equal(Sphere::from_bounding_spheres(&[one]).unwrap(), one);
    equal(
        Sphere::from_bounding_spheres(&[one, two]).unwrap(),
        one.union(two),
    );
    equal(
        Sphere::from_bounding_spheres(&[one, two, three]).unwrap(),
        sphere(DVec3::new(0.0, 1.5, 2.0), 3.5),
    );
    equal(
        Sphere::from_corner_points(-DVec3::ONE, DVec3::ONE),
        sphere(DVec3::ZERO, 3.0_f64.sqrt()),
    );
    equal(
        Sphere::from_ellipsoid(Ellipsoid::new(DVec3::new(1.0, 2.0, 3.0)).unwrap()),
        sphere(DVec3::ZERO, 3.0),
    );
}

#[test]
fn upstream_rectangle_2d_and_height_projection() {
    let geographic =
        |c: cesium_math::Cartographic| Some(DVec3::new(c.longitude, c.latitude, c.height));
    let result =
        Sphere::from_rectangle_with_heights_2d(Some(GeoRectangle::MAXIMUM), 0.0, 0.0, geographic)
            .unwrap();
    equal(
        result,
        sphere(DVec3::ZERO, (PI * PI + FRAC_PI_2 * FRAC_PI_2).sqrt()),
    );
    let rect = GeoRectangle::radians(-0.5, -0.2, 1.0, 0.4).unwrap();
    let result =
        Sphere::from_rectangle_with_heights_2d(Some(rect), 10.0, 30.0, geographic).unwrap();
    equal(
        result,
        Sphere::from_corner_points(DVec3::new(-0.5, -0.2, 10.0), DVec3::new(1.0, 0.4, 30.0)),
    );
    equal(
        Sphere::from_rectangle_with_heights_2d(None, 0.0, 0.0, geographic).unwrap(),
        sphere(DVec3::ZERO, 0.0),
    );
    assert!(Sphere::from_rectangle_with_heights_2d(Some(rect), 0.0, 0.0, |_| None).is_err());
}

#[test]
fn upstream_rectangle_3d_and_height() {
    let ellipsoid = Ellipsoid::WGS84;
    let globe = Sphere::from_rectangle_3d(Some(GeoRectangle::MAXIMUM), ellipsoid, 0.0).unwrap();
    assert!(globe.center.length() < 1e-8);
    assert!((globe.radius - ellipsoid.maximum_radius()).abs() < 1e-8);
    let rectangle = GeoRectangle::radians(0.1, -0.3, 0.2, -0.4).unwrap();
    let samples = rectangle.subsample(ellipsoid, 100_000.0).unwrap();
    equal(
        Sphere::from_rectangle_3d(Some(rectangle), ellipsoid, 100_000.0).unwrap(),
        Sphere::from_points(&samples).unwrap(),
    );
    equal(
        Sphere::from_rectangle_3d(None, ellipsoid, 0.0).unwrap(),
        sphere(DVec3::ZERO, 0.0),
    );
    assert!(Sphere::from_rectangle_3d(Some(rectangle), ellipsoid, f64::NAN).is_err());
}

#[test]
fn upstream_oriented_box_and_transformation() {
    let rotation = DQuat::from_rotation_z(0.4);
    let axes = DMat3::from_quat(rotation) * DMat3::from_diagonal(DVec3::new(1.0, 2.0, 3.0));
    let center = DVec3::new(1.0, 2.0, 3.0);
    let obb = Sphere::from_oriented_box(center, axes).unwrap();
    equal(obb, sphere(center, 14.0_f64.sqrt()));
    for x in [-1.0, 1.0] {
        for y in [-1.0, 1.0] {
            for z in [-1.0, 1.0] {
                assert!(
                    (center + axes * DVec3::new(x, y, z)).distance(center) <= obb.radius + 1e-14
                );
            }
        }
    }
    let matrix =
        DMat4::from_scale_rotation_translation(DVec3::new(1.0, 2.0, 3.0), rotation, center);
    equal(
        Sphere::from_transformation(matrix).unwrap(),
        sphere(center, 0.5 * 14.0_f64.sqrt()),
    );
    equal(
        Sphere::from_transformation(DMat4::from_scale(DVec3::ZERO)).unwrap(),
        sphere(DVec3::ZERO, 0.0),
    );
    assert!(Sphere::from_transformation(DMat4::from_scale(DVec3::splat(f64::NAN))).is_err());
}

#[test]
fn transformed_cube_sphere_encloses_all_sheared_corners() {
    let transform = DMat4::from_cols(
        DVec3::X.extend(0.0),
        DVec3::new(-1.0, 1.0, 0.0).extend(0.0),
        DVec3::Z.extend(0.0),
        DVec3::new(1.0, 2.0, 3.0).extend(1.0),
    );
    let sphere = Sphere::from_transformation(transform).unwrap();
    for x in [-0.5, 0.5] {
        for y in [-0.5, 0.5] {
            for z in [-0.5, 0.5] {
                let point = transform.transform_point3(DVec3::new(x, y, z));
                assert!(point.distance(sphere.center) <= sphere.radius + 1e-14);
            }
        }
    }
    let upstream_radius = 0.5 * 4.0_f64.sqrt();
    assert!(sphere.radius > upstream_radius);
}
