use cesium_math::CubicRealPolynomial;

#[test]
fn test_cubic_real_polynomial_discriminant() {
    let a = 3.0;
    let b = 2.0;
    let c = 1.0;
    let d = 1.0;
    let expected = b * b * c * c - 4.0 * a * c * c * c - 4.0 * b * b * b * d
        - 27.0 * a * a * d * d
        + 18.0 * a * b * c * d;
    let actual = CubicRealPolynomial::compute_discriminant(a, b, c, d);
    assert!((actual - expected).abs() < 1e-14);
}

#[test]
fn test_cubic_real_polynomial_roots_cases() {
    // Three repeated roots
    let r1 = CubicRealPolynomial::compute_real_roots(2.0, -12.0, 24.0, -16.0);
    assert_eq!(r1.len(), 3);
    assert!((r1[0] - 2.0).abs() < 1e-14);
    assert!((r1[1] - 2.0).abs() < 1e-14);
    assert!((r1[2] - 2.0).abs() < 1e-14);

    // One unique and two repeated roots
    let r2 = CubicRealPolynomial::compute_real_roots(2.0, 2.0, -2.0, -2.0);
    assert_eq!(r2.len(), 3);
    assert!((r2[0] - -1.0).abs() < 1e-14);
    assert!((r2[1] - -1.0).abs() < 1e-14);
    assert!((r2[2] - 1.0).abs() < 1e-14);

    // Three unique roots
    let r3 = CubicRealPolynomial::compute_real_roots(2.0, 6.0, -26.0, -30.0);
    assert_eq!(r3.len(), 3);
    assert!((r3[0] - -5.0).abs() < 1e-14);
    assert!((r3[1] - -1.0).abs() < 1e-14);
    assert!((r3[2] - 3.0).abs() < 1e-14);

    // Complex roots
    let r4 = CubicRealPolynomial::compute_real_roots(2.0, -6.0, 10.0, -6.0);
    assert_eq!(r4.len(), 1);
    assert!((r4[0] - 1.0).abs() < 1e-14);

    // Quadratic case (leading coeff is 0)
    let r5 = CubicRealPolynomial::compute_real_roots(0.0, 2.0, -4.0, -6.0);
    assert_eq!(r5.len(), 2);
    assert_eq!(r5[0], -1.0);
    assert_eq!(r5[1], 3.0);

    // Deflated cases
    let d1 = CubicRealPolynomial::compute_real_roots(1.0, 0.0, 1.0, 2.0);
    assert_eq!(d1.len(), 1);
    assert!((d1[0] - -1.0).abs() < 1e-14);

    let d2 = CubicRealPolynomial::compute_real_roots(1.0, 0.0, -1.0, 0.0);
    assert_eq!(d2, vec![-1.0, 0.0, 1.0]);

    let d3 = CubicRealPolynomial::compute_real_roots(1.0, 1.0, 0.0, 0.0);
    assert_eq!(d3, vec![-1.0, 0.0, 0.0]);

    let d4 = CubicRealPolynomial::compute_real_roots(1.0, -1.0, 0.0, 0.0);
    assert_eq!(d4, vec![0.0, 0.0, 1.0]);

    let d5 = CubicRealPolynomial::compute_real_roots(1.0, 1.0, 1.0, 0.0);
    assert_eq!(d5, vec![0.0]);
}
