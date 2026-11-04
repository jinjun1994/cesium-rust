use cesium_math::{PrimitiveType, SphereGeometry, SphereOutlineGeometry, VertexFormat};

#[test]
fn test_sphere_geometry_positions_and_triangles() {
    let sphere = SphereGeometry::new(
        Some(1.0),
        Some(3),
        Some(3),
        Some(VertexFormat::POSITION_ONLY),
    );
    let geom = sphere.create_geometry().unwrap();

    let num_vertices = 36;
    let num_triangles = 18;
    assert_eq!(geom.attributes.position.unwrap().values.len(), num_vertices * 3);
    assert_eq!(geom.indices.len(), num_triangles * 3);
    assert_eq!(geom.primitive_type, PrimitiveType::Triangles);

    // Degenerate radius
    let degen = SphereGeometry::new(Some(0.0), None, None, None);
    assert!(degen.create_geometry().is_none());
}

#[test]
fn test_sphere_geometry_all_attributes() {
    let sphere = SphereGeometry::new(
        Some(10.0),
        Some(3),
        Some(3),
        Some(VertexFormat::ALL),
    );
    let geom = sphere.create_geometry().unwrap();

    let num_vertices = 36;
    assert_eq!(geom.attributes.position.unwrap().values.len(), num_vertices * 3);
    assert_eq!(geom.attributes.normal.unwrap().values.len(), num_vertices * 3);
    assert_eq!(geom.attributes.st.unwrap().values.len(), num_vertices * 2);
}

#[test]
fn test_sphere_outline_geometry() {
    let outline = SphereOutlineGeometry::new(
        Some(1.0),
        Some(2),
        Some(2),
        Some(2),
    );
    let geom = outline.create_geometry().unwrap();

    assert_eq!(geom.attributes.position.unwrap().values.len(), 12 * 3);
    assert_eq!(geom.indices.len(), 6 * 2);
    assert_eq!(geom.primitive_type, PrimitiveType::Lines);

    let degen = SphereOutlineGeometry::new(Some(0.0), None, None, None);
    assert!(degen.create_geometry().is_none());
}
