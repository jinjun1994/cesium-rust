use cesium_renderer::MetadataComponentType;

#[test]
fn test_get_minimum() {
    assert_eq!(
        MetadataComponentType::get_minimum(MetadataComponentType::Int8),
        -128.0
    );
    assert_eq!(
        MetadataComponentType::get_minimum(MetadataComponentType::Uint8),
        0.0
    );
    assert_eq!(
        MetadataComponentType::get_minimum(MetadataComponentType::Int16),
        -32768.0
    );
    assert_eq!(
        MetadataComponentType::get_minimum(MetadataComponentType::Uint16),
        0.0
    );
    assert_eq!(
        MetadataComponentType::get_minimum(MetadataComponentType::Int32),
        -2147483648.0
    );
    assert_eq!(
        MetadataComponentType::get_minimum(MetadataComponentType::Uint32),
        0.0
    );
    assert_eq!(
        MetadataComponentType::get_minimum(MetadataComponentType::Float32),
        -340282346638528859811704183484516925440.0
    );
    assert_eq!(
        MetadataComponentType::get_minimum(MetadataComponentType::Float64),
        -f64::MAX
    );
    assert_eq!(
        MetadataComponentType::get_minimum(MetadataComponentType::Int64),
        -9223372036854775808.0
    );
    assert_eq!(
        MetadataComponentType::get_minimum(MetadataComponentType::Uint64),
        0.0
    );
}

#[test]
fn test_get_maximum() {
    assert_eq!(
        MetadataComponentType::get_maximum(MetadataComponentType::Int8),
        127.0
    );
    assert_eq!(
        MetadataComponentType::get_maximum(MetadataComponentType::Uint8),
        255.0
    );
    assert_eq!(
        MetadataComponentType::get_maximum(MetadataComponentType::Int16),
        32767.0
    );
    assert_eq!(
        MetadataComponentType::get_maximum(MetadataComponentType::Uint16),
        65535.0
    );
    assert_eq!(
        MetadataComponentType::get_maximum(MetadataComponentType::Int32),
        2147483647.0
    );
    assert_eq!(
        MetadataComponentType::get_maximum(MetadataComponentType::Uint32),
        4294967295.0
    );
    assert_eq!(
        MetadataComponentType::get_maximum(MetadataComponentType::Float32),
        340282346638528859811704183484516925440.0
    );
    assert_eq!(
        MetadataComponentType::get_maximum(MetadataComponentType::Float64),
        f64::MAX
    );
    assert_eq!(
        MetadataComponentType::get_maximum(MetadataComponentType::Int64),
        9223372036854775807.0
    );
    assert_eq!(
        MetadataComponentType::get_maximum(MetadataComponentType::Uint64),
        18446744073709551615.0
    );
}

#[test]
fn test_is_integer_type() {
    assert!(MetadataComponentType::Int8.is_integer_type());
    assert!(MetadataComponentType::Uint8.is_integer_type());
    assert!(MetadataComponentType::Int16.is_integer_type());
    assert!(MetadataComponentType::Uint16.is_integer_type());
    assert!(MetadataComponentType::Int32.is_integer_type());
    assert!(MetadataComponentType::Uint32.is_integer_type());
    assert!(MetadataComponentType::Int64.is_integer_type());
    assert!(MetadataComponentType::Uint64.is_integer_type());
    assert!(!MetadataComponentType::Float32.is_integer_type());
    assert!(!MetadataComponentType::Float64.is_integer_type());
}

#[test]
fn test_is_unsigned_integer_type() {
    assert!(!MetadataComponentType::Int8.is_unsigned_integer_type());
    assert!(MetadataComponentType::Uint8.is_unsigned_integer_type());
    assert!(!MetadataComponentType::Int16.is_unsigned_integer_type());
    assert!(MetadataComponentType::Uint16.is_unsigned_integer_type());
    assert!(!MetadataComponentType::Int32.is_unsigned_integer_type());
    assert!(MetadataComponentType::Uint32.is_unsigned_integer_type());
    assert!(!MetadataComponentType::Int64.is_unsigned_integer_type());
    assert!(MetadataComponentType::Uint64.is_unsigned_integer_type());
    assert!(!MetadataComponentType::Float32.is_unsigned_integer_type());
    assert!(!MetadataComponentType::Float64.is_unsigned_integer_type());
}

#[test]
fn test_normalizes_integers() {
    // Signed integers
    assert_eq!(
        MetadataComponentType::normalize(127.0, MetadataComponentType::Int8).unwrap(),
        1.0
    );
    assert_eq!(
        MetadataComponentType::normalize(-127.0, MetadataComponentType::Int8).unwrap(),
        -1.0
    );
    assert_eq!(
        MetadataComponentType::normalize(-128.0, MetadataComponentType::Int8).unwrap(),
        -1.0
    );
    assert_eq!(
        MetadataComponentType::normalize(0.0, MetadataComponentType::Int8).unwrap(),
        0.0
    );

    // Unsigned integers
    assert_eq!(
        MetadataComponentType::normalize(255.0, MetadataComponentType::Uint8).unwrap(),
        1.0
    );
    assert_eq!(
        MetadataComponentType::normalize(0.0, MetadataComponentType::Uint8).unwrap(),
        0.0
    );

    // Float type throws
    assert!(MetadataComponentType::normalize(1.0, MetadataComponentType::Float32).is_err());
}

#[test]
fn test_unnormalizes_integers() {
    assert_eq!(
        MetadataComponentType::unnormalize(1.0, MetadataComponentType::Int8).unwrap(),
        127.0
    );
    assert_eq!(
        MetadataComponentType::unnormalize(-1.0, MetadataComponentType::Int8).unwrap(),
        -127.0
    );
    assert_eq!(
        MetadataComponentType::unnormalize(0.0, MetadataComponentType::Int8).unwrap(),
        0.0
    );

    assert_eq!(
        MetadataComponentType::unnormalize(1.0, MetadataComponentType::Uint8).unwrap(),
        255.0
    );
    assert_eq!(
        MetadataComponentType::unnormalize(0.0, MetadataComponentType::Uint8).unwrap(),
        0.0
    );

    // Clamps values outside [-1, 1] or [0, 1]
    assert_eq!(
        MetadataComponentType::unnormalize(2.0, MetadataComponentType::Int8).unwrap(),
        127.0
    );
    assert_eq!(
        MetadataComponentType::unnormalize(-2.0, MetadataComponentType::Int8).unwrap(),
        -127.0
    );
    assert_eq!(
        MetadataComponentType::unnormalize(-1.0, MetadataComponentType::Uint8).unwrap(),
        0.0
    );

    // Float type throws
    assert!(MetadataComponentType::unnormalize(1.0, MetadataComponentType::Float32).is_err());
}

#[test]
fn test_get_size_in_bytes() {
    assert_eq!(
        MetadataComponentType::get_size_in_bytes(MetadataComponentType::Int8),
        1
    );
    assert_eq!(
        MetadataComponentType::get_size_in_bytes(MetadataComponentType::Uint8),
        1
    );
    assert_eq!(
        MetadataComponentType::get_size_in_bytes(MetadataComponentType::Int16),
        2
    );
    assert_eq!(
        MetadataComponentType::get_size_in_bytes(MetadataComponentType::Uint16),
        2
    );
    assert_eq!(
        MetadataComponentType::get_size_in_bytes(MetadataComponentType::Int32),
        4
    );
    assert_eq!(
        MetadataComponentType::get_size_in_bytes(MetadataComponentType::Uint32),
        4
    );
    assert_eq!(
        MetadataComponentType::get_size_in_bytes(MetadataComponentType::Float32),
        4
    );
    assert_eq!(
        MetadataComponentType::get_size_in_bytes(MetadataComponentType::Int64),
        8
    );
    assert_eq!(
        MetadataComponentType::get_size_in_bytes(MetadataComponentType::Uint64),
        8
    );
    assert_eq!(
        MetadataComponentType::get_size_in_bytes(MetadataComponentType::Float64),
        8
    );
}

#[test]
fn test_category_and_gpu_types() {
    assert_eq!(MetadataComponentType::Int8.category(), "int");
    assert_eq!(MetadataComponentType::Uint8.category(), "uint");
    assert_eq!(MetadataComponentType::Float32.category(), "float");

    assert_eq!(
        MetadataComponentType::Int64.gpu_component_type(),
        MetadataComponentType::Int32
    );
    assert_eq!(
        MetadataComponentType::Uint64.gpu_component_type(),
        MetadataComponentType::Uint32
    );
    assert_eq!(
        MetadataComponentType::Float64.gpu_component_type(),
        MetadataComponentType::Float32
    );
    assert_eq!(
        MetadataComponentType::Int16.gpu_component_type(),
        MetadataComponentType::Int16
    );

    assert!(MetadataComponentType::Int32.is_vector_compatible());
    assert!(!MetadataComponentType::Int64.is_vector_compatible());
    assert!(!MetadataComponentType::Uint64.is_vector_compatible());
}

#[test]
fn test_value_transform() {
    let transformed = MetadataComponentType::apply_value_transform(10.0, 5.0, 2.0);
    assert_eq!(transformed, 25.0);

    let untransformed = MetadataComponentType::unapply_value_transform(25.0, 5.0, 2.0);
    assert_eq!(untransformed, 10.0);

    let zero_scale = MetadataComponentType::unapply_value_transform(25.0, 5.0, 0.0);
    assert_eq!(zero_scale, 0.0);
}
