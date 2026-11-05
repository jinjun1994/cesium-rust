//! Direct port and verification against CesiumJS `IntersectionTestsSpec.js` (Apache-2.0).
#![allow(clippy::excessive_precision)]

use cesium_math::{
    AxisAlignedBoundingBox, Ellipsoid, IntersectionTests, Plane, Ray, RayInterval, Sphere,
};
use glam::DVec3;

#[test]
fn ray_plane() {
    let plane = Plane::new(DVec3::X, -1.0).unwrap();

    let ray = Ray::new(2.0 * DVec3::X, -DVec3::X).unwrap();
    assert_eq!(
        IntersectionTests::ray_plane(&ray, &plane),
        Some(DVec3::new(1.0, 0.0, 0.0))
    );

    let ray = Ray::new(2.0 * DVec3::X, DVec3::X).unwrap();
    assert_eq!(IntersectionTests::ray_plane(&ray, &plane), None);

    let ray = Ray::new(2.0 * DVec3::X, DVec3::Y).unwrap();
    assert_eq!(IntersectionTests::ray_plane(&ray, &plane), None);
}

#[test]
fn ray_triangle() {
    let p0 = DVec3::new(-1.0, 0.0, 0.0);
    let p1 = DVec3::new(1.0, 0.0, 0.0);
    let p2 = DVec3::new(0.0, 1.0, 0.0);

    // Front face
    let ray = Ray::new(DVec3::Z, -DVec3::Z).unwrap();
    assert_eq!(
        IntersectionTests::ray_triangle(&ray, p0, p1, p2, false),
        Some(DVec3::ZERO)
    );

    // Back face without culling
    let ray = Ray::new(-DVec3::Z, DVec3::Z).unwrap();
    assert_eq!(
        IntersectionTests::ray_triangle(&ray, p0, p1, p2, false),
        Some(DVec3::ZERO)
    );

    // Back face with culling
    assert_eq!(
        IntersectionTests::ray_triangle(&ray, p0, p1, p2, true),
        None
    );

    // Outside 0-1 edge
    let ray = Ray::new(DVec3::new(0.0, -1.0, 1.0), -DVec3::Z).unwrap();
    assert_eq!(
        IntersectionTests::ray_triangle(&ray, p0, p1, p2, false),
        None
    );

    // Outside 1-2 edge
    let ray = Ray::new(DVec3::new(1.0, 1.0, 1.0), -DVec3::Z).unwrap();
    assert_eq!(
        IntersectionTests::ray_triangle(&ray, p0, p1, p2, false),
        None
    );

    // Outside 2-0 edge
    let ray = Ray::new(DVec3::new(-1.0, 1.0, 1.0), -DVec3::Z).unwrap();
    assert_eq!(
        IntersectionTests::ray_triangle(&ray, p0, p1, p2, false),
        None
    );

    // Parallel
    let ray = Ray::new(DVec3::Z, DVec3::X).unwrap();
    assert_eq!(
        IntersectionTests::ray_triangle(&ray, p0, p1, p2, false),
        None
    );

    // Behind ray origin
    let ray = Ray::new(DVec3::Z, DVec3::Z).unwrap();
    assert_eq!(
        IntersectionTests::ray_triangle(&ray, p0, p1, p2, false),
        None
    );
}

#[test]
fn line_segment_triangle() {
    let p0 = DVec3::new(-1.0, 0.0, 0.0);
    let p1 = DVec3::new(1.0, 0.0, 0.0);
    let p2 = DVec3::new(0.0, 1.0, 0.0);

    // Front face
    assert_eq!(
        IntersectionTests::line_segment_triangle(DVec3::Z, -DVec3::Z, p0, p1, p2, false),
        Some(DVec3::ZERO)
    );

    // Back face without culling
    assert_eq!(
        IntersectionTests::line_segment_triangle(-DVec3::Z, DVec3::Z, p0, p1, p2, false),
        Some(DVec3::ZERO)
    );

    // Back face with culling
    assert_eq!(
        IntersectionTests::line_segment_triangle(-DVec3::Z, DVec3::Z, p0, p1, p2, true),
        None
    );

    // Outside edges
    assert_eq!(
        IntersectionTests::line_segment_triangle(
            DVec3::new(0.0, -1.0, 1.0),
            DVec3::new(0.0, -1.0, -1.0),
            p0,
            p1,
            p2,
            false
        ),
        None
    );
    assert_eq!(
        IntersectionTests::line_segment_triangle(
            DVec3::new(1.0, 1.0, 1.0),
            DVec3::new(1.0, 1.0, -1.0),
            p0,
            p1,
            p2,
            false
        ),
        None
    );
    assert_eq!(
        IntersectionTests::line_segment_triangle(
            DVec3::new(-1.0, 1.0, 1.0),
            DVec3::new(-1.0, 1.0, -1.0),
            p0,
            p1,
            p2,
            false
        ),
        None
    );

    // Parallel
    assert_eq!(
        IntersectionTests::line_segment_triangle(DVec3::Z, DVec3::X + DVec3::Z, p0, p1, p2, false),
        None
    );

    // Behind v0
    assert_eq!(
        IntersectionTests::line_segment_triangle(
            2.0 * DVec3::Z,
            DVec3::Z,
            p0,
            p1,
            p2,
            false
        ),
        None
    );

    // Behind v1
    assert_eq!(
        IntersectionTests::line_segment_triangle(
            DVec3::Z,
            0.5 * DVec3::Z,
            p0,
            p1,
            p2,
            false
        ),
        None
    );
}

#[test]
fn ray_sphere() {
    let unit_sphere = Sphere::new(DVec3::ZERO, 1.0);

    for (pos, dir) in [
        (DVec3::new(2.0, 0.0, 0.0), DVec3::new(-1.0, 0.0, 0.0)),
        (DVec3::new(0.0, 2.0, 0.0), DVec3::new(0.0, -1.0, 0.0)),
        (DVec3::new(0.0, 0.0, 2.0), DVec3::new(0.0, 0.0, -1.0)),
        (DVec3::new(-2.0, 0.0, 0.0), DVec3::new(1.0, 0.0, 0.0)),
        (DVec3::new(0.0, -2.0, 0.0), DVec3::new(0.0, 1.0, 0.0)),
        (DVec3::new(0.0, 0.0, -2.0), DVec3::new(0.0, 0.0, 1.0)),
    ] {
        let ray = Ray::new(pos, dir).unwrap();
        let hit = IntersectionTests::ray_sphere(&ray, &unit_sphere).unwrap();
        assert!((hit.start - 1.0).abs() < 1e-14);
        assert!((hit.stop - 3.0).abs() < 1e-14);
    }

    // Inside pointing in
    let ray = Ray::new(0.5 * DVec3::X, -DVec3::X).unwrap();
    let hit = IntersectionTests::ray_sphere(&ray, &unit_sphere).unwrap();
    assert_eq!(hit.start, 0.0);
    assert!((hit.stop - 1.5).abs() < 1e-14);

    // Inside pointing out
    let ray = Ray::new(0.5 * DVec3::X, DVec3::X).unwrap();
    let hit = IntersectionTests::ray_sphere(&ray, &unit_sphere).unwrap();
    assert_eq!(hit.start, 0.0);
    assert!((hit.stop - 0.5).abs() < 1e-14);

    // Tangent
    let ray = Ray::new(DVec3::new(1.0, 1.0, 0.0), -DVec3::X).unwrap();
    let hit = IntersectionTests::ray_sphere(&ray, &unit_sphere).unwrap();
    assert!((hit.start - 1.0).abs() < 1e-14);
    assert!((hit.stop - 1.0).abs() < 1e-14);

    // Miss
    let ray = Ray::new(DVec3::new(2.0, 0.0, 0.0), DVec3::Z).unwrap();
    assert_eq!(IntersectionTests::ray_sphere(&ray, &unit_sphere), None);

    // Non-origin center
    let offset_sphere = Sphere::new(DVec3::new(5.0, 0.0, 0.0), 1.0);
    let ray = Ray::new(DVec3::ZERO, DVec3::X).unwrap();
    let hit = IntersectionTests::ray_sphere(&ray, &offset_sphere).unwrap();
    assert!((hit.start - 4.0).abs() < 1e-14);
    assert!((hit.stop - 6.0).abs() < 1e-14);
}

#[test]
fn ray_ellipsoid() {
    let unit_sphere = Ellipsoid::UNIT_SPHERE;

    for (pos, dir) in [
        (DVec3::new(2.0, 0.0, 0.0), DVec3::new(-1.0, 0.0, 0.0)),
        (DVec3::new(0.0, 2.0, 0.0), DVec3::new(0.0, -1.0, 0.0)),
        (DVec3::new(0.0, 0.0, 2.0), DVec3::new(0.0, 0.0, -1.0)),
        (DVec3::new(-2.0, 0.0, 0.0), DVec3::new(1.0, 0.0, 0.0)),
        (DVec3::new(0.0, -2.0, 0.0), DVec3::new(0.0, 1.0, 0.0)),
        (DVec3::new(0.0, 0.0, -2.0), DVec3::new(0.0, 0.0, 1.0)),
    ] {
        let ray = Ray::new(pos, dir).unwrap();
        let hit = IntersectionTests::ray_ellipsoid(&ray, &unit_sphere).unwrap();
        assert!((hit.start - 1.0).abs() < 1e-14);
        assert!((hit.stop - 3.0).abs() < 1e-14);
    }

    // Inside pointing in
    let ellipsoid = Ellipsoid::WGS84;
    let origin = DVec3::new(20000.0, 0.0, 0.0);
    let direction = -origin.normalize();
    let ray = Ray::new(origin, direction).unwrap();
    let hit = IntersectionTests::ray_ellipsoid(&ray, &ellipsoid).unwrap();
    assert_eq!(hit.start, 0.0);
    assert!((hit.stop - (ellipsoid.radii().x + origin.x)).abs() < 1e-8);

    // Inside pointing out
    let direction = origin.normalize();
    let ray = Ray::new(origin, direction).unwrap();
    let hit = IntersectionTests::ray_ellipsoid(&ray, &ellipsoid).unwrap();
    assert_eq!(hit.start, 0.0);
    assert!((hit.stop - (ellipsoid.radii().x - origin.x)).abs() < 1e-8);

    // Tangent
    let ray = Ray::new(DVec3::X, DVec3::Z).unwrap();
    assert_eq!(IntersectionTests::ray_ellipsoid(&ray, &unit_sphere), None);

    // Miss
    let ray = Ray::new(DVec3::new(2.0, 0.0, 0.0), DVec3::Z).unwrap();
    assert_eq!(IntersectionTests::ray_ellipsoid(&ray, &unit_sphere), None);
}

#[test]
fn grazing_altitude_location() {
    let unit_sphere = Ellipsoid::UNIT_SPHERE;

    // Origin of ray
    let ray = Ray::new(DVec3::new(3.0, 0.0, 0.0), DVec3::X).unwrap();
    assert_eq!(
        IntersectionTests::grazing_altitude_location(&ray, &unit_sphere),
        Some(ray.origin)
    );

    // Outside ellipsoid
    let ray = Ray::new(DVec3::new(-2.0, -2.0, 0.0), DVec3::X).unwrap();
    let expected = DVec3::new(0.0, -2.0, 0.0);
    let actual = IntersectionTests::grazing_altitude_location(&ray, &unit_sphere).unwrap();
    assert!(actual.distance(expected) < 1e-14);

    let ray = Ray::new(DVec3::new(0.0, 2.0, 2.0), -DVec3::Y).unwrap();
    let expected = DVec3::new(0.0, 0.0, 2.0);
    let actual = IntersectionTests::grazing_altitude_location(&ray, &unit_sphere).unwrap();
    assert!(actual.distance(expected) < 1e-14);

    // Outside WGS84
    let ellipsoid = Ellipsoid::WGS84;
    let origin = DVec3::new(6502435.411150063, -6350860.759819263, -7230794.954832983);
    let direction = DVec3::new(-0.6053473557455881, 0.002372596412575323, 0.7959578818493397);
    let ray = Ray::new(origin, direction).unwrap();
    let expected = DVec3::new(628106.8386515155, -6327836.936616249, 493230.07552381355);
    let actual = IntersectionTests::grazing_altitude_location(&ray, &ellipsoid).unwrap();
    assert!(actual.distance(expected) < 1e-7);

    // Inside ellipsoid
    let ray = Ray::new(DVec3::new(0.5, 0.0, 0.0), DVec3::Z).unwrap();
    assert_eq!(
        IntersectionTests::grazing_altitude_location(&ray, &unit_sphere),
        Some(ray.origin)
    );

    // Undefined at center
    let ray = Ray::new(DVec3::ZERO, DVec3::Z).unwrap();
    assert_eq!(
        IntersectionTests::grazing_altitude_location(&ray, &unit_sphere),
        None
    );
}

#[test]
fn line_segment_plane() {
    let plane = Plane::new(DVec3::Y, -2.0).unwrap();

    let p = DVec3::new(1.0, 1.0, 0.0);
    let q = DVec3::new(1.0, 3.0, 0.0);
    assert_eq!(
        IntersectionTests::line_segment_plane(p, q, &plane),
        Some(DVec3::new(1.0, 2.0, 0.0))
    );

    // Entire segment behind plane
    let p0 = -2.0 * DVec3::X;
    let p1 = -5.0 * DVec3::X;
    let x_plane = Plane::new(DVec3::X, 0.0).unwrap();
    assert_eq!(IntersectionTests::line_segment_plane(p0, p1, &x_plane), None);

    // Entire segment in front of plane
    let p0 = 5.0 * DVec3::X;
    let p1 = 2.0 * DVec3::X;
    assert_eq!(IntersectionTests::line_segment_plane(p0, p1, &x_plane), None);

    // Parallel
    let p0 = -DVec3::Y;
    let p1 = DVec3::Y;
    assert_eq!(IntersectionTests::line_segment_plane(p0, p1, &x_plane), None);
}

#[test]
fn triangle_plane_intersection() {
    let plane = Plane::new(DVec3::Z, 0.0).unwrap();

    // Triangle in front
    let p0 = DVec3::new(0.0, 0.0, 1.0);
    let p1 = DVec3::new(1.0, 0.0, 1.0);
    let p2 = DVec3::new(0.0, 1.0, 1.0);
    assert_eq!(
        IntersectionTests::triangle_plane_intersection(p0, p1, p2, &plane),
        None
    );

    // Triangle behind
    let p0 = DVec3::new(0.0, 0.0, -1.0);
    let p1 = DVec3::new(1.0, 0.0, -1.0);
    let p2 = DVec3::new(0.0, 1.0, -1.0);
    assert_eq!(
        IntersectionTests::triangle_plane_intersection(p0, p1, p2, &plane),
        None
    );

    // Triangle with p0 behind
    let p0 = DVec3::new(0.0, 0.0, -1.0);
    let p1 = DVec3::new(1.0, 0.0, 1.0);
    let p2 = DVec3::new(0.0, 1.0, 1.0);
    let split = IntersectionTests::triangle_plane_intersection(p0, p1, p2, &plane).unwrap();
    assert_eq!(split.positions.len(), 5);
    assert_eq!(split.indices.len(), 9);

    // Triangle with p0 in front
    let p0 = DVec3::new(0.0, 0.0, 1.0);
    let p1 = DVec3::new(1.0, 0.0, -1.0);
    let p2 = DVec3::new(0.0, 1.0, -1.0);
    let split = IntersectionTests::triangle_plane_intersection(p0, p1, p2, &plane).unwrap();
    assert_eq!(split.positions.len(), 5);
    assert_eq!(split.indices.len(), 9);
}

#[test]
fn ray_axis_aligned_box_and_line_segment_sphere() {
    let aabb = AxisAlignedBoundingBox::new(DVec3::splat(-1.0), DVec3::ONE, None);

    let ray = Ray::new(2.0 * DVec3::X, -DVec3::X).unwrap();
    let hit = IntersectionTests::ray_axis_aligned_bounding_box(&ray, &aabb).unwrap();
    assert_eq!(hit, RayInterval { start: 1.0, stop: 3.0 });

    let sphere = Sphere::new(DVec3::ZERO, 1.0);
    let hit = IntersectionTests::line_segment_sphere(-2.0 * DVec3::X, 2.0 * DVec3::X, &sphere).unwrap();
    assert_eq!(hit, RayInterval { start: 1.0, stop: 3.0 });

    let miss = IntersectionTests::line_segment_sphere(2.0 * DVec3::X, 3.0 * DVec3::X, &sphere);
    assert_eq!(miss, None);
}
