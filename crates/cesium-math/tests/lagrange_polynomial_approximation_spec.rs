use cesium_math::LagrangePolynomialApproximation;

fn assert_close(a: f64, b: f64, eps: f64) {
    assert!((a - b).abs() <= eps, "expected {b}, got {a}");
}

#[test]
fn test_lagrange_interpolation_produces_correct_results() {
    let x_table = [0.0, 60.0, 120.0, 180.0, 240.0, 300.0, 360.0, 420.0];
    let y_table = [
        13378137.0, 0.0, 0.0, 13374128.3576279, 327475.593690065, 0.0, 13362104.8328212,
        654754.936954423, 0.0, 13342073.6310691, 981641.896976832, 0.0,
        13314046.7567223, 1307940.57608951, 0.0, 13278041.005799, 1633455.42917117, 0.0,
        13234077.9559193, 1957991.38083385, 0.0, 13182183.953374, 2281353.94232816, 0.0,
    ];

    let x = 100.0;
    let result =
        LagrangePolynomialApproximation::interpolate_order_zero(x, &x_table, &y_table, 3).unwrap();

    let expected = [13367002.870928623, 545695.7388100647, 0.0];
    assert_eq!(result.len(), 3);
    assert_close(result[0], expected[0], 1e-15);
    assert_close(result[1], expected[1], 1e-15);
    assert_close(result[2], expected[2], 1e-15);
}

#[test]
fn test_lagrange_get_required_data_points() {
    assert_eq!(LagrangePolynomialApproximation::get_required_data_points(0), 2);
    assert_eq!(LagrangePolynomialApproximation::get_required_data_points(1), 2);
    assert_eq!(LagrangePolynomialApproximation::get_required_data_points(2), 3);
    assert_eq!(LagrangePolynomialApproximation::get_required_data_points(3), 4);
}
