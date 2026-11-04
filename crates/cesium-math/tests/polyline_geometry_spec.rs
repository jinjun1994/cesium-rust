use cesium_math::{
    ArcType, Cartesian3, Color, Ellipsoid, PolylineGeometry, PolylineGeometryOptions,
    PrimitiveType, VertexFormat,
};
use std::f64::consts::PI;

#[test]
fn test_polyline_geometry_negative_width_returns_none() {
    let positions = vec![
        Cartesian3::new(1.0, 0.0, 0.0),
        Cartesian3::new(0.0, 1.0, 0.0),
        Cartesian3::new(0.0, 0.0, 1.0),
    ];
    let line = PolylineGeometry::new(PolylineGeometryOptions {
        positions,
        width: Some(-1.0),
        vertex_format: Some(VertexFormat::ALL),
        granularity: Some(PI),
        ellipsoid: Some(Ellipsoid::UNIT_SPHERE),
        ..Default::default()
    });

    assert!(line.create_geometry().is_none());
}

#[test]
fn test_polyline_geometry_computes_all_vertex_attributes() {
    let positions = vec![
        Cartesian3::new(1.0, 0.0, 0.0),
        Cartesian3::new(0.0, 1.0, 0.0),
        Cartesian3::new(0.0, 0.0, 1.0),
    ];
    let line = PolylineGeometry::new(PolylineGeometryOptions {
        positions: positions.clone(),
        width: Some(10.0),
        vertex_format: Some(VertexFormat::ALL),
        granularity: Some(PI),
        ellipsoid: Some(Ellipsoid::UNIT_SPHERE),
        ..Default::default()
    })
    .create_geometry()
    .expect("geometry expected");

    assert!(line.attributes.position.is_some());
    assert!(line.attributes.prev_position.is_some());
    assert!(line.attributes.next_position.is_some());
    assert!(line.attributes.expand_and_width.is_some());
    assert!(line.attributes.st.is_some());

    let num_vertices = positions.len() * 4 - 4;
    assert_eq!(
        line.attributes.position.unwrap().values.len(),
        num_vertices * 3
    );
    assert_eq!(
        line.attributes.prev_position.unwrap().values.len(),
        num_vertices * 3
    );
    assert_eq!(
        line.attributes.next_position.unwrap().values.len(),
        num_vertices * 3
    );
    assert_eq!(
        line.attributes.expand_and_width.unwrap().values.len(),
        num_vertices * 2
    );
    assert_eq!(line.attributes.st.unwrap().values.len(), num_vertices * 2);
    assert_eq!(line.indices.len(), positions.len() * 6 - 6);
    assert_eq!(line.primitive_type, PrimitiveType::Triangles);
}

#[test]
fn test_polyline_geometry_computes_per_segment_colors() {
    let positions = vec![
        Cartesian3::new(1.0, 0.0, 0.0),
        Cartesian3::new(0.0, 1.0, 0.0),
        Cartesian3::new(0.0, 0.0, 1.0),
    ];
    let colors = vec![
        Color::new(1.0, 0.0, 0.0, 1.0),
        Color::new(0.0, 1.0, 0.0, 1.0),
        Color::new(0.0, 0.0, 1.0, 1.0),
    ];
    let line = PolylineGeometry::new(PolylineGeometryOptions {
        positions: positions.clone(),
        colors: Some(colors),
        width: Some(10.0),
        vertex_format: Some(VertexFormat::ALL),
        granularity: Some(PI),
        ellipsoid: Some(Ellipsoid::UNIT_SPHERE),
        ..Default::default()
    })
    .create_geometry()
    .expect("geometry expected");

    assert!(line.attributes.color.is_some());
    let num_vertices = positions.len() * 4 - 4;
    assert_eq!(line.attributes.color.unwrap().values.len(), num_vertices * 4);
}

#[test]
fn test_polyline_geometry_computes_per_vertex_colors() {
    let positions = vec![
        Cartesian3::new(1.0, 0.0, 0.0),
        Cartesian3::new(0.0, 1.0, 0.0),
        Cartesian3::new(0.0, 0.0, 1.0),
    ];
    let colors = vec![
        Color::new(1.0, 0.0, 0.0, 1.0),
        Color::new(0.0, 1.0, 0.0, 1.0),
        Color::new(0.0, 0.0, 1.0, 1.0),
    ];
    let line = PolylineGeometry::new(PolylineGeometryOptions {
        positions: positions.clone(),
        colors: Some(colors),
        colors_per_vertex: Some(true),
        width: Some(10.0),
        vertex_format: Some(VertexFormat::ALL),
        granularity: Some(PI),
        ellipsoid: Some(Ellipsoid::UNIT_SPHERE),
        ..Default::default()
    })
    .create_geometry()
    .expect("geometry expected");

    assert!(line.attributes.color.is_some());
    let num_vertices = positions.len() * 4 - 4;
    assert_eq!(line.attributes.color.unwrap().values.len(), num_vertices * 4);
}

#[test]
fn test_polyline_geometry_duplicate_positions_returns_none() {
    let pos = Cartesian3::new(100000.0, -200000.0, 300000.0);
    let positions = vec![pos, pos];
    let line = PolylineGeometry::new(PolylineGeometryOptions {
        positions,
        width: Some(10.0),
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        arc_type: Some(ArcType::None),
        ..Default::default()
    });

    assert!(line.create_geometry().is_none());
}
