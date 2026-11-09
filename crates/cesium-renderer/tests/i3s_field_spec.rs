use cesium_renderer::i3s_field::{
    I3SAttributeStorageInfo, I3SField, I3SHeaderItem, I3SValueDesc,
};
use serde_json::Value;
use std::collections::HashMap;

#[test]
fn validate_field_header_with_invalid_buffer_size() {
    let field = I3SField::new(I3SAttributeStorageInfo {
        name: "testField".to_string(),
        header: vec![I3SHeaderItem {
            property: "count".to_string(),
            value_type: "UInt32".to_string(),
        }],
        ..Default::default()
    });

    let short_buffer = vec![0u8; 3];
    let err = field.validate_header(&short_buffer).unwrap_err();
    assert!(err.message.contains("Invalid attribute buffer size"));
}

#[test]
fn validate_field_body_without_attribute_byte_counts() {
    let mut custom = HashMap::new();
    custom.insert(
        "testAttr".to_string(),
        I3SValueDesc {
            value_type: "String".to_string(),
            ..Default::default()
        },
    );

    let mut field = I3SField::new(I3SAttributeStorageInfo {
        name: "testField".to_string(),
        ordering: vec!["testAttr".to_string()],
        custom_props: custom,
        ..Default::default()
    });

    let mut header = HashMap::new();
    header.insert("count".to_string(), 1);
    field.header = Some(header);

    let buffer = vec![0u8; 5];
    let err = field.validate_body(&buffer, 4).unwrap_err();
    assert!(err.message.contains("attributeByteCounts is missing"));
}

#[test]
fn validate_field_body_with_invalid_offset() {
    let mut field = I3SField::new(I3SAttributeStorageInfo {
        name: "testField".to_string(),
        ordering: Vec::new(),
        ..Default::default()
    });

    let mut header = HashMap::new();
    header.insert("count".to_string(), 1);
    field.header = Some(header);

    let buffer = vec![0u8; 3];
    let err = field.validate_body(&buffer, 4).unwrap_err();
    assert!(err.message.contains("Invalid attribute buffer size"));
}

#[test]
fn validate_field_body_with_missing_property() {
    let mut field = I3SField::new(I3SAttributeStorageInfo {
        name: "testField".to_string(),
        ordering: vec!["ObjectIds".to_string()],
        ..Default::default()
    });

    let mut header = HashMap::new();
    header.insert("count".to_string(), 0);
    field.header = Some(header);

    let buffer = vec![0u8; 5];
    let err = field.validate_body(&buffer, 4).unwrap_err();
    assert!(err.message.contains("objectIds is missing"));
}

#[test]
fn load_field_with_unavailable_resource_404() {
    let mut field = I3SField::new(I3SAttributeStorageInfo::default());
    let buffer = b"{404} not found".to_vec();

    field.load_binary(buffer).unwrap();
    assert!(field.data.is_none());
}

#[test]
fn load_field_with_invalid_header() {
    let mut field = I3SField::new(I3SAttributeStorageInfo {
        header: vec![I3SHeaderItem {
            property: "count".to_string(),
            value_type: "UInt16".to_string(),
        }],
        ..Default::default()
    });

    let short_buffer = vec![1u8]; // 1 byte but UInt16 requires 2
    field.load_binary(short_buffer).unwrap();

    assert!(field.header.is_none());
}

#[test]
fn load_field_with_invalid_body() {
    let mut field = I3SField::new(I3SAttributeStorageInfo {
        header: vec![I3SHeaderItem {
            property: "count".to_string(),
            value_type: "UInt8".to_string(),
        }],
        ordering: vec!["attributeValues".to_string()],
        attribute_values: Some(I3SValueDesc {
            value_type: "UInt32".to_string(),
            ..Default::default()
        }),
        ..Default::default()
    });

    // Count is 1, but total buffer length is only 2 (need 1 + 4 = 5)
    let buffer = vec![1u8, 0u8];
    field.load_binary(buffer).unwrap();

    assert!(field.header.is_some());
    assert!(field.values.is_empty());
}

#[test]
fn load_field_with_valid_string_buffer() {
    let text = "Pass";
    let mut buffer = Vec::new();
    buffer.push(1u8); // header count = 1
    buffer.push((text.len() + 1) as u8); // byte count for text + null terminator
    buffer.extend_from_slice(text.as_bytes());
    buffer.push(0u8); // null terminator

    let mut field = I3SField::new(I3SAttributeStorageInfo {
        name: "testStrings".to_string(),
        header: vec![I3SHeaderItem {
            property: "count".to_string(),
            value_type: "UInt8".to_string(),
        }],
        ordering: vec![
            "attributeByteCounts".to_string(),
            "ObjectIds".to_string(),
        ],
        attribute_byte_counts: Some(I3SValueDesc {
            value_type: "UInt8".to_string(),
            ..Default::default()
        }),
        object_ids: Some(I3SValueDesc {
            value_type: "String".to_string(),
            ..Default::default()
        }),
        ..Default::default()
    });

    field.load_binary(buffer).unwrap();

    assert_eq!(field.values, vec![Value::String("Pass".to_string())]);
}

#[test]
fn load_field_with_valid_numeric_buffer() {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&2u32.to_le_bytes()); // header count = 2
    buffer.extend_from_slice(&10.5f32.to_le_bytes());
    buffer.extend_from_slice(&20.25f32.to_le_bytes());

    let mut field = I3SField::new(I3SAttributeStorageInfo {
        name: "testFloats".to_string(),
        header: vec![I3SHeaderItem {
            property: "count".to_string(),
            value_type: "UInt32".to_string(),
        }],
        ordering: vec!["attributeValues".to_string()],
        attribute_values: Some(I3SValueDesc {
            value_type: "Float32".to_string(),
            ..Default::default()
        }),
        ..Default::default()
    });

    field.load_binary(buffer).unwrap();

    assert_eq!(field.values.len(), 2);
    assert_eq!(field.values[0], Value::from(10.5f32));
    assert_eq!(field.values[1], Value::from(20.25f32));
}
