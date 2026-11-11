//! SunLight matching CesiumJS `Scene/SunLight.js` (Apache-2.0).

use cesium_math::Color;

use crate::light::Light;

/// Construction options for [`SunLight`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SunLightOptions {
    /// The light's color. Default is `Color::WHITE`.
    pub color: Option<Color>,
    /// The light's intensity. Default is `2.0`.
    pub intensity: Option<f64>,
}

/// A directional light source that originates from the Sun.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SunLight {
    /// The color of the light.
    pub color: Color,
    /// The intensity of the light.
    pub intensity: f64,
}

impl Default for SunLight {
    fn default() -> Self {
        Self::new(None)
    }
}

impl SunLight {
    /// Creates a new `SunLight` with default or provided options.
    pub fn new(options: Option<SunLightOptions>) -> Self {
        let options = options.unwrap_or_default();
        Self {
            color: options.color.unwrap_or(Color::WHITE),
            intensity: options.intensity.unwrap_or(2.0),
        }
    }
}

impl Light for SunLight {
    fn color(&self) -> Color {
        self.color
    }

    fn intensity(&self) -> f64 {
        self.intensity
    }
}
