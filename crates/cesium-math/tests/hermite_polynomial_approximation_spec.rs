use cesium_math::HermitePolynomialApproximation;

fn assert_close(a: f64, b: f64, eps: f64) {
    assert!((a - b).abs() <= eps, "expected {b}, got {a}");
}

#[test]
fn test_hermite_interpolating_produces_correct_results() {
    let x_table = [0.0, 60.0, 120.0, 180.0, 240.0, 300.0, 360.0, 420.0];
    let y_table = [
        13378137.0, 0.0, 13374128.3576279, 0.0, 13362104.8328212, 0.0, 13342073.6310691,
        0.0, 13314046.7567223, 0.0, 13278041.005799, 0.0, 13234077.9559193, 0.0,
        13182183.953374, 0.0,
    ];
    let dy_table = [
        0.0, 0.0, -133.614738921601, 0.0, -267.149404854867, 0.0, -400.523972797808, 0.0,
        -533.658513692378, 0.0, -666.473242324565, 0.0, -798.888565138278, 0.0,
        -930.82512793439, 0.0,
    ];

    let mut y_table_combined = vec![0.0; y_table.len() * 2];
    for i in 0..(y_table.len() / 2) {
        y_table_combined[i * 4] = y_table[i * 2];
        y_table_combined[i * 4 + 1] = y_table[i * 2 + 1];
        y_table_combined[i * 4 + 2] = dy_table[i * 2];
        y_table_combined[i * 4 + 3] = dy_table[i * 2 + 1];
    }

    let x = 100.0;
    let result = HermitePolynomialApproximation::interpolate_order_zero(
        x,
        &x_table,
        &y_table_combined,
        4,
    )
    .unwrap();

    let expected = 13367002.870928625;
    assert_close(result[0], expected, 1e-6);
}

#[test]
fn test_hermite_get_required_data_points() {
    assert_eq!(HermitePolynomialApproximation::get_required_data_points(0, 0), 2);
    assert_eq!(HermitePolynomialApproximation::get_required_data_points(1, 0), 2);
    assert_eq!(HermitePolynomialApproximation::get_required_data_points(2, 0), 3);
    assert_eq!(HermitePolynomialApproximation::get_required_data_points(3, 0), 4);
    assert_eq!(HermitePolynomialApproximation::get_required_data_points(3, 1), 2);
    assert_eq!(HermitePolynomialApproximation::get_required_data_points(5, 1), 3);
    assert_eq!(HermitePolynomialApproximation::get_required_data_points(7, 1), 4);
}

#[test]
fn test_hermite_higher_order_interpolation_produces_correct_results() {
    let x_table = [0.0, 60.0, 120.0, 180.0, 240.0, 300.0, 360.0, 420.0];
    let y_table = [
        13378137.0, 0.0, 13374128.3576279, 0.0, 13362104.8328212, 0.0, 13342073.6310691,
        0.0, 13314046.7567223, 0.0, 13278041.005799, 0.0, 13234077.9559193, 0.0,
        13182183.953374, 0.0,
    ];
    let dy_table = [
        0.0, 0.0, -133.614738921601, 0.0, -267.149404854867, 0.0, -400.523972797808, 0.0,
        -533.658513692378, 0.0, -666.473242324565, 0.0, -798.888565138278, 0.0,
        -930.82512793439, 0.0,
    ];

    let mut y_table_combined = vec![0.0; y_table.len() * 2];
    for i in 0..(y_table.len() / 2) {
        y_table_combined[i * 4] = y_table[i * 2];
        y_table_combined[i * 4 + 1] = y_table[i * 2 + 1];
        y_table_combined[i * 4 + 2] = dy_table[i * 2];
        y_table_combined[i * 4 + 3] = dy_table[i * 2 + 1];
    }

    let x = 100.0;
    let result = HermitePolynomialApproximation::interpolate(
        x,
        &x_table,
        &y_table_combined,
        2,
        1,
        1,
    )
    .unwrap();

    let expected = [13367002.870928625, 0.0, -222.65168787012135, 0.0];
    assert_eq!(result.len(), 4);
    assert_close(result[0], expected[0], 1e-8);
    assert_close(result[1], expected[1], 1e-8);
    assert_close(result[2], expected[2], 1e-8);
    assert_close(result[3], expected[3], 1e-8);
}
