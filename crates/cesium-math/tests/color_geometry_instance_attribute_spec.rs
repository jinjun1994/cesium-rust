use cesium_math::{Color, ColorGeometryInstanceAttribute, ComponentDatatype};

#[test]
fn test_color_geometry_instance_attribute_constructor() {
    let attr = ColorGeometryInstanceAttribute::new(1.0, 1.0, 0.0, 0.5);
    assert_eq!(attr.component_datatype, ComponentDatatype::UnsignedByte);
    assert_eq!(attr.components_per_attribute, 4);
    assert!(attr.normalize);

    let expected = Color::new(1.0, 1.0, 0.0, 0.5).to_bytes();
    assert_eq!(attr.value, expected);
}

#[test]
fn test_color_geometry_instance_attribute_from_color() {
    let color = Color::CYAN;
    let attr = ColorGeometryInstanceAttribute::from_color(color);
    assert_eq!(attr.component_datatype, ComponentDatatype::UnsignedByte);
    assert_eq!(attr.components_per_attribute, 4);
    assert!(attr.normalize);
    assert_eq!(attr.value, color.to_bytes());
}

#[test]
fn test_color_geometry_instance_attribute_to_value() {
    let color = Color::new(0.2, 0.4, 0.6, 0.8);
    let val = ColorGeometryInstanceAttribute::to_value(&color);
    assert_eq!(val, color.to_bytes());
}

#[test]
fn test_color_geometry_instance_attribute_equals() {
    let a1 = ColorGeometryInstanceAttribute::new(0.1, 0.2, 0.3, 0.4);
    let a2 = ColorGeometryInstanceAttribute::new(0.1, 0.2, 0.3, 0.4);
    let diff_r = ColorGeometryInstanceAttribute::new(0.5, 0.2, 0.3, 0.4);
    let diff_g = ColorGeometryInstanceAttribute::new(0.1, 0.5, 0.3, 0.4);
    let diff_b = ColorGeometryInstanceAttribute::new(0.1, 0.2, 0.5, 0.4);
    let diff_a = ColorGeometryInstanceAttribute::new(0.1, 0.2, 0.3, 0.5);

    assert!(a1.equals(&a2));
    assert!(!a1.equals(&diff_r));
    assert!(!a1.equals(&diff_g));
    assert!(!a1.equals(&diff_b));
    assert!(!a1.equals(&diff_a));
}
