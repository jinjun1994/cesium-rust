use cesium_math::{ComponentDatatype, ShowGeometryInstanceAttribute};

#[test]
fn test_show_geometry_instance_attribute_constructor() {
    let attr = ShowGeometryInstanceAttribute::new(false);
    assert_eq!(attr.component_datatype, ComponentDatatype::UnsignedByte);
    assert_eq!(attr.components_per_attribute, 1);
    assert!(!attr.normalize);
    assert_eq!(attr.value, [0]);

    let attr_true = ShowGeometryInstanceAttribute::new(true);
    assert_eq!(attr_true.value, [1]);
}

#[test]
fn test_show_geometry_instance_attribute_to_value() {
    assert_eq!(ShowGeometryInstanceAttribute::to_value(true), [1]);
    assert_eq!(ShowGeometryInstanceAttribute::to_value(false), [0]);
}
