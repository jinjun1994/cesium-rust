use cesium_math::{
    Cartesian2, Cartesian3, CornerType, Ellipsoid, PolylineVolumeGeometry,
    PolylineVolumeGeometryOptions, VertexFormat,
};

fn test_shape() -> Vec<Cartesian2> {
    vec![
        Cartesian2::new(-100.0, -100.0),
        Cartesian2::new(100.0, -100.0),
        Cartesian2::new(100.0, 100.0),
        Cartesian2::new(-100.0, 100.0),
    ]
}

#[test]
fn test_polyline_volume_geometry_returns_none_without_two_unique_polyline_positions() {
    let geom = PolylineVolumeGeometry::new(PolylineVolumeGeometryOptions {
        polyline_positions: vec![Cartesian3::ZERO],
        shape_positions: test_shape(),
        ellipsoid: None,
        corner_type: None,
        vertex_format: None,
        granularity: None,
    });
    assert!(geom.create_geometry().is_none());
}

#[test]
fn test_polyline_volume_geometry_returns_none_without_three_unique_shape_positions() {
    let geom = PolylineVolumeGeometry::new(PolylineVolumeGeometryOptions {
        polyline_positions: vec![Cartesian3::UNIT_X, Cartesian3::UNIT_Y],
        shape_positions: vec![
            Cartesian2::UNIT_X,
            Cartesian2::UNIT_X,
            Cartesian2::UNIT_X,
        ],
        ellipsoid: None,
        corner_type: None,
        vertex_format: None,
        granularity: None,
    });
    assert!(geom.create_geometry().is_none());
}

#[test]
fn test_polyline_volume_geometry_computes_positions() {
    let polyline_positions =
        Cartesian3::from_degrees_array(&[90.0, -30.0, 90.0, -35.0], None).unwrap();

    let geom = PolylineVolumeGeometry::new(PolylineVolumeGeometryOptions {
        polyline_positions,
        shape_positions: test_shape(),
        ellipsoid: None,
        corner_type: Some(CornerType::Mitered),
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        granularity: None,
    });

    let m = geom.create_geometry().expect("create_geometry failed");
    let pos = m.attributes.position.expect("missing position");
    // 6 positions * 4 box positions * 2 duplicate + 4 positions * 2 ends = 56 vertices
    assert_eq!(pos.values.len(), 56 * 3);
    // 5 segments * 8 triangles per segment + 2 triangles * 2 ends = 44 triangles
    assert_eq!(m.indices.len(), 44 * 3);
}

#[test]
fn test_polyline_volume_geometry_computes_positions_clockwise_shape() {
    let polyline_positions =
        Cartesian3::from_degrees_array(&[90.0, -30.0, 90.0, -35.0], None).unwrap();

    let mut shape = test_shape();
    shape.reverse();

    let geom = PolylineVolumeGeometry::new(PolylineVolumeGeometryOptions {
        polyline_positions,
        shape_positions: shape,
        ellipsoid: None,
        corner_type: Some(CornerType::Mitered),
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        granularity: None,
    });

    let m = geom.create_geometry().expect("create_geometry failed");
    let pos = m.attributes.position.expect("missing position");
    assert_eq!(pos.values.len(), 56 * 3);
    assert_eq!(m.indices.len(), 44 * 3);
}

#[test]
fn test_polyline_volume_geometry_computes_most_vertex_attributes() {
    let polyline_positions =
        Cartesian3::from_degrees_array(&[90.0, -30.0, 90.0, -35.0], None).unwrap();

    let geom = PolylineVolumeGeometry::new(PolylineVolumeGeometryOptions {
        polyline_positions,
        shape_positions: test_shape(),
        ellipsoid: None,
        corner_type: Some(CornerType::Mitered),
        vertex_format: Some(VertexFormat::POSITION_NORMAL_AND_ST),
        granularity: None,
    });

    let m = geom.create_geometry().expect("create_geometry failed");
    assert!(m.attributes.position.is_some());
    assert!(m.attributes.normal.is_some());
    assert!(m.attributes.st.is_some());
    assert!(m.attributes.tangent.is_none());
    assert!(m.attributes.bitangent.is_none());
    assert_eq!(m.attributes.position.unwrap().values.len(), 56 * 3);
    assert_eq!(m.attributes.normal.unwrap().values.len(), 56 * 3);
    assert_eq!(m.attributes.st.unwrap().values.len(), 56 * 2);
    assert_eq!(m.indices.len(), 44 * 3);
}

#[test]
fn test_polyline_volume_geometry_computes_all_vertex_attributes() {
    let polyline_positions =
        Cartesian3::from_degrees_array(&[90.0, -30.0, 90.0, -35.0], None).unwrap();

    let geom = PolylineVolumeGeometry::new(PolylineVolumeGeometryOptions {
        polyline_positions,
        shape_positions: test_shape(),
        ellipsoid: None,
        corner_type: Some(CornerType::Mitered),
        vertex_format: Some(VertexFormat::ALL),
        granularity: None,
    });

    let m = geom.create_geometry().expect("create_geometry failed");
    assert!(m.attributes.position.is_some());
    assert!(m.attributes.normal.is_some());
    assert!(m.attributes.st.is_some());
    assert!(m.attributes.tangent.is_some());
    assert!(m.attributes.bitangent.is_some());
    assert_eq!(m.attributes.position.unwrap().values.len(), 56 * 3);
    assert_eq!(m.attributes.normal.unwrap().values.len(), 56 * 3);
    assert_eq!(m.attributes.st.unwrap().values.len(), 56 * 2);
    assert_eq!(m.attributes.tangent.unwrap().values.len(), 56 * 3);
    assert_eq!(m.attributes.bitangent.unwrap().values.len(), 56 * 3);
    assert_eq!(m.indices.len(), 44 * 3);
}

#[test]
fn test_polyline_volume_geometry_pack_unpack() {
    let geom = PolylineVolumeGeometry::new(PolylineVolumeGeometryOptions {
        polyline_positions: vec![
            Cartesian3::new(1.0, 2.0, 3.0),
            Cartesian3::new(4.0, 5.0, 6.0),
        ],
        shape_positions: test_shape(),
        ellipsoid: Some(Ellipsoid::WGS84),
        corner_type: Some(CornerType::Beveled),
        vertex_format: Some(VertexFormat::POSITION_AND_NORMAL),
        granularity: Some(0.05),
    });

    let mut packed = Vec::new();
    geom.pack(&mut packed, None);
    assert_eq!(packed.len(), geom.packed_length());

    let unpacked = PolylineVolumeGeometry::unpack(&packed, None).expect("unpack failed");
    assert_eq!(geom, unpacked);
}
