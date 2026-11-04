use cesium_math::{ArcType, Cartesian3, Ellipsoid, PolygonGeometryLibrary, EPSILON7};

#[test]
fn test_polygon_geometry_library_subdivide_rhumb_line() {
    let p0 = Cartesian3::new(3813220.0, -5085291.0, 527179.0);
    let p1 = Cartesian3::new(3813220.0, -5085291.0, 527179.0);
    let positions = PolygonGeometryLibrary::subdivide_rhumb_line(
        &Ellipsoid::WGS84,
        p0,
        p1,
        2.0,
    );
    assert_eq!(positions.len(), 3);
    assert_eq!(positions, vec![3813220.0, -5085291.0, 527179.0]);

    let p1_close = Cartesian3::new(3813220.0, -5085291.0, 527179.0 + 1.0);
    let positions_close = PolygonGeometryLibrary::subdivide_rhumb_line(
        &Ellipsoid::WGS84,
        p0,
        p1_close,
        2.0,
    );
    assert_eq!(positions_close.len(), 3);
    assert_eq!(positions_close, vec![3813220.0, -5085291.0, 527179.0]);

    let p1_far = Cartesian3::new(3813220.0, -5085291.0, 527179.0 + 5.0);
    let positions_far = PolygonGeometryLibrary::subdivide_rhumb_line(
        &Ellipsoid::WGS84,
        p0,
        p1_far,
        2.0,
    );
    assert_eq!(positions_far.len(), 12); // 4 vertices * 3

    let expected = [
        3813220.447295841, -5085291.596511482, 527179.0622555692,
        3813220.3851130935, -5085291.513584885, 527180.3036009098,
        3813220.3229302, -5085291.430658091, 527181.5449462304,
        3813220.2607471617, -5085291.347731101, 527182.7862915307,
    ];

    for i in 0..12 {
        assert!(
            (positions_far[i] - expected[i]).abs() < EPSILON7,
            "Mismatch at index {}: got {}, expected {}",
            i,
            positions_far[i],
            expected[i]
        );
    }
}

#[test]
fn test_polygon_geometry_library_split_polygons_on_equator_simple() {
    let raw = [
        3813220.0, -5085291.0, 527179.0,
        3701301.0, -5097773.0, -993503.0,
        5037375.0, -3776794.0, -1017021.0,
        5049166.0, -3865306.0, 494270.0,
    ];
    let positions = Cartesian3::unpack_array(&raw, 0).unwrap();

    let polygons = PolygonGeometryLibrary::split_polygons_on_equator(
        std::slice::from_ref(&positions),
        &Ellipsoid::WGS84,
        ArcType::Geodesic,
    );

    let expected_inter1 = Cartesian3::new(3799258.6687873346, -5123110.886796548, 0.0);
    let expected_inter2 = Cartesian3::new(5077099.353935631, -3860530.240917096, 0.0);

    assert_eq!(polygons.len(), 2);
    assert_eq!(polygons[0].len(), 4);
    assert_eq!(polygons[0][0], positions[0]);
    assert!((polygons[0][1].x - expected_inter1.x).abs() < EPSILON7);
    assert!((polygons[0][1].y - expected_inter1.y).abs() < EPSILON7);
    assert!((polygons[0][2].x - expected_inter2.x).abs() < EPSILON7);
    assert!((polygons[0][2].y - expected_inter2.y).abs() < EPSILON7);
    assert_eq!(polygons[0][3], positions[3]);

    assert_eq!(polygons[1].len(), 4);
    assert!((polygons[1][0].x - expected_inter1.x).abs() < EPSILON7);
    assert_eq!(polygons[1][1], positions[1]);
    assert_eq!(polygons[1][2], positions[2]);
    assert!((polygons[1][3].x - expected_inter2.x).abs() < EPSILON7);
}

#[test]
fn test_polygon_geometry_library_split_polygons_on_equator_touching() {
    let raw = [
        3813220.0, -5085291.0, 527179.0,
        3701301.0, -5097773.0, 0.0,
        5049166.0, -3865306.0, 494270.0,
    ];
    let positions = Cartesian3::unpack_array(&raw, 0).unwrap();

    let polygons = PolygonGeometryLibrary::split_polygons_on_equator(
        std::slice::from_ref(&positions),
        &Ellipsoid::WGS84,
        ArcType::Geodesic,
    );

    assert_eq!(polygons.len(), 1);
    assert_eq!(polygons[0].len(), 3);
    assert_eq!(polygons[0][0], positions[0]);
    assert_eq!(polygons[0][1], positions[1]);
    assert_eq!(polygons[0][2], positions[2]);
}

#[test]
fn test_polygon_geometry_library_split_polygons_on_equator_array() {
    let raw = [
        3813220.0, -5085291.0, 527179.0,
        3701301.0, -5097773.0, -993503.0,
        5037375.0, -3776794.0, -1017021.0,
        5049166.0, -3865306.0, 494270.0,
    ];
    let positions = Cartesian3::unpack_array(&raw, 0).unwrap();

    let polygons = PolygonGeometryLibrary::split_polygons_on_equator(
        &[positions.clone(), positions],
        &Ellipsoid::WGS84,
        ArcType::Geodesic,
    );

    assert_eq!(polygons.len(), 4);
}
