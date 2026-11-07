use cesium_math::{Cartesian2, Cartesian3, Cartesian4, Color, Matrix2, Matrix3, Matrix4};
use cesium_renderer::{Uniform, UniformMap, UniformType, UniformValue};

#[test]
fn test_sets_float_uniform() {
    let mut map = UniformMap::new();
    map.set_getter("u", || UniformValue::Float(1.0));

    let mut uniform = Uniform::new("u", UniformType::Float, 0);
    let val = map.evaluate("u").unwrap();
    assert!(uniform.set(val).is_ok());
    assert_eq!(uniform.value, Some(UniformValue::Float(1.0)));
}

#[test]
fn test_sets_vec2_uniform() {
    let mut map = UniformMap::new();
    map.set_getter("u", || Cartesian2::new(0.25, 0.5).into());

    let mut uniform = Uniform::new("u", UniformType::FloatVec2, 0);
    let val = map.evaluate("u").unwrap();
    assert!(uniform.set(val).is_ok());
    assert_eq!(uniform.value, Some(UniformValue::Vec2([0.25, 0.5])));
}

#[test]
fn test_sets_vec3_uniform() {
    let mut map = UniformMap::new();
    map.set_getter("u_cartesian", || Cartesian3::new(0.25, 0.5, 0.75).into());
    map.set_getter("u_color", || {
        let c = Color::new(0.25, 0.5, 0.75, 1.0);
        UniformValue::Vec3([c.red, c.green, c.blue])
    });

    let mut u1 = Uniform::new("u_cartesian", UniformType::FloatVec3, 0);
    assert!(u1.set(map.evaluate("u_cartesian").unwrap()).is_ok());
    assert_eq!(u1.value, Some(UniformValue::Vec3([0.25, 0.5, 0.75])));

    let mut u2 = Uniform::new("u_color", UniformType::FloatVec3, 1);
    assert!(u2.set(map.evaluate("u_color").unwrap()).is_ok());
    assert_eq!(u2.value, Some(UniformValue::Vec3([0.25, 0.5, 0.75])));
}

#[test]
fn test_sets_vec4_uniform() {
    let mut map = UniformMap::new();
    map.set_getter("u_cartesian", || Cartesian4::new(0.25, 0.5, 0.75, 1.0).into());
    map.set_getter("u_color", || Color::new(0.25, 0.5, 0.75, 1.0).into());

    let mut u1 = Uniform::new("u_cartesian", UniformType::FloatVec4, 0);
    assert!(u1.set(map.evaluate("u_cartesian").unwrap()).is_ok());
    assert_eq!(u1.value, Some(UniformValue::Vec4([0.25, 0.5, 0.75, 1.0])));

    let mut u2 = Uniform::new("u_color", UniformType::FloatVec4, 1);
    assert!(u2.set(map.evaluate("u_color").unwrap()).is_ok());
    assert_eq!(u2.value, Some(UniformValue::Vec4([0.25, 0.5, 0.75, 1.0])));
}

#[test]
fn test_sets_int_uniforms() {
    let mut map = UniformMap::new();
    map.set_value("u_int", 1);
    map.set_value("u_ivec2", [1, 2]);
    map.set_value("u_ivec3", [1, 2, 3]);
    map.set_value("u_ivec4", [1, 2, 3, 4]);

    let mut u_int = Uniform::new("u_int", UniformType::Int, 0);
    assert!(u_int.set(map.evaluate("u_int").unwrap()).is_ok());
    assert_eq!(u_int.value, Some(UniformValue::Int(1)));

    let mut u_ivec2 = Uniform::new("u_ivec2", UniformType::IntVec2, 1);
    assert!(u_ivec2.set(map.evaluate("u_ivec2").unwrap()).is_ok());
    assert_eq!(u_ivec2.value, Some(UniformValue::IVec2([1, 2])));

    let mut u_ivec3 = Uniform::new("u_ivec3", UniformType::IntVec3, 2);
    assert!(u_ivec3.set(map.evaluate("u_ivec3").unwrap()).is_ok());
    assert_eq!(u_ivec3.value, Some(UniformValue::IVec3([1, 2, 3])));

    let mut u_ivec4 = Uniform::new("u_ivec4", UniformType::IntVec4, 3);
    assert!(u_ivec4.set(map.evaluate("u_ivec4").unwrap()).is_ok());
    assert_eq!(u_ivec4.value, Some(UniformValue::IVec4([1, 2, 3, 4])));
}

#[test]
fn test_sets_bool_uniforms() {
    let mut map = UniformMap::new();
    map.set_value("u_bool", true);
    map.set_value("u_bvec2", [true, false]);
    map.set_value("u_bvec3", [true, false, true]);
    map.set_value("u_bvec4", [true, false, true, false]);

    let mut u_b = Uniform::new("u_bool", UniformType::Bool, 0);
    assert!(u_b.set(map.evaluate("u_bool").unwrap()).is_ok());
    assert_eq!(u_b.value, Some(UniformValue::Bool(true)));

    let mut u_bv2 = Uniform::new("u_bvec2", UniformType::BoolVec2, 1);
    assert!(u_bv2.set(map.evaluate("u_bvec2").unwrap()).is_ok());
    assert_eq!(u_bv2.value, Some(UniformValue::BVec2([true, false])));

    let mut u_bv3 = Uniform::new("u_bvec3", UniformType::BoolVec3, 2);
    assert!(u_bv3.set(map.evaluate("u_bvec3").unwrap()).is_ok());
    assert_eq!(u_bv3.value, Some(UniformValue::BVec3([true, false, true])));

    let mut u_bv4 = Uniform::new("u_bvec4", UniformType::BoolVec4, 3);
    assert!(u_bv4.set(map.evaluate("u_bvec4").unwrap()).is_ok());
    assert_eq!(u_bv4.value, Some(UniformValue::BVec4([true, false, true, false])));
}

#[test]
fn test_sets_matrix_uniforms() {
    let mut map = UniformMap::new();
    map.set_value("u_m2", Matrix2::new(1.0, 2.0, 3.0, 4.0));
    map.set_value(
        "u_m3",
        Matrix3::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0),
    );
    map.set_value(
        "u_m4",
        Matrix4::new(
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
            16.0,
        ),
    );

    let mut u_m2 = Uniform::new("u_m2", UniformType::FloatMat2, 0);
    assert!(u_m2.set(map.evaluate("u_m2").unwrap()).is_ok());
    assert_eq!(
        u_m2.value,
        Some(UniformValue::Mat2([1.0, 3.0, 2.0, 4.0]))
    );

    let mut u_m3 = Uniform::new("u_m3", UniformType::FloatMat3, 1);
    assert!(u_m3.set(map.evaluate("u_m3").unwrap()).is_ok());
    assert_eq!(
        u_m3.value,
        Some(UniformValue::Mat3([
            1.0, 4.0, 7.0, 2.0, 5.0, 8.0, 3.0, 6.0, 9.0
        ]))
    );

    let mut u_m4 = Uniform::new("u_m4", UniformType::FloatMat4, 2);
    assert!(u_m4.set(map.evaluate("u_m4").unwrap()).is_ok());
    assert_eq!(
        u_m4.value,
        Some(UniformValue::Mat4([
            1.0, 5.0, 9.0, 13.0, 2.0, 6.0, 10.0, 14.0, 3.0, 7.0, 11.0, 15.0, 4.0, 8.0, 12.0,
            16.0
        ]))
    );
}

#[test]
fn test_sets_struct_uniform() {
    let mut map = UniformMap::new();
    map.set_value("u.f", 2.5);
    map.set_value("u.v", Cartesian4::new(0.25, 0.5, 0.75, 1.0));

    let mut u_f = Uniform::new("u.f", UniformType::Float, 0);
    let mut u_v = Uniform::new("u.v", UniformType::FloatVec4, 1);

    assert!(u_f.set(map.evaluate("u.f").unwrap()).is_ok());
    assert!(u_v.set(map.evaluate("u.v").unwrap()).is_ok());

    assert_eq!(u_f.value, Some(UniformValue::Float(2.5)));
    assert_eq!(
        u_v.value,
        Some(UniformValue::Vec4([0.25, 0.5, 0.75, 1.0]))
    );
}

#[test]
fn test_sets_uniform_arrays() {
    let mut map = UniformMap::new();
    map.set_value("u_floats", UniformValue::FloatArray(vec![0.25, 0.5]));
    map.set_value(
        "u_vec2s",
        UniformValue::Vec2Array(vec![[0.25, 0.5], [1.25, 1.5]]),
    );
    map.set_value(
        "u_vec3s",
        UniformValue::Vec3Array(vec![[0.25, 0.5, 0.75], [1.25, 1.5, 1.75]]),
    );
    map.set_value(
        "u_vec4s",
        UniformValue::Vec4Array(vec![[0.25, 0.5, 0.75, 1.0], [1.25, 1.5, 1.75, 2.0]]),
    );
    map.set_value("u_ints", UniformValue::IntArray(vec![1, 2]));
    map.set_value("u_bools", UniformValue::BoolArray(vec![true, false]));
    map.set_value(
        "u_mat2s",
        UniformValue::Mat2Array(vec![[1.0, 2.0, 3.0, 4.0], [5.0, 6.0, 7.0, 8.0]]),
    );

    let mut u_f = Uniform::new("u_floats", UniformType::Float, 0);
    assert!(u_f.set(map.evaluate("u_floats").unwrap()).is_ok());

    let mut u_v2 = Uniform::new("u_vec2s", UniformType::FloatVec2, 1);
    assert!(u_v2.set(map.evaluate("u_vec2s").unwrap()).is_ok());

    let mut u_v3 = Uniform::new("u_vec3s", UniformType::FloatVec3, 2);
    assert!(u_v3.set(map.evaluate("u_vec3s").unwrap()).is_ok());

    let mut u_v4 = Uniform::new("u_vec4s", UniformType::FloatVec4, 3);
    assert!(u_v4.set(map.evaluate("u_vec4s").unwrap()).is_ok());

    let mut u_i = Uniform::new("u_ints", UniformType::Int, 4);
    assert!(u_i.set(map.evaluate("u_ints").unwrap()).is_ok());

    let mut u_b = Uniform::new("u_bools", UniformType::Bool, 5);
    assert!(u_b.set(map.evaluate("u_bools").unwrap()).is_ok());

    let mut u_m2 = Uniform::new("u_mat2s", UniformType::FloatMat2, 6);
    assert!(u_m2.set(map.evaluate("u_mat2s").unwrap()).is_ok());
}

#[test]
fn test_throws_on_type_mismatch() {
    let mut u = Uniform::new("u_float", UniformType::Float, 0);
    // Setting Mat4 to Float uniform throws DeveloperError
    let err = u.set(UniformValue::Mat4([0.0; 16]));
    assert!(err.is_err());
}
