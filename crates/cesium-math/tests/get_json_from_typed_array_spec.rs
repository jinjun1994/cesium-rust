use cesium_math::get_json_from_typed_array;
use serde_json::{json, Value};

#[test]
fn test_converts_typed_array_to_json() {
    let expected = json!({
        "a": [0, 1, 2],
        "b": "b",
        "c": {
            "d": true
        }
    });

    let s = serde_json::to_string(&expected).unwrap();
    let bytes = s.as_bytes();

    let result: Value = get_json_from_typed_array(bytes, None, None).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_converts_sub_region_of_typed_array_to_json() {
    let parent = json!({
        "a": [0, 1, 2],
        "b": "b",
        "c": {
            "d": true
        }
    });

    let s = serde_json::to_string(&parent).unwrap();
    let bytes = s.as_bytes();

    // Find the substring corresponding to {"d":true}
    let target = r#"{"d":true}"#;
    let start = s.find(target).expect("target substring should exist");
    let len = target.len();

    let result: Value = get_json_from_typed_array(bytes, Some(start), Some(len)).unwrap();
    assert_eq!(result, parent["c"]);
}
