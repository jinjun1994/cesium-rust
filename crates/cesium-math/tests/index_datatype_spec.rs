use cesium_math::{IndexDatatype, SIXTY_FOUR_KILOBYTES};

#[test]
fn test_index_datatype_validate() {
    assert!(IndexDatatype::validate(IndexDatatype::UNSIGNED_SHORT.as_gl_enum()));
    assert!(IndexDatatype::validate(IndexDatatype::UNSIGNED_BYTE.as_gl_enum()));
    assert!(IndexDatatype::validate(IndexDatatype::UNSIGNED_INT.as_gl_enum()));
    assert!(!IndexDatatype::validate(0));
    assert!(!IndexDatatype::validate(99999));
}

#[test]
fn test_index_datatype_create_typed_array() {
    let arr16 = IndexDatatype::create_typed_array(3, 3);
    assert_eq!(arr16.bytes_per_element(), 2);
    assert_eq!(arr16.len(), 3);

    let arr32 = IndexDatatype::create_typed_array(SIXTY_FOUR_KILOBYTES + 1, 3);
    assert_eq!(arr32.bytes_per_element(), 4);
    assert_eq!(arr32.len(), 3);
}

#[test]
fn test_index_datatype_create_typed_array_from_buffer() {
    let source_u16: Vec<u16> = (0..10).collect();
    let mut buffer16 = Vec::new();
    for v in &source_u16 {
        buffer16.extend_from_slice(&v.to_ne_bytes());
    }

    let index_buffer16 =
        IndexDatatype::create_typed_array_from_buffer(3, &buffer16, 0, 5).unwrap();
    assert_eq!(index_buffer16.bytes_per_element(), 2);
    assert_eq!(index_buffer16.len(), 5);
    assert_eq!(index_buffer16.get(0), Some(0));

    let index_buffer16_offset =
        IndexDatatype::create_typed_array_from_buffer(3, &buffer16, 2 * 5, 5).unwrap();
    assert_eq!(index_buffer16_offset.bytes_per_element(), 2);
    assert_eq!(index_buffer16_offset.len(), 5);
    assert_eq!(index_buffer16_offset.get(0), Some(5));

    let source_u32: Vec<u32> = (0..10).collect();
    let mut buffer32 = Vec::new();
    for v in &source_u32 {
        buffer32.extend_from_slice(&v.to_ne_bytes());
    }

    let index_buffer32 = IndexDatatype::create_typed_array_from_buffer(
        SIXTY_FOUR_KILOBYTES + 1,
        &buffer32,
        0,
        5,
    )
    .unwrap();
    assert_eq!(index_buffer32.bytes_per_element(), 4);
    assert_eq!(index_buffer32.len(), 5);
    assert_eq!(index_buffer32.get(0), Some(0));

    let index_buffer32_offset = IndexDatatype::create_typed_array_from_buffer(
        SIXTY_FOUR_KILOBYTES + 1,
        &buffer32,
        4 * 5,
        5,
    )
    .unwrap();
    assert_eq!(index_buffer32_offset.bytes_per_element(), 4);
    assert_eq!(index_buffer32_offset.len(), 5);
    assert_eq!(index_buffer32_offset.get(0), Some(5));
}

#[test]
fn test_index_datatype_get_size_in_bytes() {
    assert_eq!(IndexDatatype::UNSIGNED_BYTE.get_size_in_bytes(), 1);
    assert_eq!(IndexDatatype::UNSIGNED_SHORT.get_size_in_bytes(), 2);
    assert_eq!(IndexDatatype::UNSIGNED_INT.get_size_in_bytes(), 4);
}

#[test]
fn test_index_datatype_from_size_in_bytes() {
    assert_eq!(
        IndexDatatype::from_size_in_bytes(1).unwrap(),
        IndexDatatype::UNSIGNED_BYTE
    );
    assert_eq!(
        IndexDatatype::from_size_in_bytes(2).unwrap(),
        IndexDatatype::UNSIGNED_SHORT
    );
    assert_eq!(
        IndexDatatype::from_size_in_bytes(4).unwrap(),
        IndexDatatype::UNSIGNED_INT
    );
    assert!(IndexDatatype::from_size_in_bytes(3).is_err());
}
