use cesium_math::{
    Iau2000Orientation, IauOrientationAxes, JulianDate, Matrix3, TimeStandard,
};

#[test]
#[allow(clippy::excessive_precision)]
fn test_iau_orientation_axes_compute_icrf_to_moon_fixed() {
    let date = JulianDate::new(2451545.0, -32.184, TimeStandard::TAI);
    let axes = IauOrientationAxes::new(Some(Iau2000Orientation::compute_moon));

    let expected_matrix = Matrix3::from_row_major_array([
        0.784227052091917,
        0.55784711246016394,
        0.27165148607559436,
        -0.62006191525085563,
        0.7205566654668133,
        0.31035675134719942,
        -0.022608671404182448,
        -0.41183090094261243,
        0.91097977859342938,
    ]);

    let mtx = axes.evaluate(&date);

    for i in 0..9 {
        assert!(
            (mtx.values[i] - expected_matrix.values[i]).abs() <= 1e-13,
            "matrix values at index {} mismatch: {} vs {}",
            i,
            mtx.values[i],
            expected_matrix.values[i]
        );
    }
}
