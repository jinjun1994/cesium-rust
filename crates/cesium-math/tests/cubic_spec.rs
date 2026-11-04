//! Reference scenarios from CesiumJS CubicRealPolynomialSpec (Apache-2.0).
use cesium_math::{cubic_discriminant, cubic_real_roots};

fn check(coeff: [f64; 4], expected: &[f64], tolerance: f64) {
    let roots = cubic_real_roots(coeff[0], coeff[1], coeff[2], coeff[3]);
    assert_eq!(roots.len(), expected.len(), "{coeff:?}: {roots:?}");
    for (&actual, &want) in roots.iter().zip(expected) {
        assert!((actual - want).abs() <= tolerance, "{coeff:?}: {roots:?}");
    }
}

#[test]
fn upstream_discriminant_and_general_roots() {
    assert_eq!(cubic_discriminant(3.0, 2.0, 1.0, 1.0), -175.0);
    check([2.0, -12.0, 24.0, -16.0], &[2.0, 2.0, 2.0], 1e-15);
    check([2.0, 2.0, -2.0, -2.0], &[-1.0, -1.0, 1.0], 1e-15);
    check([2.0, 6.0, -26.0, -30.0], &[-5.0, -1.0, 3.0], 1e-15);
    check([2.0, -6.0, 10.0, -6.0], &[1.0], 1e-15);
}

#[test]
fn upstream_deflation_and_zero_roots() {
    check([0.0, 2.0, -4.0, -6.0], &[-1.0, 3.0], 0.0);
    check([1.0, 0.0, 1.0, 2.0], &[-1.0], 1e-14);
    check([1.0, 0.0, 0.0, -8.0], &[2.0, 2.0, 2.0], 1e-14);
    check([1.0, 0.0, -1.0, 0.0], &[-1.0, 0.0, 1.0], 0.0);
    check([1.0, 1.0, 0.0, 0.0], &[-1.0, 0.0, 0.0], 0.0);
    check([1.0, -1.0, 0.0, 0.0], &[0.0, 0.0, 1.0], 0.0);
    check([1.0, 1.0, 1.0, 0.0], &[0.0], 0.0);
    // Cesium source's `roots.Length` typo produces undefined roots here;
    // the mathematically valid real root is zero.
    check([1.0, 0.0, 1.0, 0.0], &[0.0], 0.0);
    assert!(cubic_real_roots(f64::NAN, 0.0, 0.0, 1.0).is_empty());
}

#[test]
fn distinct_integer_roots_satisfy_original_polynomial() {
    for first in -5..=3 {
        for second in first + 1..=4 {
            for third in second + 1..=5 {
                let (x, y, z) = (first as f64, second as f64, third as f64);
                let coefficients = [1.0, -(x + y + z), x * y + x * z + y * z, -x * y * z];
                check(coefficients, &[x, y, z], 1e-10);
            }
        }
    }
}
