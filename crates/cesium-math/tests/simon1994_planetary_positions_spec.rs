#![allow(clippy::excessive_precision)]

use cesium_math::{
    equals_epsilon, negative_pi_to_pi, CelestialFrameTransforms, JulianDate,
    Simon1994PlanetaryPositions, TimeStandard, EPSILON2, EPSILON3, EPSILON4,
};

fn assert_eps(a: f64, b: f64, eps: f64) {
    assert!(
        equals_epsilon(a, b, Some(eps), Some(eps)),
        "{} != {} (eps: {})",
        a,
        b,
        eps
    );
}

#[test]
fn test_computes_correct_sun_position() {
    let date = JulianDate::new(2451545.0, 0.0, TimeStandard::TAI);
    let sun = Simon1994PlanetaryPositions::compute_sun_position_in_earth_inertial_frame(Some(&date));
    let x = 26500268539.790234;
    let y = -132756447253.27325;
    let z = -57556483362.533806;
    assert_eps(x, sun.x, EPSILON2);
    assert_eps(y, sun.y, EPSILON2);
    assert_eps(z, sun.z, EPSILON2);

    let date = JulianDate::new(2456401.5, 0.0, TimeStandard::TAI);
    let sun = Simon1994PlanetaryPositions::compute_sun_position_in_earth_inertial_frame(Some(&date));
    let x = 131512388940.33589;
    let y = 66661342667.949928;
    let z = 28897975607.905258;
    assert_eps(x, sun.x, EPSILON3);
    assert_eps(y, sun.y, EPSILON3);
    assert_eps(z, sun.z, EPSILON3);

    let date = JulianDate::new(2455998.591667, 0.0, TimeStandard::TAI);
    let sun = Simon1994PlanetaryPositions::compute_sun_position_in_earth_inertial_frame(Some(&date));
    let x = 147109989956.19534;
    let y = -19599996881.217579;
    let z = -8497578102.7696457;
    assert_eps(x, sun.x, EPSILON3);
    assert_eps(y, sun.y, EPSILON3);
    assert_eps(z, sun.z, EPSILON3);
}

#[test]
fn test_computes_correct_moon_position() {
    let date = JulianDate::new(2451545.0, 0.0, TimeStandard::TAI);
    let moon =
        Simon1994PlanetaryPositions::compute_moon_position_in_earth_inertial_frame(Some(&date));
    let x = -291632410.61232185;
    let y = -266522146.36821631;
    let z = -75994518.081043154;
    assert_eps(x, moon.x, EPSILON4);
    assert_eps(y, moon.y, EPSILON4);
    assert_eps(z, moon.z, EPSILON4);

    let date = JulianDate::new(2456401.5, 0.0, TimeStandard::TAI);
    let moon =
        Simon1994PlanetaryPositions::compute_moon_position_in_earth_inertial_frame(Some(&date));
    let x = -223792974.4736526;
    let y = 315772435.34490639;
    let z = 97913011.236112773;
    assert_eps(x, moon.x, EPSILON4);
    assert_eps(y, moon.y, EPSILON4);
    assert_eps(z, moon.z, EPSILON4);

    let date = JulianDate::new(2455998.591667, 0.0, TimeStandard::TAI);
    let moon =
        Simon1994PlanetaryPositions::compute_moon_position_in_earth_inertial_frame(Some(&date));
    let x = -268426117.00202647;
    let y = -220468861.73998192;
    let z = -110670164.58446842;
    assert_eps(x, moon.x, EPSILON4);
    assert_eps(y, moon.y, EPSILON4);
    assert_eps(z, moon.z, EPSILON4);
}

#[test]
fn test_sun_rising_east_setting_west() {
    let mut times_of_day = Vec::new();
    for hour in 1..=24 {
        let iso = if hour < 24 {
            format!("2011-07-06T{:02}:00:00Z", hour)
        } else {
            "2011-07-07T00:00:00Z".to_string()
        };
        let date = JulianDate::from_iso8601(&iso).unwrap();
        times_of_day.push(date);
    }

    let mut angles = Vec::new();
    for time in &times_of_day {
        let transform_matrix =
            CelestialFrameTransforms::compute_icrf_to_central_body_fixed_matrix(time);
        let position =
            Simon1994PlanetaryPositions::compute_sun_position_in_earth_inertial_frame(Some(time));
        let fixed_position = transform_matrix.multiply_by_vector(position);
        let angle = negative_pi_to_pi(fixed_position.y.atan2(fixed_position.x));
        angles.push(angle);
    }

    // Expect a clockwise motion
    for i in 1..24 {
        assert!(
            angles[i] < angles[i - 1],
            "Expected angles[{}] ({}) < angles[{}] ({})",
            i,
            angles[i],
            i - 1,
            angles[i - 1]
        );
    }
}

#[test]
fn test_works_without_a_time() {
    let sun = Simon1994PlanetaryPositions::compute_sun_position_in_earth_inertial_frame(None);
    assert!(sun.x.is_finite());
    assert!(sun.y.is_finite());
    assert!(sun.z.is_finite());

    let moon = Simon1994PlanetaryPositions::compute_moon_position_in_earth_inertial_frame(None);
    assert!(moon.x.is_finite());
    assert!(moon.y.is_finite());
    assert!(moon.z.is_finite());
}
