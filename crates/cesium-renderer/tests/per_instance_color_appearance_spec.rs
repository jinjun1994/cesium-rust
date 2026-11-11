//! PerInstanceColorAppearanceSpec matching CesiumJS `Specs/Scene/PerInstanceColorAppearanceSpec.js` (Apache-2.0).

use cesium_math::VertexFormat;
use cesium_renderer::appearance::Appearance;
use cesium_renderer::per_instance_color_appearance::{
    PerInstanceColorAppearance, PerInstanceColorAppearanceOptions,
};
use cesium_renderer::render_state::{BlendingState, CullFace};

#[test]
fn test_constructor() {
    let a = PerInstanceColorAppearance::default();

    assert!(a.material.is_none());
    assert!(!a.vertex_shader_source.is_empty());
    assert!(!a.fragment_shader_source.is_empty());
    assert_eq!(
        a.render_state,
        Appearance::get_default_render_state(true, false, None)
    );
    assert_eq!(a.vertex_format, PerInstanceColorAppearance::VERTEX_FORMAT);
    assert!(!a.flat);
    assert!(a.face_forward);
    assert!(a.translucent);
    assert!(!a.closed);
    assert!(a.is_translucent());
}

#[test]
fn test_renders_default_lighting() {
    let a = PerInstanceColorAppearance::new(PerInstanceColorAppearanceOptions {
        flat: Some(false),
        translucent: Some(true),
        closed: Some(false),
        ..Default::default()
    });

    assert_eq!(a.vertex_format, VertexFormat::POSITION_AND_NORMAL);
    assert!(a.is_translucent());

    let fs = a.get_fragment_shader_source();
    assert!(fs.contains("#define FACE_FORWARD"));
    assert!(fs.contains("czm_phong"));
    assert!(fs.contains("czm_gammaCorrect"));

    let rs = a.get_render_state();
    assert_eq!(rs.blending, BlendingState::ALPHA_BLEND);
    assert!(!rs.depth_mask);
}

#[test]
fn test_renders_flat() {
    let a = PerInstanceColorAppearance::new(PerInstanceColorAppearanceOptions {
        flat: Some(true),
        translucent: Some(false),
        closed: Some(true),
        ..Default::default()
    });

    assert_eq!(
        a.vertex_format,
        PerInstanceColorAppearance::FLAT_VERTEX_FORMAT
    );
    assert_eq!(a.vertex_format, VertexFormat::POSITION_ONLY);
    assert!(a.flat);
    assert!(!a.translucent);
    assert!(a.closed);
    assert!(!a.face_forward);
    assert!(!a.is_translucent());

    let fs = a.get_fragment_shader_source();
    assert!(fs.contains("#define FLAT"));
    assert!(!fs.contains("#define FACE_FORWARD"));
    assert!(fs.contains("czm_gammaCorrect"));

    let rs = a.get_render_state();
    assert_eq!(rs.blending, BlendingState::DISABLED);
    assert!(rs.depth_mask);
    assert!(rs.cull.enabled);
    assert_eq!(rs.cull.face, CullFace::Back);
}
