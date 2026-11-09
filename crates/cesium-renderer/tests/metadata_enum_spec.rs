use cesium_renderer::{MetadataComponentType, MetadataEnum};
use serde_json::json;

#[test]
fn test_creates_enum_with_default_values() {
    let json_options = json!({
        "id": "color",
        "enum": {
            "values": [
                {
                    "name": "RED",
                    "value": 0
                },
                {
                    "name": "GREEN",
                    "value": 1
                },
                {
                    "name": "BLUE",
                    "value": 2
                }
            ]
        }
    });

    let color_enum = MetadataEnum::from_json_options(&json_options).unwrap();

    assert_eq!(color_enum.values[0].name, "RED");
    assert_eq!(color_enum.values[1].name, "GREEN");
    assert_eq!(color_enum.values[2].name, "BLUE");
    assert_eq!(color_enum.value_type, MetadataComponentType::Uint16);
    assert_eq!(color_enum.id, "color");
    assert_eq!(color_enum.name, None);
    assert_eq!(color_enum.description, None);
    assert_eq!(color_enum.extras, None);
    assert_eq!(color_enum.extensions, None);
}

#[test]
fn test_creates_enum() {
    let extras = json!({
        "gain": 0.5,
        "offset": 0.1
    });
    let extensions = json!({
        "EXT_other_extension": {}
    });

    let json_options = json!({
        "id": "color",
        "enum": {
            "name": "Color",
            "description": "Common colors",
            "extras": extras,
            "extensions": extensions,
            "valueType": "UINT64",
            "values": [
                {
                    "name": "RED",
                    "value": 0
                },
                {
                    "name": "GREEN",
                    "value": 1
                },
                {
                    "name": "BLUE",
                    "value": 2
                }
            ]
        }
    });

    let color_enum = MetadataEnum::from_json_options(&json_options).unwrap();

    assert_eq!(color_enum.values[0].name, "RED");
    assert_eq!(color_enum.values[1].name, "GREEN");
    assert_eq!(color_enum.values[2].name, "BLUE");
    assert_eq!(color_enum.value_type, MetadataComponentType::Uint64);
    assert_eq!(color_enum.id, "color");
    assert_eq!(color_enum.name.as_deref(), Some("Color"));
    assert_eq!(color_enum.description.as_deref(), Some("Common colors"));
    assert_eq!(color_enum.extras, Some(extras));
    assert_eq!(color_enum.extensions, Some(extensions));
}

#[test]
fn test_constructor_throws_without_id() {
    let json_options = json!({
        "enum": {
            "values": [
                {
                    "name": "RED",
                    "value": 0
                }
            ]
        }
    });

    assert!(MetadataEnum::from_json_options(&json_options).is_err());
}

#[test]
fn test_constructor_throws_without_enum() {
    let json_options = json!({
        "id": "enumId"
    });

    assert!(MetadataEnum::from_json_options(&json_options).is_err());
}
