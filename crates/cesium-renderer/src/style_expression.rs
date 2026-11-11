//! StyleExpression matching CesiumJS `Scene/StyleExpression.js` (Apache-2.0).

use std::collections::HashMap;

use cesium_math::{Color, DeveloperError};
use serde_json::Value;

/// Result value of evaluating a 3D Tiles style expression.
#[derive(Clone, Debug, PartialEq)]
pub enum StyleValue {
    Boolean(bool),
    Number(f64),
    String(String),
    Color(Color),
    Array(Vec<StyleValue>),
    Null,
}

/// Common trait for 3D tile features whose properties are evaluated in styling expressions.
pub trait TileFeature {
    /// Retrieves an inherited or direct property value by name.
    fn get_property_inherited(&self, name: &str) -> Option<Value>;
}

/// An expression for a style applied to a 3D Tileset or model features.
///
/// This type describes an interface and is not intended to be instantiated directly.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StyleExpression;

impl StyleExpression {
    /// Creates a new `StyleExpression`.
    pub fn new() -> Self {
        Self
    }

    /// Evaluates the result of an expression, optionally using the provided feature's properties.
    pub fn evaluate<F: TileFeature>(&self, _feature: &F) -> Result<StyleValue, DeveloperError> {
        Err(DeveloperError::new(
            "This type describes an interface and is not intended to be instantiated directly.",
        ))
    }

    /// Evaluates the result of a Color expression, optionally using the provided feature's properties.
    pub fn evaluate_color<F: TileFeature>(&self, _feature: &F) -> Result<Color, DeveloperError> {
        Err(DeveloperError::new(
            "This type describes an interface and is not intended to be instantiated directly.",
        ))
    }

    /// Gets the shader function for this expression.
    pub fn get_shader_function(
        &self,
        _function_signature: &str,
        _variable_substitution_map: &HashMap<String, String>,
        _shader_state: &Value,
        _return_type: &str,
    ) -> Result<String, DeveloperError> {
        Err(DeveloperError::new(
            "This type describes an interface and is not intended to be instantiated directly.",
        ))
    }

    /// Gets the variables used by the expression.
    pub fn get_variables(&self, _prefix: &str) -> Result<Vec<String>, DeveloperError> {
        Err(DeveloperError::new(
            "This type describes an interface and is not intended to be instantiated directly.",
        ))
    }
}
