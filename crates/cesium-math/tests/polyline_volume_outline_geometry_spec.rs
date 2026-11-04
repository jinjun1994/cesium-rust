use cesium_math::{
    Cartesian2, Cartesian3, CornerType, Ellipsoid, PolylineVolumeOutlineGeometry,
    PolylineVolumeOutlineGeometryOptions, PrimitiveType,
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
fn test_polyline_volume_outline_geometry_returns_none_without_two_unique_positions() {
    let geom = PolylineVolumeOutlineGeometry::new(PolylineVolumeOutlineGeometryOptions {
        polyline_positions: vec![Cartesian3::ZERO],
        shape_positions: test_shape(),
        ellipsoid: None,
        corner_type: None,
        granularity: None,
    });
    assert!(geom.create_geometry().is_none());
}

#[test]
fn test_polyline_volume_outline_geometry_returns_none_without_three_unique_shape_positions() {
    let geom = PolylineVolumeOutlineGeometry::new(PolylineVolumeOutlineGeometryOptions {
        polyline_positions: vec![Cartesian3::UNIT_X, Cartesian3::UNIT_Y],
        shape_positions: vec![
            Cartesian2::UNIT_X,
            Cartesian2::UNIT_X,
            Cartesian2::UNIT_X,
        ],
        ellipsoid: None,
        corner_type: None,
        granularity: None,
    });
    assert!(geom.create_geometry().is_none());
}

#[test]
fn test_polyline_volume_outline_geometry_computes_positions() {
    let polyline_positions =
        Cartesian3::from_degrees_array(&[90.0, -30.0, 90.0, -35.0], None).unwrap();

    let geom = PolylineVolumeOutlineGeometry::new(PolylineVolumeOutlineGeometryOptions {
        polyline_positions,
        shape_positions: test_shape(),
        ellipsoid: None,
        corner_type: Some(CornerType::Mitered),
        granularity: None,
    });

    let m = geom.create_geometry().expect("create_geometry failed");
    assert_eq!(m.primitive_type, PrimitiveType::Lines);
    let pos = m.attributes.position.expect("missing position");
    // 6 segments * 4 vertices per shape = 24 vertices
    assert_eq!(pos.values.len(), 24 * 3);
    // 4 lines * 5 positions + 4 lines * 2 end caps = 28 lines = 28 * 2 indices
    assert_eq!(m.indices.len(), 28 * 2);
}

#[test]
fn test_polyline_volume_outline_geometry_computes_positions_clockwise_shape() {
    let polyline_positions =
        Cartesian3::from_degrees_array(&[90.0, -30.0, 90.0, -35.0], None).unwrap();

    let mut shape = test_shape();
    shape.reverse();

    let geom = PolylineVolumeOutlineGeometry::new(PolylineVolumeOutlineGeometryOptions {
        polyline_positions,
        shape_positions: shape,
        ellipsoid: None,
        corner_type: Some(CornerType::Mitered),
        granularity: None,
    });

    let m = geom.create_geometry().expect("create_geometry failed");
    let pos = m.attributes.position.expect("missing position");
    assert_eq!(pos.values.len(), 24 * 3);
    assert_eq!(m.indices.len(), 28 * 2);
}

#[test]
fn test_polyline_volume_outline_geometry_pack_unpack() {
    let geom = PolylineVolumeOutlineGeometry::new(PolylineVolumeOutlineGeometryOptions {
        polyline_positions: vec![
            Cartesian3::new(1.0, 2.0, 3.0),
            Cartesian3::new(4.0, 5.0, 6.0),
        ],
        shape_positions: test_shape(),
        ellipsoid: Some(Ellipsoid::WGS84),
        corner_type: Some(CornerType::Beveled),
        granularity: Some(0.05),
    });

    let mut packed = Vec::new();
    geom.pack(&mut packed, None);
    assert_eq!(packed.len(), geom.packed_length());

    let unpacked = PolylineVolumeOutlineGeometry::unpack(&packed, None).expect("unpack failed");
    assert_eq!(geom, unpacked);
}
