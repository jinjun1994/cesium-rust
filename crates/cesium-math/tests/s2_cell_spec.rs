use cesium_math::{Cartesian3, Ellipsoid, S2Cell, EPSILON10, EPSILON15};

fn from_degrees(lon_deg: f64, lat_deg: f64) -> Cartesian3 {
    let carto = cesium_math::Cartographic::from_radians(
        lon_deg.to_radians(),
        lat_deg.to_radians(),
    );
    let dvec = Ellipsoid::WGS84
        .cartographic_to_cartesian(carto)
        .unwrap();
    Cartesian3::new(dvec.x, dvec.y, dvec.z)
}

fn assert_cartesian_equals_epsilon(actual: Cartesian3, expected: Cartesian3, epsilon: f64) {
    assert!(
        actual.equals_epsilon(&expected, Some(epsilon), None),
        "Actual {:?} not within epsilon {} of expected {:?}",
        actual,
        epsilon,
        expected
    );
}

#[test]
fn test_s2_cell_constructor_and_validation() {
    let cell = S2Cell::from_id(3458764513820540928).unwrap();
    assert_eq!(cell.cell_id(), 3458764513820540928);

    assert!(S2Cell::from_id(0).is_err());
    assert!(!S2Cell::is_valid_id(0));
    assert!(!S2Cell::is_valid_id(u64::MAX));

    // Valid cell IDs
    assert!(S2Cell::is_valid_id(3383782026967071428));
    assert!(S2Cell::is_valid_id(3458764513820540928));

    // Invalid cell IDs
    assert!(!S2Cell::is_valid_id(0b0010101000000000000000000000000000000000000000000000000000000000));
}

#[test]
fn test_s2_cell_from_token() {
    let cell = S2Cell::from_token("3").unwrap();
    assert_eq!(cell.cell_id(), 3458764513820540928);

    assert!(S2Cell::from_token("XX").is_err());
    assert!(S2Cell::from_token("LOL").is_err());
    assert!(S2Cell::from_token("----").is_err());
    assert!(S2Cell::from_token("0").is_err());

    assert!(S2Cell::is_valid_token("1"));
    assert!(S2Cell::is_valid_token("2ef59bd34"));
    assert!(S2Cell::is_valid_token("2ef59bd352b93ac3"));
}

#[test]
fn test_s2_cell_token_id_roundtrip() {
    assert_eq!(S2Cell::get_id_from_token("04").unwrap(), 288230376151711744);
    assert_eq!(S2Cell::get_id_from_token("3").unwrap(), 3458764513820540928);
    assert_eq!(
        S2Cell::get_id_from_token("2ef59bd352b93ac3").unwrap(),
        3383782026967071427
    );

    assert_eq!(S2Cell::get_token_from_id(288230376151711744), "04");
    assert_eq!(S2Cell::get_token_from_id(3458764513820540928), "3");
    assert_eq!(
        S2Cell::get_token_from_id(3383782026967071427),
        "2ef59bd352b93ac3"
    );
}

#[test]
fn test_s2_cell_from_face_position_level() {
    let cell = S2Cell::from_face_position_level(0, 0, 1).unwrap();
    assert_eq!(S2Cell::get_token_from_id(cell.cell_id()), "04");

    let cell = S2Cell::from_face_position_level(0, 1, 1).unwrap();
    assert_eq!(S2Cell::get_token_from_id(cell.cell_id()), "0c");

    let cell = S2Cell::from_face_position_level(0, 2, 1).unwrap();
    assert_eq!(S2Cell::get_token_from_id(cell.cell_id()), "14");

    let cell = S2Cell::from_face_position_level(0, 3, 1).unwrap();
    assert_eq!(S2Cell::get_token_from_id(cell.cell_id()), "1c");

    let cell = S2Cell::from_face_position_level(2, 0, 1).unwrap();
    assert_eq!(S2Cell::get_token_from_id(cell.cell_id()), "44");

    let cell = S2Cell::from_face_position_level(4, 0, 1).unwrap();
    assert_eq!(S2Cell::get_token_from_id(cell.cell_id()), "84");

    let cell = S2Cell::from_face_position_level(1, 538969508876688737, 30).unwrap();
    assert_eq!(S2Cell::get_token_from_id(cell.cell_id()), "2ef59bd352b93ac3");

    // Invalid face, position, level
    assert!(S2Cell::from_face_position_level(-1, 0, 1).is_err());
    assert!(S2Cell::from_face_position_level(6, 0, 1).is_err());
    assert!(S2Cell::from_face_position_level(0, 4, 1).is_err());
    assert!(S2Cell::from_face_position_level(0, 0, -1).is_err());
    assert!(S2Cell::from_face_position_level(0, 0, 31).is_err());
}

#[test]
fn test_s2_cell_get_level() {
    assert_eq!(S2Cell::get_level(3170534137668829184).unwrap(), 1);
    assert_eq!(S2Cell::get_level(3383782026921377792).unwrap(), 16);
    assert_eq!(S2Cell::get_level(3383782026967071427).unwrap(), 30);

    assert!(S2Cell::get_level(0).is_err());
}

#[test]
fn test_s2_cell_parent() {
    let cell = S2Cell::from_id(3383782026967515136).unwrap();
    let parent = cell.get_parent().unwrap();
    assert_eq!(parent.cell_id(), 3383782026971709440);

    // Parent at level
    let cell = S2Cell::from_id(3383782026967056384).unwrap();
    let p21 = cell.get_parent_at_level(21).unwrap();
    assert_eq!(p21.cell_id(), 3383782026967252992);

    let p7 = cell.get_parent_at_level(7).unwrap();
    assert_eq!(p7.cell_id(), 3383821801271328768);

    let p0 = cell.get_parent_at_level(0).unwrap();
    assert_eq!(p0.cell_id(), 3458764513820540928);

    // Level 0 cells have no parent
    let root = S2Cell::from_token("3").unwrap();
    assert!(root.get_parent().is_err());
    assert!(root.get_parent_at_level(0).is_err());
}

#[test]
fn test_s2_cell_children() {
    let cell = S2Cell::from_id(3383782026971709440).unwrap();
    let expected = [
        3383782026959126528_u64,
        3383782026967515136_u64,
        3383782026975903744_u64,
        3383782026984292352_u64,
    ];

    for (i, &expected_id) in expected.iter().enumerate() {
        assert_eq!(cell.get_child(i).unwrap().cell_id(), expected_id);
    }

    assert!(cell.get_child(4).is_err());

    let leaf = S2Cell::from_id(3383782026967071427).unwrap();
    assert_eq!(leaf.level(), 30);
    assert!(leaf.get_child(0).is_err());
}

#[test]
fn test_s2_cell_get_center() {
    assert_cartesian_equals_epsilon(
        S2Cell::from_token("1").unwrap().get_center(None),
        from_degrees(0.0, 0.0),
        EPSILON10,
    );
    assert_cartesian_equals_epsilon(
        S2Cell::from_token("3").unwrap().get_center(None),
        from_degrees(90.0, 0.0),
        EPSILON10,
    );
    assert_cartesian_equals_epsilon(
        S2Cell::from_token("5").unwrap().get_center(None),
        from_degrees(-180.0, 90.0),
        EPSILON10,
    );
    assert_cartesian_equals_epsilon(
        S2Cell::from_token("7").unwrap().get_center(None),
        from_degrees(-180.0, 0.0),
        EPSILON10,
    );
    assert_cartesian_equals_epsilon(
        S2Cell::from_token("9").unwrap().get_center(None),
        from_degrees(-90.0, 0.0),
        EPSILON10,
    );
    assert_cartesian_equals_epsilon(
        S2Cell::from_token("b").unwrap().get_center(None),
        from_degrees(0.0, -90.0),
        EPSILON10,
    );
    assert_cartesian_equals_epsilon(
        S2Cell::from_token("2ef59bd352b93ac3").unwrap().get_center(None),
        from_degrees(105.64131803774308, -10.490091033598308),
        EPSILON10,
    );
    assert_cartesian_equals_epsilon(
        S2Cell::from_token("1234567").unwrap().get_center(None),
        from_degrees(9.868307318504081, 27.468392925827605),
        EPSILON10,
    );
}

#[test]
fn test_s2_cell_get_vertices() {
    let cell = S2Cell::from_token("2ef59bd352b93ac3").unwrap();

    assert_cartesian_equals_epsilon(
        cell.get_vertex(0, None).unwrap(),
        from_degrees(105.64131799299665, -10.490091077431977),
        EPSILON15,
    );
    assert_cartesian_equals_epsilon(
        cell.get_vertex(1, None).unwrap(),
        from_degrees(105.64131808248949, -10.490091072946313),
        EPSILON15,
    );
    assert_cartesian_equals_epsilon(
        cell.get_vertex(2, None).unwrap(),
        from_degrees(105.64131808248948, -10.490090989764633),
        EPSILON15,
    );
    assert_cartesian_equals_epsilon(
        cell.get_vertex(3, None).unwrap(),
        from_degrees(105.64131799299665, -10.4900909942503),
        EPSILON15,
    );

    assert!(cell.get_vertex(4, None).is_err());
}
