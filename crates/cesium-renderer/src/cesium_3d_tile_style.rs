//! 3D Tiles Styling document matching CesiumJS `Scene/Cesium3DTileStyle.js`.
use std::collections::HashMap;

use cesium_math::{Color, DeveloperError};
use serde_json::Value;

use crate::conditions_expression::ConditionsExpression;
use crate::expression::Expression;
use crate::style_expression::{StyleValue, TileFeature};

/// Either a single [`Expression`] or a [`ConditionsExpression`].
#[derive(Clone, Debug, PartialEq)]
pub enum TileStyleExpression {
    Expression(Expression),
    Conditions(ConditionsExpression),
}

impl TileStyleExpression {
    /// Parses a style property value (string, boolean, number, or `{ "conditions": [...] }`).
    pub fn from_value(
        val: &Value,
        defines: &HashMap<String, String>,
    ) -> Result<Self, DeveloperError> {
        match val {
            Value::String(s) => Ok(Self::Expression(Expression::with_defines(
                s.clone(),
                defines.clone(),
            )?)),
            Value::Bool(b) => Ok(Self::Expression(Expression::with_defines(
                b.to_string(),
                defines.clone(),
            )?)),
            Value::Number(n) => Ok(Self::Expression(Expression::with_defines(
                n.to_string(),
                defines.clone(),
            )?)),
            Value::Object(obj) => {
                if let Some(arr) = obj.get("conditions").and_then(Value::as_array) {
                    let mut pairs = Vec::with_capacity(arr.len());
                    for item in arr {
                        let pair = item.as_array().ok_or_else(|| {
                            DeveloperError::new("Each condition entry must be a 2-element array.")
                        })?;
                        if pair.len() != 2 {
                            return Err(DeveloperError::new(
                                "Each condition entry must have [condition, expression].",
                            ));
                        }
                        let cond = pair[0]
                            .as_str()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| pair[0].to_string());
                        let expr = pair[1]
                            .as_str()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| pair[1].to_string());
                        pairs.push((cond, expr));
                    }
                    Ok(Self::Conditions(ConditionsExpression::new(
                        pairs,
                        Some(defines.clone()),
                    )?))
                } else {
                    Err(DeveloperError::new(
                        "Style object must contain a 'conditions' array.",
                    ))
                }
            }
            _ => Err(DeveloperError::new("Unsupported style expression value.")),
        }
    }

    pub fn expression_source(&self) -> String {
        format!("{self:?}")
    }

    pub fn evaluate<F: TileFeature>(&self, feature: &F) -> Result<StyleValue, DeveloperError> {
        match self {
            Self::Expression(e) => e.evaluate(feature),
            Self::Conditions(c) => c.evaluate(feature),
        }
    }

    pub fn evaluate_color<F: TileFeature>(&self, feature: &F) -> Result<Color, DeveloperError> {
        match self {
            Self::Expression(e) => e.evaluate_color(feature),
            Self::Conditions(c) => c.evaluate_color(feature),
        }
    }
}

/// Full 3D Tiles style document matching CesiumJS `Cesium3DTileStyle`.
#[derive(Clone, Debug, Default)]
pub struct Cesium3DTileStyleDocument {
    pub style: Value,
    pub ready: bool,
    pub defines: HashMap<String, String>,
    pub show: Option<TileStyleExpression>,
    pub color: Option<TileStyleExpression>,
    pub point_size: Option<TileStyleExpression>,
    pub point_outline_color: Option<TileStyleExpression>,
    pub point_outline_width: Option<TileStyleExpression>,
    pub label_color: Option<TileStyleExpression>,
    pub label_outline_color: Option<TileStyleExpression>,
    pub label_outline_width: Option<TileStyleExpression>,
    pub font: Option<TileStyleExpression>,
    pub label_style: Option<TileStyleExpression>,
    pub label_text: Option<TileStyleExpression>,
    pub background_color: Option<TileStyleExpression>,
    pub background_padding: Option<TileStyleExpression>,
    pub background_enabled: Option<TileStyleExpression>,
    pub scale_by_distance: Option<TileStyleExpression>,
    pub translucency_by_distance: Option<TileStyleExpression>,
    pub distance_display_condition: Option<TileStyleExpression>,
    pub height_offset: Option<TileStyleExpression>,
    pub anchor_line_enabled: Option<TileStyleExpression>,
    pub anchor_line_color: Option<TileStyleExpression>,
    pub image: Option<TileStyleExpression>,
    pub disable_depth_test_distance: Option<TileStyleExpression>,
    pub horizontal_origin: Option<TileStyleExpression>,
    pub vertical_origin: Option<TileStyleExpression>,
    pub label_horizontal_origin: Option<TileStyleExpression>,
    pub label_vertical_origin: Option<TileStyleExpression>,
    pub line_width: Option<TileStyleExpression>,
    pub meta: HashMap<String, TileStyleExpression>,
}

impl Cesium3DTileStyleDocument {
    /// Parses a 3D Tiles style JSON object into a [`Cesium3DTileStyleDocument`].
    pub fn from_json(style_json: &Value) -> Result<Self, DeveloperError> {
        let obj = style_json
            .as_object()
            .ok_or_else(|| DeveloperError::new("style JSON must be an object."))?;

        let mut defines = HashMap::new();
        if let Some(defs) = obj.get("defines").and_then(Value::as_object) {
            for (k, v) in defs {
                let expr_str = v
                    .as_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| v.to_string());
                defines.insert(k.clone(), expr_str);
            }
        }

        let parse_opt = |key: &str| -> Result<Option<TileStyleExpression>, DeveloperError> {
            match obj.get(key) {
                Some(v) if !v.is_null() => Ok(Some(TileStyleExpression::from_value(v, &defines)?)),
                _ => Ok(None),
            }
        };

        let mut meta = HashMap::new();
        if let Some(meta_obj) = obj.get("meta").and_then(Value::as_object) {
            for (k, v) in meta_obj {
                meta.insert(k.clone(), TileStyleExpression::from_value(v, &defines)?);
            }
        }

        let show = parse_opt("show")?;
        let color = parse_opt("color")?;
        let point_size = parse_opt("pointSize")?;
        let point_outline_color = parse_opt("pointOutlineColor")?;
        let point_outline_width = parse_opt("pointOutlineWidth")?;
        let label_color = parse_opt("labelColor")?;
        let label_outline_color = parse_opt("labelOutlineColor")?;
        let label_outline_width = parse_opt("labelOutlineWidth")?;
        let font = parse_opt("font")?;
        let label_style = parse_opt("labelStyle")?;
        let label_text = parse_opt("labelText")?;
        let background_color = parse_opt("backgroundColor")?;
        let background_padding = parse_opt("backgroundPadding")?;
        let background_enabled = parse_opt("backgroundEnabled")?;
        let scale_by_distance = parse_opt("scaleByDistance")?;
        let translucency_by_distance = parse_opt("translucencyByDistance")?;
        let distance_display_condition = parse_opt("distanceDisplayCondition")?;
        let height_offset = parse_opt("heightOffset")?;
        let anchor_line_enabled = parse_opt("anchorLineEnabled")?;
        let anchor_line_color = parse_opt("anchorLineColor")?;
        let image = parse_opt("image")?;
        let disable_depth_test_distance = parse_opt("disableDepthTestDistance")?;
        let horizontal_origin = parse_opt("horizontalOrigin")?;
        let vertical_origin = parse_opt("verticalOrigin")?;
        let label_horizontal_origin = parse_opt("labelHorizontalOrigin")?;
        let label_vertical_origin = parse_opt("labelVerticalOrigin")?;
        let line_width = parse_opt("lineWidth")?;

        Ok(Self {
            style: style_json.clone(),
            ready: true,
            defines,
            show,
            color,
            point_size,
            point_outline_color,
            point_outline_width,
            label_color,
            label_outline_color,
            label_outline_width,
            font,
            label_style,
            label_text,
            background_color,
            background_padding,
            background_enabled,
            scale_by_distance,
            translucency_by_distance,
            distance_display_condition,
            height_offset,
            anchor_line_enabled,
            anchor_line_color,
            image,
            disable_depth_test_distance,
            horizontal_origin,
            vertical_origin,
            label_horizontal_origin,
            label_vertical_origin,
            line_width,
            meta,
        })
    }
}
