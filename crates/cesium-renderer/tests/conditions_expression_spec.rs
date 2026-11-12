use std::collections::HashMap;

use cesium_math::Color;
use cesium_renderer::{ConditionsExpression, StyleValue, TileFeature};
use serde_json::{json, Value};

struct MockHeightFeature {
    height: f64,
    floor_color: Option<Color>,
}

impl MockHeightFeature {
    fn new(height: f64) -> Self {
        Self {
            height,
            floor_color: None,
        }
    }

    fn with_color(height: f64, color: Color) -> Self {
        Self {
            height,
            floor_color: Some(color),
        }
    }
}

impl TileFeature for MockHeightFeature {
    fn get_property_inherited(&self, name: &str) -> Option<Value> {
        match name {
            "Height" => Some(json!(self.height)),
            "FloorColor" => self.floor_color.map(|c| {
                json!({
                    "red": c.red,
                    "green": c.green,
                    "blue": c.blue,
                    "alpha": c.alpha
                })
            }),
            _ => None,
        }
    }
}

fn sample_conditions() -> Vec<(String, String)> {
    vec![
        ("${Height} > 100".to_string(), "color(\"blue\")".to_string()),
        ("${Height} > 50".to_string(), "color(\"red\")".to_string()),
        ("true".to_string(), "color(\"lime\")".to_string()),
    ]
}

#[test]
fn constructs() {
    let expr = ConditionsExpression::new(sample_conditions(), None).unwrap();
    assert_eq!(expr.conditions(), sample_conditions().as_slice());
}

#[test]
fn evaluates_conditional() {
    let expr = ConditionsExpression::new(sample_conditions(), None).unwrap();
    assert_eq!(
        expr.evaluate_color(&MockHeightFeature::new(101.0)).unwrap(),
        Color::BLUE
    );
    assert_eq!(
        expr.evaluate_color(&MockHeightFeature::new(52.0)).unwrap(),
        Color::RED
    );
    assert_eq!(
        expr.evaluate_color(&MockHeightFeature::new(3.0)).unwrap(),
        Color::LIME
    );
}

#[test]
fn evaluates_conditional_with_defines() {
    let mut defines = HashMap::new();
    defines.insert("halfHeight".to_string(), "${Height}/2".to_string());
    defines.insert("quarterHeight".to_string(), "${Height}/4".to_string());

    let conditions = vec![
        (
            "${halfHeight} > 50 && ${halfHeight} < 100".to_string(),
            "color(\"blue\")".to_string(),
        ),
        (
            "${quarterHeight} > 50 && ${quarterHeight} < 52".to_string(),
            "color(\"red\")".to_string(),
        ),
        ("true".to_string(), "color(\"lime\")".to_string()),
    ];

    let expr = ConditionsExpression::new(conditions, Some(defines)).unwrap();
    assert_eq!(
        expr.evaluate_color(&MockHeightFeature::new(101.0)).unwrap(),
        Color::BLUE
    );
    assert_eq!(
        expr.evaluate_color(&MockHeightFeature::new(52.0)).unwrap(),
        Color::LIME
    );
    assert_eq!(
        expr.evaluate_color(&MockHeightFeature::new(3.0)).unwrap(),
        Color::LIME
    );
}

#[test]
fn evaluates_with_multiple_variables_and_unmatched_fallback() {
    let conditions = vec![
        ("${Height} > 100".to_string(), "${FloorColor}".to_string()),
        (
            "${Height} > 50".to_string(),
            "${FloorColor} * 0.5".to_string(),
        ),
        ("true".to_string(), "color(\"lime\")".to_string()),
    ];
    let expr = ConditionsExpression::new(conditions, None).unwrap();

    let f1 = MockHeightFeature::with_color(101.0, Color::RED);
    assert_eq!(expr.evaluate_color(&f1).unwrap(), Color::RED);

    let f2 = MockHeightFeature::with_color(60.0, Color::new(1.0, 0.0, 0.0, 1.0));
    assert_eq!(
        expr.evaluate_color(&f2).unwrap(),
        Color::new(0.5, 0.0, 0.0, 0.5)
    );

    let empty = ConditionsExpression::new(
        vec![("${Height} > 1000".to_string(), "color(\"red\")".to_string())],
        None,
    )
    .unwrap();
    assert_eq!(
        empty.evaluate(&MockHeightFeature::new(10.0)).unwrap(),
        StyleValue::Null
    );
    assert_eq!(
        empty.evaluate_color(&MockHeightFeature::new(10.0)).unwrap(),
        Color::WHITE
    );
}
