use cesium_math::{
    Cartesian3, EllipsoidGeometry, EllipsoidOutlineGeometry, PrimitiveType, VertexFormat,
};

#[test]
fn test_ellipsoid_geometry_triangles() {
    let ell = EllipsoidGeometry::new(
        Some(Cartesian3::new(10.0, 20.0, 30.0)),
        None,
        None,
        None,
        None,
        None,
        Some(3),
        Some(3),
        Some(VertexFormat::POSITION_ONLY),
    );
    let geom = ell.create_geometry().unwrap();

    let num_vertices = 36;
    let num_triangles = 18;
    assert_eq!(geom.attributes.position.unwrap().values.len(), num_vertices * 3);
    assert_eq!(geom.indices.len(), num_triangles * 3);
    assert_eq!(geom.primitive_type, PrimitiveType::Triangles);

    // Negative radius
    let degen = EllipsoidGeometry::new(
        Some(Cartesian3::new(-1.0, 1.0, 1.0)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    assert!(degen.create_geometry().is_none());
}

#[test]
fn test_ellipsoid_outline_geometry() {
    let outline = EllipsoidOutlineGeometry::new(
        Some(Cartesian3::new(10.0, 20.0, 30.0)),
        None,
        None,
        None,
        None,
        None,
        Some(2),
        Some(2),
        Some(2),
    );
    let geom = outline.create_geometry().unwrap();

    assert_eq!(geom.attributes.position.unwrap().values.len(), 12 * 3);
    assert_eq!(geom.indices.len(), 6 * 2);
    assert_eq!(geom.primitive_type, PrimitiveType::Lines);
}
