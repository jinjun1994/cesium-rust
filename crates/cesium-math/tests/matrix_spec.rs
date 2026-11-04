use cesium_math::{Cartesian2, Cartesian3, Cartesian4, Matrix2, Matrix3, Matrix4};

#[test]
fn test_matrix2_upstream() {
    // Row-major argument constructor:
    // [1, 2]
    // [3, 4]
    let m = Matrix2::new(1.0, 2.0, 3.0, 4.0);
    // Column-major internal storage:
    // col0: 1, 3
    // col1: 2, 4
    assert_eq!(m[0], 1.0);
    assert_eq!(m[1], 3.0);
    assert_eq!(m[2], 2.0);
    assert_eq!(m[3], 4.0);

    let v = Cartesian2::new(5.0, 6.0);
    let mv = m.multiply_by_vector(v);
    // [1*5 + 2*6, 3*5 + 4*6] = [17, 39]
    assert_eq!(mv, Cartesian2::new(17.0, 39.0));

    let t = m.transpose();
    assert_eq!(t[0], 1.0);
    assert_eq!(t[1], 2.0);
    assert_eq!(t[2], 3.0);
    assert_eq!(t[3], 4.0);
}

#[test]
fn test_matrix3_upstream() {
    let m = Matrix3::IDENTITY;
    assert_eq!(m[0], 1.0);
    assert_eq!(m[4], 1.0);
    assert_eq!(m[8], 1.0);

    let scale = Matrix3::from_scale(Cartesian3::new(2.0, 3.0, 4.0));
    let v = Cartesian3::new(1.0, 1.0, 1.0);
    let scaled = scale.multiply_by_vector(v);
    assert_eq!(scaled, Cartesian3::new(2.0, 3.0, 4.0));

    let rot_z = Matrix3::from_rotation_z(std::f64::consts::FRAC_PI_2);
    let rot_x = rot_z.multiply_by_vector(Cartesian3::UNIT_X);
    assert!((rot_x.x).abs() < 1e-12);
    assert!((rot_x.y - 1.0).abs() < 1e-12);
    assert!((rot_x.z).abs() < 1e-12);

    let inv = scale.inverse().unwrap();
    let id = scale.multiply(&inv);
    assert!(id.equals_epsilon(&Matrix3::IDENTITY, 1e-12));
}

#[test]
fn test_matrix4_upstream() {
    let m = Matrix4::IDENTITY;
    assert_eq!(m[0], 1.0);
    assert_eq!(m[5], 1.0);
    assert_eq!(m[10], 1.0);
    assert_eq!(m[15], 1.0);

    let trans = Matrix4::from_translation(Cartesian3::new(10.0, 20.0, 30.0));
    assert_eq!(trans.get_translation(), Cartesian3::new(10.0, 20.0, 30.0));

    let pt = Cartesian3::new(1.0, 2.0, 3.0);
    let pt_trans = trans.multiply_by_point(pt);
    assert_eq!(pt_trans, Cartesian3::new(11.0, 22.0, 33.0));

    // Direction should ignore translation
    let dir = Cartesian3::new(1.0, 0.0, 0.0);
    let dir_trans = trans.multiply_by_direction(dir);
    assert_eq!(dir_trans, Cartesian3::new(1.0, 0.0, 0.0));

    let mut packed = [0.0; 16];
    trans.pack(&mut packed, 0).unwrap();
    assert_eq!(packed.len(), 16);
    let unpacked = Matrix4::unpack(&packed, 0).unwrap();
    assert_eq!(unpacked, trans);

    let inv = trans.inverse_transformation().unwrap();
    let roundtrip = trans.multiply(&inv);
    assert!(roundtrip.equals_epsilon(&Matrix4::IDENTITY, 1e-12));

    let v4 = Cartesian4::new(1.0, 2.0, 3.0, 1.0);
    let mv4 = trans.multiply_by_vector(v4);
    assert_eq!(mv4, Cartesian4::new(11.0, 22.0, 33.0, 1.0));
}
