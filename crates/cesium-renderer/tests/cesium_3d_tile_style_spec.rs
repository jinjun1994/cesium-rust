use std::collections::HashMap;

use cesium_math::Color;
use cesium_renderer::{
    Cesium3DTileStyleDocument, StyleValue, TileFeature, TileStyleExpression,
};
use serde_json::{json, Value};

struct MockStyleFeature {
    props: HashMap<String, Value>,
}

impl MockStyleFeature {
    fn new() -> Self {
        Self {
            props: HashMap::new(),
        }
    }

    fn add(&mut self, name: &str, value: Value) {
        self.props.insert(name.to_string(), value);
    }
}

impl TileFeature for MockStyleFeature {
    fn get_property_inherited(&self, name: &str) -> Option<Value> {
        self.props.get(name).cloned()
    }
}

#[test]
fn parses_full_style_json_properties() {
    let style_json = json!({
        "color": "color('red')",
        "show": "${id} < 100.0",
        "pointSize": "${id} / 100.0",
        "pointOutlineColor": "color('blue')",
        "pointOutlineWidth": "5.0",
        "labelColor": "color('yellow')",
        "labelOutlineColor": "color('orange')",
        "labelOutlineWidth": "6.0",
        "font": "'24px Helvetica'",
        "labelStyle": "1",
        "labelText": "'label text'",
        "backgroundColor": "color('coral')",
        "backgroundPadding": "vec2(1.0, 2.0)",
        "backgroundEnabled": "true",
        "scaleByDistance": "vec4(1.0, 2.0, 3.0, 4.0)",
        "translucencyByDistance": "vec4(5.0, 6.0, 7.0, 8.0)",
        "distanceDisplayCondition": "vec2(3.0, 4.0)",
        "heightOffset": "10.0",
        "anchorLineEnabled": "true",
        "anchorLineColor": "color('fuchsia')",
        "image": "'url/to/invalid/image'",
        "disableDepthTestDistance": "1000.0",
        "horizontalOrigin": "0",
        "verticalOrigin": "0",
        "labelHorizontalOrigin": "0",
        "labelVerticalOrigin": "0",
        "lineWidth": "10.0"
    });

    let doc = Cesium3DTileStyleDocument::from_json(&style_json).unwrap();
    assert!(doc.ready);
    assert_eq!(doc.style, style_json);

    let mut f1 = MockStyleFeature::new();
    f1.add("id", json!(11));

    assert_eq!(doc.color.as_ref().unwrap().evaluate_color(&f1).unwrap(), Color::RED);
    assert_eq!(
        doc.show.as_ref().unwrap().evaluate(&f1).unwrap(),
        StyleValue::Boolean(true)
    );
    assert_eq!(
        doc.point_size.as_ref().unwrap().evaluate(&f1).unwrap(),
        StyleValue::Number(0.11)
    );
    assert_eq!(
        doc.point_outline_color
            .as_ref()
            .unwrap()
            .evaluate_color(&f1)
            .unwrap(),
        Color::BLUE
    );
    assert_eq!(
        doc.label_color.as_ref().unwrap().evaluate_color(&f1).unwrap(),
        Color::YELLOW
    );
    assert_eq!(
        doc.anchor_line_color
            .as_ref()
            .unwrap()
            .evaluate_color(&f1)
            .unwrap(),
        Color::MAGENTA
    );
    assert_eq!(
        doc.line_width.as_ref().unwrap().evaluate(&f1).unwrap(),
        StyleValue::Number(10.0)
    );
}

#[test]
fn evaluates_conditions_defines_and_meta() {
    let style_json = json!({
        "defines": {
            "halfHeight": "${Height} / 2"
        },
        "color": {
            "conditions": [
                ["${halfHeight} >= 50", "color('red')"],
                ["true", "color('blue')"]
            ]
        },
        "show": "${Height} > 0",
        "meta": {
            "description": "'Building ${id} height ${Height}'"
        }
    });

    let doc = Cesium3DTileStyleDocument::from_json(&style_json).unwrap();
    assert!(matches!(doc.color, Some(TileStyleExpression::Conditions(_))));

    let mut tall = MockStyleFeature::new();
    tall.add("id", json!(11));
    tall.add("Height", json!(100));

    let mut short = MockStyleFeature::new();
    short.add("id", json!(12));
    short.add("Height", json!(38));

    assert_eq!(
        doc.color.as_ref().unwrap().evaluate_color(&tall).unwrap(),
        Color::RED
    );
    assert_eq!(
        doc.color.as_ref().unwrap().evaluate_color(&short).unwrap(),
        Color::BLUE
    );
    assert_eq!(
        doc.meta["description"].evaluate(&tall).unwrap(),
        StyleValue::String("Building 11 height 100".to_string())
    );
}
