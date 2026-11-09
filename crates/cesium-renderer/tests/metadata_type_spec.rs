use cesium_renderer::{MetadataMathType, MetadataType};

#[test]
fn test_is_vector_type_works() {
    assert!(MetadataType::Vec2.is_vector_type());
    assert!(MetadataType::Vec3.is_vector_type());
    assert!(MetadataType::Vec4.is_vector_type());
    assert!(!MetadataType::Mat2.is_vector_type());
    assert!(!MetadataType::Mat3.is_vector_type());
    assert!(!MetadataType::Mat4.is_vector_type());
    assert!(!MetadataType::Scalar.is_vector_type());
    assert!(!MetadataType::Enum.is_vector_type());
    assert!(!MetadataType::Boolean.is_vector_type());
    assert!(!MetadataType::String.is_vector_type());
}

#[test]
fn test_is_matrix_type_works() {
    assert!(!MetadataType::Vec2.is_matrix_type());
    assert!(!MetadataType::Vec3.is_matrix_type());
    assert!(!MetadataType::Vec4.is_matrix_type());
    assert!(MetadataType::Mat2.is_matrix_type());
    assert!(MetadataType::Mat3.is_matrix_type());
    assert!(MetadataType::Mat4.is_matrix_type());
    assert!(!MetadataType::Scalar.is_matrix_type());
    assert!(!MetadataType::Enum.is_matrix_type());
    assert!(!MetadataType::Boolean.is_matrix_type());
    assert!(!MetadataType::String.is_matrix_type());
}

#[test]
fn test_get_component_count_works() {
    assert_eq!(MetadataType::Vec2.get_component_count(), 2);
    assert_eq!(MetadataType::Vec3.get_component_count(), 3);
    assert_eq!(MetadataType::Vec4.get_component_count(), 4);
    assert_eq!(MetadataType::Mat2.get_component_count(), 4);
    assert_eq!(MetadataType::Mat3.get_component_count(), 9);
    assert_eq!(MetadataType::Mat4.get_component_count(), 16);
    assert_eq!(MetadataType::Scalar.get_component_count(), 1);
    assert_eq!(MetadataType::Enum.get_component_count(), 1);
    assert_eq!(MetadataType::Boolean.get_component_count(), 1);
    assert_eq!(MetadataType::String.get_component_count(), 1);
}

#[test]
fn test_get_component_count_throws_for_invalid_type() {
    assert!(MetadataType::get_component_count_from_str("NOT_A_TYPE").is_err());
}

#[test]
fn test_get_math_type_works() {
    assert_eq!(
        MetadataType::Vec2.get_math_type(),
        Some(MetadataMathType::Cartesian2)
    );
    assert_eq!(
        MetadataType::Vec3.get_math_type(),
        Some(MetadataMathType::Cartesian3)
    );
    assert_eq!(
        MetadataType::Vec4.get_math_type(),
        Some(MetadataMathType::Cartesian4)
    );
    assert_eq!(
        MetadataType::Mat2.get_math_type(),
        Some(MetadataMathType::Matrix2)
    );
    assert_eq!(
        MetadataType::Mat3.get_math_type(),
        Some(MetadataMathType::Matrix3)
    );
    assert_eq!(
        MetadataType::Mat4.get_math_type(),
        Some(MetadataMathType::Matrix4)
    );
    assert_eq!(MetadataType::Scalar.get_math_type(), None);
    assert_eq!(MetadataType::Enum.get_math_type(), None);
    assert_eq!(MetadataType::Boolean.get_math_type(), None);
    assert_eq!(MetadataType::String.get_math_type(), None);
}
