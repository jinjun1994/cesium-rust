use cesium_math::{Color, DeveloperError};

/// Common interface describing a light source.
pub trait Light: Send + Sync {
    /// The color of the light.
    fn color(&self) -> Color;
    /// The intensity controlling the strength of the light.
    fn intensity(&self) -> f64;
}

/// Abstract light base struct matching CesiumJS `Scene/Light.js`.
///
/// Direct access to properties on the unspecialized base instance throws an instantiation error.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LightBase;

impl LightBase {
    /// Constructs a base light.
    pub fn new() -> Self {
        Self
    }

    /// Accessing color directly on base Light throws DeveloperError.
    pub fn color(&self) -> Result<Color, DeveloperError> {
        Err(DeveloperError::new(
            "This type describes an abstract interface and cannot be instantiated directly.",
        ))
    }

    /// Accessing intensity directly on base Light throws DeveloperError.
    pub fn intensity(&self) -> Result<f64, DeveloperError> {
        Err(DeveloperError::new(
            "This type describes an abstract interface and cannot be instantiated directly.",
        ))
    }
}
