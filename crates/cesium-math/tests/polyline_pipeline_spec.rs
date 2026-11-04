use cesium_math::{
    Cartesian3, Ellipsoid, PolylineArcOptions, PolylineHeight, PolylinePipeline,
    EPSILON4, EPSILON6, PI_OVER_FOUR, PI_OVER_TWO,
};

#[test]
fn test_wrap_longitude() {
    let positions = Cartesian3::from_degrees_array(
        &[-75.163789, 39.952335, -80.2264393, 25.7889689],
        None,
    )
    .unwrap();
    let segments = PolylinePipeline::wrap_longitude(&positions, None);
    assert_eq!(segments.lengths.len(), 1);
    assert_eq!(segments.lengths[0], 2);
}

#[test]
fn test_wrap_longitude_works_with_empty_array() {
    let segments = PolylinePipeline::wrap_longitude(&[], None);
    assert_eq!(segments.lengths.len(), 0);
    assert_eq!(segments.positions.len(), 0);
}

#[test]
fn test_wrap_longitude_breaks_polyline_into_segments() {
    let positions =
        Cartesian3::from_degrees_array(&[-179.0, 39.0, 2.0, 25.0], None).unwrap();
    let segments = PolylinePipeline::wrap_longitude(&positions, None);
    assert_eq!(segments.lengths.len(), 2);
    assert_eq!(segments.lengths[0], 2);
    assert_eq!(segments.lengths[1], 2);
}

#[test]
fn test_wrap_longitude_breaks_polyline_into_segments_with_model_matrix() {
    let center = Cartesian3::from_degrees(-179.0, 39.0, None, None).unwrap();
    let matrix = Ellipsoid::WGS84
        .east_north_up_to_fixed_frame(center.into())
        .unwrap();

    let positions = vec![
        Cartesian3::new(0.0, 0.0, 0.0),
        Cartesian3::new(0.0, 100000000.0, 0.0),
    ];
    let segments = PolylinePipeline::wrap_longitude(&positions, Some(matrix));
    assert_eq!(segments.lengths.len(), 2);
    assert_eq!(segments.lengths[0], 2);
    assert_eq!(segments.lengths[1], 2);
}

#[test]
fn test_generate_arc_accepts_a_height_array_for_single_value() {
    let positions = vec![Cartesian3::from_degrees(0.0, 0.0, None, None).unwrap()];
    let height = vec![30.0];

    let new_positions = PolylinePipeline::generate_arc(&PolylineArcOptions {
        positions: &positions,
        height: Some(PolylineHeight::PerPosition(height)),
        ..Default::default()
    });

    assert_eq!(new_positions.len(), 3);
    let p = Cartesian3::new(new_positions[0], new_positions[1], new_positions[2]);
    let expected = Cartesian3::from_degrees(0.0, 0.0, Some(30.0), None).unwrap();
    assert!(p.equals_epsilon(&expected, Some(EPSILON6), Some(EPSILON6)));
}

#[test]
fn test_generate_arc_subdivides_in_half() {
    let p1 = Cartesian3::from_degrees(0.0, 0.0, None, None).unwrap();
    let p2 = Cartesian3::from_degrees(90.0, 0.0, None, None).unwrap();
    let p3 = Cartesian3::from_degrees(45.0, 0.0, None, None).unwrap();
    let positions = vec![p1, p2];

    let new_positions = PolylinePipeline::generate_arc(&PolylineArcOptions {
        positions: &positions,
        granularity: Some(PI_OVER_TWO / 2.0),
        ellipsoid: Some(Ellipsoid::WGS84),
        ..Default::default()
    });

    assert_eq!(new_positions.len(), 3 * 3);
    let p1n = Cartesian3::new(new_positions[0], new_positions[1], new_positions[2]);
    let p3n = Cartesian3::new(new_positions[3], new_positions[4], new_positions[5]);
    let p2n = Cartesian3::new(new_positions[6], new_positions[7], new_positions[8]);
    assert!(p1.equals_epsilon(&p1n, Some(EPSILON4), Some(EPSILON4)));
    assert!(p2.equals_epsilon(&p2n, Some(EPSILON4), Some(EPSILON4)));
    assert!(p3.equals_epsilon(&p3n, Some(EPSILON4), Some(EPSILON4)));
}

#[test]
fn test_generate_arc_works_with_empty_array() {
    let new_positions = PolylinePipeline::generate_arc(&PolylineArcOptions {
        positions: &[],
        ..Default::default()
    });
    assert_eq!(new_positions.len(), 0);
}

#[test]
fn test_generate_arc_works_one_position() {
    let new_positions = PolylinePipeline::generate_arc(&PolylineArcOptions {
        positions: &[Cartesian3::UNIT_Z],
        ellipsoid: Some(Ellipsoid::UNIT_SPHERE),
        ..Default::default()
    });
    assert_eq!(new_positions.len(), 3);
    assert_eq!(new_positions, vec![0.0, 0.0, 1.0]);
}

#[test]
fn test_generate_rhumb_arc_accepts_a_height_array_for_single_value() {
    let positions = vec![Cartesian3::from_degrees(0.0, 0.0, None, None).unwrap()];
    let height = vec![30.0];

    let new_positions = PolylinePipeline::generate_rhumb_arc(&PolylineArcOptions {
        positions: &positions,
        height: Some(PolylineHeight::PerPosition(height)),
        ..Default::default()
    });

    assert_eq!(new_positions.len(), 3);
    let p = Cartesian3::new(new_positions[0], new_positions[1], new_positions[2]);
    let expected = Cartesian3::from_degrees(0.0, 0.0, Some(30.0), None).unwrap();
    assert!(p.equals_epsilon(&expected, Some(EPSILON6), Some(EPSILON6)));
}

#[test]
fn test_generate_rhumb_arc_subdivides_in_half() {
    let p1 = Cartesian3::from_degrees(0.0, 30.0, None, None).unwrap();
    let p2 = Cartesian3::from_degrees(90.0, 30.0, None, None).unwrap();
    let p3 = Cartesian3::from_degrees(45.0, 30.0, None, None).unwrap();
    let positions = vec![p1, p2];

    let new_positions = PolylinePipeline::generate_rhumb_arc(&PolylineArcOptions {
        positions: &positions,
        granularity: Some(PI_OVER_FOUR),
        ellipsoid: Some(Ellipsoid::WGS84),
        ..Default::default()
    });

    assert_eq!(new_positions.len(), 3 * 3);
    let p1n = Cartesian3::new(new_positions[0], new_positions[1], new_positions[2]);
    let p3n = Cartesian3::new(new_positions[3], new_positions[4], new_positions[5]);
    let p2n = Cartesian3::new(new_positions[6], new_positions[7], new_positions[8]);
    assert!(p1.equals_epsilon(&p1n, Some(EPSILON4), Some(EPSILON4)));
    assert!(p2.equals_epsilon(&p2n, Some(EPSILON4), Some(EPSILON4)));
    assert!(p3.equals_epsilon(&p3n, Some(EPSILON4), Some(EPSILON4)));
}

#[test]
fn test_generate_rhumb_arc_works_with_empty_array() {
    let new_positions = PolylinePipeline::generate_rhumb_arc(&PolylineArcOptions {
        positions: &[],
        ..Default::default()
    });
    assert_eq!(new_positions.len(), 0);
}

#[test]
fn test_generate_rhumb_arc_works_one_position() {
    let new_positions = PolylinePipeline::generate_rhumb_arc(&PolylineArcOptions {
        positions: &[Cartesian3::UNIT_Z],
        ellipsoid: Some(Ellipsoid::UNIT_SPHERE),
        ..Default::default()
    });
    assert_eq!(new_positions.len(), 3);
    assert_eq!(new_positions, vec![0.0, 0.0, 1.0]);
}

#[test]
fn test_generate_rhumb_arc_return_values_for_each_position() {
    let positions =
        Cartesian3::from_degrees_array(&[0.0, 0.0, 10.0, 0.0, 10.0, 5.0], None).unwrap();
    let new_positions = PolylinePipeline::generate_rhumb_arc(&PolylineArcOptions {
        positions: &positions,
        ..Default::default()
    });
    assert!(!new_positions.is_empty());
    for val in &new_positions {
        assert!(val.is_finite());
    }
}

#[test]
fn test_generate_cartesian_arc_and_rhumb_arc() {
    let positions =
        Cartesian3::from_degrees_array(&[0.0, 0.0, 10.0, 0.0, 10.0, 5.0], None).unwrap();
    let arc_pts = PolylinePipeline::generate_cartesian_arc(&PolylineArcOptions {
        positions: &positions,
        ..Default::default()
    });
    assert!(!arc_pts.is_empty());

    let rhumb_pts = PolylinePipeline::generate_cartesian_rhumb_arc(&PolylineArcOptions {
        positions: &positions,
        ..Default::default()
    });
    assert!(!rhumb_pts.is_empty());
}

#[test]
fn test_extract_heights() {
    let p1 = Cartesian3::from_degrees(10.0, 20.0, Some(100.0), None).unwrap();
    let p2 = Cartesian3::from_degrees(30.0, 40.0, Some(250.0), None).unwrap();
    let heights = PolylinePipeline::extract_heights(&[p1, p2], &Ellipsoid::WGS84);
    assert_eq!(heights.len(), 2);
    assert!((heights[0] - 100.0).abs() < 1e-4);
    assert!((heights[1] - 250.0).abs() < 1e-4);
}
