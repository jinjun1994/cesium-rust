use cesium_math::LinearApproximation;

#[test]
fn test_should_produce_correct_results() {
    let x_table = [2.0, 4.0];
    let y_table = [2.0, 3.0, 4.0, 34.0];

    let results =
        LinearApproximation::interpolate_order_zero(3.0, &x_table, &y_table, 2).unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0], 3.0);
    assert_eq!(results[1], 18.5);
}

#[test]
fn test_should_produce_correct_results_2() {
    let x_table2 = [40.0, 120.0];
    let y_table2 = [20.0, 40.0, 60.0, 80.0, 90.0, 100.0];

    let results =
        LinearApproximation::interpolate_order_zero(80.0, &x_table2, &y_table2, 3).unwrap();

    assert_eq!(results.len(), 3);
    assert_eq!(results[0], 50.0);
    assert_eq!(results[1], 65.0);
    assert_eq!(results[2], 80.0);
}

#[test]
fn test_should_produce_correct_results_3() {
    let x_table3 = [20.0, 30.0];
    let y_table3 = [
        10.0, 20.0, 30.0, 20.0, 30.0, 40.0, 20.0, 40.0, 60.0, 80.0, 90.0, 100.0,
    ];

    let results =
        LinearApproximation::interpolate_order_zero(40.0, &x_table3, &y_table3, 1).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0], 30.0);
}

#[test]
fn test_should_throw_if_length_is_greater_than_2() {
    let x_table = [44.0, 99.0, 230.0];
    let y_table = [2.3, 4.5, 6.6, 3.2, 4.4, 12.23];

    assert!(LinearApproximation::interpolate_order_zero(2.3, &x_table, &y_table, 3).is_err());
}

#[test]
fn test_should_throw_when_y_stride_equals_zero() {
    let x_table = [4.0, 8.0];
    let y_table = [4.0, 8.0];

    assert!(LinearApproximation::interpolate_order_zero(6.0, &x_table, &y_table, 0).is_err());
}

#[test]
fn test_get_required_data_points_returns_2() {
    assert_eq!(LinearApproximation::get_required_data_points(1), 2);
}
