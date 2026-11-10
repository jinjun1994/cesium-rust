use cesium_math::{AttributeType, ComponentDatatype};
use cesium_renderer::model_components::{Attribute, Quantization};
use cesium_renderer::model_reader::ModelReader;

#[test]
fn test_returns_compact_float_data_for_vec3_float() {
    let raw_data: [f32; 6] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let bytes: Vec<u8> = raw_data.iter().flat_map(|f| f.to_le_bytes()).collect();

    let attr = Attribute {
        count: 2,
        attribute_type: Some(AttributeType::Vec3),
        component_datatype: Some(ComponentDatatype::Float),
        typed_array: Some(bytes),
        byte_offset: 0,
        byte_stride: None,
        normalized: false,
        ..Default::default()
    };

    let result = ModelReader::read_attribute_as_typed_array(&attr).unwrap();
    assert_eq!(result, raw_data);
}

#[test]
fn test_returns_compact_float_data_for_vec2_float() {
    let raw_data: [f32; 4] = [0.1, 0.2, 0.3, 0.4];
    let bytes: Vec<u8> = raw_data.iter().flat_map(|f| f.to_le_bytes()).collect();

    let attr = Attribute {
        count: 2,
        attribute_type: Some(AttributeType::Vec2),
        component_datatype: Some(ComponentDatatype::Float),
        typed_array: Some(bytes),
        byte_offset: 0,
        byte_stride: None,
        normalized: false,
        ..Default::default()
    };

    let result = ModelReader::read_attribute_as_typed_array(&attr).unwrap();
    assert_eq!(result, raw_data);
}

#[test]
fn test_returns_compact_data_for_scalar_float() {
    let raw_data: [f32; 3] = [10.0, 20.0, 30.0];
    let bytes: Vec<u8> = raw_data.iter().flat_map(|f| f.to_le_bytes()).collect();

    let attr = Attribute {
        count: 3,
        attribute_type: Some(AttributeType::Scalar),
        component_datatype: Some(ComponentDatatype::Float),
        typed_array: Some(bytes),
        byte_offset: 0,
        byte_stride: None,
        normalized: false,
        ..Default::default()
    };

    let result = ModelReader::read_attribute_as_typed_array(&attr).unwrap();
    assert_eq!(result, raw_data);
}

#[test]
fn test_deinterleaves_data_when_byte_stride_exceeds_default() {
    // Interleaved stride: 16 bytes (3 floats pos = 12 bytes, + 4 bytes padding)
    let mut bytes = Vec::new();
    let pos0: [f32; 3] = [1.0, 2.0, 3.0];
    let pad0: [u8; 4] = [0, 0, 0, 0];
    let pos1: [f32; 3] = [4.0, 5.0, 6.0];
    let pad1: [u8; 4] = [0, 0, 0, 0];

    bytes.extend_from_slice(&pos0[0].to_le_bytes());
    bytes.extend_from_slice(&pos0[1].to_le_bytes());
    bytes.extend_from_slice(&pos0[2].to_le_bytes());
    bytes.extend_from_slice(&pad0);

    bytes.extend_from_slice(&pos1[0].to_le_bytes());
    bytes.extend_from_slice(&pos1[1].to_le_bytes());
    bytes.extend_from_slice(&pos1[2].to_le_bytes());
    bytes.extend_from_slice(&pad1);

    let attr = Attribute {
        count: 2,
        attribute_type: Some(AttributeType::Vec3),
        component_datatype: Some(ComponentDatatype::Float),
        typed_array: Some(bytes),
        byte_offset: 0,
        byte_stride: Some(16),
        normalized: false,
        ..Default::default()
    };

    let result = ModelReader::read_attribute_as_typed_array(&attr).unwrap();
    assert_eq!(result, [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
}

#[test]
fn test_respects_byte_offset_when_deinterleaving() {
    // Offset 4 bytes, then 2 floats uv (8 bytes), total stride 12 bytes
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0u8; 4]); // prefix
    bytes.extend_from_slice(&1.0f32.to_le_bytes());
    bytes.extend_from_slice(&2.0f32.to_le_bytes());

    bytes.extend_from_slice(&[0u8; 4]); // prefix
    bytes.extend_from_slice(&3.0f32.to_le_bytes());
    bytes.extend_from_slice(&4.0f32.to_le_bytes());

    let attr = Attribute {
        count: 2,
        attribute_type: Some(AttributeType::Vec2),
        component_datatype: Some(ComponentDatatype::Float),
        typed_array: Some(bytes),
        byte_offset: 4,
        byte_stride: Some(12),
        normalized: false,
        ..Default::default()
    };

    let result = ModelReader::read_attribute_as_typed_array(&attr).unwrap();
    assert_eq!(result, [1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn test_applies_normalization_to_unsigned_byte_data() {
    let bytes = vec![0u8, 128u8, 255u8];
    let attr = Attribute {
        count: 3,
        attribute_type: Some(AttributeType::Scalar),
        component_datatype: Some(ComponentDatatype::UnsignedByte),
        typed_array: Some(bytes),
        byte_offset: 0,
        byte_stride: None,
        normalized: true,
        ..Default::default()
    };

    let result = ModelReader::read_attribute_as_typed_array(&attr).unwrap();
    assert_eq!(result[0], 0.0);
    assert!((result[1] - (128.0 / 255.0)).abs() < 1e-6);
    assert_eq!(result[2], 1.0);
}

#[test]
fn test_applies_normalization_to_unsigned_short_data() {
    let u16_data = [0u16, 65535u16];
    let bytes: Vec<u8> = u16_data.iter().flat_map(|u| u.to_le_bytes()).collect();

    let attr = Attribute {
        count: 2,
        attribute_type: Some(AttributeType::Scalar),
        component_datatype: Some(ComponentDatatype::UnsignedShort),
        typed_array: Some(bytes),
        byte_offset: 0,
        byte_stride: None,
        normalized: true,
        ..Default::default()
    };

    let result = ModelReader::read_attribute_as_typed_array(&attr).unwrap();
    assert_eq!(result[0], 0.0);
    assert_eq!(result[1], 1.0);
}

#[test]
fn test_dequantizes_vec3_data_with_step_size_and_offset() {
    let quantized_data = [0.0f32, 10.0, 20.0];
    let step_size = [0.5, 0.5, 0.5];
    let offset = [1.0, 2.0, 3.0];

    let result = ModelReader::dequantize_3d(&quantized_data, 1, step_size, offset);
    assert_eq!(result, [1.0, 7.0, 13.0]);
}

#[test]
fn test_dequantizes_vec2_data_with_step_size_and_offset() {
    let quantized_data = [4.0f32, 8.0];
    let step_size = [2.0, 0.5];
    let offset = [10.0, -1.0];

    let result = ModelReader::dequantize_2d(&quantized_data, 1, step_size, offset);
    assert_eq!(result, [18.0, 3.0]);
}

#[test]
fn test_dequantizes_scalar_data_with_step_size_and_offset() {
    let quantized_data = [10.0f32, 20.0];
    let step_size = 3.0;
    let offset = 5.0;

    let result = ModelReader::dequantize_1d(&quantized_data, 2, step_size, offset);
    assert_eq!(result, [35.0, 65.0]);
}

#[test]
fn test_oct_decodes_quantized_normals() {
    // Oct-decode [128.0, 128.0] with range 255
    let quantized = [128.0f32, 128.0];
    let decoded = ModelReader::oct_decode(&quantized, 1, 255.0).unwrap();
    assert_eq!(decoded.len(), 3);
    // Length of unit normal should be close to 1.0
    let len = (decoded[0] * decoded[0] + decoded[1] * decoded[1] + decoded[2] * decoded[2]).sqrt();
    assert!((len - 1.0).abs() < 1e-2);
}

#[test]
fn test_convert_zxy_to_xyz() {
    let zxy = [3.0f32, 1.0, 2.0];
    let xyz = ModelReader::convert_zxy_to_xyz(&zxy, 1);
    assert_eq!(xyz, [1.0, 2.0, 3.0]);
}

#[test]
fn test_dequantize_pipeline_integration() {
    let raw_bytes: Vec<u8> = vec![10u8, 20u8];
    let attr = Attribute {
        count: 2,
        attribute_type: Some(AttributeType::Scalar),
        component_datatype: Some(ComponentDatatype::UnsignedByte),
        typed_array: Some(raw_bytes),
        byte_offset: 0,
        byte_stride: None,
        normalized: false,
        quantization: Some(Quantization {
            quantized_volume_step_size: Some(vec![2.0]),
            quantized_volume_offset: Some(vec![5.0]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = ModelReader::read_attribute_as_typed_array(&attr).unwrap();
    assert_eq!(result, [25.0, 45.0]);
}
