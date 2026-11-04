use cesium_math::{
    Cartesian3, EllipseGeometry, EllipseOutlineGeometry, Ellipsoid, PrimitiveType, VertexFormat,
};

#[test]
fn test_ellipse_outline_geometry_positions() {
    let center = Cartesian3::from_degrees(0.0, 0.0, Some(0.0), Some(&Ellipsoid::WGS84)).unwrap();
    let outline = EllipseOutlineGeometry::new(
        center,
        1.0,
        1.0,
        Some(Ellipsoid::WGS84),
        Some(0.0),
        None,
        Some(0.1),
        Some(0.0),
        None,
    );
    let geom = outline.create_geometry().unwrap();

    let pos = geom.attributes.position.unwrap();
    assert_eq!(pos.values.len(), 8 * 3);
    assert_eq!(geom.indices.len(), 8 * 2);
    assert_eq!(geom.primitive_type, PrimitiveType::Lines);
    assert_eq!(geom.bounding_sphere.unwrap().radius, 1.0);

    let degen = EllipseOutlineGeometry::new(
        center, 0.0, 1.0, None, None, None, None, None, None,
    );
    assert!(degen.create_geometry().is_none());

    let invalid_order = EllipseOutlineGeometry::new(
        center, 1.0, 2.0, None, None, None, None, None, None,
    );
    assert!(invalid_order.create_geometry().is_none());
}

#[test]
fn test_ellipse_geometry_positions_and_triangles() {
    let center = Cartesian3::from_degrees(0.0, 0.0, Some(0.0), Some(&Ellipsoid::WGS84)).unwrap();
    let ellipse = EllipseGeometry::new(
        center,
        1.0,
        1.0,
        Some(Ellipsoid::WGS84),
        Some(0.0),
        None,
        Some(0.1),
        Some(0.0),
        Some(0.0),
        Some(VertexFormat::POSITION_ONLY),
    );
    let geom = ellipse.create_geometry().unwrap();

    let pos = geom.attributes.position.unwrap();
    assert_eq!(pos.values.len(), 16 * 3);
    assert_eq!(geom.indices.len(), 22 * 3);
    assert_eq!(geom.primitive_type, PrimitiveType::Triangles);
    assert_eq!(geom.bounding_sphere.unwrap().radius, 1.0);
}

#[test]
fn test_ellipse_geometry_all_attributes() {
    let center = Cartesian3::from_degrees(0.0, 0.0, Some(0.0), Some(&Ellipsoid::WGS84)).unwrap();
    let ellipse = EllipseGeometry::new(
        center,
        1.0,
        1.0,
        Some(Ellipsoid::WGS84),
        Some(0.0),
        None,
        Some(0.1),
        Some(0.0),
        Some(0.0),
        Some(VertexFormat::ALL),
    );
    let geom = ellipse.create_geometry().unwrap();

    assert!(geom.attributes.position.is_some());
    assert!(geom.attributes.normal.is_some());
    assert!(geom.attributes.st.is_some());
}
