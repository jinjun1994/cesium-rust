use cesium_formats::gltf_pipeline::{component_type_size_in_bytes, get_component_reader};

fn test_component_reader_type(
    component_type: u32,
    raw_bytes: &[u8],
    expected: &[f64],
) {
    let component_reader = get_component_reader(component_type).unwrap();
    let byte_offset = component_type_size_in_bytes(component_type);
    let number_of_components = 2;
    let mut result = vec![0.0; number_of_components];
    component_reader(raw_bytes, byte_offset, number_of_components, &mut result);
    assert_eq!(result, expected);
}

#[test]
fn test_reads_values() {
    // 5120 BYTE: [0i8, 1i8, 2i8]
    test_component_reader_type(5120, &[0, 1, 2], &[1.0, 2.0]);

    // 5121 UNSIGNED_BYTE: [0u8, 1u8, 2u8]
    test_component_reader_type(5121, &[0, 1, 2], &[1.0, 2.0]);

    // 5122 SHORT: [0i16, 1i16, 2i16]
    let mut short_bytes = Vec::new();
    for v in [0i16, 1i16, 2i16] {
        short_bytes.extend_from_slice(&v.to_le_bytes());
    }
    test_component_reader_type(5122, &short_bytes, &[1.0, 2.0]);

    // 5123 UNSIGNED_SHORT: [0u16, 1u16, 2u16]
    let mut ushort_bytes = Vec::new();
    for v in [0u16, 1u16, 2u16] {
        ushort_bytes.extend_from_slice(&v.to_le_bytes());
    }
    test_component_reader_type(5123, &ushort_bytes, &[1.0, 2.0]);

    // 5124 INT: [0i32, 1i32, 2i32]
    let mut int_bytes = Vec::new();
    for v in [0i32, 1i32, 2i32] {
        int_bytes.extend_from_slice(&v.to_le_bytes());
    }
    test_component_reader_type(5124, &int_bytes, &[1.0, 2.0]);

    // 5125 UNSIGNED_INT: [0u32, 1u32, 2u32]
    let mut uint_bytes = Vec::new();
    for v in [0u32, 1u32, 2u32] {
        uint_bytes.extend_from_slice(&v.to_le_bytes());
    }
    test_component_reader_type(5125, &uint_bytes, &[1.0, 2.0]);

    // 5126 FLOAT: [0.0f32, 1.0f32, 2.0f32]
    let mut float_bytes = Vec::new();
    for v in [0.0f32, 1.0f32, 2.0f32] {
        float_bytes.extend_from_slice(&v.to_le_bytes());
    }
    test_component_reader_type(5126, &float_bytes, &[1.0, 2.0]);

    // 5130 DOUBLE: [0.0f64, 1.0f64, 2.0f64]
    let mut double_bytes = Vec::new();
    for v in [0.0f64, 1.0f64, 2.0f64] {
        double_bytes.extend_from_slice(&v.to_le_bytes());
    }
    test_component_reader_type(5130, &double_bytes, &[1.0, 2.0]);
}

#[test]
fn test_throws_for_unsupported_component_type() {
    assert!(get_component_reader(0).is_err());
}
