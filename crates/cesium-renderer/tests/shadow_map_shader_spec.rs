use cesium_renderer::shader_source::ShaderSource;
use cesium_renderer::shadow_map_shader::ShadowMapShader;

fn make_shader_source(source: &str, defines: Vec<&str>) -> ShaderSource {
    ShaderSource {
        defines: defines.into_iter().map(String::from).collect(),
        sources: vec![source.to_string()],
        pick_color_qualifier: None,
        include_built_ins: true,
    }
}

#[test]
fn test_has_clipping_for_shadow_cast() {
    let fs_planes = make_shader_source(
        "void main() { out_FragColor = vec4(1.0); }",
        vec!["HAS_CLIPPING_PLANES"],
    );
    assert!(ShadowMapShader::has_clipping_for_shadow_cast(&fs_planes));

    let fs_polygons = make_shader_source(
        "void main() { out_FragColor = vec4(1.0); }",
        vec!["ENABLE_CLIPPING_POLYGONS"],
    );
    assert!(ShadowMapShader::has_clipping_for_shadow_cast(&fs_polygons));

    let fs_other = make_shader_source(
        "void main() { out_FragColor = vec4(1.0); }",
        vec!["SOME_OTHER_DEFINE"],
    );
    assert!(!ShadowMapShader::has_clipping_for_shadow_cast(&fs_other));
}

#[test]
fn test_get_shadow_cast_shader_keyword() {
    let keyword1 =
        ShadowMapShader::get_shadow_cast_shader_keyword(false, false, true, true, false);
    let keyword2 =
        ShadowMapShader::get_shadow_cast_shader_keyword(false, false, true, true, true);
    assert_ne!(keyword1, keyword2);
    assert_eq!(keyword1, "castShadow false false true true false");
    assert_eq!(keyword2, "castShadow false false true true true");
}

#[test]
fn test_create_shadow_cast_fragment_shader_opaque_without_clipping() {
    let fs = make_shader_source("void main() { out_FragColor = vec4(1.0); }", vec![]);
    let result =
        ShadowMapShader::create_shadow_cast_fragment_shader(&fs, false, true, true, false);
    let new_main = result.sources.last().expect("must contain new main source");
    assert!(!new_main.contains("czm_shadow_cast_main()"));
    assert!(result.defines.contains(&"SHADOW_MAP".to_string()));
}

#[test]
fn test_create_shadow_cast_fragment_shader_opaque_with_clipping() {
    let fs = make_shader_source(
        "void main() { if (clipped) { discard; } out_FragColor = vec4(1.0); }",
        vec![],
    );
    let result =
        ShadowMapShader::create_shadow_cast_fragment_shader(&fs, false, true, true, true);
    let new_main = result.sources.last().expect("must contain new main source");
    assert!(new_main.contains("czm_shadow_cast_main()"));
}

#[test]
fn test_create_shadow_cast_fragment_shader_translucent() {
    let fs = make_shader_source(
        "void main() { out_FragColor = vec4(1.0, 1.0, 1.0, 0.5); }",
        vec![],
    );
    let result =
        ShadowMapShader::create_shadow_cast_fragment_shader(&fs, false, true, false, false);
    let new_main = result.sources.last().expect("must contain new main source");
    assert!(new_main.contains("czm_shadow_cast_main()"));
    assert!(new_main.contains("if (out_FragColor.a == 0.0)"));
}

#[test]
fn test_create_shadow_cast_fragment_shader_opaque_with_clipping_and_point_light() {
    let fs = make_shader_source(
        "void main() { if (clipped) { discard; } out_FragColor = vec4(1.0); }",
        vec![],
    );
    let result =
        ShadowMapShader::create_shadow_cast_fragment_shader(&fs, true, false, true, true);
    let new_main = result.sources.last().expect("must contain new main source");
    assert!(new_main.contains("czm_shadow_cast_main()"));
    assert!(new_main.contains("shadowMap_lightPositionEC"));
    assert!(new_main.contains("czm_packDepth(distance)"));
}

#[test]
fn test_create_shadow_cast_vertex_shader() {
    let vs = make_shader_source(
        "void main() { gl_Position = czm_modelViewProjection * vec4(position, 1.0); }",
        vec![],
    );
    let result = ShadowMapShader::create_shadow_cast_vertex_shader(&vs, true, true);
    assert!(result.defines.contains(&"SHADOW_MAP".to_string()));
    assert!(result.defines.contains(&"GENERATE_POSITION".to_string()));
    let new_vs = result.sources.last().expect("must append shadow_vs");
    assert!(new_vs.contains("czm_shadow_cast_main()"));
    assert!(new_vs.contains("v_positionEC = (czm_inverseProjection * gl_Position).xyz;"));
}

#[test]
fn test_get_shadow_receive_shader_keyword() {
    let keyword = ShadowMapShader::get_shadow_receive_shader_keyword(
        true, false, false, false, true, false, true, true, false, false,
    );
    assert_eq!(
        keyword,
        "receiveShadow truefalsefalsefalsetruefalsetruetruefalsefalse"
    );
}
