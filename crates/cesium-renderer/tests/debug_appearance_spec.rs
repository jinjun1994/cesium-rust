use cesium_math::DeveloperError;
use cesium_renderer::{Appearance, DebugAppearance, DebugAppearanceOptions};

#[test]
fn test_constructor_throws_without_attribute_name() {
    let err = DebugAppearance::new(DebugAppearanceOptions::default())
        .expect_err("should throw without attribute_name");
    assert_eq!(err.name(), DeveloperError::NAME);
}

#[test]
fn test_default_construct_with_normal_attribute_name() {
    let a = DebugAppearance::new(DebugAppearanceOptions {
        attribute_name: Some("normal".to_string()),
        per_instance_attribute: Some(false),
        ..Default::default()
    })
    .expect("should construct DebugAppearance");

    assert!(a.vertex_shader_source.contains("normal"));
    assert!(a.vertex_shader_source.contains("v_normal"));
    assert!(a.fragment_shader_source.contains("v_normal"));

    assert!(a.material.is_none());
    assert_eq!(a.attribute_name, "normal");
    assert_eq!(a.glsl_datatype, "vec3");
    assert_eq!(a.render_state, Appearance::get_default_render_state(false, false, None));
    assert!(!a.translucent);
    assert!(!a.closed);
}

#[test]
fn test_default_construct_with_st_attribute_name() {
    let a = DebugAppearance::new(DebugAppearanceOptions {
        attribute_name: Some("st".to_string()),
        per_instance_attribute: Some(false),
        ..Default::default()
    })
    .expect("should construct DebugAppearance");

    assert!(a.vertex_shader_source.contains("st"));
    assert!(a.vertex_shader_source.contains("v_st"));
    assert!(a.fragment_shader_source.contains("v_st"));

    assert!(a.material.is_none());
    assert_eq!(a.attribute_name, "st");
    assert_eq!(a.glsl_datatype, "vec2");
    assert_eq!(a.render_state, Appearance::get_default_render_state(false, false, None));
    assert!(!a.translucent);
    assert!(!a.closed);
}

#[test]
fn test_debug_appearance_with_float_attribute_name() {
    let a = DebugAppearance::new(DebugAppearanceOptions {
        attribute_name: Some("rotation".to_string()),
        glsl_datatype: Some("float".to_string()),
        per_instance_attribute: Some(true),
        ..Default::default()
    })
    .expect("should construct DebugAppearance");

    assert!(a.vertex_shader_source.contains("rotation"));
    assert!(a.vertex_shader_source.contains("v_rotation"));
    assert!(a.fragment_shader_source.contains("v_rotation"));

    assert!(a.material.is_none());
    assert_eq!(a.attribute_name, "rotation");
    assert_eq!(a.glsl_datatype, "float");
    assert_eq!(a.render_state, Appearance::get_default_render_state(false, false, None));
    assert!(!a.translucent);
    assert!(!a.closed);
}

#[test]
fn test_debug_appearance_with_vec3_attribute_name() {
    let a = DebugAppearance::new(DebugAppearanceOptions {
        attribute_name: Some("str".to_string()),
        glsl_datatype: Some("vec3".to_string()),
        per_instance_attribute: Some(false),
        ..Default::default()
    })
    .expect("should construct DebugAppearance");

    assert!(a.vertex_shader_source.contains("str"));
    assert!(a.vertex_shader_source.contains("v_str"));
    assert!(a.fragment_shader_source.contains("v_str"));

    assert!(a.material.is_none());
    assert_eq!(a.attribute_name, "str");
    assert_eq!(a.glsl_datatype, "vec3");
    assert_eq!(a.render_state, Appearance::get_default_render_state(false, false, None));
    assert!(!a.translucent);
    assert!(!a.closed);
}

#[test]
fn test_constructor_throws_with_invalid_glsl_datatype() {
    let err = DebugAppearance::new(DebugAppearanceOptions {
        attribute_name: Some("invalid".to_string()),
        glsl_datatype: Some("mat4".to_string()),
        ..Default::default()
    })
    .expect_err("should throw with invalid glsl_datatype");

    assert_eq!(err.name(), DeveloperError::NAME);
}
