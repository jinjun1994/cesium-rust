use cesium_renderer::{
    MetadataClass, MetadataComponentType, MetadataEnum, MetadataEnumJson, MetadataEnumValue,
    MetadataType,
};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_creates_class_with_default_values() {
    let building_class = MetadataClass::from_json(&json!({
        "id": "building",
        "class": {},
    }))
    .unwrap();

    assert_eq!(building_class.id, "building");
    assert!(building_class.properties.is_empty());
    assert!(building_class.properties_by_semantic.is_empty());
    assert_eq!(building_class.name, None);
    assert_eq!(building_class.description, None);
    assert_eq!(building_class.extras, None);
    assert_eq!(building_class.extensions, None);
}

#[test]
fn test_creates_class() {
    let extras = json!({
        "cityInfo": {
            "name": "city"
        }
    });
    let extensions = json!({
        "EXT_other_extension": {}
    });

    let building_class = MetadataClass::from_json(&json!({
        "id": "building",
        "class": {
            "name": "Building",
            "description": "Building Class",
            "extras": extras,
            "extensions": extensions,
            "properties": {
                "height": {
                    "type": "SCALAR",
                    "componentType": "FLOAT32"
                },
                "position": {
                    "type": "SCALAR",
                    "componentType": "FLOAT32",
                    "array": true,
                    "count": 3,
                    "semantic": "_POSITION"
                },
                "color": {
                    "type": "STRING",
                    "semantic": "_COLOR"
                }
            }
        }
    }))
    .unwrap();

    assert_eq!(building_class.id, "building");
    assert_eq!(building_class.name.as_deref(), Some("Building"));
    assert_eq!(
        building_class.description.as_deref(),
        Some("Building Class")
    );
    assert_eq!(building_class.extras, Some(extras));
    assert_eq!(building_class.extensions, Some(extensions));

    let properties = &building_class.properties;
    let height_prop = &properties["height"];
    let position_prop = &properties["position"];
    let color_prop = &properties["color"];

    assert_eq!(height_prop.r#type, MetadataType::Scalar);
    assert_eq!(
        height_prop.component_type,
        Some(MetadataComponentType::Float32)
    );

    assert_eq!(position_prop.r#type, MetadataType::Scalar);
    assert_eq!(
        position_prop.component_type,
        Some(MetadataComponentType::Float32)
    );

    assert_eq!(color_prop.r#type, MetadataType::String);
    assert_eq!(color_prop.component_type, None);

    let mut keys: Vec<&String> = properties.keys().collect();
    keys.sort();
    assert_eq!(keys, vec!["color", "height", "position"]);

    let by_semantic = &building_class.properties_by_semantic;
    assert_eq!(by_semantic["_COLOR"], *color_prop);
    assert_eq!(by_semantic["_POSITION"], *position_prop);

    let mut sem_keys: Vec<&String> = by_semantic.keys().collect();
    sem_keys.sort();
    assert_eq!(sem_keys, vec!["_COLOR", "_POSITION"]);
}

#[test]
fn test_creates_class_with_enum_property() {
    let color_enum = MetadataEnum::from_json(
        "color".to_string(),
        MetadataEnumJson {
            values: vec![MetadataEnumValue {
                name: "RED".to_string(),
                value: 0,
                description: None,
                extras: None,
                extensions: None,
            }],
            ..Default::default()
        },
    )
    .unwrap();

    let mut enums = HashMap::new();
    enums.insert("color".to_string(), color_enum.clone());

    let building_class = MetadataClass::from_json_with_enums(
        &json!({
            "id": "building",
            "class": {
                "properties": {
                    "color": {
                        "type": "ENUM",
                        "enumType": "color"
                    }
                }
            }
        }),
        Some(&enums),
    )
    .unwrap();

    let color_prop = &building_class.properties["color"];
    assert_eq!(color_prop.r#type, MetadataType::Enum);
    assert_eq!(color_prop.component_type, None);
    assert_eq!(color_prop.enum_type, Some(color_enum));
}

#[test]
fn test_constructor_throws_without_id() {
    let res = MetadataClass::from_json(&json!({
        "class": {}
    }));
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().message, "options.id is required.");
}

#[test]
fn test_constructor_throws_without_class() {
    let res = MetadataClass::from_json(&json!({
        "id": "classId"
    }));
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().message, "options.class is required.");
}
