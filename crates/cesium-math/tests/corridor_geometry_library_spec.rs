use cesium_math::{
    Cartesian3, CornerType, CorridorGeometryLibrary, CorridorParameters, Ellipsoid,
};

#[test]
fn test_corridor_geometry_library_collinear_points() {
    let positions = vec![
        Cartesian3::new(1.0, 1.0, 1.0),
        Cartesian3::new(2.0, 2.0, 2.0),
        Cartesian3::new(3.0, 3.0, 3.0),
        Cartesian3::new(4.0, 4.0, 4.0),
    ];

    let result = CorridorGeometryLibrary::compute_positions(CorridorParameters {
        positions: &positions,
        width: 0.15,
        ellipsoid: Ellipsoid::WGS84,
        corner_type: CornerType::Rounded,
        granularity: 1.0_f64.to_radians(),
        save_attributes: false,
    });

    assert_eq!(result.corners.len(), 0);
}

#[test]
fn test_corridor_geometry_library_non_collinear_points() {
    let positions = vec![
        Cartesian3::new(0.0, 0.0, 1.0),
        Cartesian3::new(1.0, 0.0, 2.0),
        Cartesian3::new(1.0, 1.0, 3.0),
        Cartesian3::new(0.0, 1.0, 4.0),
    ];

    let result = CorridorGeometryLibrary::compute_positions(CorridorParameters {
        positions: &positions,
        width: 0.15,
        ellipsoid: Ellipsoid::WGS84,
        corner_type: CornerType::Rounded,
        granularity: 1.0_f64.to_radians(),
        save_attributes: false,
    });

    assert_eq!(result.corners.len(), 2);
}
