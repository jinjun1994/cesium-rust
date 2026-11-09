//! FrustumCommands matching CesiumJS `Scene/FrustumCommands.js` (Apache-2.0).

use crate::commands::{DrawCommand, NUMBER_OF_PASSES};

/// Defines a list of commands whose geometry are bound by near and far distances from the camera.
#[derive(Debug, Clone, PartialEq)]
pub struct FrustumCommands {
    /// The lower bound or closest distance from the camera.
    pub near: f64,
    /// The upper bound or farthest distance from the camera.
    pub far: f64,
    /// Commands per pass.
    pub commands: Vec<Vec<DrawCommand>>,
    /// Number of commands currently active per pass.
    pub indices: Vec<usize>,
}

impl Default for FrustumCommands {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl FrustumCommands {
    /// Creates a new `FrustumCommands` with optional near and far plane bounds.
    pub fn new(near: Option<f64>, far: Option<f64>) -> Self {
        let mut commands = Vec::with_capacity(NUMBER_OF_PASSES);
        let mut indices = Vec::with_capacity(NUMBER_OF_PASSES);

        for _ in 0..NUMBER_OF_PASSES {
            commands.push(Vec::new());
            indices.push(0);
        }

        Self {
            near: near.unwrap_or(0.0),
            far: far.unwrap_or(0.0),
            commands,
            indices,
        }
    }
}
