use cesium_math::VertexFormat;
use cesium_renderer::appearance::Appearance;
use cesium_renderer::material::Material;
use cesium_renderer::polyline_material_appearance::{
    PolylineMaterialAppearance, PolylineMaterialAppearanceOptions,
};

#[test]
fn test_default_constructor() {
    let a = PolylineMaterialAppearance::default();

    assert_eq!(a.material.material_type, Material::COLOR_TYPE);
    assert!(!a.vertex_shader_source().is_empty());
    assert!(!a.fragment_shader_source.is_empty());
    assert_eq!(
        a.render_state,
        Appearance::get_default_render_state(true, false, None)
    );
    assert_eq!(a.vertex_format, PolylineMaterialAppearance::VERTEX_FORMAT);
    assert_eq!(a.vertex_format, VertexFormat::POSITION_AND_ST);
    assert!(a.translucent);
    assert!(!a.closed);
    assert!(a.is_translucent());
    assert_eq!(a.get_fragment_shader_source(), a.fragment_shader_source);
}

#[test]
fn test_custom_material_and_options() {
    let outline_mat = Material::from_type(Material::POLYLINE_OUTLINE_TYPE).unwrap();
    let a = PolylineMaterialAppearance::new(PolylineMaterialAppearanceOptions {
        material: Some(outline_mat.clone()),
        translucent: Some(false),
        ..Default::default()
    });

    assert_eq!(a.material.material_type, Material::POLYLINE_OUTLINE_TYPE);
    assert!(!a.translucent);
    assert_eq!(
        a.render_state,
        Appearance::get_default_render_state(false, false, None)
    );
}

#[test]
fn test_dashed_material_vertex_shader_macro() {
    let mut dashed_mat = Material::from_type(Material::POLYLINE_DASH_TYPE).unwrap();
    dashed_mat.shader_source = "in float v_polylineAngle;\nczm_material czm_getMaterial(czm_materialInput mi) { return czm_getDefaultMaterial(mi); }".to_string();

    let a = PolylineMaterialAppearance::new(PolylineMaterialAppearanceOptions {
        material: Some(dashed_mat),
        ..Default::default()
    });

    assert!(a.vertex_shader_source().contains("#define POLYLINE_DASH"));
}
