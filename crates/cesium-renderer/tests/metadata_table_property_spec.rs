use cesium_renderer::{
    MetadataClassProperty, MetadataComponentType, MetadataTableProperty,
    MetadataTablePropertyOptions, MetadataType,
};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_creates_metadata_table_property() {
    let extras = json!({
        "other": 0
    });
    let extensions = json!({
        "EXT_other_extension": {}
    });

    let mut buffer_views = HashMap::new();
    buffer_views.insert("0".to_string(), vec![0u8; 8]);

    let class_prop = MetadataClassProperty {
        id: "property".to_string(),
        r#type: MetadataType::Scalar,
        component_type: Some(MetadataComponentType::Float32),
        ..Default::default()
    };

    let prop_json = json!({
        "values": 0,
        "extras": extras,
        "extensions": extensions
    });

    let property = MetadataTableProperty::new(MetadataTablePropertyOptions {
        count: 2,
        property: prop_json,
        class_property: class_prop,
        buffer_views,
    })
    .unwrap();

    assert_eq!(property.extras(), Some(&extras));
    assert_eq!(property.extensions(), Some(&extensions));
    assert_eq!(property.byte_length(), 8);
}

#[test]
fn test_constructs_properties_with_string_offsets_and_array_offsets() {
    let extras = json!({ "other": 0 });
    let extensions = json!({ "EXT_other_extension": {} });

    let mut buffer_views = HashMap::new();
    // bufferView 0: characters (5 bytes: a, b, c, d, e)
    buffer_views.insert("0".to_string(), vec![97u8, 98, 99, 100, 101]);
    // bufferView 1: string offsets (UINT16: 0, 2, 5) -> 3 * 2 = 6 bytes
    let mut str_offsets = vec![0u8; 6];
    str_offsets[0..2].copy_from_slice(&0u16.to_le_bytes());
    str_offsets[2..4].copy_from_slice(&2u16.to_le_bytes());
    str_offsets[4..6].copy_from_slice(&5u16.to_le_bytes());
    buffer_views.insert("1".to_string(), str_offsets);
    // bufferView 2: array offsets (UINT8: 0, 1, 2) -> 3 bytes
    buffer_views.insert("2".to_string(), vec![0u8, 1, 2]);

    let class_prop = MetadataClassProperty {
        id: "property".to_string(),
        r#type: MetadataType::String,
        is_array: true,
        is_variable_length_array: true,
        ..Default::default()
    };

    let prop_json = json!({
        "values": 0,
        "extras": extras,
        "extensions": extensions,
        "stringOffsetType": "UINT16",
        "stringOffsets": 1,
        "arrayOffsetType": "UINT8",
        "arrayOffsets": 2
    });

    let property = MetadataTableProperty::new(MetadataTablePropertyOptions {
        count: 2,
        property: prop_json,
        class_property: class_prop,
        buffer_views,
    })
    .unwrap();

    assert_eq!(property.extras(), Some(&extras));
    assert_eq!(property.extensions(), Some(&extensions));
    // 5 + 6 + 3 = 14 bytes
    assert_eq!(property.byte_length(), 14);
}

#[test]
fn test_constructor_throws_without_count() {
    let class_prop = MetadataClassProperty::default();
    let prop_json = json!({ "values": 0 });
    assert!(MetadataTableProperty::new(MetadataTablePropertyOptions {
        count: 0,
        property: prop_json,
        class_property: class_prop,
        buffer_views: HashMap::new(),
    })
    .is_err());
}

#[test]
fn test_get_and_set_scalar_values() {
    let mut buffer_views = HashMap::new();
    let mut raw_bytes = vec![0u8; 8];
    raw_bytes[0..4].copy_from_slice(&1.5f32.to_le_bytes());
    raw_bytes[4..8].copy_from_slice(&2.5f32.to_le_bytes());
    buffer_views.insert("0".to_string(), raw_bytes);

    let class_prop = MetadataClassProperty {
        id: "height".to_string(),
        r#type: MetadataType::Scalar,
        component_type: Some(MetadataComponentType::Float32),
        ..Default::default()
    };

    let mut property = MetadataTableProperty::new(MetadataTablePropertyOptions {
        count: 2,
        property: json!({ "values": 0 }),
        class_property: class_prop,
        buffer_views,
    })
    .unwrap();

    let val0 = property.get(0).unwrap();
    assert_eq!(val0.as_f64().unwrap(), 1.5);
    let val1 = property.get(1).unwrap();
    assert_eq!(val1.as_f64().unwrap(), 2.5);

    // Set new value
    property.set(0, json!(4.25)).unwrap();
    assert_eq!(property.get(0).unwrap().as_f64().unwrap(), 4.25);

    // Throws for NaN or Infinity
    assert!(property.set(0, json!(f64::NAN)).is_err());
    assert!(property.set(0, json!(f64::INFINITY)).is_err());
}
