//! Mathematical constants and helper functions matching CesiumJS `Core/Math`.
//! Adapted from CesiumJS (Apache-2.0).

pub const EPSILON1: f64 = 0.1;
pub const EPSILON2: f64 = 0.01;
pub const EPSILON3: f64 = 0.001;
pub const EPSILON4: f64 = 0.0001;
pub const EPSILON5: f64 = 0.00001;
pub const EPSILON6: f64 = 0.000001;
pub const EPSILON7: f64 = 0.0000001;
pub const EPSILON8: f64 = 0.00000001;
pub const EPSILON9: f64 = 0.000000001;
pub const EPSILON10: f64 = 0.0000000001;
pub const EPSILON11: f64 = 0.00000000001;
pub const EPSILON12: f64 = 0.000000000001;
pub const EPSILON13: f64 = 0.0000000000001;
pub const EPSILON14: f64 = 0.00000000000001;
pub const EPSILON15: f64 = 0.000000000000001;
pub const EPSILON16: f64 = 0.0000000000000001;
pub const EPSILON17: f64 = 0.00000000000000001;
pub const EPSILON18: f64 = 0.000000000000000001;
pub const EPSILON19: f64 = 0.0000000000000000001;
pub const EPSILON20: f64 = 0.00000000000000000001;

pub const PI: f64 = std::f64::consts::PI;
pub const TWO_PI: f64 = 2.0 * PI;
pub const HALF_PI: f64 = std::f64::consts::FRAC_PI_2;
pub const PI_OVER_TWO: f64 = HALF_PI;
pub const PI_OVER_THREE: f64 = PI / 3.0;
pub const PI_OVER_FOUR: f64 = std::f64::consts::FRAC_PI_4;
pub const PI_OVER_SIX: f64 = PI / 6.0;
pub const THREE_PI_OVER_TWO: f64 = 3.0 * PI / 2.0;

pub const RADIANS_PER_DEGREE: f64 = PI / 180.0;
pub const DEGREES_PER_RADIAN: f64 = 180.0 / PI;

#[inline]
pub fn sign(value: f64) -> f64 {
    if value > 0.0 {
        1.0
    } else if value < 0.0 {
        -1.0
    } else if value == 0.0 {
        if value.is_sign_negative() {
            -0.0
        } else {
            0.0
        }
    } else {
        f64::NAN
    }
}

#[inline]
pub fn sign_not_zero(value: f64) -> f64 {
    if value < 0.0 {
        -1.0
    } else {
        1.0
    }
}

#[inline]
pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

#[inline]
pub fn lerp(p: f64, q: f64, time: f64) -> f64 {
    (1.0 - time) * p + time * q
}

#[inline]
pub fn to_snorm(value: f64, range_maximum: Option<f64>) -> f64 {
    let max = range_maximum.unwrap_or(255.0);
    ((clamp(value, -1.0, 1.0) * 0.5 + 0.5) * max).round()
}

#[inline]
pub fn from_snorm(value: f64, range_maximum: Option<f64>) -> f64 {
    let max = range_maximum.unwrap_or(255.0);
    (clamp(value, 0.0, max) / max) * 2.0 - 1.0
}

#[inline]
pub fn normalize(value: f64, range_minimum: f64, range_maximum: f64) -> f64 {
    let span = (range_maximum - range_minimum).max(0.0);
    if span == 0.0 {
        0.0
    } else {
        clamp((value - range_minimum) / span, 0.0, 1.0)
    }
}

#[inline]
pub fn asin_clamped(value: f64) -> f64 {
    clamp(value, -1.0, 1.0).asin()
}

#[inline]
pub fn acos_clamped(value: f64) -> f64 {
    clamp(value, -1.0, 1.0).acos()
}

#[inline]
pub fn chord_length(angle: f64, radius: f64) -> f64 {
    2.0 * radius * (angle * 0.5).sin()
}

#[inline]
pub fn equals_epsilon(
    left: f64,
    right: f64,
    relative_epsilon: Option<f64>,
    absolute_epsilon: Option<f64>,
) -> bool {
    let rel = relative_epsilon.unwrap_or(0.0);
    let abs = absolute_epsilon.unwrap_or(rel);
    let abs_diff = (left - right).abs();
    abs_diff <= abs || abs_diff <= rel * left.abs().max(right.abs())
}

#[inline]
pub fn negative_pi_to_pi(angle: f64) -> f64 {
    if (-PI..=PI).contains(&angle) {
        angle
    } else {
        zero_to_two_pi(angle + PI) - PI
    }
}

pub const SIXTY_FOUR_KILOBYTES: usize = 65536;
pub const SIXTY_FOUR_KILOBYTES_F64: f64 = 65536.0;

#[inline]
pub fn zero_to_two_pi(angle: f64) -> f64 {
    if (0.0..TWO_PI).contains(&angle) {
        angle
    } else {
        let m = angle % TWO_PI;
        if m < 0.0 {
            m + TWO_PI
        } else {
            m
        }
    }
}

/// Computes the factorial of a non-negative integer.
#[inline]
pub fn factorial(n: usize) -> f64 {
    const FACTORIALS: [f64; 21] = [
        1.0, 1.0, 2.0, 6.0, 24.0, 120.0, 720.0, 5040.0, 40320.0, 362880.0,
        3628800.0, 39916800.0, 479001600.0, 6227020800.0, 87178291200.0,
        1307674368000.0, 20922789888000.0, 355687428096000.0, 6402373705728000.0,
        121645100408832000.0, 2432902008176640000.0,
    ];
    if n < FACTORIALS.len() {
        FACTORIALS[n]
    } else {
        let mut result = FACTORIALS[FACTORIALS.len() - 1];
        for i in FACTORIALS.len()..=n {
            result *= i as f64;
        }
        result
    }
}

/// Increments a number with wrapping to a minimum value if the number exceeds the maximum value.
#[inline]
pub fn increment_wrap(n: f64, maximum_value: f64, minimum_value: Option<f64>) -> f64 {
    let min = minimum_value.unwrap_or(0.0);
    let mut next = n + 1.0;
    if next > maximum_value {
        next = min;
    }
    next
}

