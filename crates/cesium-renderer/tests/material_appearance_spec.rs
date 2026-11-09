use cesium_math::VertexFormat;
use cesium_renderer::{
    Appearance, BlendingState, CullFace, Material, MaterialAppearance, MaterialAppearanceOptions,
    MaterialSupport,
};

#[test]
fn test_constructor() {
    let a = MaterialAppearance::default();

    assert_eq!(a.material_support, MaterialSupport::Textured);
    assert_eq!(a.material.material_type, Material::COLOR_TYPE);
    assert_eq!(
        a.vertex_shader_source,
        MaterialSupport::Textured.vertex_shader_source()
    );
    assert_eq!(
        a.fragment_shader_source,
        MaterialSupport::Textured.fragment_shader_source()
    );
    assert_eq!(
        a.render_state,
        Appearance::get_default_render_state(true, false, None)
    );
    assert_eq!(a.vertex_format, MaterialSupport::Textured.vertex_format());
    assert!(!a.flat);
    assert!(a.face_forward);
    assert!(a.translucent);
    assert!(!a.closed);
}

#[test]
fn test_renders_basic() {
    let a = MaterialAppearance::new(MaterialAppearanceOptions {
        material_support: Some(MaterialSupport::Basic),
        translucent: Some(false),
        closed: Some(true),
        material: Some(Material::from_type(Material::DOT_TYPE).unwrap()),
        ..Default::default()
    });

    assert_eq!(a.material_support, MaterialSupport::Basic);
    assert_eq!(a.vertex_format, VertexFormat::POSITION_AND_NORMAL);
    assert!(!a.translucent);
    assert!(a.closed);
    assert!(!a.face_forward); // !closed -> false
    assert_eq!(a.material.material_type, Material::DOT_TYPE);

    let rs = a.render_state;
    assert!(rs.cull.enabled);
    assert_eq!(rs.cull.face, CullFace::Back);
}

#[test]
fn test_renders_textured() {
    let a = MaterialAppearance::new(MaterialAppearanceOptions {
        material_support: Some(MaterialSupport::Textured),
        translucent: Some(false),
        closed: Some(true),
        material: Some(Material::from_type(Material::IMAGE_TYPE).unwrap()),
        ..Default::default()
    });

    assert_eq!(a.material_support, MaterialSupport::Textured);
    assert_eq!(a.vertex_format, VertexFormat::POSITION_NORMAL_AND_ST);
    assert!(!a.translucent);
    assert!(a.closed);
    assert_eq!(a.material.material_type, Material::IMAGE_TYPE);

    let rs = a.render_state;
    assert!(rs.cull.enabled);
    assert_eq!(rs.cull.face, CullFace::Back);
}

#[test]
fn test_renders_all() {
    let a = MaterialAppearance::new(MaterialAppearanceOptions {
        material_support: Some(MaterialSupport::All),
        translucent: Some(false),
        closed: Some(true),
        material: Some(Material::from_type(Material::NORMAL_MAP_TYPE).unwrap()),
        ..Default::default()
    });

    assert_eq!(a.material_support, MaterialSupport::All);
    assert_eq!(a.vertex_format, VertexFormat::ALL);
    assert!(!a.translucent);
    assert!(a.closed);
    assert_eq!(a.material.material_type, Material::NORMAL_MAP_TYPE);

    let rs = a.render_state;
    assert!(rs.cull.enabled);
    assert_eq!(rs.cull.face, CullFace::Back);
}

#[test]
fn test_get_fragment_shader_source() {
    let a = MaterialAppearance::new(MaterialAppearanceOptions {
        flat: Some(true),
        face_forward: Some(true),
        ..Default::default()
    });

    let fs = a.get_fragment_shader_source();
    assert!(fs.contains("#define FLAT"));
    assert!(fs.contains("#define FACE_FORWARD"));
    assert!(fs.contains(&a.material.shader_source));
    assert!(fs.contains(&a.fragment_shader_source));
}

#[test]
fn test_translucency_and_render_state() {
    let a = MaterialAppearance::new(MaterialAppearanceOptions {
        material: Some(Material::from_type(Material::NORMAL_MAP_TYPE).unwrap()), // non-translucent
        ..Default::default()
    });

    assert!(!a.is_translucent());
    let rs = a.get_render_state();
    assert!(rs.depth_mask);
    assert_eq!(rs.blending, BlendingState::DISABLED);
}
