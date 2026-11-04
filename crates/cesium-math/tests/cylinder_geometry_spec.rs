use cesium_math::{CylinderGeometry, CylinderOutlineGeometry, PrimitiveType, VertexFormat};

#[test]
fn test_cylinder_geometry_positions_and_triangles() {
    let cylinder = CylinderGeometry::new(
        1.0,
        1.0,
        1.0,
        Some(3),
        Some(VertexFormat::POSITION_ONLY),
    );
    let geom = cylinder.create_geometry().unwrap();

    let num_vertices = 12;
    let num_triangles = 8;
    assert_eq!(geom.attributes.position.unwrap().values.len(), num_vertices * 3);
    assert_eq!(geom.indices.len(), num_triangles * 3);
    assert_eq!(geom.primitive_type, PrimitiveType::Triangles);

    // Degenerate checks
    let degen1 = CylinderGeometry::new(0.0, 1.0, 1.0, None, None);
    assert!(degen1.create_geometry().is_none());

    let degen2 = CylinderGeometry::new(1.0, 0.0, 0.0, None, None);
    assert!(degen2.create_geometry().is_none());
}

#[test]
fn test_cylinder_outline_geometry() {
    let outline = CylinderOutlineGeometry::new(
        1.0,
        1.0,
        1.0,
        Some(3),
        Some(0), // no vertical lines
    );
    let geom = outline.create_geometry().unwrap();

    assert_eq!(geom.attributes.position.unwrap().values.len(), 6 * 3);
    assert_eq!(geom.indices.len(), 6 * 2); // 6 lines (3 top + 3 bottom)
    assert_eq!(geom.primitive_type, PrimitiveType::Lines);

    // With vertical lines
    let outline_vertical = CylinderOutlineGeometry::new(
        1.0,
        1.0,
        1.0,
        Some(4),
        Some(4),
    );
    let geom_v = outline_vertical.create_geometry().unwrap();
    assert_eq!(geom_v.attributes.position.unwrap().values.len(), 8 * 3);
    // 4 bottom + 4 top + 4 vertical = 12 lines = 24 indices
    assert_eq!(geom_v.indices.len(), 24);
}
