//! Easing functions matching CesiumJS `Core/EasingFunction.js` (Apache-2.0).

/// Easing curves for animations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EasingFunction {
    #[default]
    LinearNone,
    QuadraticIn,
    QuadraticOut,
    QuadraticInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    QuarticIn,
    QuarticOut,
    QuarticInOut,
    QuinticIn,
    QuinticOut,
    QuinticInOut,
    SinusoidalIn,
    SinusoidalOut,
    SinusoidalInOut,
    ExponentialIn,
    ExponentialOut,
    ExponentialInOut,
    CircularIn,
    CircularOut,
    CircularInOut,
}

impl EasingFunction {
    /// Evaluates the easing function for parameter `k` in range `[0.0, 1.0]`.
    pub fn evaluate(&self, k: f64) -> f64 {
        let k = k.clamp(0.0, 1.0);
        match self {
            Self::LinearNone => k,
            Self::QuadraticIn => k * k,
            Self::QuadraticOut => k * (2.0 - k),
            Self::QuadraticInOut => {
                let k2 = k * 2.0;
                if k2 < 1.0 {
                    0.5 * k2 * k2
                } else {
                    -0.5 * ((k2 - 1.0) * (k2 - 3.0) - 1.0)
                }
            }
            Self::CubicIn => k * k * k,
            Self::CubicOut => {
                let km1 = k - 1.0;
                km1 * km1 * km1 + 1.0
            }
            Self::CubicInOut => {
                let k2 = k * 2.0;
                if k2 < 1.0 {
                    0.5 * k2 * k2 * k2
                } else {
                    let km2 = k2 - 2.0;
                    0.5 * (km2 * km2 * km2 + 2.0)
                }
            }
            Self::QuarticIn => k * k * k * k,
            Self::QuarticOut => {
                let km1 = k - 1.0;
                1.0 - km1 * km1 * km1 * km1
            }
            Self::QuarticInOut => {
                let k2 = k * 2.0;
                if k2 < 1.0 {
                    0.5 * k2 * k2 * k2 * k2
                } else {
                    let km2 = k2 - 2.0;
                    -0.5 * (km2 * km2 * km2 * km2 - 2.0)
                }
            }
            Self::QuinticIn => k * k * k * k * k,
            Self::QuinticOut => {
                let km1 = k - 1.0;
                km1 * km1 * km1 * km1 * km1 + 1.0
            }
            Self::QuinticInOut => {
                let k2 = k * 2.0;
                if k2 < 1.0 {
                    0.5 * k2 * k2 * k2 * k2 * k2
                } else {
                    let km2 = k2 - 2.0;
                    0.5 * (km2 * km2 * km2 * km2 * km2 + 2.0)
                }
            }
            Self::SinusoidalIn => 1.0 - (k * std::f64::consts::FRAC_PI_2).cos(),
            Self::SinusoidalOut => (k * std::f64::consts::FRAC_PI_2).sin(),
            Self::SinusoidalInOut => 0.5 * (1.0 - (std::f64::consts::PI * k).cos()),
            Self::ExponentialIn => {
                if k == 0.0 {
                    0.0
                } else {
                    (10.0 * (k - 1.0)).exp2()
                }
            }
            Self::ExponentialOut => {
                if (k - 1.0).abs() < 1e-9 {
                    1.0
                } else {
                    1.0 - (-10.0 * k).exp2()
                }
            }
            Self::ExponentialInOut => {
                if k == 0.0 {
                    0.0
                } else if (k - 1.0).abs() < 1e-9 {
                    1.0
                } else {
                    let k2 = k * 2.0;
                    if k2 < 1.0 {
                        0.5 * (10.0 * (k2 - 1.0)).exp2()
                    } else {
                        0.5 * (2.0 - (-10.0 * (k2 - 1.0)).exp2())
                    }
                }
            }
            Self::CircularIn => 1.0 - (1.0 - k * k).sqrt(),
            Self::CircularOut => {
                let km1 = k - 1.0;
                (1.0 - km1 * km1).sqrt()
            }
            Self::CircularInOut => {
                let k2 = k * 2.0;
                if k2 < 1.0 {
                    -0.5 * ((1.0 - k2 * k2).sqrt() - 1.0)
                } else {
                    let km2 = k2 - 2.0;
                    0.5 * ((1.0 - km2 * km2).sqrt() + 1.0)
                }
            }
        }
    }
}
