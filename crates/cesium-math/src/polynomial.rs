//! Real polynomial operations adapted from CesiumJS (Apache-2.0).

/// Discriminant of `a*x² + b*x + c`.
pub fn quadratic_discriminant(a: f64, b: f64, c: f64) -> f64 {
    b * b - 4.0 * a * c
}

fn add_with_cancellation_check(left: f64, right: f64, tolerance: f64) -> f64 {
    let sum = left + right;
    if left.signum() != right.signum() && (sum / left.abs().max(right.abs())).abs() < tolerance {
        0.0
    } else {
        sum
    }
}

/// Real roots, retaining repeated roots and Cesium's degenerate-case behavior.
/// Non-finite inputs are rejected with an empty result.
pub fn quadratic_real_roots(a: f64, b: f64, c: f64) -> Vec<f64> {
    if !a.is_finite() || !b.is_finite() || !c.is_finite() {
        return vec![];
    }
    if a == 0.0 {
        return if b == 0.0 { vec![] } else { vec![-c / b] };
    }
    if b == 0.0 {
        if c == 0.0 {
            return vec![0.0, 0.0];
        }
        let am = a.abs();
        let cm = c.abs();
        if cm < am && cm / am < 1e-14 {
            return vec![0.0, 0.0];
        }
        if cm > am && am / cm < 1e-14 {
            return vec![];
        }
        let ratio = -c / a;
        return if ratio < 0.0 {
            vec![]
        } else {
            let root = ratio.sqrt();
            vec![-root, root]
        };
    }
    if c == 0.0 {
        let ratio = -b / a;
        return if ratio < 0.0 {
            vec![ratio, 0.0]
        } else {
            vec![0.0, ratio]
        };
    }
    let radicand = add_with_cancellation_check(b * b, -4.0 * a * c, 1e-14);
    if radicand < 0.0 {
        return vec![];
    }
    let q = -0.5 * add_with_cancellation_check(b, b.signum() * radicand.sqrt(), 1e-14);
    if b > 0.0 {
        vec![q / a, c / q]
    } else {
        vec![c / q, q / a]
    }
}

/// Discriminant of `a*x³ + b*x² + c*x + d`.
pub fn cubic_discriminant(a: f64, b: f64, c: f64, d: f64) -> f64 {
    let (a2, b2, c2, d2) = (a * a, b * b, c * c, d * d);
    18.0 * a * b * c * d + b2 * c2 - 27.0 * a2 * d2 - 4.0 * (a * c2 * c + b2 * b * d)
}

/// Real roots following Cesium's conventions, including its three identical
/// outputs for the monomial case `a*x³ + d` even when the root is simple.
/// Non-finite inputs are rejected with an empty result.
pub fn cubic_real_roots(a: f64, b: f64, c: f64, d: f64) -> Vec<f64> {
    if ![a, b, c, d].iter().all(|x| x.is_finite()) {
        return vec![];
    }
    if a == 0.0 {
        return quadratic_real_roots(b, c, d);
    }
    if b == 0.0 && c == 0.0 {
        let root = (-d / a).cbrt();
        return vec![root, root, root];
    }
    if d == 0.0 {
        let mut roots = quadratic_real_roots(a, b, c);
        roots.push(0.0);
        roots.sort_by(f64::total_cmp);
        return roots;
    }

    // Use both direct and reciprocal forms to reduce cancellation, matching
    // Cesium's direct and reciprocal formulation.
    let (b, c) = (b / 3.0, c / 3.0);
    let (ac, bd, b2, c2) = (a * c, b * d, b * b, c * c);
    let (delta1, delta2, delta3) = (ac - b2, a * d - b * c, bd - c2);
    let discriminant = 4.0 * delta1 * delta3 - delta2 * delta2;
    let direct = b2 * bd >= ac * c2;
    if discriminant < 0.0 {
        let (abar, cbar, dbar) = if direct {
            (a, delta1, -2.0 * b * delta1 + a * delta2)
        } else {
            (d, delta3, -d * delta2 + 2.0 * c * delta3)
        };
        let sign = if dbar < 0.0 { -1.0 } else { 1.0 };
        let t0 = -sign * abar.abs() * (-discriminant).sqrt();
        let t1 = -dbar + t0;
        let p = (t1 / 2.0).cbrt();
        let q = if t1 == t0 { -p } else { -cbar / p };
        let t = if cbar <= 0.0 {
            p + q
        } else {
            -dbar / (p * p + q * q + cbar)
        };
        return vec![if direct { (t - b) / a } else { -d / (t + c) }];
    }
    let (cbara, dbara) = (delta1, -2.0 * b * delta1 + a * delta2);
    let (cbard, dbard) = (delta3, -d * delta2 + 2.0 * c * delta3);
    let sqrt_disc = discriminant.sqrt();
    let half_sqrt3 = 3.0_f64.sqrt() / 2.0;
    let theta = (a * sqrt_disc).atan2(-dbara).abs() / 3.0;
    let scale = 2.0 * (-cbara).sqrt();
    let t1 = scale * theta.cos();
    let t3 = scale * (-theta.cos() / 2.0 - half_sqrt3 * theta.sin());
    let numerator_large = if t1 + t3 > 2.0 * b { t1 - b } else { t3 - b };
    let root1 = numerator_large / a;
    let theta = (d * sqrt_disc).atan2(-dbard).abs() / 3.0;
    let scale = 2.0 * (-cbard).sqrt();
    let t1 = scale * theta.cos();
    let t3 = scale * (-theta.cos() / 2.0 - half_sqrt3 * theta.sin());
    let numerator_small = -d;
    let denominator_small = if t1 + t3 < 2.0 * c { t1 + c } else { t3 + c };
    let root3 = numerator_small / denominator_small;
    let e = a * denominator_small;
    let f = -numerator_large * denominator_small - a * numerator_small;
    let g = numerator_large * numerator_small;
    let root2 = (c * f - b * g) / (-b * f + c * e);
    let mut roots = vec![root1, root2, root3];
    roots.sort_by(f64::total_cmp);
    roots
}

/// Discriminant of `a*x⁴ + b*x³ + c*x² + d*x + e`.
pub fn quartic_discriminant(a: f64, b: f64, c: f64, d: f64, e: f64) -> f64 {
    let (a2, b2, c2, d2, e2) = (a * a, b * b, c * c, d * d, e * e);
    let (a3, b3, c3, d3, e3) = (a2 * a, b2 * b, c2 * c, d2 * d, e2 * e);
    b2 * c2 * d2 - 4.0 * b3 * d3 - 4.0 * a * c3 * d2 + 18.0 * a * b * c * d3 - 27.0 * a2 * d2 * d2
        + 256.0 * a3 * e3
        + e * (18.0 * b3 * c * d - 4.0 * b2 * c3 + 16.0 * a * c2 * c2
            - 80.0 * a * b * c2 * d
            - 6.0 * a * b2 * d2
            + 144.0 * a2 * c * d2)
        + e2 * (144.0 * a * b2 * c - 27.0 * b2 * b2 - 128.0 * a2 * c2 - 192.0 * a2 * b * d)
}

fn quartic_original(a3: f64, a2: f64, a1: f64, a0: f64) -> Vec<f64> {
    let a3sq = a3 * a3;
    let p = a2 - 3.0 * a3sq / 8.0;
    let q = a1 - a2 * a3 / 2.0 + a3sq * a3 / 8.0;
    let r = a0 - a1 * a3 / 4.0 + a2 * a3sq / 16.0 - 3.0 * a3sq * a3sq / 256.0;
    let cubic = cubic_real_roots(1.0, 2.0 * p, p * p - 4.0 * r, -q * q);
    let Some(&h2) = cubic.last() else {
        return vec![];
    };
    let offset = -a3 / 4.0;
    if h2.abs() < 1e-14 {
        let roots = quadratic_real_roots(1.0, p, r);
        if roots.len() != 2 {
            return vec![];
        }
        let mut result = Vec::new();
        for &root in &roots {
            if root >= 0.0 {
                let y = root.sqrt();
                result.extend([offset - y, offset + y]);
            }
        }
        result.sort_by(f64::total_cmp);
        return result;
    }
    if h2 <= 0.0 || !h2.is_finite() {
        return vec![];
    }
    let h = h2.sqrt();
    let m = (p + h2 - q / h) / 2.0;
    let n = (p + h2 + q / h) / 2.0;
    let mut result = quadratic_real_roots(1.0, h, m);
    result.extend(quadratic_real_roots(1.0, -h, n));
    for x in &mut result {
        *x += offset;
    }
    result.sort_by(f64::total_cmp);
    result
}

fn quartic_neumark(a3: f64, a2: f64, a1: f64, a0: f64) -> Vec<f64> {
    let (a1sq, a2sq, a3sq) = (a1 * a1, a2 * a2, a3 * a3);
    let cubic = cubic_real_roots(
        1.0,
        -2.0 * a2,
        a1 * a3 + a2sq - 4.0 * a0,
        a3sq * a0 - a1 * a2 * a3 + a1sq,
    );
    let Some(&y) = cubic.first() else {
        return vec![];
    };
    let temp = a2 - y;
    let (g1, h1) = (a3 / 2.0, temp / 2.0);
    let m = temp * temp - 4.0 * a0;
    let m_error = temp * temp + 4.0 * a0.abs();
    let n = a3sq - 4.0 * y;
    let n_error = a3sq + 4.0 * y.abs();
    let (g2, h2) = if y < 0.0 || m * n_error < n * m_error {
        let root = n.sqrt();
        (
            root / 2.0,
            if root == 0.0 {
                0.0
            } else {
                (a3 * h1 - a1) / root
            },
        )
    } else {
        let root = m.sqrt();
        (
            if root == 0.0 {
                0.0
            } else {
                (a3 * h1 - a1) / root
            },
            root / 2.0,
        )
    };
    // JS Math.sign treats positive and negative zero as equal.
    let sign = |v: f64| if v == 0.0 { 0.0 } else { v.signum() };
    let (big_g, small_g) = if g1 == 0.0 && g2 == 0.0 {
        (0.0, 0.0)
    } else if sign(g1) == sign(g2) {
        let big = g1 + g2;
        (big, y / big)
    } else {
        let small = g1 - g2;
        (y / small, small)
    };
    let (big_h, small_h) = if h1 == 0.0 && h2 == 0.0 {
        (0.0, 0.0)
    } else if sign(h1) == sign(h2) {
        let big = h1 + h2;
        (big, a0 / big)
    } else {
        let small = h1 - h2;
        (a0 / small, small)
    };
    let mut result = quadratic_real_roots(1.0, big_g, big_h);
    result.extend(quadratic_real_roots(1.0, small_g, small_h));
    result.sort_by(f64::total_cmp);
    result
}

/// Real roots using Cesium's two quartic factorizations; multiplicities are retained.
/// Non-finite coefficients are rejected. A near-zero leading coefficient uses the cubic solver.
pub fn quartic_real_roots(a: f64, b: f64, c: f64, d: f64, e: f64) -> Vec<f64> {
    if ![a, b, c, d, e].iter().all(|v| v.is_finite()) {
        return vec![];
    }
    if a.abs() < 1e-15 {
        return cubic_real_roots(b, c, d, e);
    }
    let (a3, a2, a1, a0) = (b / a, c / a, d / a, e / a);
    if ![a3, a2, a1, a0].iter().all(|v| v.is_finite()) {
        return vec![];
    }
    let key =
        ((a3 < 0.0) as u8) * 8 + ((a2 < 0.0) as u8) * 4 + ((a1 < 0.0) as u8) * 2 + (a0 < 0.0) as u8;
    if matches!(key, 1 | 2 | 5 | 8 | 11) {
        quartic_neumark(a3, a2, a1, a0)
    } else {
        quartic_original(a3, a2, a1, a0)
    }
}
