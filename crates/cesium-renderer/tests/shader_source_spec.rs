use cesium_renderer::{ShaderContextInfo, ShaderSource, ShaderSourceOptions};

#[test]
fn test_combines_defines() {
    let source = ShaderSource::new(ShaderSourceOptions {
        defines: vec!["A".to_string(), "B".to_string(), String::new()],
        ..Default::default()
    })
    .unwrap();

    let context = ShaderContextInfo::default();
    let shader_text = source.create_combined_vertex_shader(&context);
    assert!(shader_text.contains("#define A"));
    assert!(shader_text.contains("#define B"));
    let count = shader_text.matches("#define").count();
    assert_eq!(count, 2);
}

#[test]
fn test_combines_sources() {
    let source = ShaderSource::new(ShaderSourceOptions {
        sources: vec!["void func() {}".to_string(), "void main() {}".to_string()],
        ..Default::default()
    })
    .unwrap();

    let context = ShaderContextInfo::default();
    let shader_text = source.create_combined_vertex_shader(&context);
    assert!(shader_text.contains("#line 0\nvoid func() {}"));
    assert!(shader_text.contains("#line 0\nvoid main() {}"));
}

#[test]
fn test_combines_defines_and_sources() {
    let source = ShaderSource::new(ShaderSourceOptions {
        defines: vec!["A".to_string(), "B".to_string(), String::new()],
        sources: vec!["void func() {}".to_string(), "void main() {}".to_string()],
        ..Default::default()
    })
    .unwrap();

    let context = ShaderContextInfo::default();
    let shader_text = source.create_combined_vertex_shader(&context);
    assert!(shader_text.contains("#define A"));
    assert!(shader_text.contains("#define B"));
    let count = shader_text.matches("#define").count();
    assert_eq!(count, 2);
    assert!(shader_text.contains("#line 0\nvoid func() {}"));
    assert!(shader_text.contains("#line 0\nvoid main() {}"));
}

#[test]
fn test_creates_pick_shader_with_uniform() {
    let source = ShaderSource::new(ShaderSourceOptions {
        sources: vec!["void main() { out_FragColor = vec4(1.0); }".to_string()],
        pick_color_qualifier: Some("uniform".to_string()),
        ..Default::default()
    })
    .unwrap();

    let context = ShaderContextInfo::default();
    let shader_text = source.create_combined_fragment_shader(&context);
    assert!(shader_text.contains("uniform vec4 czm_pickColor;"));
    assert!(shader_text.contains("out_FragColor = czm_pickColor;"));
}

#[test]
fn test_creates_pick_shader_with_varying() {
    let source = ShaderSource::new(ShaderSourceOptions {
        sources: vec!["void main() { out_FragColor = vec4(1.0); }".to_string()],
        pick_color_qualifier: Some("in".to_string()),
        ..Default::default()
    })
    .unwrap();

    let context = ShaderContextInfo::default();
    let shader_text = source.create_combined_fragment_shader(&context);
    assert!(shader_text.contains("in vec4 czm_pickColor;"));
    assert!(shader_text.contains("out_FragColor = czm_pickColor;"));
}

#[test]
fn test_throws_with_invalid_qualifier() {
    let res = ShaderSource::new(ShaderSourceOptions {
        pick_color_qualifier: Some("const".to_string()),
        ..Default::default()
    });
    assert!(res.is_err());
}

#[test]
fn test_combines_version_to_shader() {
    let source = ShaderSource::new(ShaderSourceOptions {
        sources: vec!["#version 300 es\nvoid main() {out_FragColor = vec4(1.0); }".to_string()],
        ..Default::default()
    })
    .unwrap();

    let context = ShaderContextInfo::default();
    let shader_text = source.create_combined_vertex_shader(&context);
    assert!(shader_text.starts_with("#version 300 es\n"));
}

#[test]
fn test_clones() {
    let source = ShaderSource::new(ShaderSourceOptions {
        defines: vec!["A".to_string()],
        sources: vec!["void main() { out_FragColor = vec4(1.0); }".to_string()],
        pick_color_qualifier: Some("in".to_string()),
        include_built_ins: Some(false),
    })
    .unwrap();

    let cloned = source.clone();
    assert_eq!(cloned.defines.len(), 1);
    assert_eq!(cloned.defines[0], source.defines[0]);
    assert_eq!(cloned.sources.len(), 1);
    assert_eq!(cloned.sources[0], source.sources[0]);
    assert_eq!(cloned.pick_color_qualifier, source.pick_color_qualifier);
    assert_eq!(cloned.include_built_ins, source.include_built_ins);
}

#[test]
fn test_creates_cache_key_for_empty_shader() {
    let source = ShaderSource::default();
    assert_eq!(source.get_cache_key(), ":undefined:true:");
}

#[test]
fn test_creates_cache_key() {
    let source = ShaderSource::new(ShaderSourceOptions {
        defines: vec!["A".to_string(), "B".to_string(), "C".to_string()],
        sources: vec!["void main() { out_FragColor = vec4(1.0); }".to_string()],
        pick_color_qualifier: Some("in".to_string()),
        include_built_ins: Some(false),
    })
    .unwrap();

    assert_eq!(
        source.get_cache_key(),
        "A,B,C:in:false:void main() { out_FragColor = vec4(1.0); }"
    );
}

#[test]
fn test_uses_sorted_list_of_defines_in_cache_key() {
    let source1 = ShaderSource::new(ShaderSourceOptions {
        defines: vec!["A".to_string(), "B".to_string(), "C".to_string()],
        ..Default::default()
    })
    .unwrap();

    let source2 = ShaderSource::new(ShaderSourceOptions {
        defines: vec!["B".to_string(), "C".to_string(), "A".to_string()],
        ..Default::default()
    })
    .unwrap();

    let key1 = source1.get_cache_key();
    assert_eq!(key1, source2.get_cache_key());
    assert_eq!(key1, "A,B,C:undefined:true:");
}

#[test]
fn test_cache_key_includes_all_sources() {
    let source = ShaderSource::new(ShaderSourceOptions {
        sources: vec![
            "vec4 getColor() { return vec4(1.0, 0.0, 0.0, 1.0); }".to_string(),
            "void main() { out_FragColor = getColor(); }".to_string(),
        ],
        ..Default::default()
    })
    .unwrap();

    assert_eq!(
        source.get_cache_key(),
        ":undefined:true:vec4 getColor() { return vec4(1.0, 0.0, 0.0, 1.0); }\nvoid main() { out_FragColor = getColor(); }"
    );
}

#[test]
fn test_adds_layout_declaration_for_out_frag_color_if_it_does_not_already_exist() {
    let source = ShaderSource::new(ShaderSourceOptions {
        defines: vec!["A".to_string()],
        sources: vec!["void main() { out_FragColor = vec4(1.0); }".to_string()],
        pick_color_qualifier: Some("in".to_string()),
        include_built_ins: Some(false),
    })
    .unwrap();

    let context = ShaderContextInfo::default();
    let shader_text = source.create_combined_fragment_shader(&context);
    let count = shader_text
        .matches("layout(location = 0) out vec4 out_FragColor;")
        .count();
    assert_eq!(count, 1);
}

#[test]
fn test_does_not_add_layout_declaration_for_out_frag_color_if_it_already_exists() {
    let source = ShaderSource::new(ShaderSourceOptions {
        defines: vec!["A".to_string()],
        sources: vec![
            "layout(location = 0) out vec4 out_FragColor; void main() { out_FragColor = vec4(1.0); }".to_string(),
        ],
        pick_color_qualifier: Some("in".to_string()),
        include_built_ins: Some(false),
    })
    .unwrap();

    let context = ShaderContextInfo::default();
    let shader_text = source.create_combined_fragment_shader(&context);
    let count = shader_text
        .matches("layout(location = 0) out vec4 out_FragColor;")
        .count();
    assert_eq!(count, 1);
}

#[test]
fn test_find_normal_and_position_varying() {
    let s1 = ShaderSource::new(ShaderSourceOptions {
        defines: vec!["HAS_NORMALS".to_string()],
        sources: vec!["#ifdef HAS_NORMALS\nin vec3 v_normalEC;\n#endif".to_string()],
        ..Default::default()
    })
    .unwrap();
    assert_eq!(s1.find_normal_varying().as_deref(), Some("v_normalEC"));

    let s2 = ShaderSource::new(ShaderSourceOptions {
        defines: vec![],
        sources: vec!["#ifdef HAS_NORMALS\nin vec3 v_normalEC;\n#endif".to_string()],
        ..Default::default()
    })
    .unwrap();
    assert_eq!(s2.find_normal_varying(), None);

    let s3 = ShaderSource::new(ShaderSourceOptions {
        sources: vec!["in vec3 v_positionEC;".to_string()],
        ..Default::default()
    })
    .unwrap();
    assert_eq!(s3.find_position_varying().as_deref(), Some("v_positionEC"));
}
