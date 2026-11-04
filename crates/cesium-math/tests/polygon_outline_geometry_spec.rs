use cesium_math::{
    ArcType, BoundingSphere, Cartesian3, Ellipsoid, GeometryOffsetAttribute,
    PolygonHierarchy, PolygonOutlineGeometry, PolygonOutlineGeometryFromPositionsOptions,
    PolygonOutlineGeometryOptions, EPSILON6, EPSILON9, PI_OVER_THREE, RADIANS_PER_DEGREE,
};

#[test]
fn test_polygon_outline_geometry_throws_with_height_when_per_position_height() {
    let res = PolygonOutlineGeometry::new(PolygonOutlineGeometryOptions {
        polygon_hierarchy: PolygonHierarchy::new(vec![Cartesian3::ZERO]),
        height: Some(30.0),
        extruded_height: None,
        ellipsoid: None,
        granularity: None,
        per_position_height: Some(true),
        arc_type: None,
        offset_attribute: None,
    });
    assert!(res.is_err());
}

#[test]
fn test_polygon_outline_geometry_throws_if_arc_type_invalid() {
    let p = Cartesian3::from_degrees_array(&[0.0, 0.0, 1.0, 0.0, 1.0, 1.0], None).unwrap();
    let res = PolygonOutlineGeometry::from_positions(PolygonOutlineGeometryFromPositionsOptions {
        positions: p,
        height: None,
        extruded_height: None,
        ellipsoid: None,
        granularity: None,
        per_position_height: None,
        arc_type: Some(ArcType::None),
        offset_attribute: None,
    });
    assert!(res.is_err());
}

#[test]
fn test_polygon_outline_geometry_returns_none_with_less_than_three_positions() {
    let geom = PolygonOutlineGeometry::from_positions(PolygonOutlineGeometryFromPositionsOptions {
        positions: vec![Cartesian3::new(0.0, 0.0, 0.0)],
        height: None,
        extruded_height: None,
        ellipsoid: None,
        granularity: None,
        per_position_height: None,
        arc_type: None,
        offset_attribute: None,
    })
    .unwrap();
    assert!(geom.create_geometry().is_none());

    let geom2 = PolygonOutlineGeometry::from_positions(PolygonOutlineGeometryFromPositionsOptions {
        positions: Cartesian3::from_degrees_array(&[-72.0, 40.0, -68.0, 40.0], None).unwrap(),
        height: None,
        extruded_height: None,
        ellipsoid: None,
        granularity: None,
        per_position_height: None,
        arc_type: None,
        offset_attribute: None,
    })
    .unwrap();
    assert!(geom2.create_geometry().is_none());
}

#[test]
fn test_polygon_outline_geometry_returns_none_due_to_duplicate_positions() {
    let positions =
        Cartesian3::from_degrees_array(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.0], None).unwrap();
    let geom = PolygonOutlineGeometry::from_positions(PolygonOutlineGeometryFromPositionsOptions {
        positions: positions.clone(),
        height: None,
        extruded_height: None,
        ellipsoid: None,
        granularity: None,
        per_position_height: None,
        arc_type: None,
        offset_attribute: None,
    })
    .unwrap();
    assert!(geom.create_geometry().is_none());

    let geom_ext =
        PolygonOutlineGeometry::from_positions(PolygonOutlineGeometryFromPositionsOptions {
            positions,
            height: None,
            extruded_height: Some(2.0),
            ellipsoid: None,
            granularity: None,
            per_position_height: None,
            arc_type: None,
            offset_attribute: None,
        })
        .unwrap();
    assert!(geom_ext.create_geometry().is_none());
}

#[test]
fn test_polygon_outline_geometry_duplicate_hierarchy_positions() {
    let hierarchy = PolygonHierarchy {
        positions: Cartesian3::from_degrees_array(&[1.0, 1.0, 1.0, 1.0, 1.0, 1.0], None).unwrap(),
        holes: vec![PolygonHierarchy {
            positions: Cartesian3::from_degrees_array(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.0], None)
                .unwrap(),
            holes: Vec::new(),
        }],
    };
    let geom = PolygonOutlineGeometry::new(PolygonOutlineGeometryOptions {
        polygon_hierarchy: hierarchy,
        height: None,
        extruded_height: None,
        ellipsoid: None,
        granularity: None,
        per_position_height: None,
        arc_type: None,
        offset_attribute: None,
    })
    .unwrap();
    assert!(geom.create_geometry().is_none());
}

#[test]
fn test_polygon_outline_geometry_duplicate_hierarchy_positions_with_different_heights() {
    let hierarchy = PolygonHierarchy {
        positions: Cartesian3::from_degrees_array_heights(
            &[1.0, 1.0, 10.0, 1.0, 1.0, 20.0, 1.0, 1.0, 30.0],
            None,
        )
        .unwrap(),
        holes: vec![PolygonHierarchy {
            positions: Cartesian3::from_degrees_array_heights(
                &[0.0, 0.0, 10.0, 0.0, 0.0, 20.0, 0.0, 0.0, 30.0],
                None,
            )
            .unwrap(),
            holes: Vec::new(),
        }],
    };

    let geom_no_pph = PolygonOutlineGeometry::new(PolygonOutlineGeometryOptions {
        polygon_hierarchy: hierarchy.clone(),
        height: None,
        extruded_height: None,
        ellipsoid: None,
        granularity: None,
        per_position_height: Some(false),
        arc_type: None,
        offset_attribute: None,
    })
    .unwrap();
    assert!(geom_no_pph.create_geometry().is_none());

    let geom_pph = PolygonOutlineGeometry::new(PolygonOutlineGeometryOptions {
        polygon_hierarchy: hierarchy,
        height: None,
        extruded_height: None,
        ellipsoid: None,
        granularity: None,
        per_position_height: Some(true),
        arc_type: None,
        offset_attribute: None,
    })
    .unwrap();
    assert!(geom_pph.create_geometry().is_some());
}

#[test]
fn test_polygon_outline_geometry_computes_positions() {
    let geom = PolygonOutlineGeometry::from_positions(PolygonOutlineGeometryFromPositionsOptions {
        positions: Cartesian3::from_degrees_array(
            &[-1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0],
            None,
        )
        .unwrap(),
        height: None,
        extruded_height: None,
        ellipsoid: None,
        granularity: None,
        per_position_height: None,
        arc_type: None,
        offset_attribute: None,
    })
    .unwrap();

    let p = geom.create_geometry().unwrap();
    assert_eq!(p.attributes.position.as_ref().unwrap().values.len(), 8 * 3);
    assert_eq!(p.indices.len(), 8 * 2);
}

#[test]
fn test_polygon_outline_geometry_computes_positions_with_per_position_heights() {
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

    let geom = PolygonOutlineGeometry::from_positions(PolygonOutlineGeometryFromPositionsOptions {
        positions,
        height: None,
        extruded_height: None,
        ellipsoid: Some(ellipsoid),
        granularity: None,
        per_position_height: Some(true),
        arc_type: None,
        offset_attribute: None,
    })
    .unwrap();

    let p = geom.create_geometry().unwrap();
    let pos_values = &p.attributes.position.as_ref().unwrap().values;
    let p0 = Cartesian3::new(pos_values[0], pos_values[1], pos_values[2]);
    let c0 = ellipsoid.cartesian_to_cartographic(p0.into()).unwrap();
    assert!((c0.height - height).abs() < EPSILON6);

    let p1 = Cartesian3::new(pos_values[3], pos_values[4], pos_values[5]);
    let c1 = ellipsoid.cartesian_to_cartographic(p1.into()).unwrap();
    assert!(c1.height.abs() < EPSILON6);
}

#[test]
fn test_polygon_outline_geometry_creates_with_rhumb_lines() {
    let geom = PolygonOutlineGeometry::from_positions(PolygonOutlineGeometryFromPositionsOptions {
        positions: Cartesian3::from_degrees_array(
            &[-1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0],
            None,
        )
        .unwrap(),
        height: None,
        extruded_height: None,
        ellipsoid: None,
        granularity: Some(RADIANS_PER_DEGREE),
        per_position_height: None,
        arc_type: Some(ArcType::Rhumb),
        offset_attribute: None,
    })
    .unwrap();

    let p = geom.create_geometry().unwrap();
    assert_eq!(p.attributes.position.as_ref().unwrap().values.len(), 8 * 3);
    assert_eq!(p.indices.len(), 16);
}

#[test]
fn test_polygon_outline_geometry_different_subdivisions_geodesic_and_rhumb() {
    let positions = Cartesian3::from_degrees_array(
        &[-80.0, 75.0, 80.0, 75.0, 80.0, 45.0, -80.0, 45.0],
        None,
    )
    .unwrap();

    let geo = PolygonOutlineGeometry::from_positions(PolygonOutlineGeometryFromPositionsOptions {
        positions: positions.clone(),
        height: None,
        extruded_height: None,
        ellipsoid: None,
        granularity: Some(RADIANS_PER_DEGREE),
        per_position_height: None,
        arc_type: Some(ArcType::Geodesic),
        offset_attribute: None,
    })
    .unwrap()
    .create_geometry()
    .unwrap();

    let rhumb = PolygonOutlineGeometry::from_positions(PolygonOutlineGeometryFromPositionsOptions {
        positions,
        height: None,
        extruded_height: None,
        ellipsoid: None,
        granularity: Some(RADIANS_PER_DEGREE),
        per_position_height: None,
        arc_type: Some(ArcType::Rhumb),
        offset_attribute: None,
    })
    .unwrap()
    .create_geometry()
    .unwrap();

    assert_ne!(
        geo.attributes.position.as_ref().unwrap().values.len(),
        rhumb.attributes.position.as_ref().unwrap().values.len()
    );
    assert_ne!(geo.indices.len(), rhumb.indices.len());
}

#[test]
fn test_polygon_outline_geometry_computes_positions_with_per_position_heights_rhumb() {
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

    let geom = PolygonOutlineGeometry::from_positions(PolygonOutlineGeometryFromPositionsOptions {
        positions,
        height: None,
        extruded_height: None,
        ellipsoid: Some(ellipsoid),
        granularity: None,
        per_position_height: Some(true),
        arc_type: Some(ArcType::Rhumb),
        offset_attribute: None,
    })
    .unwrap();

    let p = geom.create_geometry().unwrap();
    let pos_values = &p.attributes.position.as_ref().unwrap().values;
    let p0 = Cartesian3::new(pos_values[0], pos_values[1], pos_values[2]);
    let c0 = ellipsoid.cartesian_to_cartographic(p0.into()).unwrap();
    assert!((c0.height - height).abs() < EPSILON6);

    let p1 = Cartesian3::new(pos_values[3], pos_values[4], pos_values[5]);
    let c1 = ellipsoid.cartesian_to_cartographic(p1.into()).unwrap();
    assert!(c1.height.abs() < EPSILON6);
}

#[test]
fn test_polygon_outline_geometry_extruded_height_and_per_position_height() {
    let ellipsoid = Ellipsoid::WGS84;
    let max_height = 100.0;
    let min_height = 60.0;
    let extruded_height = 50.0;
    let positions = Cartesian3::from_degrees_array_heights(
        &[
            -1.0, -1.0, max_height,
            1.0, -1.0, min_height,
            1.0, 1.0, min_height,
            -1.0, 1.0, min_height,
        ],
        None,
    )
    .unwrap();

    let geom = PolygonOutlineGeometry::from_positions(PolygonOutlineGeometryFromPositionsOptions {
        positions,
        height: None,
        extruded_height: Some(extruded_height),
        ellipsoid: Some(ellipsoid),
        granularity: None,
        per_position_height: Some(true),
        arc_type: None,
        offset_attribute: None,
    })
    .unwrap();

    let p = geom.create_geometry().unwrap();
    let pos_values = &p.attributes.position.as_ref().unwrap().values;

    let p0 = Cartesian3::new(pos_values[0], pos_values[1], pos_values[2]);
    let c0 = ellipsoid.cartesian_to_cartographic(p0.into()).unwrap();
    assert!((c0.height - max_height).abs() < EPSILON6);

    let p1 = Cartesian3::new(pos_values[3], pos_values[4], pos_values[5]);
    let c1 = ellipsoid.cartesian_to_cartographic(p1.into()).unwrap();
    assert!((c1.height - min_height).abs() < EPSILON6);

    let p24 = Cartesian3::new(pos_values[24], pos_values[25], pos_values[26]);
    let c24 = ellipsoid.cartesian_to_cartographic(p24.into()).unwrap();
    assert!((c24.height - extruded_height).abs() < EPSILON6);
}

#[test]
fn test_polygon_outline_geometry_creates_from_hierarchy() {
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

    let geom = PolygonOutlineGeometry::new(PolygonOutlineGeometryOptions {
        polygon_hierarchy: hierarchy,
        height: None,
        extruded_height: None,
        ellipsoid: None,
        granularity: Some(PI_OVER_THREE),
        per_position_height: None,
        arc_type: None,
        offset_attribute: None,
    })
    .unwrap();

    let p = geom.create_geometry().unwrap();
    assert_eq!(p.attributes.position.as_ref().unwrap().values.len(), 12 * 3);
    assert_eq!(p.indices.len(), 12 * 2);
}

#[test]
fn test_polygon_outline_geometry_creates_from_clockwise_hierarchy() {
    let hierarchy = PolygonHierarchy {
        positions: Cartesian3::from_degrees_array(
            &[-124.0, 35.0, -124.0, 40.0, -110.0, 40.0, -110.0, 35.0],
            None,
        )
        .unwrap(),
        holes: vec![PolygonHierarchy {
            positions: Cartesian3::from_degrees_array(
                &[-122.0, 36.0, -112.0, 36.0, -112.0, 39.0, -122.0, 39.0],
                None,
            )
            .unwrap(),
            holes: vec![PolygonHierarchy {
                positions: Cartesian3::from_degrees_array(
                    &[-120.0, 36.5, -120.0, 38.5, -114.0, 38.5, -114.0, 36.5],
                    None,
                )
                .unwrap(),
                holes: Vec::new(),
            }],
        }],
    };

    let geom = PolygonOutlineGeometry::new(PolygonOutlineGeometryOptions {
        polygon_hierarchy: hierarchy,
        height: None,
        extruded_height: None,
        ellipsoid: None,
        granularity: Some(PI_OVER_THREE),
        per_position_height: None,
        arc_type: None,
        offset_attribute: None,
    })
    .unwrap();

    let p = geom.create_geometry().unwrap();
    assert_eq!(p.attributes.position.as_ref().unwrap().values.len(), 12 * 3);
    assert_eq!(p.indices.len(), 12 * 2);
}

#[test]
fn test_polygon_outline_geometry_does_not_reverse_input_array() {
    let p = Cartesian3::from_degrees_array(
        &[-124.0, 35.0, -124.0, 40.0, -110.0, 40.0, -110.0, 35.0],
        None,
    )
    .unwrap();
    let h1 = Cartesian3::from_degrees_array(
        &[-122.0, 36.0, -112.0, 36.0, -112.0, 39.0, -122.0, 39.0],
        None,
    )
    .unwrap();
    let h2 = Cartesian3::from_degrees_array(
        &[-120.0, 36.5, -120.0, 38.5, -114.0, 38.5, -114.0, 36.5],
        None,
    )
    .unwrap();

    let hierarchy = PolygonHierarchy {
        positions: p.clone(),
        holes: vec![PolygonHierarchy {
            positions: h1.clone(),
            holes: vec![PolygonHierarchy {
                positions: h2.clone(),
                holes: Vec::new(),
            }],
        }],
    };

    let geom = PolygonOutlineGeometry::new(PolygonOutlineGeometryOptions {
        polygon_hierarchy: hierarchy,
        height: None,
        extruded_height: None,
        ellipsoid: None,
        granularity: Some(PI_OVER_THREE),
        per_position_height: None,
        arc_type: None,
        offset_attribute: None,
    })
    .unwrap();

    let _ = geom.create_geometry().unwrap();

    let expected_p = Cartesian3::from_degrees_array(
        &[-124.0, 35.0, -124.0, 40.0, -110.0, 40.0, -110.0, 35.0],
        None,
    )
    .unwrap();
    for i in 0..p.len() {
        assert!(p[i].equals_epsilon(&expected_p[i], Some(EPSILON9), Some(EPSILON9)));
    }
}

#[test]
fn test_polygon_outline_geometry_bounding_sphere() {
    let geom = PolygonOutlineGeometry::from_positions(PolygonOutlineGeometryFromPositionsOptions {
        positions: Cartesian3::from_degrees_array(
            &[-108.0, 1.0, -108.0, -1.0, -106.0, -1.0, -106.0, 1.0],
            None,
        )
        .unwrap(),
        height: None,
        extruded_height: None,
        ellipsoid: None,
        granularity: Some(PI_OVER_THREE),
        per_position_height: None,
        arc_type: None,
        offset_attribute: None,
    })
    .unwrap();

    let p = geom.create_geometry().unwrap();
    let bs = BoundingSphere::from_vertices(
        &p.attributes.position.as_ref().unwrap().values,
        Cartesian3::ZERO.into(),
        3,
    )
    .unwrap();

    let sphere = p.bounding_sphere.unwrap();
    assert!(sphere.center.distance(bs.center) < EPSILON9);
    assert!((sphere.radius - bs.radius).abs() < EPSILON9);
}

#[test]
fn test_polygon_outline_geometry_bounding_sphere_high_altitude() {
    let height = 40000000.0;
    let positions = Cartesian3::from_degrees_array(
        &[-108.0, 1.0, -108.0, -1.0, -106.0, -1.0, -106.0, 1.0],
        None,
    )
    .unwrap();

    let geom = PolygonOutlineGeometry::from_positions(PolygonOutlineGeometryFromPositionsOptions {
        positions,
        height: Some(height),
        extruded_height: None,
        ellipsoid: None,
        granularity: None,
        per_position_height: None,
        arc_type: None,
        offset_attribute: None,
    })
    .unwrap();

    let p = geom.create_geometry().unwrap();
    let pts = Cartesian3::from_degrees_array_heights(
        &[
            -108.0, 1.0, height,
            -108.0, -1.0, height,
            -106.0, -1.0, height,
            -106.0, 1.0, height,
        ],
        None,
    )
    .unwrap();
    let pts_dvec: Vec<glam::DVec3> = pts.iter().map(|p| (*p).into()).collect();

    let bs = BoundingSphere::from_points(&pts_dvec).unwrap();
    let sphere = p.bounding_sphere.unwrap();
    assert!((sphere.radius - bs.radius).abs() < 100.0);
}

#[test]
fn test_polygon_outline_geometry_computes_positions_extruded() {
    let geom = PolygonOutlineGeometry::from_positions(PolygonOutlineGeometryFromPositionsOptions {
        positions: Cartesian3::from_degrees_array(
            &[-1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0],
            None,
        )
        .unwrap(),
        height: None,
        extruded_height: Some(30000.0),
        ellipsoid: None,
        granularity: None,
        per_position_height: None,
        arc_type: None,
        offset_attribute: None,
    })
    .unwrap();

    let p = geom.create_geometry().unwrap();
    assert_eq!(p.attributes.position.as_ref().unwrap().values.len(), 16 * 3);
    assert_eq!(p.indices.len(), 20 * 2);
}

#[test]
fn test_polygon_outline_geometry_creates_from_hierarchy_extruded() {
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

    let geom = PolygonOutlineGeometry::new(PolygonOutlineGeometryOptions {
        polygon_hierarchy: hierarchy,
        height: None,
        extruded_height: Some(30000.0),
        ellipsoid: None,
        granularity: Some(PI_OVER_THREE),
        per_position_height: None,
        arc_type: None,
        offset_attribute: None,
    })
    .unwrap();

    let p = geom.create_geometry().unwrap();
    assert_eq!(p.attributes.position.as_ref().unwrap().values.len(), 24 * 3);
    assert_eq!(p.indices.len(), 36 * 2);
}

#[test]
fn test_polygon_outline_geometry_offset_attribute() {
    let geom = PolygonOutlineGeometry::from_positions(PolygonOutlineGeometryFromPositionsOptions {
        positions: Cartesian3::from_degrees_array(
            &[-1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0],
            None,
        )
        .unwrap(),
        height: None,
        extruded_height: None,
        ellipsoid: None,
        granularity: None,
        per_position_height: None,
        arc_type: None,
        offset_attribute: Some(GeometryOffsetAttribute::Top),
    })
    .unwrap();

    let p = geom.create_geometry().unwrap();
    let num_verts = 8;
    assert_eq!(
        p.attributes.position.as_ref().unwrap().values.len(),
        num_verts * 3
    );
    let offset = &p.attributes.apply_offset.as_ref().unwrap().values;
    assert_eq!(offset.len(), num_verts);
    let expected = vec![1.0; num_verts];
    assert_eq!(offset, &expected);
}

#[test]
fn test_polygon_outline_geometry_offset_attribute_extruded_top() {
    let geom = PolygonOutlineGeometry::from_positions(PolygonOutlineGeometryFromPositionsOptions {
        positions: Cartesian3::from_degrees_array(
            &[-1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0],
            None,
        )
        .unwrap(),
        height: None,
        extruded_height: Some(30000.0),
        ellipsoid: None,
        granularity: None,
        per_position_height: None,
        arc_type: None,
        offset_attribute: Some(GeometryOffsetAttribute::Top),
    })
    .unwrap();

    let p = geom.create_geometry().unwrap();
    let num_verts = 16;
    assert_eq!(
        p.attributes.position.as_ref().unwrap().values.len(),
        num_verts * 3
    );
    let offset = &p.attributes.apply_offset.as_ref().unwrap().values;
    assert_eq!(offset.len(), num_verts);
    let mut expected = vec![0.0; num_verts];
    for item in expected.iter_mut().take(8) {
        *item = 1.0;
    }
    assert_eq!(offset, &expected);
}

#[test]
fn test_polygon_outline_geometry_offset_attribute_extruded_all() {
    let geom = PolygonOutlineGeometry::from_positions(PolygonOutlineGeometryFromPositionsOptions {
        positions: Cartesian3::from_degrees_array(
            &[-1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0],
            None,
        )
        .unwrap(),
        height: None,
        extruded_height: Some(30000.0),
        ellipsoid: None,
        granularity: None,
        per_position_height: None,
        arc_type: None,
        offset_attribute: Some(GeometryOffsetAttribute::All),
    })
    .unwrap();

    let p = geom.create_geometry().unwrap();
    let num_verts = 16;
    assert_eq!(
        p.attributes.position.as_ref().unwrap().values.len(),
        num_verts * 3
    );
    let offset = &p.attributes.apply_offset.as_ref().unwrap().values;
    assert_eq!(offset.len(), num_verts);
    let expected = vec![1.0; num_verts];
    assert_eq!(offset, &expected);
}

#[test]
fn test_polygon_outline_geometry_pack_unpack() {
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

    let geom = PolygonOutlineGeometry::new(PolygonOutlineGeometryOptions {
        polygon_hierarchy: hierarchy,
        height: None,
        extruded_height: None,
        ellipsoid: None,
        granularity: Some(PI_OVER_THREE),
        per_position_height: Some(true),
        arc_type: Some(ArcType::Geodesic),
        offset_attribute: None,
    })
    .unwrap();

    let mut packed = Vec::new();
    let len = geom.pack(&mut packed, None);
    assert_eq!(len, geom.packed_length());
    assert_eq!(packed.len(), geom.packed_length());

    let unpacked = PolygonOutlineGeometry::unpack(&packed, None, None).unwrap();
    assert_eq!(unpacked.height(), geom.height());
    assert_eq!(unpacked.extruded_height(), geom.extruded_height());
    assert_eq!(unpacked.granularity(), geom.granularity());
    assert_eq!(unpacked.per_position_height(), geom.per_position_height());
    assert_eq!(unpacked.arc_type(), geom.arc_type());
    assert_eq!(unpacked.offset_attribute(), geom.offset_attribute());
    assert_eq!(unpacked.packed_length(), geom.packed_length());
    assert_eq!(
        unpacked.polygon_hierarchy().positions.len(),
        geom.polygon_hierarchy().positions.len()
    );
}
