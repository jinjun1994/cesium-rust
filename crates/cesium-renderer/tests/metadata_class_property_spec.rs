use cesium_renderer::{
    MetadataClassProperty, MetadataComponentType, MetadataEnum, MetadataEnumJson,
    MetadataEnumValue, MetadataType,
};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_creates_property_with_default_values() {
    let prop_json = json!({
        "type": "SCALAR",
        "componentType": "FLOAT32"
    });

    let property =
        MetadataClassProperty::from_json("height".to_string(), &prop_json, None).unwrap();

    assert_eq!(property.id, "height");
    assert_eq!(property.name, None);
    assert_eq!(property.description, None);
    assert_eq!(property.r#type, MetadataType::Scalar);
    assert_eq!(property.enum_type, None);
    assert_eq!(
        property.component_type,
        Some(MetadataComponentType::Float32)
    );
    assert_eq!(property.value_type, Some(MetadataComponentType::Float32));
    assert!(!property.is_array);
    assert!(!property.is_variable_length_array);
    assert_eq!(property.array_length, None);
    assert!(!property.normalized);
    assert_eq!(property.max, None);
    assert_eq!(property.min, None);
    assert_eq!(property.default, None);
    assert!(!property.required);
    assert_eq!(property.semantic, None);
    assert_eq!(property.extras, None);
    assert_eq!(property.extensions, None);
    assert!(!property.is_legacy_extension);
}

#[test]
fn test_creates_property() {
    let max = json!([32767, 0, 100]);
    let min = json!([-32768, 0, -100]);
    let property_default = json!([0, 0, 0]);
    let extras = json!({"coordinates": [0, 1, 2]});
    let extensions = json!({"EXT_other_extension": {}});

    let prop_json = json!({
        "name": "Position",
        "description": "Position (X, Y, Z)",
        "array": true,
        "count": 3,
        "type": "SCALAR",
        "componentType": "INT16",
        "normalized": true,
        "max": max,
        "min": min,
        "default": property_default,
        "required": true,
        "semantic": "_POSITION",
        "extras": extras,
        "extensions": extensions,
    });

    let property =
        MetadataClassProperty::from_json("position".to_string(), &prop_json, None).unwrap();

    assert_eq!(property.id, "position");
    assert_eq!(property.name.as_deref(), Some("Position"));
    assert_eq!(
        property.description.as_deref(),
        Some("Position (X, Y, Z)")
    );
    assert_eq!(property.r#type, MetadataType::Scalar);
    assert_eq!(property.enum_type, None);
    assert_eq!(property.component_type, Some(MetadataComponentType::Int16));
    assert_eq!(property.value_type, Some(MetadataComponentType::Int16));
    assert!(property.is_array);
    assert!(!property.is_variable_length_array);
    assert_eq!(property.array_length, Some(3));
    assert!(property.normalized);
    assert_eq!(property.max, Some(max));
    assert_eq!(property.min, Some(min));
    assert_eq!(property.default, Some(property_default));
    assert!(property.required);
    assert_eq!(property.semantic.as_deref(), Some("_POSITION"));
    assert_eq!(property.extras, Some(extras));
    assert_eq!(property.extensions, Some(extensions));
    assert!(!property.is_legacy_extension);
}

#[test]
fn test_enum_property() {
    let color_enum = MetadataEnum::from_json(
        "color".to_string(),
        MetadataEnumJson {
            name: Some("Color".to_string()),
            description: None,
            value_type: Some("UINT8".to_string()),
            values: vec![
                MetadataEnumValue {
                    name: "RED".to_string(),
                    value: 0,
                    description: None,
                    extras: None,
                    extensions: None,
                },
                MetadataEnumValue {
                    name: "GREEN".to_string(),
                    value: 1,
                    description: None,
                    extras: None,
                    extensions: None,
                },
            ],
            extras: None,
            extensions: None,
        },
    )
    .unwrap();

    let mut enums = HashMap::new();
    enums.insert("color".to_string(), color_enum.clone());

    let prop_json = json!({
        "type": "ENUM",
        "enumType": "color"
    });

    let property =
        MetadataClassProperty::from_json("colorProp".to_string(), &prop_json, Some(&enums))
            .unwrap();

    assert_eq!(property.r#type, MetadataType::Enum);
    assert_eq!(property.enum_type, Some(color_enum));
    assert_eq!(property.value_type, Some(MetadataComponentType::Uint8));
}

#[test]
fn test_is_gpu_compatible() {
    // Scalar FLOAT32 -> 4 bytes -> compatible with 4 channels
    let float_prop = MetadataClassProperty::from_json(
        "val".to_string(),
        &json!({"type": "SCALAR", "componentType": "FLOAT32"}),
        None,
    )
    .unwrap();
    assert!(float_prop.is_gpu_compatible(4).unwrap());
    assert!(!float_prop.is_gpu_compatible(2).unwrap());

    // Boolean -> not compatible
    let bool_prop = MetadataClassProperty::from_json(
        "flag".to_string(),
        &json!({"type": "BOOLEAN"}),
        None,
    )
    .unwrap();
    assert!(!bool_prop.is_gpu_compatible(4).unwrap());

    // String -> not compatible
    let str_prop =
        MetadataClassProperty::from_json("str".to_string(), &json!({"type": "STRING"}), None)
            .unwrap();
    assert!(!str_prop.is_gpu_compatible(4).unwrap());

    // Channels length 0 throws DeveloperError
    assert!(float_prop.is_gpu_compatible(0).is_err());
}

#[test]
fn test_get_glsl_type() {
    let scalar_float = MetadataClassProperty::from_json(
        "f".to_string(),
        &json!({"type": "SCALAR", "componentType": "FLOAT32"}),
        None,
    )
    .unwrap();
    assert_eq!(scalar_float.get_glsl_type(), "float");

    let vec3_float = MetadataClassProperty::from_json(
        "v3".to_string(),
        &json!({"type": "VEC3", "componentType": "FLOAT32"}),
        None,
    )
    .unwrap();
    assert_eq!(vec3_float.get_glsl_type(), "vec3");

    let scalar_int = MetadataClassProperty::from_json(
        "i".to_string(),
        &json!({"type": "SCALAR", "componentType": "INT32"}),
        None,
    )
    .unwrap();
    assert_eq!(scalar_int.get_glsl_type(), "int");

    let scalar_uint = MetadataClassProperty::from_json(
        "u".to_string(),
        &json!({"type": "SCALAR", "componentType": "UINT32"}),
        None,
    )
    .unwrap();
    assert_eq!(scalar_uint.get_glsl_type(), "uint");
}

#[test]
fn test_validate() {
    let required_prop = MetadataClassProperty::from_json(
        "req".to_string(),
        &json!({"type": "SCALAR", "componentType": "FLOAT32", "required": true}),
        None,
    )
    .unwrap();
    assert!(required_prop.validate(&json!(null)).is_err());
    assert!(required_prop.validate(&json!(12.5)).is_ok());

    let array_prop = MetadataClassProperty::from_json(
        "arr".to_string(),
        &json!({"type": "SCALAR", "componentType": "FLOAT32", "array": true, "count": 2}),
        None,
    )
    .unwrap();
    assert!(array_prop.validate(&json!([1.0, 2.0])).is_ok());
    assert!(array_prop.validate(&json!([1.0, 2.0, 3.0])).is_err());
}
