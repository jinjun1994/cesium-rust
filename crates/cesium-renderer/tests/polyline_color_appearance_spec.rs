use cesium_math::VertexFormat;
use cesium_renderer::appearance::Appearance;
use cesium_renderer::polyline_color_appearance::{
    PolylineColorAppearance, PolylineColorAppearanceOptions,
};

#[test]
fn test_default_constructor() {
    let a = PolylineColorAppearance::default();

    assert!(a.material.is_none());
    assert!(!a.vertex_shader_source.is_empty());
    assert!(!a.fragment_shader_source.is_empty());
    assert_eq!(
        a.render_state,
        Appearance::get_default_render_state(true, false, None)
    );
    assert_eq!(a.vertex_format, PolylineColorAppearance::VERTEX_FORMAT);
    assert_eq!(a.vertex_format, VertexFormat::POSITION_ONLY);
    assert!(a.translucent);
    assert!(!a.closed);
    assert!(a.is_translucent());
    assert_eq!(a.get_fragment_shader_source(), a.fragment_shader_source);
}

#[test]
fn test_custom_options() {
    let a = PolylineColorAppearance::new(PolylineColorAppearanceOptions {
        translucent: Some(false),
        ..Default::default()
    });

    assert!(!a.translucent);
    assert!(!a.is_translucent());
    assert_eq!(
        a.render_state,
        Appearance::get_default_render_state(false, false, None)
    );
}
