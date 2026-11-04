use cesium_math::{Iau2000Orientation, JulianDate, TimeStandard};

#[test]
#[allow(clippy::excessive_precision)]
fn test_iau2000_orientation_compute_moon() {
    let date = JulianDate::new(2451545.0, -32.184, TimeStandard::TAI);
    let param = Iau2000Orientation::compute_moon(&date);

    let expected_right_ascension = 4.6575460830237914;
    let expected_declination = 1.1456533675897986;
    let expected_rotation = 0.71899299269222972;
    let expected_rotation_rate = 0.0000026518066425764541;

    assert!((param.right_ascension - expected_right_ascension).abs() <= 1e-13);
    assert!((param.declination - expected_declination).abs() <= 1e-13);
    assert!((param.rotation - expected_rotation).abs() <= 1e-13);
    assert!((param.rotation_rate - expected_rotation_rate).abs() <= 1e-15);
}
