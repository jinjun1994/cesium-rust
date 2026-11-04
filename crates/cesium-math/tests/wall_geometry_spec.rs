use cesium_math::{
    Cartesian3, Ellipsoid, PrimitiveType, VertexFormat, WallGeometry, WallOutlineGeometry,
};

#[test]
fn test_wall_geometry_relative_to_ellipsoid() {
    let ellipsoid = Ellipsoid::WGS84;
    let p1 = Cartesian3::from_degrees(49.0, 18.0, Some(1000.0), Some(&ellipsoid)).unwrap();
    let p2 = Cartesian3::from_degrees(50.0, 18.0, Some(1000.0), Some(&ellipsoid)).unwrap();

    let wall = WallGeometry::new(
        vec![p1, p2],
        Some(std::f64::consts::PI),
        None,
        None,
        Some(ellipsoid),
        Some(VertexFormat::POSITION_ONLY),
    );

    let geom = wall.create_geometry().unwrap();
    let pos = geom.attributes.position.unwrap();

    let num_positions = 4;
    let num_triangles = 2;
    assert_eq!(pos.values.len(), num_positions * 3);
    assert_eq!(geom.indices.len(), num_triangles * 3);
    assert_eq!(geom.primitive_type, PrimitiveType::Triangles);

    // Bottom vertex height is 0
    let c0 = ellipsoid.cartesian_to_cartographic([pos.values[0], pos.values[1], pos.values[2]].into()).unwrap();
    assert!((c0.height - 0.0).abs() < 1e-4);

    // Top vertex height is 1000
    let c1 = ellipsoid.cartesian_to_cartographic([pos.values[3], pos.values[4], pos.values[5]].into()).unwrap();
    assert!((c1.height - 1000.0).abs() < 1e-4);
}

#[test]
fn test_wall_geometry_degenerate_cases() {
    let ellipsoid = Ellipsoid::WGS84;
    let p1 = Cartesian3::from_degrees(49.0, 18.0, Some(1000.0), Some(&ellipsoid)).unwrap();

    // Less than 2 positions
    let wall_single = WallGeometry::new(vec![p1], None, None, None, None, None);
    assert!(wall_single.create_geometry().is_none());

    // Duplicate positions
    let wall_dups = WallGeometry::new(vec![p1, p1], None, None, None, None, None);
    assert!(wall_dups.create_geometry().is_none());

    // Heights mismatch
    let p2 = Cartesian3::from_degrees(50.0, 18.0, Some(1000.0), Some(&ellipsoid)).unwrap();
    let wall_mismatch = WallGeometry::new(
        vec![p1, p2],
        None,
        Some(vec![1000.0]),
        None,
        None,
        None,
    );
    assert!(wall_mismatch.create_geometry().is_none());
}

#[test]
fn test_wall_outline_geometry_create() {
    let ellipsoid = Ellipsoid::WGS84;
    let p1 = Cartesian3::from_degrees(49.0, 18.0, Some(1000.0), Some(&ellipsoid)).unwrap();
    let p2 = Cartesian3::from_degrees(50.0, 18.0, Some(1000.0), Some(&ellipsoid)).unwrap();

    let outline = WallOutlineGeometry::new(
        vec![p1, p2],
        Some(std::f64::consts::PI),
        None,
        None,
        Some(ellipsoid),
    );

    let geom = outline.create_geometry().unwrap();
    let pos = geom.attributes.position.unwrap();

    assert_eq!(pos.values.len(), 4 * 3);
    assert_eq!(geom.indices.len(), 4 * 2);
    assert_eq!(geom.primitive_type, PrimitiveType::Lines);
}

#[test]
fn test_wall_outline_geometry_with_min_max_heights() {
    let ellipsoid = Ellipsoid::WGS84;
    let p1 = Cartesian3::from_degrees(49.0, 18.0, Some(1000.0), Some(&ellipsoid)).unwrap();
    let p2 = Cartesian3::from_degrees(50.0, 18.0, Some(1000.0), Some(&ellipsoid)).unwrap();

    let outline = WallOutlineGeometry::new(
        vec![p1, p2],
        Some(std::f64::consts::PI),
        Some(vec![3000.0, 4000.0]),
        Some(vec![1000.0, 2000.0]),
        Some(ellipsoid),
    );

    let geom = outline.create_geometry().unwrap();
    let pos = geom.attributes.position.unwrap();

    assert_eq!(pos.values.len(), 4 * 3);
    assert_eq!(geom.indices.len(), 4 * 2);

    let c_bot = ellipsoid.cartesian_to_cartographic([pos.values[0], pos.values[1], pos.values[2]].into()).unwrap();
    assert!((c_bot.height - 1000.0).abs() < 1e-4);

    let c_top = ellipsoid.cartesian_to_cartographic([pos.values[3], pos.values[4], pos.values[5]].into()).unwrap();
    assert!((c_top.height - 3000.0).abs() < 1e-4);
}
