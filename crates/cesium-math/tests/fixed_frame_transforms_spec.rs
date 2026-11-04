use cesium_math::{
    Cartesian2, Cartesian3, Cartesian4, Ellipsoid, FixedFrameTransforms, GeographicProjection,
    HeadingPitchRoll, LocalFrameAxis, Matrix4, EPSILON14,
};

#[test]
fn test_east_north_up_to_fixed_frame_works() {
    let origin = Cartesian3::new(1.0, 0.0, 0.0);
    let expected_translation = Cartesian4::new(origin.x, origin.y, origin.z, 1.0);

    let result = FixedFrameTransforms::east_north_up_to_fixed_frame(
        origin,
        Some(Ellipsoid::UNIT_SPHERE),
    )
    .unwrap();

    assert_eq!(result.get_column(0).unwrap(), Cartesian4::UNIT_Y); // east
    assert_eq!(result.get_column(1).unwrap(), Cartesian4::UNIT_Z); // north
    assert_eq!(result.get_column(2).unwrap(), Cartesian4::UNIT_X); // up
    assert_eq!(result.get_column(3).unwrap(), expected_translation); // translation
}

#[test]
fn test_east_north_up_to_fixed_frame_at_north_pole() {
    let origin = Cartesian3::new(0.0, 0.0, 1.0);
    let expected_translation = Cartesian4::new(origin.x, origin.y, origin.z, 1.0);

    let result = FixedFrameTransforms::east_north_up_to_fixed_frame(
        origin,
        Some(Ellipsoid::UNIT_SPHERE),
    )
    .unwrap();

    assert_eq!(result.get_column(0).unwrap(), Cartesian4::UNIT_Y); // east
    assert_eq!(result.get_column(1).unwrap(), Cartesian4::new(-1.0, 0.0, 0.0, 0.0)); // north
    assert_eq!(result.get_column(2).unwrap(), Cartesian4::UNIT_Z); // up
    assert_eq!(result.get_column(3).unwrap(), expected_translation);
}

#[test]
fn test_east_north_up_to_fixed_frame_at_south_pole() {
    let origin = Cartesian3::new(0.0, 0.0, -1.0);
    let expected_translation = Cartesian4::new(origin.x, origin.y, origin.z, 1.0);

    let result = FixedFrameTransforms::east_north_up_to_fixed_frame(
        origin,
        Some(Ellipsoid::UNIT_SPHERE),
    )
    .unwrap();

    assert_eq!(result.get_column(0).unwrap(), Cartesian4::UNIT_Y); // east
    assert_eq!(result.get_column(1).unwrap(), Cartesian4::new(1.0, 0.0, 0.0, 0.0)); // north
    assert_eq!(result.get_column(2).unwrap(), Cartesian4::new(0.0, 0.0, -1.0, 0.0)); // up
    assert_eq!(result.get_column(3).unwrap(), expected_translation);
}

#[test]
fn test_east_north_up_to_fixed_frame_at_origin() {
    let origin = Cartesian3::ZERO;
    let expected_translation = Cartesian4::new(0.0, 0.0, 0.0, 1.0);

    let result = FixedFrameTransforms::east_north_up_to_fixed_frame(
        origin,
        Some(Ellipsoid::UNIT_SPHERE),
    )
    .unwrap();

    assert_eq!(result.get_column(0).unwrap(), Cartesian4::UNIT_Y); // east
    assert_eq!(result.get_column(1).unwrap(), Cartesian4::new(-1.0, 0.0, 0.0, 0.0)); // north
    assert_eq!(result.get_column(2).unwrap(), Cartesian4::UNIT_Z); // up
    assert_eq!(result.get_column(3).unwrap(), expected_translation);
}

#[test]
fn test_north_east_down_to_fixed_frame() {
    let origin = Cartesian3::new(1.0, 0.0, 0.0);
    let expected_translation = Cartesian4::new(origin.x, origin.y, origin.z, 1.0);

    let result = FixedFrameTransforms::north_east_down_to_fixed_frame(
        origin,
        Some(Ellipsoid::UNIT_SPHERE),
    )
    .unwrap();

    assert_eq!(result.get_column(0).unwrap(), Cartesian4::UNIT_Z); // north
    assert_eq!(result.get_column(1).unwrap(), Cartesian4::UNIT_Y); // east
    assert_eq!(result.get_column(2).unwrap(), Cartesian4::new(-1.0, 0.0, 0.0, 0.0)); // down
    assert_eq!(result.get_column(3).unwrap(), expected_translation);
}

#[test]
fn test_north_west_up_to_fixed_frame() {
    let origin = Cartesian3::new(1.0, 0.0, 0.0);
    let expected_translation = Cartesian4::new(origin.x, origin.y, origin.z, 1.0);

    let result = FixedFrameTransforms::north_west_up_to_fixed_frame(
        origin,
        Some(Ellipsoid::UNIT_SPHERE),
    )
    .unwrap();

    assert_eq!(result.get_column(0).unwrap(), Cartesian4::UNIT_Z); // north
    assert_eq!(result.get_column(1).unwrap(), Cartesian4::new(0.0, -1.0, 0.0, 0.0)); // west
    assert_eq!(result.get_column(2).unwrap(), Cartesian4::UNIT_X); // up
    assert_eq!(result.get_column(3).unwrap(), expected_translation);
}

#[test]
fn test_local_frame_axis_cross() {
    assert_eq!(LocalFrameAxis::East.cross(LocalFrameAxis::North).unwrap(), LocalFrameAxis::Up);
    assert_eq!(LocalFrameAxis::North.cross(LocalFrameAxis::East).unwrap(), LocalFrameAxis::Down);
    assert_eq!(LocalFrameAxis::North.cross(LocalFrameAxis::Up).unwrap(), LocalFrameAxis::East);
    assert_eq!(LocalFrameAxis::North.cross(LocalFrameAxis::West).unwrap(), LocalFrameAxis::Up);
}

#[test]
fn test_heading_pitch_roll_to_fixed_frame_zero() {
    let origin = Cartesian3::new(1.0, 0.0, 0.0);
    let hpr = HeadingPitchRoll::new(0.0, 0.0, 0.0);

    let result = FixedFrameTransforms::heading_pitch_roll_to_fixed_frame(
        origin,
        &hpr,
        Some(Ellipsoid::UNIT_SPHERE),
        None,
    )
    .unwrap();

    let expected = FixedFrameTransforms::east_north_up_to_fixed_frame(
        origin,
        Some(Ellipsoid::UNIT_SPHERE),
    )
    .unwrap();

    assert!(result.equals_epsilon(&expected, EPSILON14));
}

#[test]
fn test_fixed_frame_to_heading_pitch_roll_roundtrip() {
    let origin = Cartesian3::new(1000.0, 2000.0, 3000.0);
    let hpr = HeadingPitchRoll::new(0.3, 0.4, 0.5);

    let transform = FixedFrameTransforms::heading_pitch_roll_to_fixed_frame(
        origin,
        &hpr,
        Some(Ellipsoid::WGS84),
        None,
    )
    .unwrap();

    let computed_hpr = FixedFrameTransforms::fixed_frame_to_heading_pitch_roll(
        &transform,
        Some(Ellipsoid::WGS84),
        None,
    )
    .unwrap();

    assert!((computed_hpr.heading - hpr.heading).abs() < 1e-10);
    assert!((computed_hpr.pitch - hpr.pitch).abs() < 1e-10);
    assert!((computed_hpr.roll - hpr.roll).abs() < 1e-10);
}

#[test]
fn test_heading_pitch_roll_quaternion() {
    let origin = Cartesian3::new(1.0, 0.0, 0.0);
    let hpr = HeadingPitchRoll::new(0.0, 0.0, 0.0);

    let q = FixedFrameTransforms::heading_pitch_roll_quaternion(
        origin,
        &hpr,
        Some(Ellipsoid::UNIT_SPHERE),
        None,
    )
    .unwrap();

    let transform = FixedFrameTransforms::heading_pitch_roll_to_fixed_frame(
        origin,
        &hpr,
        Some(Ellipsoid::UNIT_SPHERE),
        None,
    )
    .unwrap();
    let rot = transform.get_matrix3();
    let expected_q = cesium_math::Quaternion::from_rotation_matrix(&rot);

    assert!(q.equals_epsilon(&expected_q, EPSILON14));
}

#[test]
fn test_point_to_window_coordinates() {
    let mvp = Matrix4::IDENTITY;
    let viewport = Matrix4::new(
        100.0, 0.0, 0.0, 100.0,
        0.0, 100.0, 0.0, 100.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );
    let pt = Cartesian3::new(0.0, 0.0, 0.0);
    let gl_win = FixedFrameTransforms::point_to_gl_window_coordinates(&mvp, &viewport, pt);
    assert_eq!(gl_win, Cartesian2::new(100.0, 100.0));

    let win = FixedFrameTransforms::point_to_window_coordinates(&mvp, &viewport, pt);
    assert_eq!(win, Cartesian2::new(100.0, 100.0));
}

#[test]
fn test_rotation_matrix_from_position_velocity() {
    let pos = Cartesian3::UNIT_X;
    let vel = Cartesian3::UNIT_Y;
    let rot = FixedFrameTransforms::rotation_matrix_from_position_velocity(
        pos,
        vel,
        Some(Ellipsoid::UNIT_SPHERE),
    )
    .unwrap();

    let expected = cesium_math::Matrix3::new(
        0.0, 0.0, 1.0,
        1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
    );
    assert!(rot.equals_epsilon(&expected, EPSILON14));
}

#[test]
fn test_basis_to_2d_and_ellipsoid_to_2d() {
    let projection = GeographicProjection::default();
    let center = Cartesian3::new(Ellipsoid::WGS84.radii().x, 0.0, 0.0);

    let m2d = FixedFrameTransforms::ellipsoid_to_2d_model_matrix(&projection, center).unwrap();
    assert!(m2d.values[15] == 1.0);

    let m3d = FixedFrameTransforms::east_north_up_to_fixed_frame(center, Some(Ellipsoid::WGS84)).unwrap();
    let basis2d = FixedFrameTransforms::basis_to_2d(&projection, &m3d).unwrap();
    assert!(basis2d.values[15] == 1.0);
}
