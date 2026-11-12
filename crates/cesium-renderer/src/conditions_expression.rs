//! Conditions expression matching CesiumJS `Scene/ConditionsExpression.js`.
use std::collections::HashMap;

use cesium_math::{Color, DeveloperError};

use crate::expression::Expression;
use crate::style_expression::{StyleValue, TileFeature};

#[derive(Clone, Debug, PartialEq)]
struct RuntimeStatement {
    condition: Expression,
    expression: Expression,
}

/// Evaluates an ordered list of `[condition, expression]` pairs for 3D Tiles Styling.
#[derive(Clone, Debug, PartialEq)]
pub struct ConditionsExpression {
    conditions: Vec<(String, String)>,
    runtime_conditions: Vec<RuntimeStatement>,
}

impl ConditionsExpression {
    /// Creates a new [`ConditionsExpression`] from `(condition, expression)` pairs and optional `defines`.
    pub fn new(
        conditions: Vec<(String, String)>,
        defines: Option<HashMap<String, String>>,
    ) -> Result<Self, DeveloperError> {
        let defines = defines.unwrap_or_default();
        let mut runtime_conditions = Vec::with_capacity(conditions.len());
        for (cond_str, expr_str) in &conditions {
            let condition = Expression::with_defines(cond_str.clone(), defines.clone())?;
            let expression = Expression::with_defines(expr_str.clone(), defines.clone())?;
            runtime_conditions.push(RuntimeStatement {
                condition,
                expression,
            });
        }
        Ok(Self {
            conditions,
            runtime_conditions,
        })
    }

    /// Returns the raw condition/expression pairs.
    pub fn conditions(&self) -> &[(String, String)] {
        &self.conditions
    }

    /// Evaluates the first matching condition against `feature`.
    pub fn evaluate<F: TileFeature>(&self, feature: &F) -> Result<StyleValue, DeveloperError> {
        for stmt in &self.runtime_conditions {
            let cond = stmt.condition.evaluate(feature)?;
            if matches!(cond, StyleValue::Boolean(true)) {
                return stmt.expression.evaluate(feature);
            }
        }
        Ok(StyleValue::Null)
    }

    /// Evaluates the first matching condition as a [`Color`].
    pub fn evaluate_color<F: TileFeature>(&self, feature: &F) -> Result<Color, DeveloperError> {
        for stmt in &self.runtime_conditions {
            let cond = stmt.condition.evaluate(feature)?;
            if matches!(cond, StyleValue::Boolean(true)) {
                return stmt.expression.evaluate_color(feature);
            }
        }
        Ok(Color::WHITE)
    }
}
