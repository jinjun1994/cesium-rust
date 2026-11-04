//! Cases adapted from CesiumJS OccluderSpec (Apache-2.0).
use cesium_math::{Ellipsoid, GeoRectangle, Occluder, Sphere, Visibility};
use glam::DVec3;

fn sphere(x: f64, y: f64, z: f64, radius: f64) -> Sphere {
    Sphere {
        center: DVec3::new(x, y, z),
        radius,
    }
}

#[test]
fn upstream_eclipse_full_and_partial_visibility() {
    let cases = [
        (
            sphere(0.0, 0.0, -1.5, 0.5),
            sphere(0.0, 0.0, -2.75, 0.25),
            Visibility::None,
        ),
        (
            sphere(0.0, 0.0, -2.75, 0.25),
            sphere(0.0, 0.0, -1.5, 0.5),
            Visibility::Full,
        ),
        (
            sphere(0.0, 0.0, -1.5, 0.5),
            sphere(0.0, 0.0, -2.5, 0.5),
            Visibility::None,
        ),
        (
            sphere(-1.25, 0.0, -1.5, 0.5),
            sphere(1.25, 0.0, -1.5, 0.5),
            Visibility::Full,
        ),
        (
            sphere(0.0, 0.0, -2.0, 1.0),
            sphere(0.5, 0.5, -3.0, 1.0),
            Visibility::Partial,
        ),
        (
            sphere(-0.5, 0.0, -1.0, 1.0),
            sphere(0.5, 0.0, -1.0, 1.0),
            Visibility::Partial,
        ),
        (
            sphere(0.0, 0.0, -2.0, 1.0),
            sphere(0.0, 0.5, -2.5, 1.0),
            Visibility::Partial,
        ),
        (
            sphere(0.0, 0.0, -1.5, 0.5),
            sphere(0.0, 0.0, -3.0, 1.0),
            Visibility::Full,
        ),
    ];
    for (occluder, occludee, expected) in cases {
        let c = Occluder::new(occluder, DVec3::ZERO).unwrap();
        assert_eq!(c.compute_visibility(occludee), expected);
        if matches!(expected, Visibility::None | Visibility::Full) {
            assert_eq!(c.is_sphere_visible(occludee), expected == Visibility::Full);
        }
    }
}

#[test]
fn point_camera_motion_and_inside_cases() {
    let mut c = Occluder::new(sphere(0.0, 0.0, -2.0, 1.0), DVec3::ZERO).unwrap();
    assert!(!c.is_point_visible(DVec3::new(0.0, 0.0, -4.0)));
    assert!(c.is_point_visible(DVec3::new(0.0, 0.0, -0.5)));
    assert!(!c.is_point_visible(DVec3::new(0.0, 0.0, -2.0)));
    c.set_camera_position(DVec3::new(0.0, 0.0, -2.0)).unwrap();
    assert!(!c.is_point_visible(DVec3::ZERO));
    assert!(!c.is_sphere_visible(sphere(0.0, 0.0, 0.0, 0.5)));
    assert_eq!(
        c.compute_visibility(sphere(0.0, 0.0, 0.0, 0.5)),
        Visibility::None
    );
    // Upstream computeVisibility special-cases larger occludees before camera containment.
    assert_eq!(
        c.compute_visibility(sphere(0.0, 0.0, 0.0, 2.0)),
        Visibility::Full
    );
    assert!(!c.is_sphere_visible(sphere(0.0, 0.0, 0.0, 2.0)));
    assert!(c.set_camera_position(DVec3::splat(f64::NAN)).is_err());
    assert_eq!(c.camera_position(), DVec3::new(0.0, 0.0, -2.0));
    assert!(Occluder::new(sphere(0.0, 0.0, 0.0, -1.0), DVec3::ZERO).is_err());
}

#[test]
fn upstream_occludee_point_and_visibility() {
    let occluder = sphere(0.0, 0.0, -8.0, 2.0);
    let points = [
        DVec3::new(-1.085, 0.0, -6.221),
        DVec3::new(1.085, 0.0, -6.221),
    ];
    let center = Sphere::from_points(&points).unwrap().center;
    let result = Occluder::compute_occludee_point(occluder, center, &points)
        .unwrap()
        .unwrap();
    assert!(result.distance(DVec3::new(0.0, 0.0, -5.0)) < 0.1);
    let points = [DVec3::new(-0.25, 0.0, -5.3), DVec3::new(0.25, 0.0, -5.3)];
    let center = Sphere::from_points(&points).unwrap().center;
    let result = Occluder::compute_occludee_point(occluder, center, &points)
        .unwrap()
        .unwrap();
    let point_sphere = Sphere {
        center: result,
        radius: 0.0,
    };
    let inside = Occluder::new(occluder, occluder.center).unwrap();
    assert!(!inside.is_sphere_visible(point_sphere));
    let side = Occluder::new(occluder, DVec3::new(3.0, 0.0, -8.0)).unwrap();
    assert!(side.is_sphere_visible(point_sphere));
}

#[test]
fn occludee_point_axes_degenerate_and_rectangle() {
    let s = sphere(5.0, 0.0, 0.0, 2.0);
    for axis in [DVec3::X, DVec3::Y, DVec3::Z, DVec3::ONE.normalize()] {
        let position = s.center + axis * 3.0;
        let point = Occluder::compute_occludee_point(s, position, &[position])
            .unwrap()
            .unwrap();
        assert!((point - (s.center + axis * 3.0)).length() < 1e-12);
    }
    assert!(Occluder::compute_occludee_point(s, s.center, &[DVec3::X]).is_err());
    assert!(Occluder::compute_occludee_point(s, DVec3::X, &[]).is_err());
    assert_eq!(
        Occluder::compute_occludee_point(s, s.center + DVec3::X, &[s.center + DVec3::X]).unwrap(),
        None
    );
    assert!(Occluder::compute_occludee_point_from_rectangle(
        GeoRectangle::MAXIMUM,
        Ellipsoid::WGS84
    )
    .is_none());
    let edge = std::f64::consts::PI / 32.0;
    let rectangle = GeoRectangle::radians(-edge, -edge, edge, edge).unwrap();
    let positions = rectangle.subsample(Ellipsoid::WGS84, 0.0).unwrap();
    let center = Sphere::from_points(&positions).unwrap().center;
    let expected = Occluder::compute_occludee_point(
        Sphere {
            center: DVec3::ZERO,
            radius: Ellipsoid::WGS84.minimum_radius(),
        },
        center,
        &positions,
    )
    .unwrap();
    assert!(expected.is_some());
    assert_eq!(
        Occluder::compute_occludee_point_from_rectangle(rectangle, Ellipsoid::WGS84),
        expected
    );
}
