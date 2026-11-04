use cesium_math::ComponentDatatype;

#[test]
fn test_component_datatype_sizes() {
    assert_eq!(ComponentDatatype::Byte.size_in_bytes(), 1);
    assert_eq!(ComponentDatatype::UnsignedByte.size_in_bytes(), 1);
    assert_eq!(ComponentDatatype::Short.size_in_bytes(), 2);
    assert_eq!(ComponentDatatype::UnsignedShort.size_in_bytes(), 2);
    assert_eq!(ComponentDatatype::Int.size_in_bytes(), 4);
    assert_eq!(ComponentDatatype::UnsignedInt.size_in_bytes(), 4);
    assert_eq!(ComponentDatatype::Float.size_in_bytes(), 4);
    assert_eq!(ComponentDatatype::Double.size_in_bytes(), 8);
}

#[test]
fn test_component_datatype_from_name() {
    assert_eq!(ComponentDatatype::from_name("BYTE"), Some(ComponentDatatype::Byte));
    assert_eq!(ComponentDatatype::from_name("UNSIGNED_BYTE"), Some(ComponentDatatype::UnsignedByte));
    assert_eq!(ComponentDatatype::from_name("SHORT"), Some(ComponentDatatype::Short));
    assert_eq!(ComponentDatatype::from_name("UNSIGNED_SHORT"), Some(ComponentDatatype::UnsignedShort));
    assert_eq!(ComponentDatatype::from_name("INT"), Some(ComponentDatatype::Int));
    assert_eq!(ComponentDatatype::from_name("UNSIGNED_INT"), Some(ComponentDatatype::UnsignedInt));
    assert_eq!(ComponentDatatype::from_name("FLOAT"), Some(ComponentDatatype::Float));
    assert_eq!(ComponentDatatype::from_name("DOUBLE"), Some(ComponentDatatype::Double));
    assert_eq!(ComponentDatatype::from_name("INVALID"), None);
}

#[test]
fn test_component_datatype_validate_and_from_value() {
    assert!(ComponentDatatype::validate(5120));
    assert!(ComponentDatatype::validate(5121));
    assert!(ComponentDatatype::validate(5122));
    assert!(ComponentDatatype::validate(5123));
    assert!(ComponentDatatype::validate(5124));
    assert!(ComponentDatatype::validate(5125));
    assert!(ComponentDatatype::validate(5126));
    assert!(ComponentDatatype::validate(5130));
    assert!(!ComponentDatatype::validate(0));
    assert!(!ComponentDatatype::validate(9999));

    assert_eq!(ComponentDatatype::from_value(5120), Some(ComponentDatatype::Byte));
    assert_eq!(ComponentDatatype::from_value(5126), Some(ComponentDatatype::Float));
    assert_eq!(ComponentDatatype::from_value(5130), Some(ComponentDatatype::Double));
    assert_eq!(ComponentDatatype::from_value(1234), None);
}
