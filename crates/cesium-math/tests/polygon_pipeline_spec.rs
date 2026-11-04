use cesium_math::{
    Cartesian2, Cartesian3, Ellipsoid, PolygonPipeline, WindingOrder, RADIANS_PER_DEGREE,
};

#[test]
fn test_polygon_pipeline_compute_area_2d() {
    let positions_ccw = vec![
        Cartesian2::new(0.0, 0.0),
        Cartesian2::new(2.0, 0.0),
        Cartesian2::new(2.0, 1.0),
        Cartesian2::new(0.0, 1.0),
    ];
    let area = PolygonPipeline::compute_area_2d(&positions_ccw).unwrap();
    assert_eq!(area, 2.0);

    let positions_cw = vec![
        Cartesian2::new(0.0, 0.0),
        Cartesian2::new(0.0, 2.0),
        Cartesian2::new(1.0, 2.0),
        Cartesian2::new(1.0, 0.0),
    ];
    let area_cw = PolygonPipeline::compute_area_2d(&positions_cw).unwrap();
    assert_eq!(area_cw, -2.0);

    assert!(PolygonPipeline::compute_area_2d(&[]).is_err());
    assert!(PolygonPipeline::compute_area_2d(&[Cartesian2::ZERO, Cartesian2::ZERO]).is_err());
}

#[test]
fn test_polygon_pipeline_compute_winding_order_2d() {
    let positions_ccw = vec![
        Cartesian2::new(0.0, 0.0),
        Cartesian2::new(2.0, 0.0),
        Cartesian2::new(2.0, 1.0),
        Cartesian2::new(0.0, 1.0),
    ];
    let order_ccw = PolygonPipeline::compute_winding_order_2d(&positions_ccw).unwrap();
    assert_eq!(order_ccw, WindingOrder::COUNTER_CLOCKWISE);

    let positions_cw = vec![
        Cartesian2::new(0.0, 0.0),
        Cartesian2::new(0.0, 2.0),
        Cartesian2::new(1.0, 2.0),
        Cartesian2::new(1.0, 0.0),
    ];
    let order_cw = PolygonPipeline::compute_winding_order_2d(&positions_cw).unwrap();
    assert_eq!(order_cw, WindingOrder::CLOCKWISE);
}

#[test]
fn test_polygon_pipeline_triangulate_triangle_and_square() {
    let tri_positions = vec![
        Cartesian2::new(0.0, 0.0),
        Cartesian2::new(1.0, 0.0),
        Cartesian2::new(0.0, 1.0),
    ];
    let tri_indices = PolygonPipeline::triangulate(&tri_positions, &[]);
    assert_eq!(tri_indices, vec![1, 2, 0]);

    let square_positions = vec![
        Cartesian2::new(0.0, 0.0),
        Cartesian2::new(1.0, 0.0),
        Cartesian2::new(1.0, 1.0),
        Cartesian2::new(0.0, 1.0),
    ];
    let square_indices = PolygonPipeline::triangulate(&square_positions, &[]);
    assert_eq!(square_indices, vec![2, 3, 0, 0, 1, 2]);
}

#[test]
fn test_polygon_pipeline_triangulate_holes() {
    let positions = vec![
        Cartesian2::new(0.0, 0.0),
        Cartesian2::new(3.0, 0.0),
        Cartesian2::new(3.0, 3.0),
        Cartesian2::new(0.0, 3.0),
    ];
    let hole = vec![
        Cartesian2::new(1.0, 1.0),
        Cartesian2::new(2.0, 1.0),
        Cartesian2::new(2.0, 2.0),
        Cartesian2::new(1.0, 2.0),
    ];
    let mut combined = positions;
    combined.extend(hole);

    let indices = PolygonPipeline::triangulate(&combined, &[4]);
    assert_eq!(indices.len(), 24); // 8 triangles

    // 检查每个三角形都在有效顶点范围内
    for &idx in &indices {
        assert!(idx < 8);
    }

    // 校验三角网总面积 = 外环面积 (9) - 内孔面积 (1) = 8.0
    let mut total_area = 0.0;
    for chunk in indices.chunks(3) {
        let p0 = combined[chunk[0] as usize];
        let p1 = combined[chunk[1] as usize];
        let p2 = combined[chunk[2] as usize];
        let tri_area = ((p1.x - p0.x) * (p2.y - p0.y) - (p2.x - p0.x) * (p1.y - p0.y)).abs() * 0.5;
        total_area += tri_area;
    }
    assert!((total_area - 8.0).abs() < 1e-10);
}

#[test]
fn test_polygon_pipeline_compute_subdivision_without_subdivisions() {
    let positions = vec![
        Cartesian3::new(0.0, 0.0, 90.0),
        Cartesian3::new(0.0, 90.0, 0.0),
        Cartesian3::new(90.0, 0.0, 0.0),
    ];
    let indices = vec![0, 1, 2];
    let geom = PolygonPipeline::compute_subdivision(
        &Ellipsoid::WGS84,
        &positions,
        &indices,
        None,
        Some(60.0),
    )
    .unwrap();

    let pos_values = geom.attributes.position.unwrap().values;
    assert_eq!(pos_values.len(), 9);
    assert_eq!(pos_values[0], 0.0);
    assert_eq!(pos_values[1], 0.0);
    assert_eq!(pos_values[2], 90.0);

    let idx = geom.indices;
    assert_eq!(idx, vec![0, 1, 2]);
}

#[test]
fn test_polygon_pipeline_compute_subdivision_with_subdivisions_and_texcoords() {
    let positions = vec![
        Cartesian3::new(6377802.759444977, -58441.30561735455, 29025.647900582237),
        Cartesian3::new(
            6377802.759444977,
            -58441.30561735455,
            -29025.647900582237,
        ),
        Cartesian3::new(6378137.0, 0.0, 0.0),
        Cartesian3::new(6377802.759444977, 58441.30561735455, -29025.647900582237),
        Cartesian3::new(6377802.759444977, 58441.30561735455, 29025.647900582237),
    ];
    let indices = vec![0, 1, 2, 2, 3, 4, 4, 0, 2];
    let texcoords = vec![
        Cartesian2::new(0.0, 1.0),
        Cartesian2::new(0.0, 0.0),
        Cartesian2::new(0.5, 0.0),
        Cartesian2::new(1.0, 0.0),
        Cartesian2::new(1.0, 1.0),
    ];

    let geom = PolygonPipeline::compute_subdivision(
        &Ellipsoid::WGS84,
        &positions,
        &indices,
        Some(&texcoords),
        None,
    )
    .unwrap();

    let pos_values = geom.attributes.position.unwrap().values;
    assert_eq!(pos_values.len(), 18); // 6 vertices
    assert_eq!(pos_values[0], 6377802.759444977);
    assert_eq!(pos_values[1], -58441.30561735455);
    assert_eq!(pos_values[2], 29025.647900582237);

    let idx = geom.indices;
    assert_eq!(idx.len(), 12);
    assert_eq!(idx[0], 5);
    assert_eq!(idx[1], 0);
    assert_eq!(idx[2], 2);

    let st = geom.attributes.st.unwrap().values;
    assert_eq!(st.len(), 12);
    assert_eq!(st[0], 0.0);
    assert_eq!(st[1], 1.0);
}

#[test]
fn test_polygon_pipeline_compute_rhumb_line_subdivision() {
    let positions = Cartesian3::from_degrees_array(&[0.0, 0.0, 1.0, 0.0, 1.0, 1.0], None).unwrap();
    let indices = vec![0, 1, 2];
    let geom = PolygonPipeline::compute_rhumb_line_subdivision(
        &Ellipsoid::WGS84,
        &positions,
        &indices,
        None,
        Some(0.5 * RADIANS_PER_DEGREE),
    )
    .unwrap();

    let pos_values = geom.attributes.position.unwrap().values;
    assert_eq!(pos_values.len(), 36); // 12 vertices
    let idx = geom.indices;
    assert_eq!(idx.len(), 36); // 12 triangles
}

#[test]
fn test_polygon_pipeline_scale_to_geodetic_height() {
    let ellipsoid = Ellipsoid::WGS84;
    let mut positions = vec![
        6378137.0, 0.0, 0.0,
        0.0, 6378137.0, 0.0,
    ];

    PolygonPipeline::scale_to_geodetic_height(&mut positions, Some(100.0), Some(&ellipsoid), Some(true));
    assert!((positions[0] - (6378137.0 + 100.0)).abs() < 1e-4);
    assert!((positions[4] - (6378137.0 + 100.0)).abs() < 1e-4);
}
