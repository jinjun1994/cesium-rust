use cesium_math::{ComponentDatatype, GeometryInstanceAttribute};

#[test]
fn test_geometry_instance_attribute_constructor() {
    let value = vec![255.0, 255.0, 0.0, 255.0];
    let color = GeometryInstanceAttribute::new(
        ComponentDatatype::UnsignedByte,
        4,
        true,
        value.clone(),
    )
    .unwrap();

    assert_eq!(color.component_datatype, ComponentDatatype::UnsignedByte);
    assert_eq!(color.components_per_attribute, 4);
    assert!(color.normalize);
    assert_eq!(color.value, value);
}

#[test]
fn test_geometry_instance_attribute_throws_invalid_components() {
    let value = vec![255.0, 255.0, 0.0, 255.0];
    assert!(GeometryInstanceAttribute::new(
        ComponentDatatype::UnsignedByte,
        0,
        false,
        value.clone(),
    )
    .is_err());

    assert!(GeometryInstanceAttribute::new(
        ComponentDatatype::UnsignedByte,
        7,
        false,
        value,
    )
    .is_err());
}
