use cesium_math::PixelDatatype;

#[test]
fn test_pixel_datatype_to_webgl_constant() {
    assert_eq!(PixelDatatype::UNSIGNED_BYTE.to_webgl_constant(true), 0x1401);
    assert_eq!(PixelDatatype::UNSIGNED_SHORT.to_webgl_constant(true), 0x1403);
    assert_eq!(PixelDatatype::UNSIGNED_INT.to_webgl_constant(true), 0x1405);
    assert_eq!(PixelDatatype::FLOAT.to_webgl_constant(true), 0x1406);
    assert_eq!(PixelDatatype::HALF_FLOAT.to_webgl_constant(true), 0x140B);
    assert_eq!(PixelDatatype::HALF_FLOAT.to_webgl_constant(false), 0x8D61);
    assert_eq!(PixelDatatype::UNSIGNED_INT_24_8.to_webgl_constant(true), 0x84FA);
    assert_eq!(PixelDatatype::UNSIGNED_SHORT_4_4_4_4.to_webgl_constant(true), 0x8033);
    assert_eq!(PixelDatatype::UNSIGNED_SHORT_5_5_5_1.to_webgl_constant(true), 0x8034);
    assert_eq!(PixelDatatype::UNSIGNED_SHORT_5_6_5.to_webgl_constant(true), 0x8363);
}

#[test]
fn test_pixel_datatype_is_packed() {
    assert!(!PixelDatatype::UNSIGNED_BYTE.is_packed());
    assert!(!PixelDatatype::UNSIGNED_SHORT.is_packed());
    assert!(!PixelDatatype::UNSIGNED_INT.is_packed());
    assert!(!PixelDatatype::FLOAT.is_packed());
    assert!(!PixelDatatype::HALF_FLOAT.is_packed());
    assert!(PixelDatatype::UNSIGNED_INT_24_8.is_packed());
    assert!(PixelDatatype::UNSIGNED_SHORT_4_4_4_4.is_packed());
    assert!(PixelDatatype::UNSIGNED_SHORT_5_5_5_1.is_packed());
    assert!(PixelDatatype::UNSIGNED_SHORT_5_6_5.is_packed());
}

#[test]
fn test_pixel_datatype_size_in_bytes() {
    assert_eq!(PixelDatatype::UNSIGNED_BYTE.size_in_bytes(), 1);
    assert_eq!(PixelDatatype::UNSIGNED_SHORT.size_in_bytes(), 2);
    assert_eq!(PixelDatatype::UNSIGNED_SHORT_4_4_4_4.size_in_bytes(), 2);
    assert_eq!(PixelDatatype::UNSIGNED_SHORT_5_5_5_1.size_in_bytes(), 2);
    assert_eq!(PixelDatatype::UNSIGNED_SHORT_5_6_5.size_in_bytes(), 2);
    assert_eq!(PixelDatatype::HALF_FLOAT.size_in_bytes(), 2);
    assert_eq!(PixelDatatype::UNSIGNED_INT.size_in_bytes(), 4);
    assert_eq!(PixelDatatype::FLOAT.size_in_bytes(), 4);
    assert_eq!(PixelDatatype::UNSIGNED_INT_24_8.size_in_bytes(), 4);
}

#[test]
fn test_pixel_datatype_validate() {
    assert!(PixelDatatype::validate(PixelDatatype::UNSIGNED_BYTE as u32));
    assert!(PixelDatatype::validate(PixelDatatype::UNSIGNED_SHORT as u32));
    assert!(PixelDatatype::validate(PixelDatatype::UNSIGNED_INT as u32));
    assert!(PixelDatatype::validate(PixelDatatype::FLOAT as u32));
    assert!(PixelDatatype::validate(PixelDatatype::HALF_FLOAT as u32));
    assert!(PixelDatatype::validate(PixelDatatype::UNSIGNED_INT_24_8 as u32));
    assert!(PixelDatatype::validate(PixelDatatype::UNSIGNED_SHORT_4_4_4_4 as u32));
    assert!(PixelDatatype::validate(PixelDatatype::UNSIGNED_SHORT_5_5_5_1 as u32));
    assert!(PixelDatatype::validate(PixelDatatype::UNSIGNED_SHORT_5_6_5 as u32));
    assert!(!PixelDatatype::validate(999999));
}
