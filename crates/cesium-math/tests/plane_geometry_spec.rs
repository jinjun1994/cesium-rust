use cesium_math::{PlaneGeometry, PlaneOutlineGeometry, PrimitiveType, VertexFormat};

#[test]
fn test_plane_geometry_position_only() {
    let plane = PlaneGeometry::new(Some(VertexFormat::POSITION_ONLY));
    let geom = plane.create_geometry().unwrap();

    let pos = geom.attributes.position.unwrap();
    assert_eq!(pos.values.len(), 4 * 3);
    assert_eq!(geom.indices.len(), 2 * 3);
    assert_eq!(geom.primitive_type, PrimitiveType::Triangles);
}

#[test]
fn test_plane_geometry_all_attributes() {
    let plane = PlaneGeometry::new(Some(VertexFormat::ALL));
    let geom = plane.create_geometry().unwrap();

    let num_vertices = 4;
    let num_triangles = 2;
    assert_eq!(geom.attributes.position.unwrap().values.len(), num_vertices * 3);
    assert_eq!(geom.attributes.normal.unwrap().values.len(), num_vertices * 3);
    assert_eq!(geom.attributes.tangent.unwrap().values.len(), num_vertices * 3);
    assert_eq!(geom.attributes.bitangent.unwrap().values.len(), num_vertices * 3);
    assert_eq!(geom.attributes.st.unwrap().values.len(), num_vertices * 2);
    assert_eq!(geom.indices.len(), num_triangles * 3);
}

#[test]
fn test_plane_outline_geometry() {
    let outline = PlaneOutlineGeometry::new();
    let geom = outline.create_geometry().unwrap();

    assert_eq!(geom.attributes.position.unwrap().values.len(), 4 * 3);
    assert_eq!(geom.indices.len(), 4 * 2);
    assert_eq!(geom.primitive_type, PrimitiveType::Lines);
}
