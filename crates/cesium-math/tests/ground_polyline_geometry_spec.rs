use cesium_math::{
    ArcType, Cartesian3, Cartographic, Ellipsoid, GroundPolylineGeometry,
    GroundPolylineGeometryOptions,
};

#[test]
fn computes_positions_and_attributes_for_polylines() {
    let start = Cartographic::from_degrees(0.01, 0.0, None);
    let end = Cartographic::from_degrees(0.02, 0.0, None);
    let ellipsoid = Ellipsoid::WGS84;

    let p1 = Cartesian3::from(ellipsoid.cartographic_to_cartesian(start).unwrap());
    let p2 = Cartesian3::from(ellipsoid.cartographic_to_cartesian(end).unwrap());

    let geometry_desc = GroundPolylineGeometry::new(GroundPolylineGeometryOptions {
        positions: vec![p1, p2],
        width: Some(1.0),
        granularity: Some(0.0),
        loop_: Some(false),
        arc_type: Some(ArcType::Geodesic),
        ellipsoid: Some(ellipsoid),
    })
    .unwrap();

    let geom = GroundPolylineGeometry::create_geometry(&geometry_desc).unwrap();

    // 1 segment = 8 vertices = 24 coordinates
    let pos_attr = geom.attributes.get("position").unwrap();
    assert_eq!(pos_attr.values.len(), 8 * 3);
    assert_eq!(pos_attr.components_per_attribute, 3);

    // 1 segment = 36 indices
    assert_eq!(geom.indices.len(), 36);

    // Additional attributes exist
    assert!(geom.attributes.get("startHiAndForwardOffsetX").is_some());
    assert!(geom.attributes.get("startLoAndForwardOffsetY").is_some());
    assert!(geom.attributes.get("startNormalAndForwardOffsetZ").is_some());
    assert!(geom
        .attributes
        .get("endNormalAndTextureCoordinateNormalizationX")
        .is_some());
    assert!(geom
        .attributes
        .get("rightNormalAndTextureCoordinateNormalizationY")
        .is_some());

    // Bounding sphere exists and has positive radius
    let bs = geom.bounding_sphere.unwrap();
    assert!(bs.radius > 0.0);
}

#[test]
fn removes_adjacent_duplicate_positions() {
    let start = Cartographic::from_degrees(0.01, 0.0, None);
    let end = Cartographic::from_degrees(0.02, 0.0, None);
    let ellipsoid = Ellipsoid::WGS84;

    let p1 = Cartesian3::from(ellipsoid.cartographic_to_cartesian(start).unwrap());
    let p2 = Cartesian3::from(ellipsoid.cartographic_to_cartesian(end).unwrap());

    // p1 duplicated adjacent
    let geometry_desc = GroundPolylineGeometry::new(GroundPolylineGeometryOptions {
        positions: vec![p1, p1, p2],
        width: None,
        granularity: Some(0.0),
        loop_: None,
        arc_type: None,
        ellipsoid: None,
    })
    .unwrap();

    let geom = GroundPolylineGeometry::create_geometry(&geometry_desc).unwrap();
    let pos_attr = geom.attributes.get("position").unwrap();
    // Still 1 segment (8 vertices)
    assert_eq!(pos_attr.values.len(), 8 * 3);
}

#[test]
fn returns_none_if_filtered_points_not_valid_geometry() {
    let start = Cartographic::from_degrees(0.01, 0.0, None);
    let ellipsoid = Ellipsoid::WGS84;
    let p1 = Cartesian3::from(ellipsoid.cartographic_to_cartesian(start).unwrap());

    let geometry_desc = GroundPolylineGeometry::new(GroundPolylineGeometryOptions {
        positions: vec![p1, p1],
        width: None,
        granularity: None,
        loop_: None,
        arc_type: None,
        ellipsoid: None,
    })
    .unwrap();

    let geom = GroundPolylineGeometry::create_geometry(&geometry_desc);
    assert!(geom.is_none());
}

#[test]
fn throws_if_less_than_two_positions() {
    let p1 = Cartesian3::new(1.0, 0.0, 0.0);
    let res = GroundPolylineGeometry::new(GroundPolylineGeometryOptions {
        positions: vec![p1],
        width: None,
        granularity: None,
        loop_: None,
        arc_type: None,
        ellipsoid: None,
    });
    assert!(res.is_err());
}

#[test]
fn interpolates_long_polyline_segments_geodesic_and_rhumb() {
    let start = Cartographic::from_degrees(0.0, 0.0, None);
    let end = Cartographic::from_degrees(1.0, 1.0, None);
    let ellipsoid = Ellipsoid::WGS84;

    let p1 = Cartesian3::from(ellipsoid.cartographic_to_cartesian(start).unwrap());
    let p2 = Cartesian3::from(ellipsoid.cartographic_to_cartesian(end).unwrap());

    // Geodesic interpolation
    let geo_desc = GroundPolylineGeometry::new(GroundPolylineGeometryOptions {
        positions: vec![p1, p2],
        width: None,
        granularity: Some(10000.0), // 10km step, 1 degree is ~157km, so >10 segments
        loop_: None,
        arc_type: Some(ArcType::Geodesic),
        ellipsoid: None,
    })
    .unwrap();

    let geo_geom = GroundPolylineGeometry::create_geometry(&geo_desc).unwrap();
    assert!(geo_geom.indices.len() > 36);

    // Rhumb interpolation
    let rhumb_desc = GroundPolylineGeometry::new(GroundPolylineGeometryOptions {
        positions: vec![p1, p2],
        width: None,
        granularity: Some(10000.0),
        loop_: None,
        arc_type: Some(ArcType::Rhumb),
        ellipsoid: None,
    })
    .unwrap();

    let rhumb_geom = GroundPolylineGeometry::create_geometry(&rhumb_desc).unwrap();
    assert!(rhumb_geom.indices.len() > 36);
}

#[test]
fn loops_when_specified_and_enough_positions() {
    let c1 = Cartographic::from_degrees(0.0, 0.0, None);
    let c2 = Cartographic::from_degrees(1.0, 0.0, None);
    let c3 = Cartographic::from_degrees(1.0, 1.0, None);
    let ellipsoid = Ellipsoid::WGS84;

    let p1 = Cartesian3::from(ellipsoid.cartographic_to_cartesian(c1).unwrap());
    let p2 = Cartesian3::from(ellipsoid.cartographic_to_cartesian(c2).unwrap());
    let p3 = Cartesian3::from(ellipsoid.cartographic_to_cartesian(c3).unwrap());

    let loop_desc = GroundPolylineGeometry::new(GroundPolylineGeometryOptions {
        positions: vec![p1, p2, p3],
        width: None,
        granularity: Some(0.0),
        loop_: Some(true),
        arc_type: None,
        ellipsoid: None,
    })
    .unwrap();

    let geom = GroundPolylineGeometry::create_geometry(&loop_desc).unwrap();
    // 3 segments for loop = 3 * 36 = 108 indices
    assert_eq!(geom.indices.len(), 3 * 36);
}

#[test]
fn pack_and_unpack() {
    let p1 = Cartesian3::new(100.0, 200.0, 300.0);
    let p2 = Cartesian3::new(400.0, 500.0, 600.0);
    let mut geom = GroundPolylineGeometry::new(GroundPolylineGeometryOptions {
        positions: vec![p1, p2],
        width: Some(3.5),
        granularity: Some(500.0),
        loop_: Some(true),
        arc_type: Some(ArcType::Rhumb),
        ellipsoid: Some(Ellipsoid::WGS84),
    })
    .unwrap();
    geom.set_projection_and_ellipsoid(1, Ellipsoid::WGS84);

    let mut packed = Vec::new();
    geom.pack(&mut packed);
    assert_eq!(packed.len(), geom.packed_length());

    let unpacked = GroundPolylineGeometry::unpack(&packed, 0).unwrap();
    assert_eq!(unpacked.positions(), geom.positions());
    assert_eq!(unpacked.width, 3.5);
    assert_eq!(unpacked.granularity, 500.0);
    assert!(unpacked.loop_);
    assert_eq!(unpacked.arc_type, ArcType::Rhumb);
    assert_eq!(unpacked.projection_index, 1);
}
