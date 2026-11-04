//! Scenarios from CesiumJS QuarticRealPolynomialSpec.js (Apache-2.0).
// Keep the exact decimal literals from the upstream numeric stability fixtures.
#![allow(clippy::excessive_precision)]
use cesium_math::{quartic_discriminant, quartic_real_roots};

fn check(coefficients: [f64; 5], expected: &[f64], tolerance: f64) {
    let [a, b, c, d, e] = coefficients;
    let roots = quartic_real_roots(a, b, c, d, e);
    assert_eq!(roots.len(), expected.len(), "{coefficients:?}: {roots:?}");
    for (&root, &want) in roots.iter().zip(expected) {
        assert!(
            (root - want).abs() <= tolerance,
            "{coefficients:?}: {roots:?}, expected {expected:?}"
        );
    }
}

#[test]
fn upstream_roots_and_discriminant() {
    assert_eq!(quartic_discriminant(1.0, 2.0, 3.0, 4.0, 5.0), 10800.0);
    check(
        [2.0, -16.0, 48.0, -64.0, 32.0],
        &[2.0, 2.0, 2.0, 2.0],
        1e-15,
    );
    check([2.0, 0.0, -4.0, 0.0, 2.0], &[-1.0, -1.0, 1.0, 1.0], 1e-15);
    check([2.0, -8.0, 16.0, -16.0, 6.0], &[1.0, 1.0], 1e-14);
    check(
        [2.0, 8.0, -6.0, -20.0, 16.0],
        &[-4.0, -2.0, 1.0, 1.0],
        1e-15,
    );
    check(
        [2.0, 4.0, -26.0, -28.0, 48.0],
        &[-4.0, -2.0, 1.0, 3.0],
        1e-15,
    );
    check([3.0, -8.0, 14.0, -8.0, 3.0], &[], 0.0);
    check([0.0, 2.0, 6.0, -26.0, -30.0], &[-5.0, -1.0, 3.0], 1e-15);
}

#[test]
fn upstream_stability_cases() {
    check(
        [1.0, -27121.309311434146, 0.0, -26760.571078686513, -1.0],
        &[-0.000037368410630733706, 27121.3093478151],
        1e-12,
    );
    check(
        [-1.0, -26959.661445199898, 0.0, -26675.609408851604, 1.0],
        &[-26959.661481901538, 0.000037487427107407711],
        1e-11,
    );
    check(
        [-1.0, 20607.270539372261, 0.0, 20333.159863900513, 1.0],
        &[-0.000049180747737409547, 20607.270587253341],
        1e-11,
    );
}

#[test]
fn distinct_integer_roots() {
    for x in -4..=1 {
        for y in x + 1..=2 {
            for z in y + 1..=3 {
                for w in z + 1..=4 {
                    let (x, y, z, w) = (x as f64, y as f64, z as f64, w as f64);
                    let coefficients = [
                        1.0,
                        -(x + y + z + w),
                        x * y + x * z + x * w + y * z + y * w + z * w,
                        -(x * y * z + x * y * w + x * z * w + y * z * w),
                        x * y * z * w,
                    ];
                    check(coefficients, &[x, y, z, w], 1e-9);
                }
            }
        }
    }
}
