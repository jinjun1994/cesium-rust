use cesium_math::*;

#[test]
fn test_sign_and_sign_not_zero() {
    assert_eq!(sign(-2.0), -1.0);
    assert_eq!(sign(2.0), 1.0);
    assert_eq!(sign(0.0), 0.0);
    assert!(sign(f64::NAN).is_nan());

    assert_eq!(sign_not_zero(-2.0), -1.0);
    assert_eq!(sign_not_zero(2.0), 1.0);
    assert_eq!(sign_not_zero(0.0), 1.0);
    assert_eq!(sign_not_zero(-0.0), 1.0); // -0.0 < 0.0 is false, yields 1.0 matching upstream
}

#[test]
fn test_snorm_conversions() {
    assert_eq!(to_snorm(-1.0, None), 0.0);
    assert_eq!(to_snorm(1.0, None), 255.0);
    assert_eq!(to_snorm(-1.0001, None), 0.0);
    assert_eq!(to_snorm(1.0001, None), 255.0);
    assert_eq!(to_snorm(0.0, None), 128.0);

    assert_eq!(from_snorm(0.0, None), -1.0);
    assert_eq!(from_snorm(255.0, None), 1.0);
    assert_eq!(from_snorm(-0.0001, None), -1.0);
    assert_eq!(from_snorm(255.00001, None), 1.0);
    assert_eq!(from_snorm(255.0 / 2.0, None), 0.0);
}

#[test]
fn test_clamped_trig_and_chord() {
    assert_eq!(asin_clamped(1.5), std::f64::consts::FRAC_PI_2);
    assert_eq!(asin_clamped(-1.5), -std::f64::consts::FRAC_PI_2);
    assert_eq!(acos_clamped(1.5), 0.0);
    assert_eq!(acos_clamped(-1.5), std::f64::consts::PI);

    let chord = chord_length(std::f64::consts::PI, 1.0);
    assert!((chord - 2.0).abs() < 1e-12);
}

#[test]
fn test_angle_normalization() {
    let pi = std::f64::consts::PI;
    assert!((negative_pi_to_pi(3.0 * pi) - -pi).abs() < 1e-12);
    assert!((negative_pi_to_pi(-3.0 * pi) - -pi).abs() < 1e-12);
    assert!((negative_pi_to_pi(0.5 * pi) - 0.5 * pi).abs() < 1e-12);

    assert!((zero_to_two_pi(3.0 * pi) - pi).abs() < 1e-12);
    assert!((zero_to_two_pi(-pi) - pi).abs() < 1e-12);
    assert!((zero_to_two_pi(0.0) - 0.0).abs() < 1e-12);
}

#[test]
fn test_equals_epsilon_upstream() {
    assert!(equals_epsilon(1.0, 1.0, Some(0.0), None));
    assert!(equals_epsilon(1.0, 1.0, Some(1.0), None));
    assert!(equals_epsilon(1.0, 1.0 + EPSILON7, Some(EPSILON7), None));
    assert!(!equals_epsilon(1.0, 1.0 + EPSILON7, Some(EPSILON9), None));

    // Upstream: relative epsilon scaling with magnitude
    assert!(equals_epsilon(3000000.0, 3000000.0, Some(0.0), None));
    assert!(equals_epsilon(3000000.0, 3000000.2, Some(EPSILON7), None));
    assert!(!equals_epsilon(3000000.0, 3000000.2, Some(EPSILON9), None));
}
