//! Grazing altitude cases from CesiumJS IntersectionTestsSpec (Apache-2.0).
#![allow(clippy::excessive_precision)]
use cesium_math::{grazing_altitude_location, WGS84_A, WGS84_B};
use glam::DVec3;

#[test]
fn sphere_reference_cases() {
    let unit = DVec3::ONE;
    assert_eq!(
        grazing_altitude_location(3.0 * DVec3::X, DVec3::X, unit),
        Some(3.0 * DVec3::X)
    );
    assert_eq!(
        grazing_altitude_location(0.5 * DVec3::X, DVec3::Z, unit),
        Some(0.5 * DVec3::X)
    );
    assert!(grazing_altitude_location(DVec3::ZERO, DVec3::Z, unit).is_none());
    for (origin, direction, expected) in [
        (
            DVec3::new(-2.0, -2.0, 0.0),
            DVec3::X,
            DVec3::new(0.0, -2.0, 0.0),
        ),
        (
            DVec3::new(0.0, 2.0, 2.0),
            -DVec3::Y,
            DVec3::new(0.0, 0.0, 2.0),
        ),
    ] {
        let actual = grazing_altitude_location(origin, direction, unit).unwrap();
        assert!(actual.distance(expected) < 1e-15, "{actual:?}");
    }
}

#[test]
fn wgs84_reference_cases() {
    let wgs = DVec3::new(WGS84_A, WGS84_A, WGS84_B);
    for (origin, direction, expected, tolerance) in [
        (
            DVec3::new(6502435.411150063, -6350860.759819263, -7230794.954832983),
            DVec3::new(
                -0.6053473557455881,
                0.002372596412575323,
                0.7959578818493397,
            ),
            DVec3::new(628106.8386515155, -6327836.936616249, 493230.07552381355),
            1e-8,
        ),
        (
            DVec3::new(-6546204.940468501, -10625195.62660887, -6933745.82875373),
            DVec3::new(0.5130076305689283, 0.38589525779680295, 0.766751603185799),
            DVec3::new(-125.9063174739769, -5701095.640722358, 2850156.57342018),
            1e-10,
        ),
    ] {
        let actual = grazing_altitude_location(origin, direction, wgs).unwrap();
        for axis in 0..3 {
            let error = (actual[axis] - expected[axis]).abs();
            assert!(
                error <= tolerance
                    || error <= tolerance * actual[axis].abs().max(expected[axis].abs()),
                "axis {axis}: actual {actual:?} expected {expected:?}, error {error}"
            );
        }
    }
}

#[test]
fn invalid_inputs_and_direction_scaling() {
    let radii = DVec3::ONE;
    let origin = DVec3::new(-2.0, -2.0, 0.0);
    let expected = grazing_altitude_location(origin, DVec3::X, radii).unwrap();
    assert_eq!(
        grazing_altitude_location(origin, 10.0 * DVec3::X, radii),
        Some(expected)
    );
    assert_eq!(grazing_altitude_location(origin, DVec3::ZERO, radii), None);
    assert_eq!(
        grazing_altitude_location(origin, DVec3::X, DVec3::ZERO),
        None
    );
    assert_eq!(
        grazing_altitude_location(DVec3::splat(f64::NAN), DVec3::X, radii),
        None
    );
}

#[test]
fn wgs84_reference_approximate_regression() {
    let origin = DVec3::new(-6546204.940468501, -10625195.62660887, -6933745.82875373);
    let direction = DVec3::new(0.5130076305689283, 0.38589525779680295, 0.766751603185799);
    let expected = DVec3::new(-125.9063174739769, -5701095.640722358, 2850156.57342018);
    let wgs = DVec3::new(WGS84_A, WGS84_A, WGS84_B);
    let actual = grazing_altitude_location(origin, direction, wgs).unwrap();
    assert!(actual.distance(expected) < 0.001, "{actual:?}");
}
