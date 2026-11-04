use cesium_math::AttributeType;

#[test]
fn test_attribute_type_get_math_type() {
    assert_eq!(AttributeType::SCALAR.get_math_type(), "Number");
    assert_eq!(AttributeType::VEC2.get_math_type(), "Cartesian2");
    assert_eq!(AttributeType::VEC3.get_math_type(), "Cartesian3");
    assert_eq!(AttributeType::VEC4.get_math_type(), "Cartesian4");
    assert_eq!(AttributeType::MAT2.get_math_type(), "Matrix2");
    assert_eq!(AttributeType::MAT3.get_math_type(), "Matrix3");
    assert_eq!(AttributeType::MAT4.get_math_type(), "Matrix4");
}

#[test]
fn test_attribute_type_invalid_throws() {
    assert!(AttributeType::from_name("Invalid").is_err());
    assert!(AttributeType::from_name("").is_err());
    assert!(AttributeType::from_name("int").is_err());
}

#[test]
fn test_attribute_type_get_glsl_type() {
    assert_eq!(AttributeType::SCALAR.get_glsl_type(), "float");
    assert_eq!(AttributeType::VEC2.get_glsl_type(), "vec2");
    assert_eq!(AttributeType::VEC3.get_glsl_type(), "vec3");
    assert_eq!(AttributeType::VEC4.get_glsl_type(), "vec4");
    assert_eq!(AttributeType::MAT2.get_glsl_type(), "mat2");
    assert_eq!(AttributeType::MAT3.get_glsl_type(), "mat3");
    assert_eq!(AttributeType::MAT4.get_glsl_type(), "mat4");
}

#[test]
fn test_attribute_type_get_number_of_components() {
    assert_eq!(AttributeType::SCALAR.get_number_of_components(), 1);
    assert_eq!(AttributeType::VEC2.get_number_of_components(), 2);
    assert_eq!(AttributeType::VEC3.get_number_of_components(), 3);
    assert_eq!(AttributeType::VEC4.get_number_of_components(), 4);
    assert_eq!(AttributeType::MAT2.get_number_of_components(), 4);
    assert_eq!(AttributeType::MAT3.get_number_of_components(), 9);
    assert_eq!(AttributeType::MAT4.get_number_of_components(), 16);
}

#[test]
fn test_attribute_type_get_attribute_location_count() {
    assert_eq!(AttributeType::SCALAR.get_attribute_location_count(), 1);
    assert_eq!(AttributeType::VEC2.get_attribute_location_count(), 1);
    assert_eq!(AttributeType::VEC3.get_attribute_location_count(), 1);
    assert_eq!(AttributeType::VEC4.get_attribute_location_count(), 1);
    assert_eq!(AttributeType::MAT2.get_attribute_location_count(), 2);
    assert_eq!(AttributeType::MAT3.get_attribute_location_count(), 3);
    assert_eq!(AttributeType::MAT4.get_attribute_location_count(), 4);
}
