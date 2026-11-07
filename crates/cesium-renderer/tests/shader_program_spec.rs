use std::collections::BTreeMap;
use cesium_renderer::{
    ShaderCache, ShaderContextInfo, ShaderProgram, ShaderProgramOptions, ShaderSource,
    ShaderSourceOptions,
};

#[test]
fn test_has_vertex_and_fragment_shader_source() {
    let vs = "void main() { gl_Position = vec4(1.0); }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";

    let context = ShaderContextInfo::default();
    let cache = ShaderCache::new(context);

    let sp = ShaderProgram::from_cache(
        &cache,
        ShaderProgramOptions {
            vertex_shader_source: vs.into(),
            fragment_shader_source: fs.into(),
            attribute_locations: BTreeMap::new(),
        },
    );

    let expected_vs_text = ShaderSource::new(ShaderSourceOptions {
        sources: vec![vs.to_string()],
        ..Default::default()
    })
    .unwrap()
    .create_combined_vertex_shader(&context);

    let expected_fs_text = ShaderSource::new(ShaderSourceOptions {
        sources: vec![fs.to_string()],
        ..Default::default()
    })
    .unwrap()
    .create_combined_fragment_shader(&context);

    assert_eq!(sp.borrow().vertex_shader_text, expected_vs_text);
    assert_eq!(sp.borrow().fragment_shader_text, expected_fs_text);
}

#[test]
fn test_has_a_position_vertex_attribute() {
    let vs = "in vec4 position;\nvoid main() { gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";

    let cache = ShaderCache::new(ShaderContextInfo::default());
    let sp = ShaderProgram::from_cache(
        &cache,
        ShaderProgramOptions {
            vertex_shader_source: vs.into(),
            fragment_shader_source: fs.into(),
            attribute_locations: BTreeMap::new(),
        },
    );

    let borrowed = sp.borrow();
    assert_eq!(borrowed.number_of_vertex_attributes(), 1);
    let attr = borrowed.vertex_attributes.get("position").unwrap();
    assert_eq!(attr.name, "position");
    assert_eq!(attr.index, 0);
}

#[test]
fn test_sets_attribute_indices() {
    let vs = "in vec4 position;\nin vec3 normal;\nin float heat;\nvoid main() { gl_Position = position + vec4(normal, 0.0) + vec4(heat); }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";

    let mut attrs = BTreeMap::new();
    attrs.insert("position".to_string(), 3);
    attrs.insert("normal".to_string(), 2);
    attrs.insert("heat".to_string(), 1);

    let cache = ShaderCache::new(ShaderContextInfo::default());
    let sp = ShaderProgram::from_cache(
        &cache,
        ShaderProgramOptions {
            vertex_shader_source: vs.into(),
            fragment_shader_source: fs.into(),
            attribute_locations: attrs.clone(),
        },
    );

    let borrowed = sp.borrow();
    assert_eq!(borrowed.number_of_vertex_attributes(), 3);

    let pos = borrowed.vertex_attributes.get("position").unwrap();
    assert_eq!(pos.name, "position");
    assert_eq!(pos.index, 3);

    let norm = borrowed.vertex_attributes.get("normal").unwrap();
    assert_eq!(norm.name, "normal");
    assert_eq!(norm.index, 2);

    let heat = borrowed.vertex_attributes.get("heat").unwrap();
    assert_eq!(heat.name, "heat");
    assert_eq!(heat.index, 1);
}

#[test]
fn test_compiles_with_version_directive() {
    let vs = "#version 300 es\nin vec4 position; void main() { gl_Position = position; }";
    let fs = "#version 300 es\nvoid main() { out_FragColor = vec4(1.0); }";

    let cache = ShaderCache::new(ShaderContextInfo::default());
    let sp = ShaderProgram::from_cache(
        &cache,
        ShaderProgramOptions {
            vertex_shader_source: vs.into(),
            fragment_shader_source: fs.into(),
            attribute_locations: BTreeMap::new(),
        },
    );

    let borrowed = sp.borrow();
    assert!(borrowed.vertex_shader_text.starts_with("#version 300 es\n"));
    assert!(borrowed.fragment_shader_text.starts_with("#version 300 es\n"));
}
