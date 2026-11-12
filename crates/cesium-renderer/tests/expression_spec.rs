use std::collections::HashMap;

use cesium_math::Color;
use cesium_renderer::{Expression, StyleValue, TileFeature};
use serde_json::{json, Value};

struct MockFeature {
    props: HashMap<String, Value>,
}

impl MockFeature {
    fn new() -> Self {
        Self {
            props: HashMap::new(),
        }
    }

    fn add(&mut self, name: &str, value: Value) {
        self.props.insert(name.to_string(), value);
    }
}

impl TileFeature for MockFeature {
    fn get_property_inherited(&self, name: &str) -> Option<Value> {
        self.props.get(name).cloned()
    }
}

#[test]
fn parses_backslashes() {
    let feature = MockFeature::new();
    let expr = Expression::new(r#""\\he\\\\\\ll\\\\o""#).unwrap();
    assert_eq!(
        expr.evaluate(&feature).unwrap(),
        StyleValue::String(r"\he\\\ll\\o".to_string())
    );
}

#[test]
fn evaluates_variable_and_string_interpolation() {
    let mut feature = MockFeature::new();
    feature.add("height", json!(10));
    feature.add("width", json!(5));
    feature.add("string", json!("hello"));
    feature.add("boolean", json!(true));
    feature.add("null", Value::Null);

    assert_eq!(
        Expression::new("${height}").unwrap().evaluate(&feature).unwrap(),
        StyleValue::Number(10.0)
    );
    assert_eq!(
        Expression::new("'${height}'").unwrap().evaluate(&feature).unwrap(),
        StyleValue::String("10".to_string())
    );
    assert_eq!(
        Expression::new("${height}/${width}")
            .unwrap()
            .evaluate(&feature)
            .unwrap(),
        StyleValue::Number(2.0)
    );
    assert_eq!(
        Expression::new("'replace ${string} multiple ${height}'")
            .unwrap()
            .evaluate(&feature)
            .unwrap(),
        StyleValue::String("replace hello multiple 10".to_string())
    );
    assert_eq!(
        Expression::new("'replace ${string'")
            .unwrap()
            .evaluate(&feature)
            .unwrap(),
        StyleValue::String("replace ${string".to_string())
    );
    assert_eq!(
        Expression::new("${boolean}").unwrap().evaluate(&feature).unwrap(),
        StyleValue::Boolean(true)
    );
    assert_eq!(
        Expression::new("'${boolean}'")
            .unwrap()
            .evaluate(&feature)
            .unwrap(),
        StyleValue::String("true".to_string())
    );
    assert_eq!(
        Expression::new("${null}").unwrap().evaluate(&feature).unwrap(),
        StyleValue::Null
    );
}

#[test]
fn evaluates_arithmetic_comparison_logical_and_ternary() {
    let mut feature = MockFeature::new();
    feature.add("Height", json!(100));
    feature.add("Temperature", json!(78));
    feature.add("ZipCode", json!("19341"));

    assert_eq!(
        Expression::new("(${Height} + 20) * 2 - 40 / 4")
            .unwrap()
            .evaluate(&feature)
            .unwrap(),
        StyleValue::Number(230.0)
    );
    assert_eq!(
        Expression::new("${Height} >= 100 && ${Temperature} < 80")
            .unwrap()
            .evaluate(&feature)
            .unwrap(),
        StyleValue::Boolean(true)
    );
    assert_eq!(
        Expression::new("${ZipCode} === '19341' ? 1 : 2")
            .unwrap()
            .evaluate(&feature)
            .unwrap(),
        StyleValue::Number(1.0)
    );
    assert_eq!(
        Expression::new("!(${Height} < 50) || false")
            .unwrap()
            .evaluate(&feature)
            .unwrap(),
        StyleValue::Boolean(true)
    );
}

#[test]
fn evaluates_colors_and_vectors() {
    let mut feature = MockFeature::new();
    feature.add("red", json!(255));
    feature.add("green", json!(0));
    feature.add("blue", json!(0));

    assert_eq!(
        Expression::new("color('red')").unwrap().evaluate_color(&feature).unwrap(),
        Color::RED
    );
    assert_eq!(
        Expression::new("color('#00FF00', 0.5)")
            .unwrap()
            .evaluate_color(&feature)
            .unwrap(),
        Color::new(0.0, 1.0, 0.0, 0.5)
    );
    assert_eq!(
        Expression::new("rgb(${red}, ${green}, ${blue})")
            .unwrap()
            .evaluate_color(&feature)
            .unwrap(),
        Color::RED
    );
    assert_eq!(
        Expression::new("vec3(1, 2, 3).y")
            .unwrap()
            .evaluate(&feature)
            .unwrap(),
        StyleValue::Number(2.0)
    );
    assert_eq!(
        Expression::new("length(vec2(3, 4))")
            .unwrap()
            .evaluate(&feature)
            .unwrap(),
        StyleValue::Number(5.0)
    );
}

#[test]
fn evaluates_defines_and_math_functions() {
    let mut feature = MockFeature::new();
    feature.add("Height", json!(100));

    let mut defines = HashMap::new();
    defines.insert("halfHeight".to_string(), "${Height} / 2".to_string());

    let expr = Expression::with_defines("clamp(${halfHeight}, 10, 40) + sqrt(16)", defines).unwrap();
    assert_eq!(expr.evaluate(&feature).unwrap(), StyleValue::Number(44.0));
}

#[test]
fn rejects_invalid_expressions() {
    assert!(Expression::new("1 + * 2").is_err());
    assert!(Expression::new("'unterminated").is_err());
    assert!(Expression::new("${unterminated").is_err());
}
