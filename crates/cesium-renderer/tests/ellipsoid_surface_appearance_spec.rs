use cesium_renderer::{
    Appearance, EllipsoidSurfaceAppearance, EllipsoidSurfaceAppearanceOptions, Material,
};

#[test]
fn test_constructor() {
    let a = EllipsoidSurfaceAppearance::default();

    assert_eq!(a.material.material_type, Material::COLOR_TYPE);
    assert!(!a.vertex_shader_source.is_empty());
    assert!(!a.fragment_shader_source.is_empty());
    assert_eq!(
        a.render_state,
        Appearance::get_default_render_state(true, true, None)
    );
    assert_eq!(a.vertex_format(), EllipsoidSurfaceAppearance::VERTEX_FORMAT);
    assert!(!a.flat);
    assert!(!a.face_forward);
    assert!(a.translucent);
    assert!(!a.above_ground);
    assert!(!a.closed);
}

#[test]
fn test_constructor_with_options() {
    let a = EllipsoidSurfaceAppearance::new(EllipsoidSurfaceAppearanceOptions {
        flat: Some(true),
        above_ground: Some(true),
        translucent: Some(false),
        ..Default::default()
    });

    assert!(a.flat);
    assert!(a.above_ground);
    assert!(a.face_forward); // defaults to above_ground when None
    assert!(!a.translucent);
    assert_eq!(
        a.render_state,
        Appearance::get_default_render_state(false, false, None)
    );
}
