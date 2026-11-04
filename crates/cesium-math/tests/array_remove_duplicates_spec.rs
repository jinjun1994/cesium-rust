use cesium_math::{array_remove_duplicates, Cartesian3};

#[test]
fn test_remove_duplicates_length_1() {
    let positions = vec![Cartesian3::ZERO];
    let no_duplicates = array_remove_duplicates(
        &positions,
        |a, b, eps| a.equals_epsilon(b, Some(eps), None),
        false,
        None,
    );
    assert_eq!(no_duplicates, positions);
}

#[test]
fn test_remove_duplicates_none_removed() {
    let positions = vec![
        Cartesian3::ZERO,
        Cartesian3::UNIT_X,
        Cartesian3::UNIT_Y,
        Cartesian3::UNIT_Z,
    ];
    let no_duplicates = array_remove_duplicates(
        &positions,
        |a, b, eps| a.equals_epsilon(b, Some(eps), None),
        false,
        None,
    );
    assert_eq!(no_duplicates, positions);
}

#[test]
fn test_remove_duplicates_wrapping_none_removed() {
    let positions = vec![
        Cartesian3::ZERO,
        Cartesian3::UNIT_X,
        Cartesian3::UNIT_Y,
        Cartesian3::UNIT_Z,
    ];
    let no_duplicates = array_remove_duplicates(
        &positions,
        |a, b, eps| a.equals_epsilon(b, Some(eps), None),
        true,
        None,
    );
    assert_eq!(no_duplicates, positions);
}

#[test]
fn test_remove_duplicates_to_remove_duplicates() {
    let positions = vec![
        Cartesian3::new(1.0, 1.0, 1.0),
        Cartesian3::new(1.0, 1.0, 1.0),
        Cartesian3::new(1.0, 1.0, 1.0),
        Cartesian3::new(1.0, 1.0, 1.0),
        Cartesian3::new(2.0, 2.0, 2.0),
        Cartesian3::new(3.0, 3.0, 3.0),
        Cartesian3::new(3.0, 3.0, 3.0),
    ];
    let expected = vec![
        Cartesian3::new(1.0, 1.0, 1.0),
        Cartesian3::new(2.0, 2.0, 2.0),
        Cartesian3::new(3.0, 3.0, 3.0),
    ];
    let no_duplicates = array_remove_duplicates(
        &positions,
        |a, b, eps| a.equals_epsilon(b, Some(eps), None),
        false,
        None,
    );
    assert_eq!(no_duplicates, expected);
}

#[test]
fn test_remove_duplicates_nonadjacent_not_removed() {
    let positions = vec![
        Cartesian3::new(1.0, 1.0, 1.0),
        Cartesian3::new(1.0, 1.0, 1.0),
        Cartesian3::new(1.0, 1.0, 1.0),
        Cartesian3::new(1.0, 1.0, 1.0),
        Cartesian3::new(2.0, 2.0, 2.0),
        Cartesian3::new(1.0, 1.0, 1.0),
        Cartesian3::new(3.0, 3.0, 3.0),
        Cartesian3::new(3.0, 3.0, 3.0),
    ];
    let expected = vec![
        Cartesian3::new(1.0, 1.0, 1.0),
        Cartesian3::new(2.0, 2.0, 2.0),
        Cartesian3::new(1.0, 1.0, 1.0),
        Cartesian3::new(3.0, 3.0, 3.0),
    ];
    let no_duplicates = array_remove_duplicates(
        &positions,
        |a, b, eps| a.equals_epsilon(b, Some(eps), None),
        false,
        None,
    );
    assert_eq!(no_duplicates, expected);
}

#[test]
fn test_remove_duplicates_with_wrap_around_and_indices() {
    let positions = vec![
        Cartesian3::new(1.0, 1.0, 1.0),
        Cartesian3::new(1.0, 1.0, 1.0),
        Cartesian3::new(2.0, 2.0, 2.0),
        Cartesian3::new(2.0, 2.0, 2.0),
        Cartesian3::new(3.0, 3.0, 3.0),
        Cartesian3::new(1.0, 1.0, 1.0),
    ];
    let mut removed_indices = Vec::new();
    let no_duplicates = array_remove_duplicates(
        &positions,
        |a, b, eps| a.equals_epsilon(b, Some(eps), None),
        true,
        Some(&mut removed_indices),
    );
    let expected = vec![
        Cartesian3::new(1.0, 1.0, 1.0),
        Cartesian3::new(2.0, 2.0, 2.0),
        Cartesian3::new(3.0, 3.0, 3.0),
    ];
    assert_eq!(no_duplicates, expected);
    assert_eq!(removed_indices, vec![1, 3, 5]);
}
