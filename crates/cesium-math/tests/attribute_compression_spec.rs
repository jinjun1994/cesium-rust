use cesium_math::{
    AttributeCompression, AttributeType, Cartesian2, Cartesian3, Cartesian4, Color,
    ComponentDatatype, EPSILON2, EPSILON5,
};

#[test]
fn test_oct_decode_zero_zero() {
    let result = AttributeCompression::oct_decode(0.0, 0.0).unwrap();
    let expected = Cartesian3::new(0.0, 0.0, -1.0);
    assert_eq!(result, expected);
}

#[test]
fn test_oct_encode_negative_unit_z() {
    let negative_unit_z = Cartesian3::new(0.0, 0.0, -1.0);
    let result = AttributeCompression::oct_encode(negative_unit_z).unwrap();
    assert_eq!(result, Cartesian2::new(255.0, 255.0));
}

#[test]
fn test_oct_encode_unit_z() {
    let unit_z = Cartesian3::UNIT_Z;
    let result = AttributeCompression::oct_encode(unit_z).unwrap();
    assert_eq!(result, Cartesian2::new(128.0, 128.0));
}

#[test]
fn test_oct_encode_negative_unit_z_to_4_components() {
    let negative_unit_z = Cartesian3::new(0.0, 0.0, -1.0);
    let result = AttributeCompression::oct_encode_to_cartesian4(negative_unit_z).unwrap();
    assert_eq!(result, Cartesian4::new(255.0, 255.0, 255.0, 255.0));
}

#[test]
fn test_oct_encode_unit_z_to_4_components() {
    let unit_z = Cartesian3::UNIT_Z;
    let result = AttributeCompression::oct_encode_to_cartesian4(unit_z).unwrap();
    assert_eq!(result, Cartesian4::new(128.0, 0.0, 128.0, 0.0));
}

#[test]
fn test_oct_extents_are_equal() {
    let negative_unit_z = Cartesian3::new(0.0, 0.0, -1.0);
    // lower left
    assert_eq!(AttributeCompression::oct_decode(0.0, 0.0).unwrap(), negative_unit_z);
    // lower right
    assert_eq!(AttributeCompression::oct_decode(255.0, 0.0).unwrap(), negative_unit_z);
    // upper right
    assert_eq!(AttributeCompression::oct_decode(255.0, 255.0).unwrap(), negative_unit_z);
    // upper left
    assert_eq!(AttributeCompression::oct_decode(0.0, 255.0).unwrap(), negative_unit_z);
}

#[test]
fn test_oct_roundtrip_cartesian4() {
    let vector = Cartesian3::new(0.2, 0.5, 0.8).normalize();
    let encoded = AttributeCompression::oct_encode_to_cartesian4(vector).unwrap();
    let decoded = AttributeCompression::oct_decode_from_cartesian4(encoded).unwrap();
    assert!(decoded.equals_epsilon(&vector, Some(1e-4), None));
}

#[test]
fn test_oct_float_encoding_roundtrip() {
    let vector = Cartesian3::new(0.3, -0.4, 0.5).normalize();
    let encoded_float = AttributeCompression::oct_encode_float(vector).unwrap();
    let decoded = AttributeCompression::oct_decode_float(encoded_float).unwrap();
    assert!(decoded.equals_epsilon(&vector, Some(0.02), None));
}

#[test]
fn test_oct_pack_and_unpack() {
    let v1 = Cartesian3::UNIT_X;
    let v2 = Cartesian3::UNIT_Y;
    let v3 = Cartesian3::UNIT_Z;

    let packed = AttributeCompression::oct_pack(v1, v2, v3).unwrap();
    let (d1, d2, d3) = AttributeCompression::oct_unpack(packed).unwrap();

    assert!(d1.equals_epsilon(&v1, Some(0.02), None));
    assert!(d2.equals_epsilon(&v2, Some(0.02), None));
    assert!(d3.equals_epsilon(&v3, Some(0.02), None));
}

#[test]
fn test_compress_and_decompress_texture_coordinates() {
    let tc = Cartesian2::new(0.5, 0.75);
    let compressed = AttributeCompression::compress_texture_coordinates(tc);
    let decompressed = AttributeCompression::decompress_texture_coordinates(compressed);
    assert!((decompressed.x - tc.x).abs() < 1.0 / 4095.0);
    assert!((decompressed.y - tc.y).abs() < 1.0 / 4095.0);
}

#[test]
fn test_zig_zag_delta_decode() {
    let mut u = [0u16, 2, 4, 1];
    let mut v = [0u16, 1, 3, 2];
    let mut h = [10u16, 2, 0, 4];

    AttributeCompression::zig_zag_delta_decode(&mut u, &mut v, Some(&mut h)).unwrap();

    // zigZagDecode: 0 -> 0, 1 -> -1, 2 -> 1, 3 -> -2, 4 -> 2
    // u: 0 -> 0; 0+1=1; 1+2=3; 3-1=2
    assert_eq!(u, [0, 1, 3, 2]);
    // v: 0 -> 0; 0-1=-1(65535); -1-2=-3(65533); -3+1=-2(65534)
    assert_eq!(v[0], 0);
    assert_eq!(v[1], 65535);
    assert_eq!(v[2], 65533);
    assert_eq!(v[3], 65534);
}

#[test]
fn test_encode_and_decode_rgb8() {
    let color = Color::new(0.2, 0.4, 0.6, 1.0);
    let encoded = AttributeCompression::encode_rgb8(&color);
    let decoded = AttributeCompression::decode_rgb8(encoded);

    assert!((decoded.red - color.red).abs() < 1.0 / 255.0);
    assert!((decoded.green - color.green).abs() < 1.0 / 255.0);
    assert!((decoded.blue - color.blue).abs() < 1.0 / 255.0);
}

#[test]
fn test_decode_rgb565() {
    // 0xFFFF: red 31, green 63, blue 31 -> all 1.0
    let values = [0xFFFFu16];
    let decoded = AttributeCompression::decode_rgb565(&values);
    assert_eq!(decoded.len(), 3);
    assert!((decoded[0] - 1.0).abs() < 1e-6);
    assert!((decoded[1] - 1.0).abs() < 1e-6);
    assert!((decoded[2] - 1.0).abs() < 1e-6);
}

#[test]
fn test_dequantize_works_with_byte() {
    let input = [-127.0, -127.0, -127.0, 0.0, 0.0, 0.0, 127.0, 127.0, 127.0];
    let expected = [-1.0f32, -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let result = AttributeCompression::dequantize_f64(
        &input,
        ComponentDatatype::Byte,
        AttributeType::Vec3,
        3,
    )
    .unwrap();
    for i in 0..input.len() {
        assert!((result[i] - expected[i]).abs() < EPSILON2 as f32);
    }
}

#[test]
fn test_dequantize_works_with_unsigned_byte() {
    let input = [0.0, 0.0, 0.0, 127.0, 127.0, 127.0, 255.0, 255.0, 255.0];
    let expected = [0.0f32, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 1.0, 1.0];
    let result = AttributeCompression::dequantize_f64(
        &input,
        ComponentDatatype::UnsignedByte,
        AttributeType::Vec3,
        3,
    )
    .unwrap();
    for i in 0..input.len() {
        assert!((result[i] - expected[i]).abs() < EPSILON2 as f32);
    }
}

#[test]
fn test_dequantize_works_with_short() {
    let input = [-32767.0, -32767.0, -32767.0, 0.0, 0.0, 0.0, 32767.0, 32767.0, 32767.0];
    let expected = [-1.0f32, -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let result = AttributeCompression::dequantize_f64(
        &input,
        ComponentDatatype::Short,
        AttributeType::Vec3,
        3,
    )
    .unwrap();
    for i in 0..input.len() {
        assert!((result[i] - expected[i]).abs() < EPSILON5 as f32);
    }
}
