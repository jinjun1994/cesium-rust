use cesium_math::{
    Cartesian3, HeadingPitchRange, HeadingPitchRoll, Matrix3, Quaternion,
};

#[test]
fn test_quaternion_upstream() {
    let q_id = Quaternion::IDENTITY;
    assert_eq!(q_id.x, 0.0);
    assert_eq!(q_id.y, 0.0);
    assert_eq!(q_id.z, 0.0);
    assert_eq!(q_id.w, 1.0);

    let q_z90 = Quaternion::from_axis_angle(Cartesian3::UNIT_Z, std::f64::consts::FRAC_PI_2);
    let m = Matrix3::from_quaternion(&q_z90);
    let rot_x = m.multiply_by_vector(Cartesian3::UNIT_X);
    assert!((rot_x.x).abs() < 1e-12);
    assert!((rot_x.y - 1.0).abs() < 1e-12);
    assert!((rot_x.z).abs() < 1e-12);

    let q_recovered = Quaternion::from_matrix3(&m);
    assert!(q_recovered.equals_epsilon(&q_z90, 1e-12));

    // slerp
    let q_z0 = Quaternion::IDENTITY;
    let half = Quaternion::slerp(&q_z0, &q_z90, 0.5);
    let half_m = Matrix3::from_quaternion(&half);
    let rot_half = half_m.multiply_by_vector(Cartesian3::UNIT_X);
    let expected_val = (std::f64::consts::FRAC_PI_4).cos();
    assert!((rot_half.x - expected_val).abs() < 1e-6);
    assert!((rot_half.y - expected_val).abs() < 1e-6);
}

#[test]
fn test_heading_pitch_roll_upstream() {
    let hpr = HeadingPitchRoll::new(0.1, 0.2, 0.3);
    let q = Quaternion::from_heading_pitch_roll(&hpr);
    let recovered = HeadingPitchRoll::from_quaternion(&q);

    assert!((recovered.heading - hpr.heading).abs() < 1e-6);
    assert!((recovered.pitch - hpr.pitch).abs() < 1e-6);
    assert!((recovered.roll - hpr.roll).abs() < 1e-6);

    let hpr_deg = HeadingPitchRoll::from_degrees(90.0, 45.0, 0.0);
    assert!((hpr_deg.heading - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
    assert!((hpr_deg.pitch - std::f64::consts::FRAC_PI_4).abs() < 1e-12);
    assert!(hpr_deg.roll.abs() < 1e-12);
}

#[test]
fn test_heading_pitch_range_upstream() {
    let hpr = HeadingPitchRange::new(1.0, 2.0, 300.0);
    assert_eq!(hpr.heading, 1.0);
    assert_eq!(hpr.pitch, 2.0);
    assert_eq!(hpr.range, 300.0);

    let cloned = hpr;
    assert_eq!(cloned, hpr);
}
