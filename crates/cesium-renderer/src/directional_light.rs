use cesium_math::{Cartesian3, Color};
use thiserror::Error;
use crate::light::Light;

/// Errors thrown by `DirectionalLight`.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DirectionalLightError {
    /// Thrown when direction is zero length.
    #[error("options.direction cannot be zero-length")]
    ZeroLengthDirection,
}

/// Construction options for `DirectionalLight`.
#[derive(Debug, Clone, Default)]
pub struct DirectionalLightOptions {
    /// The direction in which light gets emitted.
    pub direction: Cartesian3,
    /// The color of the light. Defaults to Color::WHITE.
    pub color: Option<Color>,
    /// The intensity of the light. Defaults to 1.0.
    pub intensity: Option<f64>,
}

/// A light that gets emitted in a single direction from infinitely far away.
#[derive(Debug, Clone, PartialEq)]
pub struct DirectionalLight {
    /// The direction in which light gets emitted.
    pub direction: Cartesian3,
    /// The color of the light.
    pub color: Color,
    /// The intensity of the light.
    pub intensity: f64,
}

impl DirectionalLight {
    /// Creates a new `DirectionalLight`.
    pub fn new(options: DirectionalLightOptions) -> Result<Self, DirectionalLightError> {
        if options.direction == Cartesian3::ZERO {
            return Err(DirectionalLightError::ZeroLengthDirection);
        }

        Ok(Self {
            direction: options.direction,
            color: options.color.unwrap_or(Color::WHITE),
            intensity: options.intensity.unwrap_or(1.0),
        })
    }
}

impl Light for DirectionalLight {
    fn color(&self) -> Color {
        self.color
    }

    fn intensity(&self) -> f64 {
        self.intensity
    }
}
