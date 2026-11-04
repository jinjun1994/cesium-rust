use cesium_math::{ArcType, Cartesian3, Ellipsoid, PolygonGeometryLibrary, EPSILON7};

#[test]
fn test_polygon_geometry_library_subdivide_line() {
    let p0 = Cartesian3::new(0.0, 0.0, 0.0);
    let p1 = Cartesian3::new(4.0, 0.0, 0.0);
    let count = PolygonGeometryLibrary::subdivide_line_count(p0, p1, 1.5);
    assert_eq!(count, 4);

    let positions = PolygonGeometryLibrary::subdivide_line(p0, p1, 1.5, None);
    assert_eq!(positions.len(), 12);
    let expected = [
        0.0, 0.0, 0.0,
        1.0, 0.0, 0.0,
        2.0, 0.0, 0.0,
        3.0, 0.0, 0.0,
    ];
    for i in 0..12 {
        assert!((positions[i] - expected[i]).abs() < EPSILON7);
    }
}

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
fn test_polygon_geometry_library_edge_on_equator_above() {
    let raw = [
        -3219367.0, -5491259.0, 401098.0,
        -3217795.0, -5506913.0, 0.0,
        -2713036.0, -5772334.0, 0.0,
        -2713766.0, -5757498.0, 406910.0,
    ];
    let positions = Cartesian3::unpack_array(&raw, 0).unwrap();

    let polygons = PolygonGeometryLibrary::split_polygons_on_equator(
        std::slice::from_ref(&positions),
        &Ellipsoid::WGS84,
        ArcType::Geodesic,
    );
    assert_eq!(polygons.len(), 1);
    assert_eq!(polygons[0].len(), 4);
    assert_eq!(polygons[0], positions);
}

#[test]
fn test_polygon_geometry_library_edge_on_equator_below() {
    let raw = [
        -3180138.0, -5441382.0, -974441.0,
        -3186540.0, -5525048.0, 0.0,
        -2198716.0, -5986569.0, 0.0,
        -2135113.0, -5925878.0, -996868.0,
    ];
    let positions = Cartesian3::unpack_array(&raw, 0).unwrap();

    let polygons = PolygonGeometryLibrary::split_polygons_on_equator(
        std::slice::from_ref(&positions),
        &Ellipsoid::WGS84,
        ArcType::Geodesic,
    );
    assert_eq!(polygons.len(), 1);
    assert_eq!(polygons[0].len(), 4);
    assert_eq!(polygons[0], positions);
}

#[test]
fn test_polygon_geometry_library_positively_concave() {
    let raw = [
        -3723536.687096985, -5140643.423654287, 622159.6094790212,
        -3706443.9124709764, -5089398.802336418, -1016836.564118223,
        -1818346.3577937474, -5988204.417556031, -1226992.0906221648,
        -1949728.2308330906, -6022778.780648997, 775419.1678640501,
        -2891108.934831509, -5659936.656854747, -534148.7427656263,
    ];
    let positions = Cartesian3::unpack_array(&raw, 0).unwrap();

    let polygons = PolygonGeometryLibrary::split_polygons_on_equator(
        std::slice::from_ref(&positions),
        &Ellipsoid::WGS84,
        ArcType::Geodesic,
    );

    let expected_inter1 = Cartesian3::new(-3746523.7934060274, -5161801.144582336, 0.0);
    let expected_inter2 = Cartesian3::new(-3298992.8935172106, -5458688.2562839165, 0.0);
    let expected_inter3 = Cartesian3::new(-2527814.313071595, -5855833.534980258, 0.0);
    let expected_inter4 = Cartesian3::new(-1921714.863778476, -6081746.753450187, 0.0);

    assert_eq!(polygons.len(), 3);
    assert_eq!(polygons[0].len(), 3);
    assert_eq!(polygons[0][0], positions[0]);
    assert!((polygons[0][1].x - expected_inter1.x).abs() < EPSILON7);
    assert!((polygons[0][1].y - expected_inter1.y).abs() < EPSILON7);
    assert!((polygons[0][2].x - expected_inter2.x).abs() < EPSILON7);
    assert!((polygons[0][2].y - expected_inter2.y).abs() < EPSILON7);

    assert_eq!(polygons[1].len(), 7);
    assert!((polygons[1][0].x - expected_inter1.x).abs() < EPSILON7);
    assert_eq!(polygons[1][1], positions[1]);
    assert_eq!(polygons[1][2], positions[2]);
    assert!((polygons[1][3].x - expected_inter4.x).abs() < EPSILON7);
    assert!((polygons[1][4].x - expected_inter3.x).abs() < EPSILON7);
    assert_eq!(polygons[1][5], positions[4]);
    assert!((polygons[1][6].x - expected_inter2.x).abs() < EPSILON7);

    assert_eq!(polygons[2].len(), 3);
    assert!((polygons[2][0].x - expected_inter4.x).abs() < EPSILON7);
    assert_eq!(polygons[2][1], positions[3]);
    assert!((polygons[2][2].x - expected_inter3.x).abs() < EPSILON7);
}

#[test]
fn test_polygon_geometry_library_negatively_concave() {
    let raw = [
        -4164072.7435535816, -4791571.5503237555, 605958.8290040599,
        -4167507.7232260685, -4800497.02674794, -508272.2109012767,
        -3712172.6000501625, -5184159.589216706, 116723.13202563708,
        -3259646.0020361557, -5455158.378873343, -532227.4715966922,
        -3283717.3855494126, -5434359.545068984, 592819.1229613343,
    ];
    let positions = Cartesian3::unpack_array(&raw, 0).unwrap();

    let polygons = PolygonGeometryLibrary::split_polygons_on_equator(
        std::slice::from_ref(&positions),
        &Ellipsoid::WGS84,
        ArcType::Geodesic,
    );

    let expected_inter1 = Cartesian3::new(-4182416.3757553473, -4815394.568525253, 0.0);
    let expected_inter2 = Cartesian3::new(-3803015.1382151386, -5120322.982906009, 0.0);
    let expected_inter3 = Cartesian3::new(-3635913.2183307745, -5240302.153458, 0.0);
    let expected_inter4 = Cartesian3::new(-3284360.5276909056, -5467504.688147503, 0.0);

    assert_eq!(polygons.len(), 3);
    assert_eq!(polygons[0].len(), 7);
    assert_eq!(polygons[0][0], positions[0]);
    assert!((polygons[0][1].x - expected_inter1.x).abs() < EPSILON7);
    assert!((polygons[0][1].y - expected_inter1.y).abs() < EPSILON7);
    assert!((polygons[0][2].x - expected_inter2.x).abs() < EPSILON7);
    assert_eq!(polygons[0][3], positions[2]);
    assert!((polygons[0][4].x - expected_inter3.x).abs() < EPSILON7);
    assert!((polygons[0][5].x - expected_inter4.x).abs() < EPSILON7);
    assert_eq!(polygons[0][6], positions[4]);

    assert_eq!(polygons[1].len(), 3);
    assert!((polygons[1][0].x - expected_inter1.x).abs() < EPSILON7);
    assert_eq!(polygons[1][1], positions[1]);
    assert!((polygons[1][2].x - expected_inter2.x).abs() < EPSILON7);

    assert_eq!(polygons[2].len(), 3);
    assert!((polygons[2][0].x - expected_inter3.x).abs() < EPSILON7);
    assert_eq!(polygons[2][1], positions[3]);
    assert!((polygons[2][2].x - expected_inter4.x).abs() < EPSILON7);
}

#[test]
fn test_polygon_geometry_library_positively_concave_point_on_equator() {
    let raw = [
        -3592289.0, -5251493.0, 433532.0,
        -3568746.0, -5245699.0, -646544.0,
        -2273628.0, -5915229.0, -715098.0,
        -2410175.0, -5885323.0, 475855.0,
        -3012338.0, -5621469.0, 0.0,
    ];
    let positions = Cartesian3::unpack_array(&raw, 0).unwrap();

    let polygons = PolygonGeometryLibrary::split_polygons_on_equator(
        std::slice::from_ref(&positions),
        &Ellipsoid::WGS84,
        ArcType::Geodesic,
    );

    let expected_inter1 = Cartesian3::new(-3595684.3882232937, -5267986.8423389485, 0.0);
    let expected_inter2 = Cartesian3::new(-2365929.6862513637, -5923091.111107741, 0.0);

    assert_eq!(polygons.len(), 3);
    assert_eq!(polygons[0].len(), 3);
    assert_eq!(polygons[0][0], positions[0]);
    assert!((polygons[0][1].x - expected_inter1.x).abs() < EPSILON7);
    assert_eq!(polygons[0][2], positions[4]);

    assert_eq!(polygons[1].len(), 5);
    assert!((polygons[1][0].x - expected_inter1.x).abs() < EPSILON7);
    assert_eq!(polygons[1][1], positions[1]);
    assert_eq!(polygons[1][2], positions[2]);
    assert!((polygons[1][3].x - expected_inter2.x).abs() < EPSILON7);
    assert_eq!(polygons[1][4], positions[4]);

    assert_eq!(polygons[2].len(), 3);
    assert!((polygons[2][0].x - expected_inter2.x).abs() < EPSILON7);
    assert_eq!(polygons[2][1], positions[3]);
    assert_eq!(polygons[2][2], positions[4]);
}

#[test]
fn test_polygon_geometry_library_negatively_concave_point_on_equator() {
    let raw = [
        -3774632.0, -5136123.0, 222459.0,
        -3714187.0, -5173580.0, -341046.0,
        -3516544.0, -5320967.0, 0.0,
        -3304860.0, -5444086.0, -342567.0,
        -3277484.0, -5466977.0, 218213.0,
    ];
    let positions = Cartesian3::unpack_array(&raw, 0).unwrap();

    let polygons = PolygonGeometryLibrary::split_polygons_on_equator(
        std::slice::from_ref(&positions),
        &Ellipsoid::WGS84,
        ArcType::Geodesic,
    );

    let expected_inter1 = Cartesian3::new(-3754485.468265927, -5156013.039098039, 0.0);
    let expected_inter2 = Cartesian3::new(-3291304.258941832, -5463327.545172482, 0.0);

    assert_eq!(polygons.len(), 3);
    assert_eq!(polygons[0].len(), 5);
    assert_eq!(polygons[0][0], positions[0]);
    assert!((polygons[0][1].x - expected_inter1.x).abs() < EPSILON7);
    assert_eq!(polygons[0][2], positions[2]);
    assert!((polygons[0][3].x - expected_inter2.x).abs() < EPSILON7);
    assert_eq!(polygons[0][4], positions[4]);

    assert_eq!(polygons[1].len(), 3);
    assert!((polygons[1][0].x - expected_inter1.x).abs() < EPSILON7);
    assert_eq!(polygons[1][1], positions[1]);
    assert_eq!(polygons[1][2], positions[2]);

    assert_eq!(polygons[2].len(), 3);
    assert_eq!(polygons[2][0], positions[2]);
    assert_eq!(polygons[2][1], positions[3]);
    assert!((polygons[2][2].x - expected_inter2.x).abs() < EPSILON7);
}

#[test]
fn test_polygon_geometry_library_edge_equator() {
    let raw = [
        -3227931.0, -5469496.0, 584508.0,
        -3150093.0, -5488360.0, -792747.0,
        -1700622.0, -6089685.0, -835364.0,
        -1786389.0, -6122714.0, 0.0,
        -2593600.0, -5826977.0, 0.0,
        -2609132.0, -5790155.0, 584508.0,
    ];
    let positions = Cartesian3::unpack_array(&raw, 0).unwrap();

    let polygons = PolygonGeometryLibrary::split_polygons_on_equator(
        std::slice::from_ref(&positions),
        &Ellipsoid::WGS84,
        ArcType::Geodesic,
    );

    let expected_inter = Cartesian3::new(-3213523.577073882, -5509437.159126084, 0.0);
    assert_eq!(polygons.len(), 2);
    assert_eq!(polygons[0].len(), 4);
    assert_eq!(polygons[0][0], positions[0]);
    assert!((polygons[0][1].x - expected_inter.x).abs() < EPSILON7);
    assert_eq!(polygons[0][2], positions[4]);
    assert_eq!(polygons[0][3], positions[5]);

    assert_eq!(polygons[1].len(), 5);
    assert!((polygons[1][0].x - expected_inter.x).abs() < EPSILON7);
    assert_eq!(polygons[1][1], positions[1]);
    assert_eq!(polygons[1][2], positions[2]);
    assert_eq!(polygons[1][3], positions[3]);
    assert_eq!(polygons[1][4], positions[4]);
}

#[test]
fn test_polygon_geometry_library_backtracking_edge_equator() {
    let raw = [
        -3491307.0, -5296123.0, 650596.0,
        -3495031.0, -5334507.0, 0.0,
        -4333607.0, -4677312.0, 0.0,
        -4275491.0, -4629182.0, -968553.0,
        -2403691.0, -5827997.0, -943662.0,
        -2484409.0, -5837281.0, 631344.0,
    ];
    let positions = Cartesian3::unpack_array(&raw, 0).unwrap();

    let polygons = PolygonGeometryLibrary::split_polygons_on_equator(
        std::slice::from_ref(&positions),
        &Ellipsoid::WGS84,
        ArcType::Geodesic,
    );

    let expected_inter = Cartesian3::new(-2471499.3842933537, -5879823.32933623, 0.0);
    assert_eq!(polygons.len(), 2);
    assert_eq!(polygons[0].len(), 4);
    assert_eq!(polygons[0][0], positions[0]);
    assert_eq!(polygons[0][1], positions[1]);
    assert!((polygons[0][2].x - expected_inter.x).abs() < EPSILON7);
    assert_eq!(polygons[0][3], positions[5]);

    assert_eq!(polygons[1].len(), 5);
    assert_eq!(polygons[1][0], positions[1]);
    assert_eq!(polygons[1][1], positions[2]);
    assert_eq!(polygons[1][2], positions[3]);
    assert_eq!(polygons[1][3], positions[4]);
    assert!((polygons[1][4].x - expected_inter.x).abs() < EPSILON7);
}

#[test]
fn test_polygon_geometry_library_split_rhumb_polygon_on_equator() {
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
        ArcType::Rhumb,
    );

    let expected_inter1 = Cartesian3::new(3799205.595277112, -5123150.245267465, 0.0);
    let expected_inter2 = Cartesian3::new(5077127.456540122, -3860493.2820580625, 0.0);

    assert_eq!(polygons.len(), 2);
    assert_eq!(polygons[0].len(), 4);
    assert_eq!(polygons[0][0], positions[0]);
    assert!((polygons[0][1].x - expected_inter1.x).abs() < EPSILON7);
    assert!((polygons[0][2].x - expected_inter2.x).abs() < EPSILON7);
    assert_eq!(polygons[0][3], positions[3]);

    assert_eq!(polygons[1].len(), 4);
    assert!((polygons[1][0].x - expected_inter1.x).abs() < EPSILON7);
    assert_eq!(polygons[1][1], positions[1]);
    assert_eq!(polygons[1][2], positions[2]);
    assert!((polygons[1][3].x - expected_inter2.x).abs() < EPSILON7);
}

#[test]
fn test_polygon_geometry_library_split_rhumb_polygon_across_idl() {
    let positions = Cartesian3::from_degrees_array(
        &[
            30.0, -30.0,
            20.0, 30.0,
            -20.0, 30.0,
            -30.0, -30.0,
        ],
        Some(&Ellipsoid::WGS84),
    )
    .unwrap();

    let polygons = PolygonGeometryLibrary::split_polygons_on_equator(
        std::slice::from_ref(&positions),
        &Ellipsoid::WGS84,
        ArcType::Rhumb,
    );

    let expected_inter1 = Cartesian3::new(5780555.229886577, 2695517.1720840395, 0.0);
    let expected_inter2 = Cartesian3::new(5780555.229886577, -2695517.1720840395, 0.0);

    assert_eq!(polygons.len(), 2);
    assert_eq!(polygons[0].len(), 4);
    assert_eq!(polygons[0][0], positions[0]);
    assert!((polygons[0][1].x - expected_inter1.x).abs() < EPSILON7);
    assert!((polygons[0][2].x - expected_inter2.x).abs() < EPSILON7);
    assert_eq!(polygons[0][3], positions[3]);

    assert_eq!(polygons[1].len(), 4);
    assert!((polygons[1][0].x - expected_inter1.x).abs() < EPSILON7);
    assert_eq!(polygons[1][1], positions[1]);
    assert_eq!(polygons[1][2], positions[2]);
    assert!((polygons[1][3].x - expected_inter2.x).abs() < EPSILON7);
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
