use cesium_math::{
    Cartesian3, CoplanarPolygonGeometry, CoplanarPolygonOutlineGeometry, Ellipsoid,
    PrimitiveType, VertexFormat,
};

#[test]
fn test_coplanar_polygon_geometry_positions() {
    let p1 = Cartesian3::from_degrees(-1.0, -1.0, Some(0.0), Some(&Ellipsoid::WGS84)).unwrap();
    let p2 = Cartesian3::from_degrees(-1.0, 0.0, Some(1.0), Some(&Ellipsoid::WGS84)).unwrap();
    let p3 = Cartesian3::from_degrees(-1.0, 1.0, Some(1.0), Some(&Ellipsoid::WGS84)).unwrap();
    let p4 = Cartesian3::from_degrees(-1.0, 2.0, Some(0.0), Some(&Ellipsoid::WGS84)).unwrap();

    let geom_spec = CoplanarPolygonGeometry::from_positions(
        vec![p1, p2, p3, p4],
        Some(VertexFormat::POSITION_ONLY),
        None,
        None,
    );

    let p = geom_spec.create_geometry().unwrap();
    let pos = p.attributes.position.unwrap();

    assert_eq!(pos.values.len(), 4 * 3);
    assert_eq!(p.indices.len(), 2 * 3);
    assert_eq!(p.primitive_type, PrimitiveType::Triangles);
}

#[test]
fn test_coplanar_polygon_geometry_all_attributes() {
    let p1 = Cartesian3::from_degrees(-1.0, -1.0, Some(0.0), Some(&Ellipsoid::WGS84)).unwrap();
    let p2 = Cartesian3::from_degrees(-1.0, 0.0, Some(1.0), Some(&Ellipsoid::WGS84)).unwrap();
    let p3 = Cartesian3::from_degrees(-1.0, 1.0, Some(1.0), Some(&Ellipsoid::WGS84)).unwrap();
    let p4 = Cartesian3::from_degrees(-1.0, 2.0, Some(0.0), Some(&Ellipsoid::WGS84)).unwrap();

    let geom_spec = CoplanarPolygonGeometry::from_positions(
        vec![p1, p2, p3, p4],
        Some(VertexFormat::ALL),
        None,
        None,
    );

    let p = geom_spec.create_geometry().unwrap();
    let num_vertices = 4;
    let num_triangles = 2;

    assert_eq!(p.attributes.position.as_ref().unwrap().values.len(), num_vertices * 3);
    assert_eq!(p.attributes.st.as_ref().unwrap().values.len(), num_vertices * 2);
    assert_eq!(p.attributes.normal.as_ref().unwrap().values.len(), num_vertices * 3);
    assert_eq!(p.attributes.tangent.as_ref().unwrap().values.len(), num_vertices * 3);
    assert_eq!(p.attributes.bitangent.as_ref().unwrap().values.len(), num_vertices * 3);
    assert_eq!(p.indices.len(), num_triangles * 3);
}

#[test]
fn test_coplanar_polygon_outline_geometry_positions() {
    let p1 = Cartesian3::from_degrees(-1.0, -1.0, Some(0.0), Some(&Ellipsoid::WGS84)).unwrap();
    let p2 = Cartesian3::from_degrees(-1.0, 0.0, Some(1.0), Some(&Ellipsoid::WGS84)).unwrap();
    let p3 = Cartesian3::from_degrees(-1.0, 1.0, Some(1.0), Some(&Ellipsoid::WGS84)).unwrap();
    let p4 = Cartesian3::from_degrees(-1.0, 2.0, Some(0.0), Some(&Ellipsoid::WGS84)).unwrap();

    let outline_spec = CoplanarPolygonOutlineGeometry::from_positions(
        vec![p1, p2, p3, p4],
    );

    let p = outline_spec.create_geometry().unwrap();
    let pos = p.attributes.position.unwrap();

    assert_eq!(pos.values.len(), 4 * 3);
    assert_eq!(p.indices.len(), 4 * 2);
    assert_eq!(p.primitive_type, PrimitiveType::Lines);
}

#[test]
fn test_coplanar_polygon_degenerate_cases() {
    let p1 = Cartesian3::from_degrees(49.0, 18.0, Some(1000.0), Some(&Ellipsoid::WGS84)).unwrap();
    let p2 = Cartesian3::from_degrees(49.0, 18.0, Some(5000.0), Some(&Ellipsoid::WGS84)).unwrap();

    // Less than 3 unique positions (only 2 distinct)
    let geom_less = CoplanarPolygonGeometry::from_positions(vec![p1, p2, p1], None, None, None);
    assert!(geom_less.create_geometry().is_none());

    // Linear positions
    let l1 = Cartesian3::new(0.0, 0.0, 1.0);
    let l2 = Cartesian3::new(0.0, 0.0, 2.0);
    let l3 = Cartesian3::new(0.0, 0.0, 3.0);
    let geom_linear = CoplanarPolygonGeometry::from_positions(vec![l1, l2, l3], None, None, None);
    assert!(geom_linear.create_geometry().is_none());
}
