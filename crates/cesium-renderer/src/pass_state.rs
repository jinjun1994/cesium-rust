//! PassState matching CesiumJS `Renderer/PassState.js` (Apache-2.0).

use cesium_math::BoundingRectangle;

/// The state for a particular rendering pass, used to supplement state in a command being executed.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PassState {
    pub context: Option<String>,
    pub framebuffer: Option<String>,
    pub blending_enabled: Option<bool>,
    pub scissor_test: Option<BoundingRectangle>,
    pub viewport: Option<BoundingRectangle>,
}

impl PassState {
    /// Creates a new PassState for a given context identifier.
    pub fn new(context: impl Into<String>) -> Self {
        Self {
            context: Some(context.into()),
            framebuffer: None,
            blending_enabled: None,
            scissor_test: None,
            viewport: None,
        }
    }
}
