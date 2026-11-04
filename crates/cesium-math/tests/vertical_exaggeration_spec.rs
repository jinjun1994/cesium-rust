use cesium_math::{Cartesian3, Ellipsoid, VerticalExaggeration, EPSILON8};

#[test]
fn test_get_height_leaves_heights_unchanged_with_scale_1() {
    let height = 100.0;
    let scale = 1.0;
    let relative_height = 0.0;

    let result = VerticalExaggeration::get_height(height, scale, relative_height);
    assert_eq!(result, height);
}

#[test]
fn test_get_height_scales_up_heights_above_relative_height() {
    let height = 150.0;
    let scale = 2.0;
    let relative_height = 100.0;

    let result = VerticalExaggeration::get_height(height, scale, relative_height);
    assert_eq!(result, 200.0);
}

#[test]
fn test_get_height_does_not_change_heights_equal_to_relative_height() {
    let height = 100.0;
    let scale = 1.0;
    let relative_height = 100.0;

    let result = VerticalExaggeration::get_height(height, scale, relative_height);
    assert_eq!(result, 100.0);
}

#[test]
fn test_get_height_scales_down_heights_below_relative_height() {
    let height = 100.0;
    let scale = 2.0;
    let relative_height = 200.0;

    let result = VerticalExaggeration::get_height(height, scale, relative_height);
    assert_eq!(result, 0.0);
}

#[test]
fn test_get_position_leaves_positions_unchanged_with_scale_1() {
    let ellipsoid = Ellipsoid::WGS84;
    let position = Cartesian3::from_radians(0.0, 0.0, Some(100.0), Some(&ellipsoid)).unwrap();
    let vertical_exaggeration = 1.0;
    let vertical_exaggeration_relative_height = 0.0;

    let result = VerticalExaggeration::get_position(
        &position,
        &ellipsoid,
        vertical_exaggeration,
        vertical_exaggeration_relative_height,
    );
    assert!(result.equals_epsilon(&position, Some(EPSILON8), None));
}

#[test]
fn test_get_position_scales_up_positions_above_relative_height() {
    let ellipsoid = Ellipsoid::WGS84;
    let position = Cartesian3::from_radians(0.0, 0.0, Some(150.0), Some(&ellipsoid)).unwrap();
    let vertical_exaggeration = 2.0;
    let vertical_exaggeration_relative_height = 100.0;

    let result = VerticalExaggeration::get_position(
        &position,
        &ellipsoid,
        vertical_exaggeration,
        vertical_exaggeration_relative_height,
    );
    let expected = Cartesian3::from_radians(0.0, 0.0, Some(200.0), Some(&ellipsoid)).unwrap();
    assert!(result.equals_epsilon(&expected, Some(EPSILON8), None));
}

#[test]
fn test_get_position_does_not_change_positions_equal_to_relative_height() {
    let ellipsoid = Ellipsoid::WGS84;
    let position = Cartesian3::from_radians(0.0, 0.0, Some(100.0), Some(&ellipsoid)).unwrap();
    let vertical_exaggeration = 1.0;
    let vertical_exaggeration_relative_height = 100.0;

    let result = VerticalExaggeration::get_position(
        &position,
        &ellipsoid,
        vertical_exaggeration,
        vertical_exaggeration_relative_height,
    );
    assert!(result.equals_epsilon(&position, Some(EPSILON8), None));
}

#[test]
fn test_get_position_scales_down_positions_below_relative_height() {
    let ellipsoid = Ellipsoid::WGS84;
    let position = Cartesian3::from_radians(0.0, 0.0, Some(100.0), Some(&ellipsoid)).unwrap();
    let vertical_exaggeration = 2.0;
    let vertical_exaggeration_relative_height = 200.0;

    let result = VerticalExaggeration::get_position(
        &position,
        &ellipsoid,
        vertical_exaggeration,
        vertical_exaggeration_relative_height,
    );
    let expected = Cartesian3::from_radians(0.0, 0.0, Some(0.0), Some(&ellipsoid)).unwrap();
    assert!(result.equals_epsilon(&expected, Some(EPSILON8), None));
}
