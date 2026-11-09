use cesium_renderer::{
    MetadataClass, MetadataClassProperty, MetadataComponentType, MetadataTable,
    MetadataTableOptions, MetadataType,
};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_creates_metadata_table_with_default_values() {
    let empty_class = MetadataClass {
        id: "empty".to_string(),
        name: None,
        description: None,
        properties: HashMap::new(),
        properties_by_semantic: HashMap::new(),
        extras: None,
        extensions: None,
    };

    let metadata_table = MetadataTable::new(MetadataTableOptions {
        count: 10,
        class_definition: Some(empty_class),
        properties: None,
        buffer_views: None,
    })
    .unwrap();

    assert_eq!(metadata_table.count(), 10);
    assert_eq!(metadata_table.byte_length(), 0);
}

#[test]
fn test_creates_metadata_table_with_properties() {
    let mut class_props = HashMap::new();
    let height_prop = MetadataClassProperty {
        id: "height".to_string(),
        r#type: MetadataType::Scalar,
        component_type: Some(MetadataComponentType::Float32),
        semantic: Some("HEIGHT_SEMANTIC".to_string()),
        ..Default::default()
    };
    let name_prop = MetadataClassProperty {
        id: "name".to_string(),
        r#type: MetadataType::String,
        ..Default::default()
    };
    class_props.insert("height".to_string(), height_prop.clone());
    class_props.insert("name".to_string(), name_prop.clone());

    let mut by_semantic = HashMap::new();
    by_semantic.insert("HEIGHT_SEMANTIC".to_string(), height_prop);

    let class_def = MetadataClass {
        id: "test_class".to_string(),
        name: None,
        description: None,
        properties: class_props,
        properties_by_semantic: by_semantic,
        extras: None,
        extensions: None,
    };

    let mut buffer_views = HashMap::new();
    // height: 2 float32 values (1.0, 2.0) -> 8 bytes
    let mut height_bytes = vec![0u8; 8];
    height_bytes[0..4].copy_from_slice(&1.0f32.to_le_bytes());
    height_bytes[4..8].copy_from_slice(&2.0f32.to_le_bytes());
    buffer_views.insert("0".to_string(), height_bytes);

    // name: 2 characters ("AB") -> 2 bytes
    buffer_views.insert("1".to_string(), vec![b'A', b'B']);
    // name string offsets: UINT32 (0, 1, 2) -> (2 + 1) * 4 = 12 bytes
    let mut name_offsets = vec![0u8; 12];
    name_offsets[0..4].copy_from_slice(&0u32.to_le_bytes());
    name_offsets[4..8].copy_from_slice(&1u32.to_le_bytes());
    name_offsets[8..12].copy_from_slice(&2u32.to_le_bytes());
    buffer_views.insert("2".to_string(), name_offsets);

    let mut properties = HashMap::new();
    properties.insert("height".to_string(), json!({ "values": 0 }));
    properties.insert(
        "name".to_string(),
        json!({
            "values": 1,
            "stringOffsets": 2,
            "stringOffsetType": "UINT32"
        }),
    );

    let mut metadata_table = MetadataTable::new(MetadataTableOptions {
        count: 2,
        class_definition: Some(class_def),
        properties: Some(properties),
        buffer_views: Some(buffer_views),
    })
    .unwrap();

    let expected_property_names = vec!["height".to_string(), "name".to_string()];
    let mut ids = metadata_table.get_property_ids();
    ids.sort();
    assert_eq!(ids, expected_property_names);

    // heightSize (8) + nameCharArraySize (2) + nameOffsetSize (12) = 22
    assert_eq!(metadata_table.byte_length(), 22);

    // has_property checks
    assert!(metadata_table.has_property("height").unwrap());
    assert!(metadata_table.has_property("name").unwrap());
    assert!(!metadata_table.has_property("non_existent").unwrap());

    // has_property_by_semantic checks
    assert!(metadata_table.has_property_by_semantic("HEIGHT_SEMANTIC").unwrap());
    assert!(!metadata_table.has_property_by_semantic("NON_EXISTENT").unwrap());

    // get_property checks
    assert_eq!(metadata_table.get_property(0, "height").unwrap(), Some(json!(1.0)));
    assert_eq!(metadata_table.get_property(1, "height").unwrap(), Some(json!(2.0)));
    assert_eq!(metadata_table.get_property(0, "name").unwrap(), Some(json!("A")));
    assert_eq!(metadata_table.get_property(1, "name").unwrap(), Some(json!("B")));

    // get_property_by_semantic
    assert_eq!(
        metadata_table.get_property_by_semantic(0, "HEIGHT_SEMANTIC").unwrap(),
        Some(json!(1.0))
    );

    // set_property checks
    assert!(metadata_table.set_property(0, "height", json!(5.0)).unwrap());
    assert_eq!(metadata_table.get_property(0, "height").unwrap(), Some(json!(5.0)));
    assert!(!metadata_table.set_property(0, "non_existent", json!(10.0)).unwrap());

    // set_property_by_semantic
    assert!(metadata_table.set_property_by_semantic(1, "HEIGHT_SEMANTIC", json!(6.0)).unwrap());
    assert_eq!(metadata_table.get_property(1, "height").unwrap(), Some(json!(6.0)));

    // get_property_typed_array
    let arr = metadata_table.get_property_typed_array("height").unwrap();
    assert!(arr.is_some());
    assert_eq!(arr.unwrap().len(), 8);

    let arr_sem = metadata_table.get_property_typed_array_by_semantic("HEIGHT_SEMANTIC").unwrap();
    assert!(arr_sem.is_some());
}

#[test]
fn test_constructor_throws_without_count_or_class() {
    assert!(MetadataTable::new(MetadataTableOptions {
        count: 0,
        class_definition: Some(MetadataClass {
            id: "test".to_string(),
            name: None,
            description: None,
            properties: HashMap::new(),
            properties_by_semantic: HashMap::new(),
            extras: None,
            extensions: None,
        }),
        properties: None,
        buffer_views: None,
    })
    .is_err());

    assert!(MetadataTable::new(MetadataTableOptions {
        count: 1,
        class_definition: None,
        properties: None,
        buffer_views: None,
    })
    .is_err());
}
