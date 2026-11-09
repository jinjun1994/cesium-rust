use cesium_math::{Cartesian2, Cartesian3, Cartesian4, Matrix2, Matrix3, Matrix4};
use cesium_renderer::{BuiltinFunctions, Ray};

#[test]
fn test_czm_transpose() {
    // 2x2
    let m2 = Matrix2::new(1.0, 2.0, 3.0, 4.0);
    let m2_t = BuiltinFunctions::transpose_mat2(&m2);
    assert_eq!(m2_t, Matrix2::new(1.0, 3.0, 2.0, 4.0));

    // 3x3
    let m3 = Matrix3::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    let m3_t = BuiltinFunctions::transpose_mat3(&m3);
    assert_eq!(
        m3_t,
        Matrix3::new(1.0, 4.0, 7.0, 2.0, 5.0, 8.0, 3.0, 6.0, 9.0)
    );

    // 4x4
    let m4 = Matrix4::new(
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
    );
    let m4_t = BuiltinFunctions::transpose_mat4(&m4);
    assert_eq!(
        m4_t,
        Matrix4::new(
            1.0, 5.0, 9.0, 13.0, 2.0, 6.0, 10.0, 14.0, 3.0, 7.0, 11.0, 15.0, 4.0, 8.0, 12.0,
            16.0
        )
    );
}

#[test]
fn test_czm_plane_distance() {
    let plane = Cartesian4::new(1.0, 0.0, 0.0, 0.0);
    let point = Cartesian3::new(1.0, 0.0, 0.0);
    let dist = BuiltinFunctions::plane_distance(plane, point);
    assert_eq!(dist, 1.0);

    let plane2 = Cartesian4::new(0.0, 1.0, 0.0, -5.0);
    let point2 = Cartesian3::new(0.0, 8.0, 0.0);
    assert_eq!(BuiltinFunctions::plane_distance(plane2, point2), 3.0);
}

#[test]
fn test_czm_line_distance() {
    let p1 = Cartesian2::new(0.0, 0.0);
    let p2 = Cartesian2::new(1.0, 0.0);
    let p = Cartesian2::new(0.5, 1.0);
    let d = BuiltinFunctions::line_distance(p1, p2, p);
    assert!((d - 1.0).abs() < 1e-6);
}

#[test]
fn test_czm_tangent_to_eye_space_matrix() {
    let tangent = Cartesian3::new(1.0, 0.0, 0.0);
    let bitangent = Cartesian3::new(0.0, 1.0, 0.0);
    let normal = Cartesian3::new(0.0, 0.0, 1.0);

    let mat = BuiltinFunctions::tangent_to_eye_space_matrix(normal, tangent, bitangent);
    assert_eq!(mat, Matrix3::IDENTITY);
}

#[test]
fn test_czm_translate_relative_to_eye() {
    let high = Cartesian3::new(6.0, 5.0, 4.0);
    let low = Cartesian3::new(0.0, 0.0, 0.0);
    let cam_high = Cartesian3::new(1.0, 2.0, 3.0);
    let cam_low = Cartesian3::new(0.0, 0.0, 0.0);

    let p = BuiltinFunctions::translate_relative_to_eye(high, low, cam_high, cam_low);
    assert_eq!(p, Cartesian4::new(5.0, 3.0, 1.0, 1.0));
}

#[test]
fn test_czm_point_along_ray() {
    let ray_x = Ray::new(Cartesian3::ZERO, Cartesian3::new(1.0, 0.0, 0.0));
    assert_eq!(
        BuiltinFunctions::point_along_ray(&ray_x, 0.0),
        Cartesian3::ZERO
    );
    assert_eq!(
        BuiltinFunctions::point_along_ray(&ray_x, 2.0),
        Cartesian3::new(2.0, 0.0, 0.0)
    );

    let ray_y = Ray::new(Cartesian3::ZERO, Cartesian3::new(0.0, 1.0, 0.0));
    assert_eq!(
        BuiltinFunctions::point_along_ray(&ray_y, -2.0),
        Cartesian3::new(0.0, -2.0, 0.0)
    );
}

#[test]
fn test_czm_oct_decode() {
    // Decodes [128.0, 128.0] to (0, 0, 1) approximately
    let v = BuiltinFunctions::oct_decode([128.0, 128.0], 255.0);
    assert!((v[0] - 0.0).abs() <= 0.01);
    assert!((v[1] - 0.0).abs() <= 0.01);
    assert!((v[2] - 1.0).abs() <= 0.01);

    // Float encoded
    let vf = BuiltinFunctions::oct_decode_float(32896.0);
    assert!((vf[0] - 0.0).abs() <= 0.01);
    assert!((vf[1] - 0.0).abs() <= 0.01);
    assert!((vf[2] - 1.0).abs() <= 0.01);
}

#[test]
fn test_czm_decompress_texture_coordinates() {
    let coords = BuiltinFunctions::decompress_texture_coordinates(8386559.0);
    assert!((coords[0] - 0.4998779).abs() <= 1e-4);
    assert!((coords[1] - 0.4998779).abs() <= 1e-4);
}

#[test]
fn test_czm_sign_not_zero() {
    assert_eq!(BuiltinFunctions::sign_not_zero_f32(0.0), 1.0);
    assert_eq!(BuiltinFunctions::sign_not_zero_f32(5.0), 1.0);
    assert_eq!(BuiltinFunctions::sign_not_zero_f32(-5.0), -1.0);

    let v2 = BuiltinFunctions::sign_not_zero_vec2([-1.0, 0.0]);
    assert_eq!(v2, [-1.0, 1.0]);

    let v3 = BuiltinFunctions::sign_not_zero_vec3([-1.0, 0.0, 1.0]);
    assert_eq!(v3, [-1.0, 1.0, 1.0]);

    let v4 = BuiltinFunctions::sign_not_zero_vec4([-1.0, 0.0, 1.0, -10.0]);
    assert_eq!(v4, [-1.0, 1.0, 1.0, -1.0]);
}

#[test]
fn test_czm_cosine_and_sine() {
    let pi_over_4 = std::f32::consts::FRAC_PI_4;
    let expected = std::f32::consts::FRAC_1_SQRT_2;
    let cs = BuiltinFunctions::cosine_and_sine(pi_over_4);
    assert!((cs[0] - expected).abs() <= 1e-4);
    assert!((cs[1] - expected).abs() <= 1e-4);
}

#[test]
fn test_czm_near_far_scalar() {
    let params = [10.0, 1.0, 20.0, 0.0];
    assert_eq!(BuiltinFunctions::near_far_scalar(params, 5.0 * 5.0), 1.0);
    assert_eq!(BuiltinFunctions::near_far_scalar(params, 10.0 * 10.0), 1.0);
    assert_eq!(BuiltinFunctions::near_far_scalar(params, 20.0 * 20.0), 0.0);
    assert_eq!(BuiltinFunctions::near_far_scalar(params, 50.0 * 50.0), 0.0);
}

#[test]
fn test_czm_unpack_float() {
    // Pack 1.0 into 4 bytes: IEEE-754 for 1.0f32 is 0x3F800000 -> little endian [0, 0, 128, 63]
    let packed = [0.0 / 255.0, 0.0 / 255.0, 128.0 / 255.0, 63.0 / 255.0];
    let unpacked = BuiltinFunctions::unpack_float(packed);
    assert!((unpacked - 1.0).abs() <= 1e-5);
}

#[test]
fn test_czm_branch_free_ternary() {
    assert_eq!(BuiltinFunctions::branch_free_ternary(true, 1.0, 0.0), 1.0);
    assert_eq!(BuiltinFunctions::branch_free_ternary(false, 1.0, 0.0), 0.0);
}

#[test]
fn test_czm_fast_approximate_atan() {
    let a0 = BuiltinFunctions::fast_approximate_atan(0.0);
    assert!(a0.abs() <= 1e-5);
    let a1 = BuiltinFunctions::fast_approximate_atan(1.0);
    assert!((a1 - std::f32::consts::FRAC_PI_4).abs() <= 1e-2);
}

#[test]
fn test_czm_antialias() {
    let c0 = [1.0, 0.0, 0.0, 1.0];
    let c1 = [0.0, 1.0, 0.0, 1.0];
    let mixed = BuiltinFunctions::antialias(c0, c1, c1, 0.5);
    assert_eq!(mixed, [0.5, 0.5, 0.0, 1.0]);
}
