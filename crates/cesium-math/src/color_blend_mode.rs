//! ColorBlendMode matching CesiumJS `Scene/ColorBlendMode.js` (Apache-2.0).

use crate::core_math::EPSILON4;

/// Defines different modes for blending between a target color and a primitive's source color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ColorBlendMode {
    #[default]
    Highlight = 0,
    Replace = 1,
    Mix = 2,
}

impl ColorBlendMode {
    /// Gets the color blend factor given a mode and blend amount.
    pub fn get_color_blend(self, blend_amount: f64) -> f64 {
        match self {
            Self::Highlight => 0.0,
            Self::Replace => 1.0,
            Self::Mix => blend_amount.clamp(EPSILON4, 1.0),
        }
    }
}
