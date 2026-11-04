use cesium_math::{
    Cartesian3, HermiteSpline, InvalidPointType, Quaternion, Spline, SplinePointType,
};

#[test]
fn test_spline_constructor_throws() {
    assert!(Spline::new().is_err());
}

#[test]
fn test_spline_wraps_time_out_of_bounds() {
    let spline = HermiteSpline::create_natural_cubic(
        vec![0.0, 1.0, 2.0],
        vec![Cartesian3::ZERO, Cartesian3::UNIT_X, Cartesian3::UNIT_Y],
    )
    .unwrap();

    assert_eq!(spline.wrap_time(-0.5), 1.5);
    assert_eq!(spline.wrap_time(2.5), 0.5);
}

#[test]
fn test_spline_clamps_time_out_of_bounds() {
    let spline = HermiteSpline::create_natural_cubic(
        vec![0.0, 1.0, 2.0],
        vec![Cartesian3::ZERO, Cartesian3::UNIT_X, Cartesian3::UNIT_Y],
    )
    .unwrap();

    assert_eq!(spline.clamp_time(-0.5), 0.0);
    assert_eq!(spline.clamp_time(2.5), 2.0);
}

#[test]
fn test_spline_find_time_interval_throws_when_time_out_of_range() {
    let spline = HermiteSpline::create_natural_cubic(
        vec![0.0, 1.0, 2.0],
        vec![Cartesian3::ZERO, Cartesian3::UNIT_X, Cartesian3::UNIT_Y],
    )
    .unwrap();

    assert!(spline.find_time_interval(4.0).is_err());
}

#[test]
fn test_spline_find_time_interval() {
    let times = vec![0.0, 1.0, 2.0, 4.0];
    let spline = HermiteSpline::create_natural_cubic(
        times.clone(),
        vec![
            Cartesian3::ZERO,
            Cartesian3::UNIT_X,
            Cartesian3::UNIT_Y,
            Cartesian3::UNIT_Z,
        ],
    )
    .unwrap();

    assert_eq!(spline.find_time_interval(times[0]).unwrap(), 0);

    // jump forward
    assert_eq!(spline.find_time_interval(times[1]).unwrap(), 1);

    // jump backward
    assert_eq!(
        spline
            .find_time_interval_with_start(times[0], Some(1))
            .unwrap(),
        0
    );

    // jump far forward
    assert_eq!(
        spline
            .find_time_interval_with_start(times[times.len() - 2], Some(0))
            .unwrap(),
        times.len() - 2
    );

    // jump far back
    assert_eq!(
        spline
            .find_time_interval_with_start(times[0], Some(times.len() - 1))
            .unwrap(),
        0
    );
}

#[test]
fn test_spline_get_point_type_throws_for_invalid_point_type() {
    assert!(Spline::get_point_type(&InvalidPointType).is_err());
}

#[test]
fn test_spline_get_point_type() {
    assert_eq!(
        Spline::get_point_type(&1.0).unwrap(),
        SplinePointType::Number
    );
    assert_eq!(
        Spline::get_point_type(&Cartesian3::ZERO).unwrap(),
        SplinePointType::Cartesian3
    );
    assert_eq!(
        Spline::get_point_type(&Quaternion::IDENTITY).unwrap(),
        SplinePointType::Quaternion
    );
}
