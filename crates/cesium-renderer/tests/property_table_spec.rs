use std::collections::HashMap;

use cesium_renderer::batch_table_hierarchy::BatchTableHierarchy;
use cesium_renderer::json_metadata_table::JsonMetadataTable;
use cesium_renderer::metadata_class::MetadataClass;
use cesium_renderer::metadata_class_property::MetadataClassProperty;
use cesium_renderer::metadata_component_type::MetadataComponentType;
use cesium_renderer::metadata_table::{MetadataTable, MetadataTableOptions};
use cesium_renderer::metadata_type::MetadataType;
use cesium_renderer::property_table::{PropertyTable, PropertyTableOptions};
use serde_json::json;

fn create_sample_metadata_table() -> MetadataTable {
    let mut class_props = HashMap::new();
    let height_prop = MetadataClassProperty {
        id: "height".to_string(),
        r#type: MetadataType::Scalar,
        component_type: Some(MetadataComponentType::Float32),
        semantic: Some("HEIGHT".to_string()),
        ..Default::default()
    };
    let name_prop = MetadataClassProperty {
        id: "name".to_string(),
        r#type: MetadataType::String,
        semantic: Some("NAME".to_string()),
        ..Default::default()
    };
    class_props.insert("height".to_string(), height_prop.clone());
    class_props.insert("name".to_string(), name_prop.clone());

    let mut by_semantic = HashMap::new();
    by_semantic.insert("HEIGHT".to_string(), height_prop);
    by_semantic.insert("NAME".to_string(), name_prop);

    let class_def = MetadataClass {
        id: "test_class".to_string(),
        name: None,
        description: None,
        properties: class_props,
        properties_by_semantic: by_semantic,
        extras: None,
        extensions: None,
    };

    let mut height_bytes = Vec::new();
    for h in [10.0f32, 20.0f32, 30.0f32] {
        height_bytes.extend_from_slice(&h.to_le_bytes());
    }

    let names = ["Building A", "Building B", "Building C"];
    let mut string_bytes = Vec::new();
    let mut offset_bytes = Vec::new();
    let mut cur_offset: u32 = 0;
    offset_bytes.extend_from_slice(&cur_offset.to_le_bytes());
    for s in names {
        string_bytes.extend_from_slice(s.as_bytes());
        cur_offset += s.len() as u32;
        offset_bytes.extend_from_slice(&cur_offset.to_le_bytes());
    }

    let mut buffer_views = HashMap::new();
    buffer_views.insert("0".to_string(), height_bytes);
    buffer_views.insert("1".to_string(), string_bytes);
    buffer_views.insert("2".to_string(), offset_bytes);

    let mut properties = HashMap::new();
    properties.insert(
        "height".to_string(),
        json!({
            "values": 0
        }),
    );
    properties.insert(
        "name".to_string(),
        json!({
            "values": 1,
            "stringOffsets": 2,
            "stringOffsetType": "UINT32"
        }),
    );

    MetadataTable::new(MetadataTableOptions {
        count: 3,
        class_definition: Some(class_def),
        properties: Some(properties),
        buffer_views: Some(buffer_views),
    })
    .unwrap()
}

fn create_sample_hierarchy() -> BatchTableHierarchy {
    let ext = json!({
        "classes": [
            {
                "name": "Wheels",
                "length": 2,
                "instances": {
                    "tireLocation": ["front", "back"]
                }
            },
            {
                "name": "Car",
                "length": 1,
                "instances": {
                    "color": ["blue"],
                    "type": ["sedan"],
                    "year": ["2020"]
                }
            }
        ],
        "instancesLength": 3,
        "classIds": [0, 0, 1],
        "parentCounts": [1, 1, 0],
        "parentIds": [2, 2]
    });
    BatchTableHierarchy::new(Some(&ext), None).unwrap()
}

#[test]
fn test_creates_property_table_with_default_values() {
    let table = PropertyTable::new(PropertyTableOptions {
        count: 3,
        ..Default::default()
    });
    assert_eq!(table.count(), 3);
    assert!(table.class().is_none());
    assert!(table.extras().is_none());
    assert!(table.extensions().is_none());
    assert_eq!(table.byte_length(), 0);
}

#[test]
fn test_creates_property_table_with_metadata_table() {
    let mt = create_sample_metadata_table();
    let extras = json!({"description": "Extra"});
    let extensions = json!({"EXT_other": {}});

    let table = PropertyTable::new(PropertyTableOptions {
        name: Some("Buildings".to_string()),
        id: Some(json!(0)),
        count: 3,
        metadata_table: Some(mt),
        extras: Some(extras.clone()),
        extensions: Some(extensions.clone()),
        ..Default::default()
    });

    assert_eq!(table.count(), 3);
    assert_eq!(table.name(), Some("Buildings"));
    assert_eq!(table.id(), Some(&json!(0)));
    assert!(table.class().is_some());
    assert_eq!(table.get_property_ids(None).len(), 2);
    assert_eq!(table.get_property(0, "name"), Some(json!("Building A")));
    assert_eq!(table.get_property(1, "name"), Some(json!("Building B")));
    assert_eq!(table.get_property(2, "name"), Some(json!("Building C")));
    assert_eq!(table.get_property(0, "height"), Some(json!(10.0)));
    assert_eq!(table.get_property(1, "height"), Some(json!(20.0)));
    assert_eq!(table.get_property(2, "height"), Some(json!(30.0)));
    assert_eq!(table.extras(), Some(&extras));
    assert_eq!(table.extensions(), Some(&extensions));
}

#[test]
fn test_has_property_and_has_property_by_semantic() {
    let mt = create_sample_metadata_table();
    let table = PropertyTable::new(PropertyTableOptions {
        count: 3,
        metadata_table: Some(mt),
        ..Default::default()
    });

    assert!(table.has_property(0, "name").unwrap());
    assert!(!table.has_property(0, "numberOfPoints").unwrap());
    assert!(table.has_property(0, "").is_err());

    assert!(table.has_property_by_semantic(0, "NAME").unwrap());
    assert!(!table.has_property_by_semantic(0, "ID").unwrap());
    assert!(table.has_property_by_semantic(0, "").is_err());
}

#[test]
fn test_property_exists_and_property_exists_by_semantic() {
    let mt = create_sample_metadata_table();
    let table = PropertyTable::new(PropertyTableOptions {
        count: 3,
        metadata_table: Some(mt),
        ..Default::default()
    });

    assert!(table.property_exists("name").unwrap());
    assert!(!table.property_exists("numberOfPoints").unwrap());
    assert!(table.property_exists("").is_err());

    assert!(table.property_exists_by_semantic("NAME").unwrap());
    assert!(!table.property_exists_by_semantic("ID").unwrap());
    assert!(table.property_exists_by_semantic("").is_err());
}

#[test]
fn test_set_property_dynamically_creates_json_metadata_table() {
    let mt = create_sample_metadata_table();
    let mut table = PropertyTable::new(PropertyTableOptions {
        count: 3,
        metadata_table: Some(mt),
        ..Default::default()
    });

    assert!(table.json_metadata_table().is_none());
    assert!(table.get_property(0, "numberOfPoints").is_none());

    table.set_property(0, "numberOfPoints", json!(10)).unwrap();
    assert!(table.json_metadata_table().is_some());
    assert_eq!(table.get_property(0, "numberOfPoints"), Some(json!(10)));

    // update existing
    table.set_property(0, "name", json!("Building New")).unwrap();
    assert_eq!(table.get_property(0, "name"), Some(json!("Building New")));
}

#[test]
fn test_combined_property_table_with_hierarchy_and_json() {
    let mt = create_sample_metadata_table();
    let hierarchy = create_sample_hierarchy();

    let mut json_props = HashMap::new();
    json_props.insert("priority".to_string(), vec![json!(2), json!(1), json!(0)]);
    json_props.insert(
        "uri".to_string(),
        vec![json!("tree.las"), json!("building.gltf"), json!("map.tif")],
    );
    let jmt = JsonMetadataTable::new(3, json_props).unwrap();

    let mut table = PropertyTable::new(PropertyTableOptions {
        count: 3,
        metadata_table: Some(mt),
        json_metadata_table: Some(jmt),
        batch_table_hierarchy: Some(hierarchy),
        ..Default::default()
    });

    // byte length
    assert!(table.byte_length() > 0);

    // property IDs for feature 0
    let mut ids = table.get_property_ids(Some(0));
    ids.sort();
    assert_eq!(
        ids,
        vec!["color", "height", "name", "priority", "tireLocation", "type", "uri", "year"]
    );

    // has_property hierarchy
    assert!(table.has_property(0, "tireLocation").unwrap());
    assert!(table.has_property(0, "color").unwrap());
    assert!(table.has_property(0, "type").unwrap());
    assert!(table.has_property(0, "year").unwrap());
    assert!(table.has_property(1, "tireLocation").unwrap());
    assert!(!table.has_property(2, "tireLocation").unwrap());
    assert!(table.has_property(2, "color").unwrap());

    // property_exists
    assert!(table.property_exists("tireLocation").unwrap());
    assert!(table.property_exists("priority").unwrap());
    assert!(table.property_exists("height").unwrap());
    assert!(!table.property_exists("widgets").unwrap());

    // get_property from all three tiers
    assert_eq!(table.get_property(0, "height"), Some(json!(10.0)));
    assert_eq!(table.get_property(0, "priority"), Some(json!(2)));
    assert_eq!(table.get_property(0, "tireLocation"), Some(json!("front")));
    assert_eq!(table.get_property(0, "color"), Some(json!("blue")));
    assert_eq!(table.get_property(2, "tireLocation"), None);
    assert_eq!(table.get_property(2, "color"), Some(json!("blue")));

    // set_property hierarchy
    table.set_property(0, "tireLocation", json!("back")).unwrap();
    table.set_property(2, "color", json!("navy")).unwrap();
    assert_eq!(table.get_property(0, "tireLocation"), Some(json!("back")));
    assert_eq!(table.get_property(0, "color"), Some(json!("navy")));
    assert_eq!(table.get_property(2, "color"), Some(json!("navy")));

    // set_property for new property
    table.set_property(0, "widgets", json!(5)).unwrap();
    assert_eq!(table.get_property(0, "widgets"), Some(json!(5)));

    // typed array
    assert!(table.get_property_typed_array("height").is_some());
    assert!(table.get_property_typed_array("priority").is_none());
    assert!(table.get_property_typed_array("tireLocation").is_none());

    // is_class
    assert!(table.is_class(0, "Wheels").unwrap());
    assert!(table.is_class(0, "Car").unwrap());
    assert!(!table.is_class(0, "Wings").unwrap());
    assert!(table.is_class(3, "Wheels").is_err()); // out of bounds

    // is_exact_class
    assert!(table.is_exact_class(0, "Wheels").unwrap());
    assert!(!table.is_exact_class(0, "Car").unwrap());
    assert!(table.is_exact_class(2, "Car").unwrap());
    assert!(!table.is_exact_class(2, "Wheels").unwrap());

    // get_exact_class_name
    assert_eq!(table.get_exact_class_name(0).unwrap(), Some("Wheels".to_string()));
    assert_eq!(table.get_exact_class_name(2).unwrap(), Some("Car".to_string()));
    assert!(table.get_exact_class_name(3).is_err());
}
