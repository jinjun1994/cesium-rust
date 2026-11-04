use cesium_math::{
    CelestialFrameTransforms, JulianDate, Matrix3, Quaternion, TimeStandard,
};

fn matrix3_determinant(m: &Matrix3) -> f64 {
    let v = &m.values;
    // column-major indexing:
    // [0] [3] [6]
    // [1] [4] [7]
    // [2] [5] [8]
    v[0] * (v[4] * v[8] - v[7] * v[5])
        - v[3] * (v[1] * v[8] - v[7] * v[2])
        + v[6] * (v[1] * v[5] - v[4] * v[2])
}

#[test]
fn test_compute_teme_to_pseudo_fixed_matrix() {
    // June 29, 2015 12:00:00 UTC
    let time = JulianDate::from_utc_components(2015, 6, 29, 12, 0, 0, 0.0);
    let t = CelestialFrameTransforms::compute_teme_to_pseudo_fixed_matrix(&time);

    let det = matrix3_determinant(&t);
    assert!((det - 1.0).abs() <= 1e-14);

    let t_inv = t.inverse().unwrap();
    let t_trans = t.transpose();
    for i in 0..9 {
        assert!((t_inv.values[i] - t_trans.values[i]).abs() <= 1e-14);
    }

    // Add one sidereal day (23.93447 hours)
    let time_next = time.add_seconds(23.93447 * 3600.0);
    let u = CelestialFrameTransforms::compute_teme_to_pseudo_fixed_matrix(&time_next);
    let t_quat = Quaternion::from_rotation_matrix(&t);
    let u_quat = Quaternion::from_rotation_matrix(&u);
    let t_angle = t_quat.compute_angle();
    let u_angle = u_quat.compute_angle();
    assert!((t_angle - u_angle).abs() <= 1e-6);
}

#[test]
#[allow(clippy::excessive_precision)]
fn test_compute_icrf_to_moon_fixed_matrix() {
    // 2011-07-03 00:00:00 UTC
    let time = JulianDate::new(2455745.0, 43200.0, TimeStandard::UTC);
    let t = CelestialFrameTransforms::compute_icrf_to_moon_fixed_matrix(&time);

    let det = matrix3_determinant(&t);
    assert!((det - 1.0).abs() <= 1e-14);

    let t_inv = t.inverse().unwrap();
    let t_trans = t.transpose();
    for i in 0..9 {
        assert!((t_inv.values[i] - t_trans.values[i]).abs() <= 1e-14);
    }

    // Expected matrix from STK Components / upstream spec
    let expected_mtx = Matrix3::from_row_major_array([
        -0.44796811269393627,
        0.8934634849604557,
        0.03236620230657612,
        0.8184479558129512,
        0.3952490953922868,
        0.4170384828971786,
        0.3598159441089767,
        0.2133099942194372,
        -0.9083123541662688,
    ]);

    for i in 0..9 {
        assert!(
            (t.values[i] - expected_mtx.values[i]).abs() <= 1e-13,
            "matrix index {} mismatch: {} vs {}",
            i,
            t.values[i],
            expected_mtx.values[i]
        );
    }

    // Add one sidereal month (27.321661 * 24 hours)
    let time_month = time.add_seconds(27.321661 * 24.0 * 3600.0);
    let u = CelestialFrameTransforms::compute_icrf_to_moon_fixed_matrix(&time_month);
    let t_quat = Quaternion::from_rotation_matrix(&t);
    let u_quat = Quaternion::from_rotation_matrix(&u);
    let t_angle = t_quat.compute_angle();
    let u_angle = u_quat.compute_angle();
    assert!((t_angle - u_angle).abs() <= 1e-3);
}
