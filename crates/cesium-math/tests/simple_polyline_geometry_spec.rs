use cesium_math::{
    ArcType, Cartesian3, Color, Ellipsoid, PrimitiveType, SimplePolylineGeometry,
    SimplePolylineGeometryOptions,
};
use std::f64::consts::PI;

#[test]
fn test_simple_polyline_geometry_computes_all_vertex_attributes() {
    let positions = vec![
        Cartesian3::new(1.0, 0.0, 0.0),
        Cartesian3::new(0.0, 1.0, 0.0),
        Cartesian3::new(0.0, 0.0, 1.0),
    ];
    let line = SimplePolylineGeometry::new(SimplePolylineGeometryOptions {
        positions: positions.clone(),
        granularity: Some(PI),
        ellipsoid: Some(Ellipsoid::UNIT_SPHERE),
        ..Default::default()
    })
    .create_geometry()
    .expect("geometry expected");

    let pos = line.attributes.position.unwrap().values;
    assert_eq!(pos, vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
    assert_eq!(line.indices, vec![0, 1, 1, 2]);
    assert_eq!(line.primitive_type, PrimitiveType::Lines);
    assert!(line.bounding_sphere.is_some());
}

#[test]
fn test_simple_polyline_geometry_computes_per_segment_colors() {
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
    let line = SimplePolylineGeometry::new(SimplePolylineGeometryOptions {
        positions: positions.clone(),
        colors: Some(colors),
        granularity: Some(PI),
        ellipsoid: Some(Ellipsoid::UNIT_SPHERE),
        ..Default::default()
    })
    .create_geometry()
    .expect("geometry expected");

    assert!(line.attributes.color.is_some());
    let num_vertices = positions.len() * 2 - 2;
    assert_eq!(line.attributes.color.unwrap().values.len(), num_vertices * 4);
}

#[test]
fn test_simple_polyline_geometry_computes_per_vertex_colors() {
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
    let line = SimplePolylineGeometry::new(SimplePolylineGeometryOptions {
        positions: positions.clone(),
        colors: Some(colors),
        colors_per_vertex: Some(true),
        granularity: Some(PI),
        ellipsoid: Some(Ellipsoid::UNIT_SPHERE),
        ..Default::default()
    })
    .create_geometry()
    .expect("geometry expected");

    assert!(line.attributes.color.is_some());
    let num_vertices = positions.len();
    assert_eq!(line.attributes.color.unwrap().values.len(), num_vertices * 4);
}

#[test]
fn test_simple_polyline_geometry_no_subdivision() {
    let positions = vec![
        Cartesian3::new(0.0, 0.0, 0.0),
        Cartesian3::new(1.0, 0.0, 0.0),
        Cartesian3::new(2.0, 0.0, 0.0),
    ];
    let line = SimplePolylineGeometry::new(SimplePolylineGeometryOptions {
        positions: positions.clone(),
        arc_type: Some(ArcType::None),
        ..Default::default()
    })
    .create_geometry()
    .expect("geometry expected");

    assert_eq!(
        line.attributes.position.unwrap().values,
        vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 2.0, 0.0, 0.0]
    );
    assert_eq!(line.indices, vec![0, 1, 1, 2]);
    assert_eq!(line.primitive_type, PrimitiveType::Lines);
}

#[test]
fn test_simple_polyline_geometry_no_subdivision_per_segment_colors() {
    let positions = vec![
        Cartesian3::new(0.0, 0.0, 0.0),
        Cartesian3::new(1.0, 0.0, 0.0),
        Cartesian3::new(2.0, 0.0, 0.0),
    ];
    let colors = vec![
        Color::new(1.0, 0.0, 0.0, 1.0),
        Color::new(0.0, 1.0, 0.0, 1.0),
        Color::new(0.0, 0.0, 1.0, 1.0),
    ];
    let line = SimplePolylineGeometry::new(SimplePolylineGeometryOptions {
        positions: positions.clone(),
        colors: Some(colors),
        arc_type: Some(ArcType::None),
        ..Default::default()
    })
    .create_geometry()
    .expect("geometry expected");

    assert!(line.attributes.color.is_some());
    let num_vertices = positions.len() * 2 - 2;
    assert_eq!(line.attributes.color.unwrap().values.len(), num_vertices * 4);
}
