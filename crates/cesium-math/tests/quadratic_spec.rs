//! Cases from CesiumJS `QuadraticRealPolynomialSpec.js` (Apache-2.0).
use cesium_math::{quadratic_discriminant, quadratic_real_roots};

#[test]
fn upstream_discriminant_and_root_cases() {
    assert_eq!(quadratic_discriminant(1.0, 2.0, 3.0), -8.0);
    for (a, b, c, expected) in [
        (2.0, -4.0, -6.0, vec![-1.0, 3.0]),
        (2.0, 4.0, -6.0, vec![-3.0, 1.0]),
        (2.0, -4.0, 6.0, vec![]),
        (0.0, 0.0, -3.0, vec![]),
        (0.0, 2.0, 8.0, vec![-4.0]),
        (3.0, 0.0, 0.0, vec![0.0, 0.0]),
        (3.0, 0.0, 18.0, vec![]),
        (2.0, 0.0, -18.0, vec![-3.0, 3.0]),
        (2.0, 6.0, 0.0, vec![-3.0, 0.0]),
        (2.0, -6.0, 0.0, vec![0.0, 3.0]),
    ] {
        assert_eq!(quadratic_real_roots(a, b, c), expected, "{a}, {b}, {c}");
    }
}

#[test]
fn cancellation_near_repeated_root_and_small_root() {
    let repeated = quadratic_real_roots(2.0, -3.999_999_999_999_999, 2.0);
    assert_eq!(repeated.len(), 2);
    assert!(repeated.iter().all(|x| (*x - 1.0).abs() < 1e-15));
    let roots = quadratic_real_roots(1.0, 1e12, 1.0);
    assert!((roots[0] + 1e12).abs() < 1e-3);
    assert!((roots[1] + 1e-12).abs() < 1e-26);
}

#[test]
fn degenerate_thresholds_and_invalid_inputs() {
    assert_eq!(quadratic_real_roots(1.0, 0.0, 1e-15), [0.0, 0.0]);
    assert!(quadratic_real_roots(1e-15, 0.0, 1.0).is_empty());
    assert_eq!(quadratic_real_roots(1.0, 0.0, -1e-12), [-1e-6, 1e-6]);
    assert!(quadratic_real_roots(f64::NAN, 1.0, 1.0).is_empty());
}
