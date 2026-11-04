use cesium_math::{
    BoundingRectangle, Cartesian2, Cartesian3, CornerType, Ellipsoid,
    PolylineVolumeGeometryLibrary,
};
use glam::DVec2;

#[test]
fn test_polyline_volume_geometry_library_collinear_positions() {
    let positions = vec![
        Cartesian3::new(1.0, 1.0, 1.0),
        Cartesian3::new(2.0, 2.0, 2.0),
        Cartesian3::new(3.0, 3.0, 3.0),
        Cartesian3::new(4.0, 4.0, 4.0),
    ];

    let shape = vec![
        Cartesian2::new(-0.15, -0.15),
        Cartesian2::new(0.15, -0.15),
        Cartesian2::new(0.15, 0.15),
        Cartesian2::new(-0.15, 0.15),
    ];

    let shape_dvec2: Vec<DVec2> = shape.iter().map(|p| DVec2::new(p.x, p.y)).collect();
    let bounding_rectangle = BoundingRectangle::from_points(&shape_dvec2).unwrap();

    let computed = PolylineVolumeGeometryLibrary::compute_positions(
        &positions,
        &shape,
        &bounding_rectangle,
        &Ellipsoid::WGS84,
        CornerType::Rounded,
        0.1,
        true,
    );

    assert!(!computed.is_empty());
    for val in &computed {
        assert!(val.is_finite());
    }
}

#[test]
fn test_polyline_volume_geometry_library_remove_duplicates_from_shape() {
    let shape = vec![
        Cartesian2::new(0.0, 0.0),
        Cartesian2::new(1.0, 0.0),
        Cartesian2::new(1.0, 0.0), // adjacent duplicate
        Cartesian2::new(1.0, 1.0),
        Cartesian2::new(0.0, 0.0), // wrap-around duplicate with first
    ];
    let cleaned = PolylineVolumeGeometryLibrary::remove_duplicates_from_shape(&shape);
    assert_eq!(cleaned.len(), 3);
    assert_eq!(cleaned[0], Cartesian2::new(1.0, 0.0));
    assert_eq!(cleaned[1], Cartesian2::new(1.0, 1.0));
    assert_eq!(cleaned[2], Cartesian2::new(0.0, 0.0));
}

#[test]
fn test_polyline_volume_geometry_library_angle_is_greater_than_pi() {
    let pos = Cartesian3::from_degrees(0.0, 0.0, None, None).unwrap();
    let forward = Cartesian3::UNIT_Y;
    let backward = Cartesian3::UNIT_X;
    let is_gt = PolylineVolumeGeometryLibrary::angle_is_greater_than_pi(
        forward,
        backward,
        pos,
        &Ellipsoid::WGS84,
    );
    // Evaluates local tangent plane orientation
    let _ = is_gt;
}
