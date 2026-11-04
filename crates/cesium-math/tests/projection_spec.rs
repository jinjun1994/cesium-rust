//! Reference cases from CesiumJS GeographicProjectionSpec and BoundingSphereSpec.
use cesium_math::{Cartographic, Ellipsoid, GeographicProjection, Sphere, WGS84_A, WGS84_B};
use glam::DVec3;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

#[test]
fn geographic_projection_upstream_roundtrip() {
    let projection = GeographicProjection::default();
    assert_eq!(projection.ellipsoid(), Ellipsoid::WGS84);
    let c = Cartographic::radians(PI, FRAC_PI_2, 0.0).unwrap();
    assert_eq!(
        projection.project(c).unwrap(),
        DVec3::new(PI * WGS84_A, FRAC_PI_2 * WGS84_A, 0.0)
    );
    let unit = GeographicProjection::new(Ellipsoid::UNIT_SPHERE).unwrap();
    let c = Cartographic::radians(-PI, FRAC_PI_2, 0.0).unwrap();
    assert_eq!(unit.project(c).unwrap(), DVec3::new(-PI, FRAC_PI_2, 0.0));
    let c = Cartographic::radians(FRAC_PI_2, FRAC_PI_4, 12.0).unwrap();
    let restored = projection
        .unproject(projection.project(c).unwrap())
        .unwrap();
    assert!(
        (restored.longitude - c.longitude).abs() < 1e-15
            && (restored.latitude - c.latitude).abs() < 1e-15
            && restored.height == 12.0
    );
    assert!(GeographicProjection::new(Ellipsoid::default()).is_err());
    assert!(unit.unproject(DVec3::splat(f64::NAN)).is_none());
}

#[test]
fn project_bounding_sphere_against_upstream_scenario() {
    let center = DVec3::new(10_000_000.0, 0.0, 0.0);
    let offsets = [
        DVec3::X,
        2.0 * DVec3::X,
        DVec3::ZERO,
        DVec3::X + DVec3::Y,
        DVec3::X - DVec3::Y,
        DVec3::X + DVec3::Z,
        DVec3::X - DVec3::Z,
    ];
    let positions = offsets.map(|offset| center + offset);
    let projection = GeographicProjection::default();
    let points: Vec<_> = positions
        .iter()
        .map(|&p| {
            projection
                .project(Ellipsoid::WGS84.cartesian_to_cartographic(p).unwrap())
                .unwrap()
        })
        .collect();
    let actual = Sphere::from_points(&points).unwrap();
    let projected = Sphere::from_points(&positions)
        .unwrap()
        .project_to_2d(Ellipsoid::WGS84, |c| projection.project(c))
        .unwrap();
    let swizzled = DVec3::new(actual.center.z, actual.center.x, actual.center.y);
    assert!(
        projected.center.distance(swizzled) < 1e-6,
        "{:?} vs {:?}",
        projected.center,
        swizzled
    );
    assert!(projected.radius > actual.radius);
}

#[test]
fn project_bounding_sphere_at_poles_and_invalid_inputs() {
    let projection = GeographicProjection::default();
    for z in [-WGS84_B, WGS84_B] {
        let s = Sphere {
            center: DVec3::new(0.0, 0.0, z),
            radius: 1000.0,
        };
        let projected = s
            .project_to_2d(Ellipsoid::WGS84, |c| projection.project(c))
            .unwrap();
        assert!(
            projected.center.is_finite() && projected.radius.is_finite() && projected.radius > 0.0
        );
    }
    let sphere = Sphere {
        center: DVec3::X * WGS84_A,
        radius: 1.0,
    };
    assert!(sphere.project_to_2d(Ellipsoid::WGS84, |_| None).is_err());
    assert!(Sphere {
        radius: -1.0,
        ..sphere
    }
    .project_to_2d(Ellipsoid::WGS84, |c| projection.project(c))
    .is_err());
}
