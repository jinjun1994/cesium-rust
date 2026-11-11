//! Spec matching CesiumJS `Specs/Scene/parseBatchTableSpec.js` (Apache-2.0).

use cesium_renderer::{parse_batch_table, MetadataClass, ParseBatchTableOptions};
use serde_json::json;

#[test]
fn test_throws_without_count() {
    let res = parse_batch_table(ParseBatchTableOptions {
        count: None,
        batch_table: Some(json!({})),
        binary_body: None,
        parse_as_property_attributes: None,
    });
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().message, "options.count is required.");
}

#[test]
fn test_throws_without_batch_table() {
    let res = parse_batch_table(ParseBatchTableOptions {
        count: Some(3),
        batch_table: None,
        binary_body: None,
        parse_as_property_attributes: None,
    });
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().message, "options.batchTable is required.");
}

#[test]
fn test_parses_batch_table_with_no_properties() {
    let metadata = parse_batch_table(ParseBatchTableOptions {
        count: Some(3),
        batch_table: Some(json!({})),
        binary_body: None,
        parse_as_property_attributes: None,
    })
    .expect("Should parse empty batch table");

    assert!(metadata.property_tables.is_empty());
}

#[test]
fn test_parses_batch_table_with_binary_properties() {
    let binary_batch_table = json!({
        "height": {
            "byteOffset": 0,
            "componentType": "FLOAT",
            "type": "SCALAR"
        }
    });

    let height_values: [f32; 3] = [10.0, 15.0, 25.0];
    let mut binary_body = Vec::new();
    for v in height_values {
        binary_body.extend_from_slice(&v.to_le_bytes());
    }

    let metadata = parse_batch_table(ParseBatchTableOptions {
        count: Some(3),
        batch_table: Some(binary_batch_table),
        binary_body: Some(&binary_body),
        parse_as_property_attributes: None,
    })
    .expect("Should parse binary batch table");

    assert_eq!(metadata.property_tables.len(), 1);
    let property_table = &metadata.property_tables[0];
    assert_eq!(property_table.id(), Some(&json!(0)));
    assert_eq!(property_table.name(), Some("Batch Table"));

    assert_eq!(property_table.get_property(0, "height"), Some(json!(10.0)));
    assert_eq!(property_table.get_property(1, "height"), Some(json!(15.0)));
    assert_eq!(property_table.get_property(2, "height"), Some(json!(25.0)));
}

#[test]
fn test_transcodes_scalars_to_correct_types() {
    let binary_batch_table = json!({
        "uint8Property": {
            "byteOffset": 0,
            "componentType": "UNSIGNED_BYTE",
            "type": "SCALAR"
        },
        "uint16Property": {
            "byteOffset": 0,
            "componentType": "UNSIGNED_SHORT",
            "type": "SCALAR"
        },
        "uint32Property": {
            "byteOffset": 0,
            "componentType": "UNSIGNED_INT",
            "type": "SCALAR"
        },
        "int8Property": {
            "byteOffset": 0,
            "componentType": "BYTE",
            "type": "SCALAR"
        },
        "int16Property": {
            "byteOffset": 0,
            "componentType": "SHORT",
            "type": "SCALAR"
        },
        "int32Property": {
            "byteOffset": 0,
            "componentType": "INT",
            "type": "SCALAR"
        },
        "float32Property": {
            "byteOffset": 0,
            "componentType": "FLOAT",
            "type": "SCALAR"
        },
        "float64Property": {
            "byteOffset": 0,
            "componentType": "DOUBLE",
            "type": "SCALAR"
        }
    });

    let binary_body = vec![0u8; 128];

    let metadata = parse_batch_table(ParseBatchTableOptions {
        count: Some(3),
        batch_table: Some(binary_batch_table),
        binary_body: Some(&binary_body),
        parse_as_property_attributes: None,
    })
    .expect("Should parse and transcode scalars");

    let class_def = metadata
        .schema
        .classes
        .get(MetadataClass::BATCH_TABLE_CLASS_NAME)
        .expect("Batch table class should exist in schema");

    let u8_prop = class_def.properties.get("uint8Property").unwrap();
    assert_eq!(format!("{:?}", u8_prop.component_type.unwrap()), "Uint8");

    let f32_prop = class_def.properties.get("float32Property").unwrap();
    assert_eq!(format!("{:?}", f32_prop.component_type.unwrap()), "Float32");

    let f64_prop = class_def.properties.get("float64Property").unwrap();
    assert_eq!(format!("{:?}", f64_prop.component_type.unwrap()), "Float64");
}

#[test]
fn test_parses_json_properties() {
    let json_batch_table = json!({
        "names": ["alpha", "beta", "gamma"],
        "numbers": [1, 2, 3]
    });

    let metadata = parse_batch_table(ParseBatchTableOptions {
        count: Some(3),
        batch_table: Some(json_batch_table),
        binary_body: None,
        parse_as_property_attributes: None,
    })
    .expect("Should parse json properties");

    assert_eq!(metadata.property_tables.len(), 1);
    let pt = &metadata.property_tables[0];
    assert_eq!(pt.get_property(0, "names"), Some(json!("alpha")));
    assert_eq!(pt.get_property(1, "numbers"), Some(json!(2)));
}
