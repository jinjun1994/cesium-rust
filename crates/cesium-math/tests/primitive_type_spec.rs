use cesium_math::PrimitiveType;

#[test]
fn test_primitive_type_validate() {
    assert!(PrimitiveType::validate(Some(PrimitiveType::Points as i32)));
    assert!(PrimitiveType::validate(Some(PrimitiveType::Lines as i32)));
    assert!(PrimitiveType::validate(Some(PrimitiveType::LineLoop as i32)));
    assert!(PrimitiveType::validate(Some(PrimitiveType::LineStrip as i32)));
    assert!(PrimitiveType::validate(Some(PrimitiveType::Triangles as i32)));
    assert!(PrimitiveType::validate(Some(PrimitiveType::TriangleStrip as i32)));
    assert!(PrimitiveType::validate(Some(PrimitiveType::TriangleFan as i32)));
    assert!(!PrimitiveType::validate(None));
    assert!(!PrimitiveType::validate(Some(7)));
    assert!(!PrimitiveType::validate(Some(-1)));
}

#[test]
fn test_primitive_type_is_lines() {
    assert!(!PrimitiveType::Points.is_lines());
    assert!(PrimitiveType::Lines.is_lines());
    assert!(PrimitiveType::LineLoop.is_lines());
    assert!(PrimitiveType::LineStrip.is_lines());
    assert!(!PrimitiveType::Triangles.is_lines());
    assert!(!PrimitiveType::TriangleStrip.is_lines());
    assert!(!PrimitiveType::TriangleFan.is_lines());
}

#[test]
fn test_primitive_type_is_triangles() {
    assert!(!PrimitiveType::Points.is_triangles());
    assert!(!PrimitiveType::Lines.is_triangles());
    assert!(!PrimitiveType::LineLoop.is_triangles());
    assert!(!PrimitiveType::LineStrip.is_triangles());
    assert!(PrimitiveType::Triangles.is_triangles());
    assert!(PrimitiveType::TriangleStrip.is_triangles());
    assert!(PrimitiveType::TriangleFan.is_triangles());
}
