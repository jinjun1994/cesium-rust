use cesium_math::{
    Cartesian3, CornerType, CorridorGeometry, CorridorGeometryOptions, CorridorOutlineGeometry,
    CorridorOutlineGeometryOptions, GeometryOffsetAttribute, VertexFormat,
};

fn sample_positions() -> Vec<Cartesian3> {
    Cartesian3::from_degrees_array(&[90.0, -30.0, 90.0, -35.0], None).unwrap()
}

#[test]
fn test_corridor_geometry_degenerate() {
    let same_pos = Cartesian3::from_degrees_array(&[90.0, -30.0, 90.0, -30.0], None).unwrap();
    let g1 = CorridorGeometry::new(CorridorGeometryOptions {
        positions: same_pos,
        width: 10000.0,
        ..Default::default()
    });
    assert!(g1.create_geometry().is_none());

    let same_lon_lat_diff_h = vec![
        Cartesian3::new(-1349511.388149118, -5063973.22857992, 3623141.6372688496),
        Cartesian3::new(-1349046.4811926484, -5062228.688739784, 3621885.0521561056),
    ];
    let g2 = CorridorGeometry::new(CorridorGeometryOptions {
        positions: same_lon_lat_diff_h,
        width: 10000.0,
        ..Default::default()
    });
    assert!(g2.create_geometry().is_none());
}

#[test]
fn test_corridor_geometry_computes_positions() {
    let g = CorridorGeometry::new(CorridorGeometryOptions {
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        positions: sample_positions(),
        corner_type: Some(CornerType::Mitered),
        width: 30000.0,
        ..Default::default()
    });

    let geom = g.create_geometry().expect("geometry expected");
    let num_vertices = 12; // 6 left + 6 right
    let num_triangles = 10; // 5 segments x 2 triangles
    assert_eq!(
        geom.attributes.position.unwrap().values.len(),
        num_vertices * 3
    );
    assert_eq!(geom.indices.len(), num_triangles * 3);
}

#[test]
fn test_corridor_geometry_all_attributes() {
    let g = CorridorGeometry::new(CorridorGeometryOptions {
        vertex_format: Some(VertexFormat::ALL),
        positions: sample_positions(),
        corner_type: Some(CornerType::Mitered),
        width: 30000.0,
        ..Default::default()
    });

    let geom = g.create_geometry().expect("geometry expected");
    let num_vertices = 12;
    let num_triangles = 10;

    assert_eq!(
        geom.attributes.position.unwrap().values.len(),
        num_vertices * 3
    );
    assert_eq!(geom.attributes.st.unwrap().values.len(), num_vertices * 2);
    assert_eq!(
        geom.attributes.normal.unwrap().values.len(),
        num_vertices * 3
    );
    assert_eq!(
        geom.attributes.tangent.unwrap().values.len(),
        num_vertices * 3
    );
    assert_eq!(
        geom.attributes.bitangent.unwrap().values.len(),
        num_vertices * 3
    );
    assert_eq!(geom.indices.len(), num_triangles * 3);
}

#[test]
fn test_corridor_geometry_extruded() {
    let g = CorridorGeometry::new(CorridorGeometryOptions {
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        positions: sample_positions(),
        corner_type: Some(CornerType::Mitered),
        width: 30000.0,
        extruded_height: Some(30000.0),
        ..Default::default()
    });

    let geom = g.create_geometry().expect("geometry expected");
    let num_vertices = 72;
    let num_triangles = 44;
    assert_eq!(
        geom.attributes.position.unwrap().values.len(),
        num_vertices * 3
    );
    assert_eq!(geom.indices.len(), num_triangles * 3);
}

#[test]
fn test_corridor_geometry_offset_attribute() {
    let g = CorridorGeometry::new(CorridorGeometryOptions {
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        positions: sample_positions(),
        corner_type: Some(CornerType::Mitered),
        width: 30000.0,
        offset_attribute: Some(GeometryOffsetAttribute::Top),
        ..Default::default()
    });

    let geom = g.create_geometry().expect("geometry expected");
    let num_vertices = 12;
    let offset = geom.attributes.apply_offset.unwrap().values;
    assert_eq!(offset.len(), num_vertices);
    assert_eq!(offset, vec![1.0; num_vertices]);
}

#[test]
fn test_corridor_geometry_offset_attribute_extruded() {
    let g_top = CorridorGeometry::new(CorridorGeometryOptions {
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        positions: sample_positions(),
        corner_type: Some(CornerType::Mitered),
        width: 30000.0,
        extruded_height: Some(30000.0),
        offset_attribute: Some(GeometryOffsetAttribute::Top),
        ..Default::default()
    });

    let geom_top = g_top.create_geometry().expect("geometry expected");
    let offset_top = geom_top.attributes.apply_offset.unwrap().values;
    assert_eq!(offset_top.len(), 72);
    let mut expected_top = vec![0.0; 72];
    for v in expected_top.iter_mut().take(12) {
        *v = 1.0;
    }
    for v in expected_top[24..48].iter_mut() {
        *v = 1.0;
    }
    assert_eq!(offset_top, expected_top);

    let g_all = CorridorGeometry::new(CorridorGeometryOptions {
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        positions: sample_positions(),
        corner_type: Some(CornerType::Mitered),
        width: 30000.0,
        extruded_height: Some(30000.0),
        offset_attribute: Some(GeometryOffsetAttribute::All),
        ..Default::default()
    });

    let geom_all = g_all.create_geometry().expect("geometry expected");
    let offset_all = geom_all.attributes.apply_offset.unwrap().values;
    assert_eq!(offset_all, vec![1.0; 72]);
}

#[test]
fn test_corridor_outline_geometry_computes_positions() {
    let g = CorridorOutlineGeometry::new(CorridorOutlineGeometryOptions {
        positions: sample_positions(),
        corner_type: Some(CornerType::Mitered),
        width: 30000.0,
        ..Default::default()
    });

    let geom = g.create_geometry().expect("geometry expected");
    assert_eq!(geom.attributes.position.unwrap().values.len(), 12 * 3);
    assert_eq!(geom.indices.len(), 12 * 2);
}

#[test]
fn test_corridor_outline_geometry_extruded() {
    let g = CorridorOutlineGeometry::new(CorridorOutlineGeometryOptions {
        positions: sample_positions(),
        corner_type: Some(CornerType::Mitered),
        width: 30000.0,
        extruded_height: Some(30000.0),
        ..Default::default()
    });

    let geom = g.create_geometry().expect("geometry expected");
    assert_eq!(geom.attributes.position.unwrap().values.len(), 24 * 3);
    assert_eq!(geom.indices.len(), 28 * 2);
}
