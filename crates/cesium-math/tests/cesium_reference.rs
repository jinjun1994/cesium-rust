//! Behavioral checks adapted from CesiumJS core Specs, pinned at
//! b8d3a36fe98a3e432eb89253c95d5f20e605e0f1 (Apache-2.0).
use cesium_math::{
    ray_ellipsoid, ray_ellipsoid_interval, Cartographic, RayInterval, Sphere, WGS84_A, WGS84_B,
};
use glam::{DMat4, DVec3};

// CartographicSpec: independently specified WGS84 Cartesian and angular values.
#[test]
fn cartographic_upstream_surface_reference_both_directions() {
    let reference = DVec3::new(
        4_094_327.792_146_529_5,
        1_909_216.404_474_774,
        4_487_348.408_865_919,
    );
    let input = Cartographic::degrees(25.0, 45.0, 0.0).unwrap();
    assert!(input.to_ecef().distance(reference) < 1e-8);
    let result = Cartographic::from_ecef(reference).unwrap();
    assert!((result.longitude - 25_f64.to_radians()).abs() < 1e-8);
    assert!((result.latitude - 45_f64.to_radians()).abs() < 1e-8);
    assert!(result.height.abs() < 1e-8);
}

// CartographicSpec: above and below the ellipsoid surface.
#[test]
fn cartographic_upstream_positive_and_negative_heights() {
    for height in [3000.0, -3000.0] {
        let input = Cartographic::degrees(35.766989, 33.333602, height).unwrap();
        let result = Cartographic::from_ecef(input.to_ecef()).unwrap();
        assert!((result.longitude - input.longitude).abs() < 1e-8);
        assert!((result.latitude - input.latitude).abs() < 1e-8);
        assert!((result.height - height).abs() < 1e-8);
    }
}

// IntersectionTestsSpec: axis-aligned outside rays, scaled to WGS84.
// Rust returns the first nonnegative surface hit rather than the full interval.
#[test]
fn ray_ellipsoid_axes_hit_and_point_away() {
    for (axis, radius) in [
        (DVec3::X, WGS84_A),
        (DVec3::Y, WGS84_A),
        (DVec3::Z, WGS84_B),
    ] {
        for sign in [-1.0, 1.0] {
            let origin = axis * (sign * 2.0 * radius);
            let inward = axis * -sign;
            assert!((ray_ellipsoid(origin, inward).unwrap() - radius).abs() < 1e-8);
            assert!(ray_ellipsoid(origin, -inward).is_none());
        }
    }
}

// IntersectionTestsSpec: rays starting inside the WGS84 ellipsoid.
#[test]
fn ray_ellipsoid_inside_returns_exit_distance() {
    let origin = DVec3::X * 20000.0;
    assert!((ray_ellipsoid(origin, DVec3::X).unwrap() - (WGS84_A - 20000.0)).abs() < 1e-8);
    assert!((ray_ellipsoid(origin, -DVec3::X).unwrap() - (WGS84_A + 20000.0)).abs() < 1e-8);
}

// BoundingSphereSpec: translation and nonuniform axis scaling.
#[test]
fn bounding_sphere_translation_and_scale() {
    let original = Sphere {
        center: DVec3::ZERO,
        radius: 1.0,
    };
    let moved = original.transformed(DMat4::from_translation(DVec3::new(1.0, 2.0, 3.0)));
    assert_eq!(moved.center, DVec3::new(1.0, 2.0, 3.0));
    assert_eq!(moved.radius, 1.0);
    let scaled = original.transformed(DMat4::from_scale(DVec3::new(1.0, 2.0, 3.0)));
    assert_eq!(scaled.center, DVec3::ZERO);
    assert_eq!(scaled.radius, 3.0);
}

#[test]
fn bounding_sphere_upstream_point_and_plane_distances() {
    let sphere = Sphere {
        center: DVec3::ZERO,
        radius: 1.0,
    };
    let outside = DVec3::new(-2.0, 1.0, 0.0);
    assert!((sphere.distance_squared_to(outside) - 1.52786405).abs() < 1e-6);
    assert_eq!(sphere.distance_squared_to(DVec3::new(-0.5, 0.5, 0.0)), 0.0);
    assert_eq!(
        sphere.compute_plane_distances(outside, DVec3::X),
        (1.0, 3.0)
    );
}

// IntersectionTestsSpec: all six axis hits, misses, and tangencies on a unit sphere.
#[test]
fn ray_interval_upstream_outside_cases() {
    let sphere = DVec3::ONE;
    for axis in [DVec3::X, DVec3::Y, DVec3::Z] {
        for sign in [-1.0, 1.0] {
            let origin = axis * (2.0 * sign);
            let direction = axis * -sign;
            assert_eq!(
                ray_ellipsoid_interval(origin, direction, sphere),
                Some(RayInterval {
                    start: 1.0,
                    stop: 3.0
                })
            );
            assert_eq!(ray_ellipsoid_interval(origin, -direction, sphere), None);
        }
    }
    for (origin, direction) in [
        (DVec3::new(1.0, 1.0, 0.0), -DVec3::X),
        (DVec3::new(-1.0, -1.0, 0.0), DVec3::X),
    ] {
        assert_eq!(
            ray_ellipsoid_interval(origin, direction, sphere),
            Some(RayInterval {
                start: 1.0,
                stop: 1.0
            })
        );
    }
    assert_eq!(
        ray_ellipsoid_interval(2.0 * DVec3::X, DVec3::Z, sphere),
        None
    );
    assert_eq!(ray_ellipsoid_interval(DVec3::X, DVec3::Z, sphere), None);
}

#[test]
fn ray_interval_upstream_inside_and_surface_cases() {
    let radii = DVec3::new(WGS84_A, WGS84_A, WGS84_B);
    let origin = 20_000.0 * DVec3::X;
    for (direction, stop) in [
        (DVec3::X, WGS84_A - 20_000.0),
        (-DVec3::X, WGS84_A + 20_000.0),
    ] {
        let hit = ray_ellipsoid_interval(origin, direction, radii).unwrap();
        assert_eq!(hit.start, 0.0);
        assert!((hit.stop - stop).abs() < 1e-8);
    }
    // Preserve Cesium's explicit on-surface branch, including its stop value.
    assert_eq!(
        ray_ellipsoid_interval(DVec3::X, -DVec3::X, DVec3::ONE),
        Some(RayInterval {
            start: 0.0,
            stop: 1.0
        })
    );
    assert_eq!(ray_ellipsoid_interval(DVec3::X, DVec3::X, DVec3::ONE), None);
    assert_eq!(ray_ellipsoid_interval(DVec3::X, DVec3::Y, DVec3::ONE), None);
}

#[test]
fn ray_interval_supports_triaxial_radii_and_non_unit_directions() {
    let radii = DVec3::new(2.0, 3.0, 4.0);
    let hit = ray_ellipsoid_interval(DVec3::new(6.0, 0.0, 0.0), -2.0 * DVec3::X, radii).unwrap();
    assert_eq!(
        hit,
        RayInterval {
            start: 2.0,
            stop: 4.0
        }
    );
    for invalid in [
        DVec3::ZERO,
        DVec3::new(-1.0, 1.0, 1.0),
        DVec3::splat(f64::NAN),
    ] {
        assert!(ray_ellipsoid_interval(DVec3::X, DVec3::X, invalid).is_none());
    }
    assert!(ray_ellipsoid_interval(DVec3::ZERO, DVec3::ZERO, radii).is_none());
    assert!(ray_ellipsoid_interval(DVec3::splat(f64::INFINITY), DVec3::X, radii).is_none());
}
