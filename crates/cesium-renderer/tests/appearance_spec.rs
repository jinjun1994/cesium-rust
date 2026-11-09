//! Direct port and verification against CesiumJS `Scene/AppearanceSpec.js` (Apache-2.0).

use cesium_renderer::{
    Appearance, AppearanceOptions, BlendingState, CullFace, DepthTestState, Material, RenderState,
};

#[test]
fn test_constructor() {
    let material = Material::from_type("Color").unwrap();
    let vs = "in vec3 position3DHigh;\nin vec3 position3DLow;\nin vec4 color;\nout vec4 v_color;\nvoid main() {\n    gl_Position = czm_modelViewProjectionRelativeToEye * czm_computePosition();\n    v_color = color;\n}\n";
    let fs = "in vec4 v_color;\nvoid main() {\n    out_FragColor = v_color;\n}\n";
    let render_state = RenderState {
        depth_test: DepthTestState {
            enabled: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let appearance = Appearance::new(AppearanceOptions {
        material: Some(material.clone()),
        vertex_shader_source: Some(vs.to_string()),
        fragment_shader_source: Some(fs.to_string()),
        render_state: Some(render_state.clone()),
        translucent: Some(false),
        closed: Some(true),
        ..Default::default()
    });

    assert_eq!(appearance.material, Some(material));
    assert_eq!(appearance.vertex_shader_source.as_deref(), Some(vs));
    assert_eq!(appearance.fragment_shader_source.as_deref(), Some(fs));
    assert_eq!(appearance.render_state, Some(render_state));
    assert!(!appearance.translucent);
    assert!(appearance.closed);
}

#[test]
fn test_get_fragment_shader_source() {
    let fs = "out vec4 v_color;\nvoid main() {\n    out_FragColor = v_color;\n}\n";
    let appearance = Appearance::new(AppearanceOptions {
        fragment_shader_source: Some(fs.to_string()),
        ..Default::default()
    });

    assert!(appearance.get_fragment_shader_source().contains(fs));
}

#[test]
fn test_get_fragment_shader_source_with_material() {
    let material = Material::from_type("Color").unwrap();
    let fs = "in vec4 v_color;\nvoid main() {\n    out_FragColor = v_color;\n}\n";
    let appearance = Appearance::new(AppearanceOptions {
        material: Some(material.clone()),
        fragment_shader_source: Some(fs.to_string()),
        ..Default::default()
    });

    let fragment_source = appearance.get_fragment_shader_source();
    assert!(fragment_source.contains(&material.shader_source));
    assert!(fragment_source.contains(fs));
}

#[test]
fn test_get_default_render_state() {
    let render_state = Appearance::get_default_render_state(true, true, None);

    assert!(render_state.depth_test.enabled);
    assert!(!render_state.depth_mask);
    assert_eq!(render_state.blending, BlendingState::ALPHA_BLEND);
    assert!(render_state.cull.enabled);
    assert_eq!(render_state.cull.face, CullFace::Back);
}

#[test]
fn test_is_translucent() {
    let mut appearance = Appearance::new(AppearanceOptions {
        translucent: Some(false),
        ..Default::default()
    });

    assert!(!appearance.is_translucent());
    appearance.translucent = true;
    assert!(appearance.is_translucent());

    let mut material = Material::from_type("Color").unwrap();
    material.translucent = false;
    appearance.material = Some(material);
    assert!(!appearance.is_translucent());

    appearance.material.as_mut().unwrap().translucent = true;
    assert!(appearance.is_translucent());
}

#[test]
fn test_get_render_state() {
    let mut appearance = Appearance::new(AppearanceOptions {
        translucent: Some(false),
        closed: Some(true),
        render_state: Some(Appearance::get_default_render_state(false, true, None)),
        ..Default::default()
    });

    let mut rs = appearance.get_render_state();
    assert!(rs.depth_mask);
    assert_eq!(rs.blending, BlendingState::DISABLED);

    appearance.translucent = true;
    rs = appearance.get_render_state();
    assert!(!rs.depth_mask);
    assert_eq!(rs.blending, BlendingState::ALPHA_BLEND);
}
