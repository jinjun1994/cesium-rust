use cesium_math::{
    ArcType, Cartesian3, Ellipsoid, PolygonGeometry, PolygonGeometryFromPositionsOptions,
    PolygonGeometryOptions, PolygonHierarchy, VertexFormat, EPSILON7,
};

#[test]
fn test_polygon_geometry_throws_with_height_when_per_position_height() {
    let res = PolygonGeometry::new(PolygonGeometryOptions {
        polygon_hierarchy: PolygonHierarchy::new(vec![Cartesian3::ZERO]),
        height: Some(30.0),
        extruded_height: None,
        vertex_format: None,
        ellipsoid: None,
        granularity: None,
        per_position_height: Some(true),
        close_top: None,
        close_bottom: None,
        arc_type: None,
        st_rotation: None,
        offset_attribute: None,
    });
    assert!(res.is_err());
}

#[test]
fn test_polygon_geometry_throws_if_arc_type_invalid() {
    let p = Cartesian3::from_degrees_array(&[0.0, 0.0, 1.0, 0.0, 1.0, 1.0], None).unwrap();
    let res = PolygonGeometry::from_positions(PolygonGeometryFromPositionsOptions {
        positions: p,
        height: None,
        extruded_height: None,
        vertex_format: None,
        ellipsoid: None,
        granularity: None,
        per_position_height: None,
        close_top: None,
        close_bottom: None,
        arc_type: Some(ArcType::None),
        st_rotation: None,
        offset_attribute: None,
    });
    assert!(res.is_err());
}

#[test]
fn test_polygon_geometry_rectangle_property() {
    let p = PolygonGeometry::new(PolygonGeometryOptions {
        vertex_format: Some(VertexFormat::POSITION_AND_ST),
        polygon_hierarchy: PolygonHierarchy {
            positions: Cartesian3::from_degrees_array_heights(
                &[
                    -100.5, 30.0, 92.0,
                    -100.0, 30.0, 92.0,
                    -100.0, 30.5, 92.0,
                    -100.5, 30.5, 92.0,
                ],
                None,
            )
            .unwrap(),
            holes: Vec::new(),
        },
        height: None,
        extruded_height: None,
        ellipsoid: None,
        granularity: Some(std::f64::consts::PI),
        per_position_height: None,
        close_top: None,
        close_bottom: None,
        arc_type: None,
        st_rotation: None,
        offset_attribute: None,
    })
    .unwrap();

    let r = p.rectangle();
    let north_deg = r.north.to_degrees();
    assert!(north_deg > 30.5);
    assert!(north_deg < 30.5999);
    assert!((r.south.to_degrees() - 30.0).abs() < 1e-5);
    assert!((r.east.to_degrees() - (-100.0)).abs() < 1e-13);
    assert!((r.west.to_degrees() - (-100.5)).abs() < 1e-13);
}

#[test]
fn test_polygon_geometry_pack_unpack() {
    let hierarchy = PolygonHierarchy {
        positions: Cartesian3::from_degrees_array(
            &[-108.0, 1.0, -108.0, -1.0, -106.0, -1.0],
            None,
        )
        .unwrap(),
        holes: vec![PolygonHierarchy {
            positions: Cartesian3::from_degrees_array(
                &[-107.5, 0.5, -107.5, -0.5, -106.5, -0.5],
                None,
            )
            .unwrap(),
            holes: Vec::new(),
        }],
    };

    let geom = PolygonGeometry::new(PolygonGeometryOptions {
        polygon_hierarchy: hierarchy,
        height: Some(10.0),
        extruded_height: Some(20.0),
        vertex_format: Some(VertexFormat::ALL),
        ellipsoid: None,
        granularity: None,
        per_position_height: Some(false),
        close_top: Some(true),
        close_bottom: Some(false),
        arc_type: Some(ArcType::Geodesic),
        st_rotation: Some(1.23),
        offset_attribute: None,
    })
    .unwrap();

    let mut packed = Vec::new();
    let len = geom.pack(&mut packed, None);
    assert_eq!(len, geom.packed_length());
    assert_eq!(packed.len(), geom.packed_length());

    let unpacked = PolygonGeometry::unpack(&packed, None, None).unwrap();
    assert_eq!(unpacked.height(), geom.height());
    assert_eq!(unpacked.extruded_height(), geom.extruded_height());
    assert_eq!(unpacked.granularity(), geom.granularity());
    assert_eq!(unpacked.per_position_height(), geom.per_position_height());
    assert_eq!(unpacked.close_top(), geom.close_top());
    assert_eq!(unpacked.close_bottom(), geom.close_bottom());
    assert_eq!(unpacked.arc_type(), geom.arc_type());
    assert_eq!(unpacked.st_rotation(), geom.st_rotation());
    assert_eq!(unpacked.packed_length(), geom.packed_length());
    assert_eq!(
        unpacked.polygon_hierarchy().positions.len(),
        geom.polygon_hierarchy().positions.len()
    );
}

#[test]
fn test_compute_rectangle_geodesic_northern_hemisphere() {
    let positions =
        Cartesian3::from_degrees_array(&[30.0, 30.0, 60.0, 60.0, 30.0, 60.0], None).unwrap();
    let result = PolygonGeometry::compute_rectangle_from_positions(
        &positions,
        Some(&Ellipsoid::WGS84),
        Some(ArcType::Geodesic),
    )
    .unwrap();

    assert!((result.west - 30.0_f64.to_radians()).abs() < EPSILON7);
    assert!((result.south - 30.0_f64.to_radians()).abs() < EPSILON7);
    assert!((result.east - 60.0_f64.to_radians()).abs() < EPSILON7);
    assert!(result.north > 60.0_f64.to_radians());
    assert!(result.north < 70.0_f64.to_radians());
}

#[test]
fn test_compute_rectangle_rhumb_northern_hemisphere() {
    let positions =
        Cartesian3::from_degrees_array(&[30.0, 30.0, 60.0, 60.0, 30.0, 60.0], None).unwrap();
    let result = PolygonGeometry::compute_rectangle_from_positions(
        &positions,
        Some(&Ellipsoid::WGS84),
        Some(ArcType::Rhumb),
    )
    .unwrap();

    assert!((result.west - 30.0_f64.to_radians()).abs() < EPSILON7);
    assert!((result.south - 30.0_f64.to_radians()).abs() < EPSILON7);
    assert!((result.east - 60.0_f64.to_radians()).abs() < EPSILON7);
    assert!((result.north - 60.0_f64.to_radians()).abs() < EPSILON7);
}

#[test]
fn test_compute_rectangle_geodesic_southern_hemisphere() {
    let positions =
        Cartesian3::from_degrees_array(&[-30.0, -30.0, -60.0, -60.0, -30.0, -60.0], None).unwrap();
    let result = PolygonGeometry::compute_rectangle_from_positions(
        &positions,
        Some(&Ellipsoid::WGS84),
        Some(ArcType::Geodesic),
    )
    .unwrap();

    assert!((result.west - (-60.0_f64).to_radians()).abs() < EPSILON7);
    assert!(result.south < (-60.0_f64).to_radians());
    assert!(result.south > (-70.0_f64).to_radians());
    assert!((result.east - (-30.0_f64).to_radians()).abs() < EPSILON7);
    assert!((result.north - (-30.0_f64).to_radians()).abs() < EPSILON7);
}

#[test]
fn test_compute_rectangle_rhumb_southern_hemisphere() {
    let positions =
        Cartesian3::from_degrees_array(&[-30.0, -30.0, -60.0, -60.0, -30.0, -60.0], None).unwrap();
    let result = PolygonGeometry::compute_rectangle_from_positions(
        &positions,
        Some(&Ellipsoid::WGS84),
        Some(ArcType::Rhumb),
    )
    .unwrap();

    assert!((result.west - (-60.0_f64).to_radians()).abs() < EPSILON7);
    assert!((result.south - (-60.0_f64).to_radians()).abs() < EPSILON7);
    assert!((result.east - (-30.0_f64).to_radians()).abs() < EPSILON7);
    assert!((result.north - (-30.0_f64).to_radians()).abs() < EPSILON7);
}

#[test]
fn test_compute_rectangle_across_idl() {
    let positions =
        Cartesian3::from_degrees_array(&[170.0, 60.0, 170.0, 30.0, -170.0, 30.0], None).unwrap();
    let result = PolygonGeometry::compute_rectangle_from_positions(
        &positions,
        Some(&Ellipsoid::WGS84),
        Some(ArcType::Geodesic),
    )
    .unwrap();

    assert!((result.west - 170.0_f64.to_radians()).abs() < EPSILON7);
    assert!((result.south - 30.0_f64.to_radians()).abs() < EPSILON7);
    assert!((result.east - (-170.0_f64).to_radians()).abs() < EPSILON7);
    assert!((result.north - 60.0_f64.to_radians()).abs() < EPSILON7);
}

#[test]
fn test_compute_rectangle_across_equator() {
    let positions = Cartesian3::from_degrees_array(
        &[
            30.0, 30.0, -30.0, 30.0, -30.0, -30.0, 30.0, -30.0,
        ],
        None,
    )
    .unwrap();
    let result = PolygonGeometry::compute_rectangle_from_positions(
        &positions,
        Some(&Ellipsoid::WGS84),
        Some(ArcType::Rhumb),
    )
    .unwrap();

    assert!((result.west - (-30.0_f64).to_radians()).abs() < EPSILON7);
    assert!((result.south - (-30.0_f64).to_radians()).abs() < EPSILON7);
    assert!((result.east - 30.0_f64.to_radians()).abs() < EPSILON7);
    assert!((result.north - 30.0_f64.to_radians()).abs() < EPSILON7);
}

#[test]
fn test_compute_rectangle_convex_around_north_pole() {
    let positions = Cartesian3::from_degrees_array(
        &[45.0, 60.0, -45.0, 60.0, -135.0, 60.0, 140.0, 60.0],
        None,
    )
    .unwrap();
    let result = PolygonGeometry::compute_rectangle_from_positions(
        &positions,
        Some(&Ellipsoid::WGS84),
        Some(ArcType::Geodesic),
    )
    .unwrap();

    assert!((result.west - (-180.0_f64).to_radians()).abs() < EPSILON7);
    assert!((result.south - 60.0_f64.to_radians()).abs() < EPSILON7);
    assert!((result.east - 180.0_f64.to_radians()).abs() < EPSILON7);
    assert!((result.north - 90.0_f64.to_radians()).abs() < EPSILON7);
}

#[test]
fn test_compute_rectangle_concave_around_north_pole() {
    let positions = Cartesian3::from_degrees_array(
        &[45.0, 60.0, -45.0, 60.0, -135.0, 60.0, -45.0, 80.0],
        None,
    )
    .unwrap();
    let result = PolygonGeometry::compute_rectangle_from_positions(
        &positions,
        Some(&Ellipsoid::WGS84),
        Some(ArcType::Geodesic),
    )
    .unwrap();

    assert!((result.west - (-135.0_f64).to_radians()).abs() < EPSILON7);
    assert!((result.south - 60.0_f64.to_radians()).abs() < EPSILON7);
    assert!((result.east - 45.0_f64.to_radians()).abs() < EPSILON7);
    assert!((result.north - 1.40485733).abs() < EPSILON7);
}

#[test]
fn test_compute_rectangle_clockwise_convex_around_north_pole() {
    let positions = Cartesian3::from_degrees_array(
        &[140.0, 60.0, -135.0, 60.0, -45.0, 60.0, 45.0, 60.0],
        None,
    )
    .unwrap();
    let result = PolygonGeometry::compute_rectangle_from_positions(
        &positions,
        Some(&Ellipsoid::WGS84),
        Some(ArcType::Geodesic),
    )
    .unwrap();

    assert!((result.west - (-180.0_f64).to_radians()).abs() < EPSILON7);
    assert!((result.south - 60.0_f64.to_radians()).abs() < EPSILON7);
    assert!((result.east - 180.0_f64.to_radians()).abs() < EPSILON7);
    assert!((result.north - 90.0_f64.to_radians()).abs() < EPSILON7);
}

#[test]
fn test_compute_rectangle_convex_around_south_pole() {
    let positions = Cartesian3::from_degrees_array(
        &[140.0, -60.0, -135.0, -60.0, -45.0, -60.0, 45.0, -60.0],
        None,
    )
    .unwrap();
    let result = PolygonGeometry::compute_rectangle_from_positions(
        &positions,
        Some(&Ellipsoid::WGS84),
        Some(ArcType::Geodesic),
    )
    .unwrap();

    assert!((result.west - (-180.0_f64).to_radians()).abs() < EPSILON7);
    assert!((result.south - (-90.0_f64).to_radians()).abs() < EPSILON7);
    assert!((result.east - 180.0_f64.to_radians()).abs() < EPSILON7);
    assert!((result.north - (-60.0_f64).to_radians()).abs() < EPSILON7);
}

#[test]
fn test_compute_rectangle_concave_rhumb_around_south_pole() {
    let positions = Cartesian3::from_degrees_array(
        &[45.0, -60.0, -45.0, -60.0, -135.0, -60.0, -45.0, -80.0],
        None,
    )
    .unwrap();
    let result = PolygonGeometry::compute_rectangle_from_positions(
        &positions,
        Some(&Ellipsoid::WGS84),
        Some(ArcType::Rhumb),
    )
    .unwrap();

    assert!((result.west - (-135.0_f64).to_radians()).abs() < EPSILON7);
    assert!((result.south - (-80.0_f64).to_radians()).abs() < EPSILON7);
    assert!((result.east - 45.0_f64.to_radians()).abs() < EPSILON7);
    assert!((result.north - (-60.0_f64).to_radians()).abs() < EPSILON7);
}

#[test]
fn test_compute_rectangle_concave_geodesic_around_south_pole() {
    let positions = Cartesian3::from_degrees_array(
        &[45.0, -60.0, -45.0, -60.0, -135.0, -60.0, -45.0, -80.0],
        None,
    )
    .unwrap();
    let result = PolygonGeometry::compute_rectangle_from_positions(
        &positions,
        Some(&Ellipsoid::WGS84),
        Some(ArcType::Geodesic),
    )
    .unwrap();

    assert!((result.west - (-135.0_f64).to_radians()).abs() < EPSILON7);
    assert!(result.south < (-80.0_f64).to_radians());
    assert!(result.south > (-90.0_f64).to_radians());
    assert!((result.east - 45.0_f64.to_radians()).abs() < EPSILON7);
    assert!((result.north - (-60.0_f64).to_radians()).abs() < EPSILON7);
}

#[test]
fn test_compute_rectangle_clockwise_convex_around_south_pole() {
    let positions = Cartesian3::from_degrees_array(
        &[45.0, -60.0, -45.0, -60.0, -135.0, -60.0, 140.0, -60.0],
        None,
    )
    .unwrap();
    let result = PolygonGeometry::compute_rectangle_from_positions(
        &positions,
        Some(&Ellipsoid::WGS84),
        Some(ArcType::Geodesic),
    )
    .unwrap();

    assert!((result.west - (-180.0_f64).to_radians()).abs() < EPSILON7);
    assert!((result.south - (-90.0_f64).to_radians()).abs() < EPSILON7);
    assert!((result.east - 180.0_f64.to_radians()).abs() < EPSILON7);
    assert!((result.north - (-60.0_f64).to_radians()).abs() < EPSILON7);
}

#[test]
fn test_polygon_geometry_computes_positions() {
    let p = PolygonGeometry::from_positions(PolygonGeometryFromPositionsOptions {
        positions: Cartesian3::from_degrees_array(
            &[-1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0],
            None,
        )
        .unwrap(),
        height: None,
        extruded_height: None,
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        ellipsoid: None,
        granularity: Some(cesium_math::RADIANS_PER_DEGREE),
        per_position_height: None,
        close_top: None,
        close_bottom: None,
        arc_type: None,
        st_rotation: None,
        offset_attribute: None,
    })
    .unwrap()
    .create_geometry()
    .unwrap();

    assert_eq!(p.attributes.position.as_ref().unwrap().values.len(), 13 * 3);
    assert_eq!(p.indices.len(), 16 * 3);
}

#[test]
fn test_polygon_geometry_creates_with_rhumb_lines() {
    let p = PolygonGeometry::from_positions(PolygonGeometryFromPositionsOptions {
        positions: Cartesian3::from_degrees_array(
            &[-1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0],
            None,
        )
        .unwrap(),
        height: None,
        extruded_height: None,
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        ellipsoid: None,
        granularity: Some(cesium_math::RADIANS_PER_DEGREE),
        per_position_height: None,
        close_top: None,
        close_bottom: None,
        arc_type: Some(ArcType::Rhumb),
        st_rotation: None,
        offset_attribute: None,
    })
    .unwrap()
    .create_geometry()
    .unwrap();

    assert_eq!(p.attributes.position.as_ref().unwrap().values.len(), 15 * 3);
    assert_eq!(p.indices.len(), 20 * 3);
}

#[test]
fn test_polygon_geometry_computes_positions_with_per_position_heights() {
    let ellipsoid = Ellipsoid::WGS84;
    let height = 100.0;
    let positions = Cartesian3::from_degrees_array_heights(
        &[
            -1.0, -1.0, height,
            1.0, -1.0, 0.0,
            1.0, 1.0, 0.0,
            -1.0, 1.0, 0.0,
        ],
        None,
    )
    .unwrap();

    let p = PolygonGeometry::from_positions(PolygonGeometryFromPositionsOptions {
        positions,
        height: None,
        extruded_height: None,
        vertex_format: None,
        ellipsoid: Some(ellipsoid),
        granularity: None,
        per_position_height: Some(true),
        close_top: None,
        close_bottom: None,
        arc_type: None,
        st_rotation: None,
        offset_attribute: None,
    })
    .unwrap()
    .create_geometry()
    .unwrap();

    let pos_values = &p.attributes.position.as_ref().unwrap().values;
    let p0 = Cartesian3::new(pos_values[0], pos_values[1], pos_values[2]);
    let c0 = ellipsoid.cartesian_to_cartographic(p0.into()).unwrap();
    assert!((c0.height - height).abs() < cesium_math::EPSILON6);

    let p1 = Cartesian3::new(pos_values[3], pos_values[4], pos_values[5]);
    let c1 = ellipsoid.cartesian_to_cartographic(p1.into()).unwrap();
    assert!(c1.height.abs() < cesium_math::EPSILON6);
}

#[test]
fn test_polygon_geometry_computes_positions_with_per_position_heights_rhumb() {
    let ellipsoid = Ellipsoid::WGS84;
    let height = 100.0;
    let positions = Cartesian3::from_degrees_array_heights(
        &[
            -1.0, -1.0, height,
            1.0, -1.0, 0.0,
            1.0, 1.0, 0.0,
            -1.0, 1.0, 0.0,
        ],
        None,
    )
    .unwrap();

    let p = PolygonGeometry::from_positions(PolygonGeometryFromPositionsOptions {
        positions,
        height: None,
        extruded_height: None,
        vertex_format: None,
        ellipsoid: Some(ellipsoid),
        granularity: None,
        per_position_height: Some(true),
        close_top: None,
        close_bottom: None,
        arc_type: Some(ArcType::Rhumb),
        st_rotation: None,
        offset_attribute: None,
    })
    .unwrap()
    .create_geometry()
    .unwrap();

    let pos_values = &p.attributes.position.as_ref().unwrap().values;
    let p0 = Cartesian3::new(pos_values[0], pos_values[1], pos_values[2]);
    let c0 = ellipsoid.cartesian_to_cartographic(p0.into()).unwrap();
    assert!((c0.height - height).abs() < cesium_math::EPSILON6);

    let p1 = Cartesian3::new(pos_values[3], pos_values[4], pos_values[5]);
    let c1 = ellipsoid.cartesian_to_cartographic(p1.into()).unwrap();
    assert!(c1.height.abs() < cesium_math::EPSILON6);
}

#[test]
fn test_polygon_geometry_creates_from_hierarchy() {
    let hierarchy = PolygonHierarchy {
        positions: Cartesian3::from_degrees_array(
            &[-124.0, 35.0, -110.0, 35.0, -110.0, 40.0, -124.0, 40.0],
            None,
        )
        .unwrap(),
        holes: vec![PolygonHierarchy {
            positions: Cartesian3::from_degrees_array(
                &[-122.0, 36.0, -122.0, 39.0, -112.0, 39.0, -112.0, 36.0],
                None,
            )
            .unwrap(),
            holes: vec![PolygonHierarchy {
                positions: Cartesian3::from_degrees_array(
                    &[-120.0, 36.5, -114.0, 36.5, -114.0, 38.5, -120.0, 38.5],
                    None,
                )
                .unwrap(),
                holes: Vec::new(),
            }],
        }],
    };

    let p = PolygonGeometry::new(PolygonGeometryOptions {
        polygon_hierarchy: hierarchy,
        height: None,
        extruded_height: None,
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        ellipsoid: None,
        granularity: Some(cesium_math::PI_OVER_THREE),
        per_position_height: None,
        close_top: None,
        close_bottom: None,
        arc_type: None,
        st_rotation: None,
        offset_attribute: None,
    })
    .unwrap()
    .create_geometry()
    .unwrap();

    assert_eq!(p.attributes.position.as_ref().unwrap().values.len(), 12 * 3);
    assert_eq!(p.indices.len(), 10 * 3);
}

#[test]
fn test_polygon_geometry_returns_none_with_invalid_positions() {
    let p = PolygonGeometry::from_positions(PolygonGeometryFromPositionsOptions {
        positions: vec![Cartesian3::ZERO],
        height: None,
        extruded_height: None,
        vertex_format: None,
        ellipsoid: None,
        granularity: None,
        per_position_height: None,
        close_top: None,
        close_bottom: None,
        arc_type: None,
        st_rotation: None,
        offset_attribute: None,
    })
    .unwrap();
    assert!(p.create_geometry().is_none());

    let p_dup = PolygonGeometry::from_positions(PolygonGeometryFromPositionsOptions {
        positions: Cartesian3::from_degrees_array(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.0], None).unwrap(),
        height: None,
        extruded_height: None,
        vertex_format: None,
        ellipsoid: None,
        granularity: None,
        per_position_height: None,
        close_top: None,
        close_bottom: None,
        arc_type: None,
        st_rotation: None,
        offset_attribute: None,
    })
    .unwrap();
    assert!(p_dup.create_geometry().is_none());
}

#[test]
fn test_polygon_geometry_computes_all_attributes() {
    let p = PolygonGeometry::from_positions(PolygonGeometryFromPositionsOptions {
        positions: Cartesian3::from_degrees_array(
            &[-1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0],
            None,
        )
        .unwrap(),
        height: None,
        extruded_height: None,
        vertex_format: Some(VertexFormat::ALL),
        ellipsoid: None,
        granularity: Some(cesium_math::RADIANS_PER_DEGREE),
        per_position_height: None,
        close_top: None,
        close_bottom: None,
        arc_type: None,
        st_rotation: None,
        offset_attribute: None,
    })
    .unwrap()
    .create_geometry()
    .unwrap();

    let num_verts = 13;
    let num_triangles = 16;
    assert_eq!(p.attributes.position.as_ref().unwrap().values.len(), num_verts * 3);
    assert_eq!(p.attributes.st.as_ref().unwrap().values.len(), num_verts * 2);
    assert_eq!(p.attributes.normal.as_ref().unwrap().values.len(), num_verts * 3);
    assert_eq!(p.indices.len(), num_triangles * 3);
}

#[test]
fn test_polygon_geometry_computes_offset_attribute() {
    let p = PolygonGeometry::from_positions(PolygonGeometryFromPositionsOptions {
        positions: Cartesian3::from_degrees_array(
            &[-1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0],
            None,
        )
        .unwrap(),
        height: None,
        extruded_height: None,
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        ellipsoid: None,
        granularity: Some(cesium_math::RADIANS_PER_DEGREE),
        per_position_height: None,
        close_top: None,
        close_bottom: None,
        arc_type: None,
        st_rotation: None,
        offset_attribute: Some(cesium_math::GeometryOffsetAttribute::Top),
    })
    .unwrap()
    .create_geometry()
    .unwrap();

    let num_verts = 13;
    let offset = &p.attributes.apply_offset.as_ref().unwrap().values;
    assert_eq!(offset.len(), num_verts);
    let expected = vec![1.0; num_verts];
    assert_eq!(offset, &expected);
}
