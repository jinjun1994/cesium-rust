use cesium_renderer::MetadataEnumValue;
use serde_json::json;

#[test]
fn test_creates_enum_value() {
    let extras = json!({
        "gain": 0.5,
        "offset": 0.1
    });
    let extensions = json!({
        "EXT_other_extension": {}
    });

    let json_value = json!({
        "name": "RED",
        "description": "Red color",
        "value": 0,
        "extras": extras,
        "extensions": extensions
    });

    let enum_value = MetadataEnumValue::from_json(Some(&json_value)).unwrap();

    assert_eq!(enum_value.name, "RED");
    assert_eq!(enum_value.description.as_deref(), Some("Red color"));
    assert_eq!(enum_value.value, 0);
    assert_eq!(enum_value.extras, Some(extras));
    assert_eq!(enum_value.extensions, Some(extensions));
}

#[test]
fn test_constructor_throws_without_value() {
    assert!(MetadataEnumValue::from_json(None).is_err());
}
