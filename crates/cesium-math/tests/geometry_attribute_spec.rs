use cesium_math::{ComponentDatatype, GeometryAttribute};

#[test]
fn test_geometry_attribute_constructor() {
    let values = vec![
        255.0, 0.0, 0.0, 255.0, 0.0, 255.0, 0.0, 255.0, 0.0, 0.0, 255.0, 255.0,
    ];
    let color = GeometryAttribute::with_normalize(
        ComponentDatatype::UnsignedByte,
        4,
        true,
        values.clone(),
    )
    .unwrap();

    assert_eq!(color.component_datatype, ComponentDatatype::UnsignedByte);
    assert_eq!(color.components_per_attribute, 4);
    assert!(color.normalize);
    assert_eq!(color.values, values);
}

#[test]
fn test_geometry_attribute_throws_invalid_components() {
    let values = vec![255.0, 0.0, 0.0];
    assert!(GeometryAttribute::with_normalize(
        ComponentDatatype::UnsignedByte,
        0,
        false,
        values.clone(),
    )
    .is_err());

    assert!(GeometryAttribute::with_normalize(
        ComponentDatatype::UnsignedByte,
        7,
        false,
        values,
    )
    .is_err());
}
