use std::collections::HashMap;

use cesium_renderer::style_expression::{StyleExpression, TileFeature};
use serde_json::Value;

struct MockFeature;

impl TileFeature for MockFeature {
    fn get_property_inherited(&self, _name: &str) -> Option<Value> {
        None
    }
}

#[test]
fn test_style_expression_interface_throws_developer_error() {
    let expression = StyleExpression::new();
    let feature = MockFeature;

    let eval_res = expression.evaluate(&feature);
    assert!(eval_res.is_err());
    assert!(eval_res
        .unwrap_err()
        .message
        .contains("This type describes an interface"));

    let eval_color_res = expression.evaluate_color(&feature);
    assert!(eval_color_res.is_err());
    assert!(eval_color_res
        .unwrap_err()
        .message
        .contains("This type describes an interface"));

    let empty_vars = HashMap::new();
    let empty_state = Value::Null;
    let shader_res =
        expression.get_shader_function("getColor()", &empty_vars, &empty_state, "vec4");
    assert!(shader_res.is_err());
    assert!(shader_res
        .unwrap_err()
        .message
        .contains("This type describes an interface"));

    let vars_res = expression.get_variables("");
    assert!(vars_res.is_err());
    assert!(vars_res
        .unwrap_err()
        .message
        .contains("This type describes an interface"));
}
