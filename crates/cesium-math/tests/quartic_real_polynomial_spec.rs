//! Tests adapted from CesiumJS QuarticRealPolynomialSpec (Apache-2.0).
#![allow(clippy::excessive_precision)]
use cesium_math::QuarticRealPolynomial;

#[test]
fn discriminant() {
    let a = 1.0;
    let b = 2.0;
    let c = 3.0;
    let d = 4.0;
    let e = 5.0;

    let a2 = a * a;
    let a3 = a2 * a;
    let b2 = b * b;
    let b3 = b2 * b;
    let c2 = c * c;
    let c3 = c2 * c;
    let d2 = d * d;
    let d3 = d2 * d;
    let e2 = e * e;
    let e3 = e2 * e;

    let expected = b2 * c2 * d2
        - 4.0 * b3 * d3
        - 4.0 * a * c3 * d2
        + 18.0 * a * b * c * d3
        - 27.0 * a2 * d2 * d2
        + 256.0 * a3 * e3
        + e * (18.0 * b3 * c * d - 4.0 * b2 * c3 + 16.0 * a * c2 * c2
            - 80.0 * a * b * c2 * d
            - 6.0 * a * b2 * d2
            + 144.0 * a2 * c * d2)
        + e2 * (144.0 * a * b2 * c - 27.0 * b2 * b2 - 128.0 * a2 * c2 - 192.0 * a2 * b * d);

    let actual = QuarticRealPolynomial::compute_discriminant(a, b, c, d, e);
    assert!((actual - expected).abs() < 1e-12);
}

#[test]
fn four_repeated_roots() {
    let roots = QuarticRealPolynomial::compute_real_roots(2.0, -16.0, 48.0, -64.0, 32.0);
    assert_eq!(roots.len(), 4);
    for &r in &roots {
        assert!((r - 2.0).abs() < 1e-14);
    }
}

#[test]
fn two_pairs_of_repeated_roots() {
    let roots = QuarticRealPolynomial::compute_real_roots(2.0, 0.0, -4.0, 0.0, 2.0);
    assert_eq!(roots.len(), 4);
    assert!((roots[0] - (-1.0)).abs() < 1e-14);
    assert!((roots[1] - (-1.0)).abs() < 1e-14);
    assert!((roots[2] - 1.0).abs() < 1e-14);
    assert!((roots[3] - 1.0).abs() < 1e-14);
}

#[test]
fn one_pair_of_repeated_roots() {
    let roots = QuarticRealPolynomial::compute_real_roots(2.0, -8.0, 16.0, -16.0, 6.0);
    assert_eq!(roots.len(), 2);
    assert!((roots[0] - 1.0).abs() < 1e-13);
    assert!((roots[1] - 1.0).abs() < 1e-13);
}

#[test]
fn two_unique_and_one_pair_of_repeated_roots() {
    let roots = QuarticRealPolynomial::compute_real_roots(2.0, 8.0, -6.0, -20.0, 16.0);
    assert_eq!(roots.len(), 4);
    assert!((roots[0] - (-4.0)).abs() < 1e-14);
    assert!((roots[1] - (-2.0)).abs() < 1e-14);
    assert!((roots[2] - 1.0).abs() < 1e-14);
    assert!((roots[3] - 1.0).abs() < 1e-14);
}

#[test]
fn four_unique_roots() {
    let roots = QuarticRealPolynomial::compute_real_roots(2.0, 4.0, -26.0, -28.0, 48.0);
    assert_eq!(roots.len(), 4);
    assert!((roots[0] - (-4.0)).abs() < 1e-14);
    assert!((roots[1] - (-2.0)).abs() < 1e-14);
    assert!((roots[2] - 1.0).abs() < 1e-14);
    assert!((roots[3] - 3.0).abs() < 1e-14);
}

#[test]
fn complex_roots() {
    let roots = QuarticRealPolynomial::compute_real_roots(3.0, -8.0, 14.0, -8.0, 3.0);
    assert_eq!(roots.len(), 0);
}

#[test]
fn cubic_case() {
    let roots = QuarticRealPolynomial::compute_real_roots(0.0, 2.0, 6.0, -26.0, -30.0);
    assert_eq!(roots.len(), 3);
    assert!((roots[0] - (-5.0)).abs() < 1e-14);
    assert!((roots[1] - (-1.0)).abs() < 1e-14);
    assert!((roots[2] - 3.0).abs() < 1e-14);
}

#[test]
fn stability_cases() {
    // Stability 1
    let a1 = 1.0;
    let b1 = -27121.309311434146;
    let c1 = 0.0;
    let d1 = -26760.571078686513;
    let e1 = -1.0;
    let roots1 = QuarticRealPolynomial::compute_real_roots(a1, b1, c1, d1, e1);
    assert_eq!(roots1.len(), 2);
    assert!((roots1[0] - (-0.000037368410630733706)).abs() < 1e-11);
    assert!((roots1[1] - 27121.3093478151).abs() < 1e-10);

    // Stability 2
    let a2 = -1.0;
    let b2 = -26959.661445199898;
    let c2 = 0.0;
    let d2 = -26675.609408851604;
    let e2 = 1.0;
    let roots2 = QuarticRealPolynomial::compute_real_roots(a2, b2, c2, d2, e2);
    assert_eq!(roots2.len(), 2);
    assert!((roots2[0] - (-26959.661481901538)).abs() < 1e-10);
    assert!((roots2[1] - 0.000037487427107407711).abs() < 1e-11);

    // Stability 3
    let a3 = -1.0;
    let b3 = 20607.270539372261;
    let c3 = 0.0;
    let d3 = 20333.159863900513;
    let e3 = 1.0;
    let roots3 = QuarticRealPolynomial::compute_real_roots(a3, b3, c3, d3, e3);
    assert_eq!(roots3.len(), 2);
    assert!((roots3[0] - (-0.000049180747737409547)).abs() < 1e-11);
    assert!((roots3[1] - 20607.270587253341).abs() < 1e-10);
}

#[test]
fn rejects_non_finite_inputs() {
    assert_eq!(QuarticRealPolynomial::compute_real_roots(f64::NAN, 1.0, 2.0, 3.0, 4.0).len(), 0);
    assert_eq!(QuarticRealPolynomial::compute_real_roots(1.0, 2.0, 3.0, f64::INFINITY, 5.0).len(), 0);
}
