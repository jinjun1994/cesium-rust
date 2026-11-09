use cesium_math::{Cartesian2, Cartesian3, Cartesian4, Matrix2, Matrix3, Matrix4};
use cesium_renderer::{
    MetadataClassProperty, MetadataComponentType, MetadataPicking, MetadataType, MetadataValue,
    PickingMetadataProperty, RawMetadataNumber,
};

const PRECISION_EPSILON: f64 = 1e-3;

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() <= PRECISION_EPSILON
}

#[test]
fn test_decode_raw_metadata_value_throws_for_invalid_index() {
    let array = [12u8, 23];
    let result = MetadataPicking::decode_raw_metadata_value(
        MetadataComponentType::Int8,
        &array,
        1234,
    );
    assert!(result.is_err());
}

#[test]
fn test_decode_raw_metadata_value_all_types() {
    // INT8
    let bytes_int8 = [12u8, 23];
    assert_eq!(
        MetadataPicking::decode_raw_metadata_value(MetadataComponentType::Int8, &bytes_int8, 1).unwrap(),
        RawMetadataNumber::Int(23)
    );

    // UINT8
    let bytes_uint8 = [12u8, 23];
    assert_eq!(
        MetadataPicking::decode_raw_metadata_value(MetadataComponentType::Uint8, &bytes_uint8, 1).unwrap(),
        RawMetadataNumber::Uint(23)
    );

    // INT16
    let i16_val = 2345i16;
    let mut bytes_int16 = vec![0u8; 4];
    bytes_int16[2..4].copy_from_slice(&i16_val.to_le_bytes());
    assert_eq!(
        MetadataPicking::decode_raw_metadata_value(MetadataComponentType::Int16, &bytes_int16, 2).unwrap(),
        RawMetadataNumber::Int(2345)
    );

    // UINT16
    let u16_val = 2345u16;
    let mut bytes_uint16 = vec![0u8; 4];
    bytes_uint16[2..4].copy_from_slice(&u16_val.to_le_bytes());
    assert_eq!(
        MetadataPicking::decode_raw_metadata_value(MetadataComponentType::Uint16, &bytes_uint16, 2).unwrap(),
        RawMetadataNumber::Uint(2345)
    );

    // INT32
    let i32_val = 234567i32;
    let mut bytes_int32 = vec![0u8; 8];
    bytes_int32[4..8].copy_from_slice(&i32_val.to_le_bytes());
    assert_eq!(
        MetadataPicking::decode_raw_metadata_value(MetadataComponentType::Int32, &bytes_int32, 4).unwrap(),
        RawMetadataNumber::Int(234567)
    );

    // UINT32
    let u32_val = 234567u32;
    let mut bytes_uint32 = vec![0u8; 8];
    bytes_uint32[4..8].copy_from_slice(&u32_val.to_le_bytes());
    assert_eq!(
        MetadataPicking::decode_raw_metadata_value(MetadataComponentType::Uint32, &bytes_uint32, 4).unwrap(),
        RawMetadataNumber::Uint(234567)
    );

    // INT64
    let i64_val = 2345i64;
    let mut bytes_int64 = vec![0u8; 16];
    bytes_int64[8..16].copy_from_slice(&i64_val.to_le_bytes());
    assert_eq!(
        MetadataPicking::decode_raw_metadata_value(MetadataComponentType::Int64, &bytes_int64, 8).unwrap(),
        RawMetadataNumber::Int(2345)
    );

    // UINT64
    let u64_val = 2345u64;
    let mut bytes_uint64 = vec![0u8; 16];
    bytes_uint64[8..16].copy_from_slice(&u64_val.to_le_bytes());
    assert_eq!(
        MetadataPicking::decode_raw_metadata_value(MetadataComponentType::Uint64, &bytes_uint64, 8).unwrap(),
        RawMetadataNumber::Uint(2345)
    );

    // FLOAT32
    let f32_val = 12.345f32;
    let mut bytes_f32 = vec![0u8; 8];
    bytes_f32[4..8].copy_from_slice(&f32_val.to_le_bytes());
    let decoded_f32 =
        MetadataPicking::decode_raw_metadata_value(MetadataComponentType::Float32, &bytes_f32, 4).unwrap();
    assert!(approx_eq(decoded_f32.as_f64(), 12.345));

    // FLOAT64
    let f64_val = 12.345f64;
    let mut bytes_f64 = vec![0u8; 16];
    bytes_f64[8..16].copy_from_slice(&f64_val.to_le_bytes());
    let decoded_f64 =
        MetadataPicking::decode_raw_metadata_value(MetadataComponentType::Float64, &bytes_f64, 8).unwrap();
    assert!(approx_eq(decoded_f64.as_f64(), 12.345));
}

#[test]
fn test_decode_raw_metadata_value_component_normalization() {
    // INT8 normalized
    let prop_int8_norm = MetadataClassProperty {
        component_type: Some(MetadataComponentType::Int8),
        normalized: true,
        ..Default::default()
    };
    let val = MetadataPicking::decode_raw_metadata_value_component(&prop_int8_norm, &[12, 23], 1).unwrap();
    assert!(approx_eq(val.as_f64(), 23.0 / 127.0));

    // UINT8 normalized
    let prop_uint8_norm = MetadataClassProperty {
        component_type: Some(MetadataComponentType::Uint8),
        normalized: true,
        ..Default::default()
    };
    let val = MetadataPicking::decode_raw_metadata_value_component(&prop_uint8_norm, &[12, 23], 1).unwrap();
    assert!(approx_eq(val.as_f64(), 23.0 / 255.0));

    // INT16 normalized
    let prop_int16_norm = MetadataClassProperty {
        component_type: Some(MetadataComponentType::Int16),
        normalized: true,
        ..Default::default()
    };
    let mut b_int16 = vec![0u8; 4];
    b_int16[2..4].copy_from_slice(&2345i16.to_le_bytes());
    let val = MetadataPicking::decode_raw_metadata_value_component(&prop_int16_norm, &b_int16, 2).unwrap();
    assert!(approx_eq(val.as_f64(), 2345.0 / 32767.0));

    // UINT16 normalized
    let prop_uint16_norm = MetadataClassProperty {
        component_type: Some(MetadataComponentType::Uint16),
        normalized: true,
        ..Default::default()
    };
    let mut b_uint16 = vec![0u8; 4];
    b_uint16[2..4].copy_from_slice(&2345u16.to_le_bytes());
    let val = MetadataPicking::decode_raw_metadata_value_component(&prop_uint16_norm, &b_uint16, 2).unwrap();
    assert!(approx_eq(val.as_f64(), 2345.0 / 65535.0));
}

#[test]
fn test_decode_raw_metadata_value_element() {
    let prop_vec2 = MetadataClassProperty {
        r#type: MetadataType::Vec2,
        component_type: Some(MetadataComponentType::Int8),
        ..Default::default()
    };
    let buf = [0u8, 1, 2, 3];
    let elem = MetadataPicking::decode_raw_metadata_value_element(&prop_vec2, &buf, 0).unwrap();
    assert_eq!(
        elem,
        MetadataValue::Array(vec![MetadataValue::Int(0), MetadataValue::Int(1)])
    );

    let prop_vec3 = MetadataClassProperty {
        r#type: MetadataType::Vec3,
        component_type: Some(MetadataComponentType::Int8),
        ..Default::default()
    };
    let elem3 = MetadataPicking::decode_raw_metadata_value_element(&prop_vec3, &buf, 0).unwrap();
    assert_eq!(
        elem3,
        MetadataValue::Array(vec![
            MetadataValue::Int(0),
            MetadataValue::Int(1),
            MetadataValue::Int(2),
        ])
    );
}

#[test]
fn test_convert_to_object_type() {
    // Unmodified types
    let s = MetadataValue::String("hello".to_string());
    assert_eq!(
        MetadataPicking::convert_to_object_type(MetadataType::String, &s).unwrap(),
        s
    );
    let b = MetadataValue::Boolean(true);
    assert_eq!(
        MetadataPicking::convert_to_object_type(MetadataType::Boolean, &b).unwrap(),
        b
    );

    // VEC2 -> Cartesian2
    let vec2_arr = MetadataValue::Array(vec![MetadataValue::Scalar(1.0), MetadataValue::Scalar(2.0)]);
    let c2 = MetadataPicking::convert_to_object_type(MetadataType::Vec2, &vec2_arr).unwrap();
    assert_eq!(c2, MetadataValue::Cartesian2(Cartesian2::new(1.0, 2.0)));

    // VEC3 -> Cartesian3
    let vec3_arr = MetadataValue::Array(vec![
        MetadataValue::Scalar(1.0),
        MetadataValue::Scalar(2.0),
        MetadataValue::Scalar(3.0),
    ]);
    let c3 = MetadataPicking::convert_to_object_type(MetadataType::Vec3, &vec3_arr).unwrap();
    assert_eq!(c3, MetadataValue::Cartesian3(Cartesian3::new(1.0, 2.0, 3.0)));

    // VEC4 -> Cartesian4
    let vec4_arr = MetadataValue::Array(vec![
        MetadataValue::Scalar(1.0),
        MetadataValue::Scalar(2.0),
        MetadataValue::Scalar(3.0),
        MetadataValue::Scalar(4.0),
    ]);
    let c4 = MetadataPicking::convert_to_object_type(MetadataType::Vec4, &vec4_arr).unwrap();
    assert_eq!(c4, MetadataValue::Cartesian4(Cartesian4::new(1.0, 2.0, 3.0, 4.0)));
}

#[test]
fn test_convert_from_object_type() {
    // Cartesian2 -> Array
    let c2 = MetadataValue::Cartesian2(Cartesian2::new(10.0, 20.0));
    let arr2 = MetadataPicking::convert_from_object_type(MetadataType::Vec2, &c2).unwrap();
    assert_eq!(
        arr2,
        MetadataValue::Array(vec![MetadataValue::Scalar(10.0), MetadataValue::Scalar(20.0)])
    );

    // Cartesian3 -> Array
    let c3 = MetadataValue::Cartesian3(Cartesian3::new(10.0, 20.0, 30.0));
    let arr3 = MetadataPicking::convert_from_object_type(MetadataType::Vec3, &c3).unwrap();
    assert_eq!(
        arr3,
        MetadataValue::Array(vec![
            MetadataValue::Scalar(10.0),
            MetadataValue::Scalar(20.0),
            MetadataValue::Scalar(30.0)
        ])
    );
}

#[test]
fn test_convert_matrix_object_types() {
    let mat2_arr = MetadataValue::Array(vec![
        MetadataValue::Scalar(1.0),
        MetadataValue::Scalar(2.0),
        MetadataValue::Scalar(3.0),
        MetadataValue::Scalar(4.0),
    ]);
    let m2 = MetadataPicking::convert_to_object_type(MetadataType::Mat2, &mat2_arr).unwrap();
    let expected_m2 = Matrix2::unpack(&[1.0, 2.0, 3.0, 4.0], 0).unwrap();
    assert_eq!(m2, MetadataValue::Matrix2(expected_m2));

    let back_m2 = MetadataPicking::convert_from_object_type(MetadataType::Mat2, &m2).unwrap();
    assert_eq!(back_m2, mat2_arr);

    let mat3_data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
    let mat3_arr = MetadataValue::Array(mat3_data.iter().map(|&v| MetadataValue::Scalar(v)).collect());
    let m3 = MetadataPicking::convert_to_object_type(MetadataType::Mat3, &mat3_arr).unwrap();
    let expected_m3 = Matrix3::unpack(&mat3_data, 0).unwrap();
    assert_eq!(m3, MetadataValue::Matrix3(expected_m3));

    let back_m3 = MetadataPicking::convert_from_object_type(MetadataType::Mat3, &m3).unwrap();
    assert_eq!(back_m3, mat3_arr);

    let mat4_data = [
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
    ];
    let mat4_arr = MetadataValue::Array(mat4_data.iter().map(|&v| MetadataValue::Scalar(v)).collect());
    let m4 = MetadataPicking::convert_to_object_type(MetadataType::Mat4, &mat4_arr).unwrap();
    let expected_m4 = Matrix4::unpack(&mat4_data, 0).unwrap();
    assert_eq!(m4, MetadataValue::Matrix4(expected_m4));

    let back_m4 = MetadataPicking::convert_from_object_type(MetadataType::Mat4, &m4).unwrap();
    assert_eq!(back_m4, mat4_arr);
}

#[test]
fn test_decode_metadata_values_with_and_without_transform() {
    // Without transform
    let prop_vec2 = MetadataClassProperty {
        r#type: MetadataType::Vec2,
        component_type: Some(MetadataComponentType::Int8),
        ..Default::default()
    };
    let meta_prop_no_xf = PickingMetadataProperty {
        has_value_transform: false,
        offset: None,
        scale: None,
    };
    let raw = [12u8, 23, 34, 45];
    let val = MetadataPicking::decode_metadata_values(&prop_vec2, &meta_prop_no_xf, &raw).unwrap();
    assert_eq!(val, MetadataValue::Cartesian2(Cartesian2::new(12.0, 23.0)));

    // With transform
    let meta_prop_xf = PickingMetadataProperty {
        has_value_transform: true,
        offset: Some(MetadataValue::Cartesian2(Cartesian2::new(100.0, 200.0))),
        scale: Some(MetadataValue::Cartesian2(Cartesian2::new(2.0, 3.0))),
    };
    let val_xf = MetadataPicking::decode_metadata_values(&prop_vec2, &meta_prop_xf, &raw).unwrap();
    assert_eq!(
        val_xf,
        MetadataValue::Cartesian2(Cartesian2::new(100.0 + 12.0 * 2.0, 200.0 + 23.0 * 3.0))
    );
}
