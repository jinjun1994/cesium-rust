use cesium_renderer::pnts_parser::PntsParser;
use serde_json::{json, Value};

/// Helper to generate point cloud tile buffer matching Cesium3DTilesTester.generatePointCloudTileBuffer
fn generate_point_cloud_tile_buffer(
    version: Option<u32>,
    feature_table_json: Option<Value>,
    feature_table_binary: Option<&[u8]>,
    feature_table_json_byte_length_override: Option<u32>,
) -> Vec<u8> {
    let json_bytes = if let Some(ref val) = feature_table_json {
        serde_json::to_vec(val).unwrap()
    } else {
        Vec::new()
    };
    let binary = feature_table_binary.unwrap_or(&[]);

    let json_len = feature_table_json_byte_length_override.unwrap_or(json_bytes.len() as u32);
    let bin_len = binary.len() as u32;
    let total_byte_length = 28 + json_len + bin_len;

    let mut data = Vec::with_capacity(total_byte_length as usize);
    data.extend_from_slice(b"pnts"); // magic
    data.extend_from_slice(&version.unwrap_or(1).to_le_bytes()); // version
    data.extend_from_slice(&total_byte_length.to_le_bytes()); // byteLength
    data.extend_from_slice(&json_len.to_le_bytes()); // featureTableJsonByteLength
    data.extend_from_slice(&bin_len.to_le_bytes()); // featureTableBinaryByteLength
    data.extend_from_slice(&0u32.to_le_bytes()); // batchTableJsonByteLength
    data.extend_from_slice(&0u32.to_le_bytes()); // batchTableBinaryByteLength
    data.extend_from_slice(&json_bytes);
    data.extend_from_slice(binary);

    data
}

#[test]
fn test_throws_without_array_buffer() {
    let result = PntsParser::parse(None);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("arrayBuffer is required"));
}

#[test]
fn test_throws_with_invalid_version() {
    let buffer = generate_point_cloud_tile_buffer(Some(2), None, None, None);
    let result = PntsParser::parse(Some(&buffer));
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("RuntimeError"));
}

#[test]
fn test_throws_if_feature_table_json_byte_length_is_0() {
    let buffer = generate_point_cloud_tile_buffer(
        Some(1),
        Some(json!({ "POINTS_LENGTH": 1 })),
        None,
        Some(0),
    );
    let result = PntsParser::parse(Some(&buffer));
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("RuntimeError"));
}

#[test]
fn test_throws_if_the_feature_table_does_not_contain_points_length() {
    let buffer = generate_point_cloud_tile_buffer(
        Some(1),
        Some(json!({
            "POSITION": {
                "byteOffset": 0
            }
        })),
        Some(&[0u8; 12]),
        None,
    );
    let result = PntsParser::parse(Some(&buffer));
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("RuntimeError"));
}

#[test]
fn test_throws_if_the_feature_table_does_not_contain_position_or_position_quantized() {
    let buffer = generate_point_cloud_tile_buffer(
        Some(1),
        Some(json!({
            "POINTS_LENGTH": 1
        })),
        None,
        None,
    );
    let result = PntsParser::parse(Some(&buffer));
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("RuntimeError"));
}

#[test]
fn test_throws_if_positions_are_quantized_and_feature_table_does_not_contain_quantized_volume_scale()
{
    let buffer = generate_point_cloud_tile_buffer(
        Some(1),
        Some(json!({
            "POINTS_LENGTH": 1,
            "POSITION_QUANTIZED": {
                "byteOffset": 0
            },
            "QUANTIZED_VOLUME_OFFSET": [0.0, 0.0, 0.0]
        })),
        Some(&[0u8; 6]),
        None,
    );
    let result = PntsParser::parse(Some(&buffer));
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("RuntimeError"));
}

#[test]
fn test_throws_if_positions_are_quantized_and_feature_table_does_not_contain_quantized_volume_offset()
{
    let buffer = generate_point_cloud_tile_buffer(
        Some(1),
        Some(json!({
            "POINTS_LENGTH": 1,
            "POSITION_QUANTIZED": {
                "byteOffset": 0
            },
            "QUANTIZED_VOLUME_SCALE": [1.0, 1.0, 1.0]
        })),
        Some(&[0u8; 6]),
        None,
    );
    let result = PntsParser::parse(Some(&buffer));
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("RuntimeError"));
}

#[test]
fn test_throws_if_batch_id_semantic_is_defined_but_batch_length_is_not() {
    let buffer = generate_point_cloud_tile_buffer(
        Some(1),
        Some(json!({
            "POINTS_LENGTH": 2,
            "POSITION": [0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            "BATCH_ID": [0, 1]
        })),
        None,
        None,
    );
    let result = PntsParser::parse(Some(&buffer));
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("RuntimeError"));
}

#[test]
fn test_parses_valid_pnts_content() {
    let buffer = generate_point_cloud_tile_buffer(
        Some(1),
        Some(json!({
            "POINTS_LENGTH": 2,
            "POSITION": [0.0, 0.0, 0.0, 1.0, 2.0, 3.0],
            "BATCH_ID": [0, 1],
            "BATCH_LENGTH": 2
        })),
        None,
        None,
    );
    let parsed = PntsParser::parse(Some(&buffer)).expect("valid pnts should parse");
    assert_eq!(parsed.points_length, 2);
    assert_eq!(parsed.batch_length, Some(2));
    assert_eq!(parsed.batch_id(0).unwrap(), Some(0));
    assert_eq!(parsed.batch_id(1).unwrap(), Some(1));
}
