//! Direct port and verification against CesiumJS `ShaderStructSpec.js` (Apache-2.0).

use cesium_renderer::ShaderStruct;

#[test]
fn test_constructs() {
    let s = ShaderStruct::new("TestStruct");
    assert_eq!(s.name, "TestStruct");
    assert!(s.fields.is_empty());
}

#[test]
fn test_add_field_adds_fields() {
    let mut s = ShaderStruct::new("TestStruct");
    s.add_field("vec3", "positionMC");
    s.add_field("float", "weights[4]");
    s.add_field("OtherStruct", "complex");

    assert_eq!(
        s.fields,
        vec![
            "    vec3 positionMC;".to_string(),
            "    float weights[4];".to_string(),
            "    OtherStruct complex;".to_string(),
        ]
    );
}

#[test]
fn test_generate_glsl_lines_generates_a_struct_definition() {
    let mut s = ShaderStruct::new("TestStruct");
    s.add_field("vec3", "positionMC");
    s.add_field("float", "weights[4]");
    s.add_field("OtherStruct", "complex");

    assert_eq!(
        s.generate_glsl_lines(),
        vec![
            "struct TestStruct",
            "{",
            "    vec3 positionMC;",
            "    float weights[4];",
            "    OtherStruct complex;",
            "};",
        ]
    );
}

#[test]
fn test_generate_glsl_lines_pads_an_empty_struct_definition() {
    let s = ShaderStruct::new("Nothing");
    assert_eq!(
        s.generate_glsl_lines(),
        vec!["struct Nothing", "{", "    float _empty;", "};",]
    );
}
