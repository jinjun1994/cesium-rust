use cesium_math::{
    Cartesian3, CircleGeometry, CircleOutlineGeometry, Ellipsoid, PrimitiveType, VertexFormat,
};

#[test]
fn test_circle_outline_geometry_positions() {
    let center = Cartesian3::from_degrees(0.0, 0.0, Some(0.0), Some(&Ellipsoid::WGS84)).unwrap();
    let outline = CircleOutlineGeometry::new(
        center,
        1.0,
        Some(Ellipsoid::WGS84),
        Some(0.0),
        None,
        Some(0.1),
        None,
    );
    let geom = outline.create_geometry().unwrap();

    let pos = geom.attributes.position.unwrap();
    assert_eq!(pos.values.len(), 8 * 3);
    assert_eq!(geom.indices.len(), 8 * 2);
    assert_eq!(geom.primitive_type, PrimitiveType::Lines);
    assert_eq!(geom.bounding_sphere.unwrap().radius, 1.0);
}

#[test]
fn test_circle_outline_geometry_extruded() {
    let center = Cartesian3::from_degrees(0.0, 0.0, Some(0.0), Some(&Ellipsoid::WGS84)).unwrap();
    let outline = CircleOutlineGeometry::new(
        center,
        1.0,
        Some(Ellipsoid::WGS84),
        Some(0.0),
        Some(5.0),
        Some(0.1),
        None,
    );
    let geom = outline.create_geometry().unwrap();

    let pos = geom.attributes.position.unwrap();
    // 8 top circle + 8 bottom circle
    assert_eq!(pos.values.len(), 16 * 3);
    // 8 top + 8 bottom + 8 vertical lines
    assert_eq!(geom.indices.len(), 24 * 2);
    assert_eq!(geom.primitive_type, PrimitiveType::Lines);
}

#[test]
fn test_circle_outline_geometry_extruded_no_vertical_lines() {
    let center = Cartesian3::from_degrees(0.0, 0.0, Some(0.0), Some(&Ellipsoid::WGS84)).unwrap();
    let outline = CircleOutlineGeometry::new(
        center,
        1.0,
        Some(Ellipsoid::WGS84),
        Some(0.0),
        Some(10000.0),
        Some(0.1),
        Some(0),
    );
    let geom = outline.create_geometry().unwrap();

    let pos = geom.attributes.position.unwrap();
    assert_eq!(pos.values.len(), 16 * 3);
    // 8 top + 8 bottom
    assert_eq!(geom.indices.len(), 16 * 2);
}

#[test]
fn test_circle_geometry_positions() {
    let center = Cartesian3::from_degrees(0.0, 0.0, Some(0.0), Some(&Ellipsoid::WGS84)).unwrap();
    let circle = CircleGeometry::new(
        center,
        1.0,
        Some(Ellipsoid::WGS84),
        Some(0.0),
        None,
        Some(0.1),
        Some(VertexFormat::POSITION_ONLY),
        Some(0.0),
    );
    let geom = circle.create_geometry().unwrap();

    let pos = geom.attributes.position.unwrap();
    assert_eq!(pos.values.len(), 16 * 3);
    assert_eq!(geom.indices.len(), 22 * 3);
    assert_eq!(geom.primitive_type, PrimitiveType::Triangles);
    assert_eq!(geom.bounding_sphere.unwrap().radius, 1.0);
}
