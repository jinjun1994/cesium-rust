//! Tests adapted from CesiumJS Scene/I3dmParserSpec.js (Apache-2.0).

use serde_json::{json, Value};

use cesium_renderer::i3dm_parser::{I3dmParser, I3dmParserError};

#[derive(Default)]
struct GenerateInstancedTileBufferOptions<'a> {
    pub magic: Option<&'a [u8; 4]>,
    pub version: Option<u32>,
    pub gltf_format: Option<u32>,
    pub gltf_uri: Option<&'a str>,
    pub feature_table_json: Option<Value>,
    pub feature_table_binary: Option<&'a [u8]>,
    pub batch_table_json: Option<Value>,
    pub batch_table_binary: Option<&'a [u8]>,
}

fn pad_string_to_byte_alignment(s: &str, byte_alignment: usize) -> String {
    let remainder = s.len() % byte_alignment;
    let padding = if remainder == 0 {
        0
    } else {
        byte_alignment - remainder
    };
    let mut res = s.to_string();
    for _ in 0..padding {
        res.push(' ');
    }
    res
}

fn generate_instanced_tile_buffer(options: GenerateInstancedTileBufferOptions) -> Vec<u8> {
    let magic = options.magic.unwrap_or(b"i3dm");
    let version = options.version.unwrap_or(1);
    let gltf_format = options.gltf_format.unwrap_or(1);
    let gltf_uri = options.gltf_uri.unwrap_or("model.gltf");
    let gltf_uri_bytes = gltf_uri.as_bytes();

    let feature_table_json_str = if let Some(ft_json) = &options.feature_table_json {
        if ft_json.is_object() && !ft_json.as_object().unwrap().is_empty() {
            serde_json::to_string(ft_json).unwrap()
        } else {
            String::new()
        }
    } else {
        serde_json::to_string(&json!({
            "INSTANCES_LENGTH": 1,
            "POSITION": [0.0, 0.0, 0.0]
        })).unwrap()
    };

    let ft_json_padded = if !feature_table_json_str.is_empty() {
        pad_string_to_byte_alignment(&feature_table_json_str, 8)
    } else {
        String::new()
    };
    let feature_table_json_byte_length = ft_json_padded.len();

    let feature_table_binary = options.feature_table_binary.unwrap_or(&[]);
    let feature_table_binary_byte_length = feature_table_binary.len();

    let batch_table_json_str = if let Some(bt_json) = &options.batch_table_json {
        if bt_json.is_object() && !bt_json.as_object().unwrap().is_empty() {
            serde_json::to_string(bt_json).unwrap()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let bt_json_padded = if !batch_table_json_str.is_empty() {
        pad_string_to_byte_alignment(&batch_table_json_str, 8)
    } else {
        String::new()
    };
    let batch_table_json_byte_length = bt_json_padded.len();

    let batch_table_binary = options.batch_table_binary.unwrap_or(&[]);
    let batch_table_binary_byte_length = batch_table_binary.len();

    let header_byte_length = 32;
    let byte_length = header_byte_length
        + feature_table_json_byte_length
        + feature_table_binary_byte_length
        + batch_table_json_byte_length
        + batch_table_binary_byte_length
        + gltf_uri_bytes.len();

    let mut buffer = Vec::with_capacity(byte_length);
    buffer.extend_from_slice(magic);
    buffer.extend_from_slice(&version.to_le_bytes());
    buffer.extend_from_slice(&(byte_length as u32).to_le_bytes());
    buffer.extend_from_slice(&(feature_table_json_byte_length as u32).to_le_bytes());
    buffer.extend_from_slice(&(feature_table_binary_byte_length as u32).to_le_bytes());
    buffer.extend_from_slice(&(batch_table_json_byte_length as u32).to_le_bytes());
    buffer.extend_from_slice(&(batch_table_binary_byte_length as u32).to_le_bytes());
    buffer.extend_from_slice(&gltf_format.to_le_bytes());

    buffer.extend_from_slice(ft_json_padded.as_bytes());
    buffer.extend_from_slice(feature_table_binary);
    buffer.extend_from_slice(bt_json_padded.as_bytes());
    buffer.extend_from_slice(batch_table_binary);
    buffer.extend_from_slice(gltf_uri_bytes);

    buffer
}

#[test]
fn throws_with_undefined_array_buffer() {
    let result = I3dmParser::parse(None, None);
    assert!(matches!(result, Err(I3dmParserError::DeveloperError(_))));
}

#[test]
fn throws_with_invalid_version() {
    let buffer = generate_instanced_tile_buffer(GenerateInstancedTileBufferOptions {
        version: Some(2),
        ..Default::default()
    });
    let result = I3dmParser::parse(Some(&buffer), None);
    assert!(matches!(result, Err(I3dmParserError::RuntimeError(_))));
    if let Err(I3dmParserError::RuntimeError(err)) = result {
        assert!(err.message.contains("Only Instanced 3D Model version 1 is supported"));
    }
}

#[test]
fn throws_if_there_is_no_feature_table_json() {
    let buffer = generate_instanced_tile_buffer(GenerateInstancedTileBufferOptions {
        feature_table_json: Some(json!({})),
        ..Default::default()
    });
    let result = I3dmParser::parse(Some(&buffer), None);
    assert!(matches!(result, Err(I3dmParserError::RuntimeError(_))));
    if let Err(I3dmParserError::RuntimeError(err)) = result {
        assert!(err.message.contains("featureTableJsonByteLength is zero"));
    }
}

#[test]
fn throws_with_invalid_gltf_format() {
    let buffer = generate_instanced_tile_buffer(GenerateInstancedTileBufferOptions {
        gltf_format: Some(2),
        ..Default::default()
    });
    let result = I3dmParser::parse(Some(&buffer), None);
    assert!(matches!(result, Err(I3dmParserError::RuntimeError(_))));
    if let Err(I3dmParserError::RuntimeError(err)) = result {
        assert!(err.message.contains("Only glTF format 0 (uri) or 1 (embedded) are supported"));
    }
}

#[test]
fn throws_if_there_is_no_gltf() {
    let buffer = generate_instanced_tile_buffer(GenerateInstancedTileBufferOptions {
        gltf_uri: Some(""),
        ..Default::default()
    });
    let result = I3dmParser::parse(Some(&buffer), None);
    assert!(matches!(result, Err(I3dmParserError::RuntimeError(_))));
    if let Err(I3dmParserError::RuntimeError(err)) = result {
        assert!(err.message.contains("glTF byte length must be greater than 0"));
    }
}

#[test]
fn detects_unaligned_gltf_warning() {
    let unaligned_feature_table_binary = [0u8; 1];
    let buffer = generate_instanced_tile_buffer(GenerateInstancedTileBufferOptions {
        feature_table_binary: Some(&unaligned_feature_table_binary),
        ..Default::default()
    });
    let result = I3dmParser::parse(Some(&buffer), None).expect("parses with unaligned gltf");
    assert!(result.is_unaligned);
}

#[test]
fn parses_i3dm() {
    let instances_length = 10;
    let mut positions_bytes = Vec::with_capacity(instances_length * 3 * 4);
    let mut heights_bytes = Vec::with_capacity(instances_length * 4);

    for i in 0..instances_length {
        let f = i as f32;
        positions_bytes.extend_from_slice(&f.to_le_bytes());
        positions_bytes.extend_from_slice(&f.to_le_bytes());
        positions_bytes.extend_from_slice(&f.to_le_bytes());
        heights_bytes.extend_from_slice(&f.to_le_bytes());
    }

    let feature_table_json = json!({
        "INSTANCES_LENGTH": instances_length,
        "POSITION": {
            "byteOffset": 0
        }
    });

    let batch_table_json = json!({
        "height": {
            "byteOffset": 0,
            "componentType": "FLOAT",
            "type": "SCALAR"
        }
    });

    let gltf_uri = "model.gltf";

    let buffer = generate_instanced_tile_buffer(GenerateInstancedTileBufferOptions {
        feature_table_json: Some(feature_table_json.clone()),
        feature_table_binary: Some(&positions_bytes),
        batch_table_json: Some(batch_table_json.clone()),
        batch_table_binary: Some(&heights_bytes),
        gltf_uri: Some(gltf_uri),
        ..Default::default()
    });

    let results = I3dmParser::parse(Some(&buffer), None).expect("successful i3dm parse");

    assert_eq!(results.gltf_format, 1);
    assert_eq!(results.feature_table_json, feature_table_json);
    assert_eq!(results.feature_table_binary, positions_bytes);
    assert_eq!(results.batch_table_json.unwrap(), batch_table_json);
    assert_eq!(results.batch_table_binary.unwrap(), heights_bytes);
    assert_eq!(std::str::from_utf8(&results.gltf).unwrap(), gltf_uri);
    assert!(!results.is_unaligned);
}
