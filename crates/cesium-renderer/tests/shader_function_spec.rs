//! Direct port and verification against CesiumJS `ShaderFunctionSpec.js` (Apache-2.0).

use cesium_renderer::ShaderFunction;

#[test]
fn test_constructs() {
    let signature = "vec3 testFunction(vec3 position)";
    let func = ShaderFunction::new(signature);
    assert_eq!(func.signature, signature);
    assert!(func.body.is_empty());
}

#[test]
fn test_add_lines_adds_lines_to_the_function_body() {
    let mut func = ShaderFunction::new("TestFunction");
    func.add_lines(&["v_color = a_color;", "return vec3(0.0, 0.0, 1.0);"]);
    assert_eq!(
        func.body,
        vec![
            "    v_color = a_color;".to_string(),
            "    return vec3(0.0, 0.0, 1.0);".to_string(),
        ]
    );
}

#[test]
fn test_add_lines_accepts_a_single_string() {
    let mut func = ShaderFunction::new("TestFunction");
    func.add_line("v_color = a_color;");
    assert_eq!(func.body, vec!["    v_color = a_color;".to_string()]);
}

#[test]
fn test_generate_glsl_lines_generates_a_function() {
    let signature = "vec3 testFunction(vec3 position)";
    let mut func = ShaderFunction::new(signature);
    func.add_lines(&["v_color = a_color;", "return vec3(0.0, 0.0, 1.0);"]);
    assert_eq!(
        func.generate_glsl_lines(),
        vec![
            signature,
            "{",
            "    v_color = a_color;",
            "    return vec3(0.0, 0.0, 1.0);",
            "}",
        ]
    );
}
