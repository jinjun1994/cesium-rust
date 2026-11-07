use cesium_renderer::{
    BuiltShaderProgram, ShaderBuilder, ShaderContextInfo, ShaderDestination, ShaderSource,
    ShaderSourceOptions,
};

fn check_vertex_shader(
    program: &BuiltShaderProgram,
    expected_defines: &[&str],
    expected_sources: &[&str],
    context: &ShaderContextInfo,
) {
    let expected_src = if expected_sources.is_empty() {
        Vec::new()
    } else {
        vec![expected_sources.join("\n")]
    };

    let expected_source = ShaderSource::new(ShaderSourceOptions {
        defines: expected_defines.iter().map(|s| s.to_string()).collect(),
        sources: expected_src,
        ..Default::default()
    })
    .unwrap();
    let expected_text = expected_source.create_combined_vertex_shader(context);
    assert_eq!(program.vertex_shader_text, expected_text);
}

fn check_fragment_shader(
    program: &BuiltShaderProgram,
    expected_defines: &[&str],
    expected_sources: &[&str],
    context: &ShaderContextInfo,
) {
    let expected_src = if expected_sources.is_empty() {
        Vec::new()
    } else {
        vec![expected_sources.join("\n")]
    };

    let expected_source = ShaderSource::new(ShaderSourceOptions {
        defines: expected_defines.iter().map(|s| s.to_string()).collect(),
        sources: expected_src,
        ..Default::default()
    })
    .unwrap();
    let expected_text = expected_source.create_combined_fragment_shader(context);
    assert_eq!(program.fragment_shader_text, expected_text);
}

#[test]
fn test_creates_an_empty_shader_by_default() {
    let builder = ShaderBuilder::new();
    let context = ShaderContextInfo::default();
    let program = builder.build_shader_program(&context);
    check_vertex_shader(&program, &[], &[], &context);
    check_fragment_shader(&program, &[], &[], &context);
}

#[test]
fn test_add_define_defines_macros_without_values() {
    let mut builder = ShaderBuilder::new();
    builder.add_define("USE_SHADOWS", None, None);
    let context = ShaderContextInfo::default();
    let program = builder.build_shader_program(&context);
    check_vertex_shader(&program, &["USE_SHADOWS"], &[], &context);
    check_fragment_shader(&program, &["USE_SHADOWS"], &[], &context);
}

#[test]
fn test_add_define_defines_macros_with_values() {
    let mut builder = ShaderBuilder::new();
    builder.add_define("PI", Some("3.1415"), None);
    let context = ShaderContextInfo::default();
    let program = builder.build_shader_program(&context);
    check_vertex_shader(&program, &["PI 3.1415"], &[], &context);
    check_fragment_shader(&program, &["PI 3.1415"], &[], &context);
}

#[test]
fn test_add_define_puts_define_in_destination_shaders() {
    let mut builder = ShaderBuilder::new();
    builder.add_define("POINT_SIZE", Some("2"), Some(ShaderDestination::VERTEX));
    builder.add_define("PI", Some("3.1415"), Some(ShaderDestination::FRAGMENT));
    builder.add_define("USE_FRAGMENT_SHADING", Some("1"), Some(ShaderDestination::BOTH));
    let context = ShaderContextInfo::default();
    let program = builder.build_shader_program(&context);
    check_vertex_shader(
        &program,
        &["POINT_SIZE 2", "USE_FRAGMENT_SHADING 1"],
        &[],
        &context,
    );
    check_fragment_shader(
        &program,
        &["PI 3.1415", "USE_FRAGMENT_SHADING 1"],
        &[],
        &context,
    );
}

#[test]
fn test_add_struct_adds_struct_to_shader() {
    let mut builder = ShaderBuilder::new();
    builder.add_struct("structVS", "TestStruct", ShaderDestination::VERTEX);
    builder.add_struct("structFS", "TestStruct", ShaderDestination::FRAGMENT);

    let context = ShaderContextInfo::default();
    let program = builder.build_shader_program(&context);
    check_vertex_shader(
        &program,
        &[],
        &["struct TestStruct", "{", "    float _empty;", "};"],
        &context,
    );
    check_fragment_shader(
        &program,
        &[],
        &["struct TestStruct", "{", "    float _empty;", "};"],
        &context,
    );
}

#[test]
fn test_add_struct_field_adds_fields() {
    let mut builder = ShaderBuilder::new();
    builder.add_struct("structVS", "TestStruct", ShaderDestination::VERTEX);
    builder.add_struct("structFS", "TestStruct", ShaderDestination::FRAGMENT);

    builder.add_struct_field("structVS", "vec3", "positionMC");
    builder.add_struct_field("structFS", "vec3", "positionMC");
    builder.add_struct_field("structFS", "float", "temperature");

    let context = ShaderContextInfo::default();
    let program = builder.build_shader_program(&context);
    check_vertex_shader(
        &program,
        &[],
        &["struct TestStruct", "{", "    vec3 positionMC;", "};"],
        &context,
    );
    check_fragment_shader(
        &program,
        &[],
        &[
            "struct TestStruct",
            "{",
            "    vec3 positionMC;",
            "    float temperature;",
            "};",
        ],
        &context,
    );
}

#[test]
fn test_add_function_and_lines() {
    let signature = "float circleMask(float radius)";
    let mut builder = ShaderBuilder::new();
    builder.add_function("testFunctionVS", signature, ShaderDestination::VERTEX);
    builder.add_function("testFunctionFS", signature, ShaderDestination::FRAGMENT);

    builder.add_function_lines(
        "testFunctionVS",
        &["v_color = vec3(0.0);", "return 1.0;"],
    );
    builder.add_function_lines("testFunctionFS", &["return 1.0 - step(0.3, radius);"]);

    let context = ShaderContextInfo::default();
    let program = builder.build_shader_program(&context);
    check_vertex_shader(
        &program,
        &[],
        &[signature, "{", "    v_color = vec3(0.0);", "    return 1.0;", "}"],
        &context,
    );
    check_fragment_shader(
        &program,
        &[],
        &[signature, "{", "    return 1.0 - step(0.3, radius);", "}"],
        &context,
    );
}

#[test]
fn test_add_uniform_puts_uniform_in_destination_shaders() {
    let mut builder = ShaderBuilder::new();
    builder.add_uniform("vec3", "u_gridDimensions", Some(ShaderDestination::VERTEX));
    builder.add_uniform("vec2", "u_mousePosition", Some(ShaderDestination::FRAGMENT));
    builder.add_uniform("float", "u_time", Some(ShaderDestination::BOTH));

    let context = ShaderContextInfo::default();
    let program = builder.build_shader_program(&context);
    check_vertex_shader(
        &program,
        &[],
        &["uniform vec3 u_gridDimensions;", "uniform float u_time;"],
        &context,
    );
    check_fragment_shader(
        &program,
        &[],
        &["uniform vec2 u_mousePosition;", "uniform float u_time;"],
        &context,
    );
}

#[test]
fn test_set_position_attribute_and_add_attribute() {
    let mut builder = ShaderBuilder::new();

    let normal_loc = builder.add_attribute("vec3", "a_normal");
    let pos_loc = builder.set_position_attribute("vec3", "a_position").unwrap();
    assert_eq!(pos_loc, 0);
    assert_eq!(normal_loc, 1);

    // Call set_position_attribute twice should fail
    assert!(builder.set_position_attribute("vec3", "a_pos2").is_err());

    let context = ShaderContextInfo::default();
    let program = builder.build_shader_program(&context);
    check_vertex_shader(
        &program,
        &[],
        &["in vec3 a_position;", "in vec3 a_normal;"],
        &context,
    );
    check_fragment_shader(&program, &[], &[], &context);

    assert_eq!(builder.attribute_locations().get("a_position"), Some(&0));
    assert_eq!(builder.attribute_locations().get("a_normal"), Some(&1));
    assert_eq!(program.attribute_locations.get("a_position"), Some(&0));
    assert_eq!(program.attribute_locations.get("a_normal"), Some(&1));
}

#[test]
fn test_add_attribute_handles_matrices() {
    let mut builder = ShaderBuilder::new();
    let matrix_loc = builder.add_attribute("mat3", "a_warpMatrix");
    let color_loc = builder.add_attribute("vec3", "a_color");
    assert_eq!(matrix_loc, 1);
    assert_eq!(color_loc, 4);

    let context = ShaderContextInfo::default();
    let program = builder.build_shader_program(&context);
    check_vertex_shader(
        &program,
        &[],
        &["in mat3 a_warpMatrix;", "in vec3 a_color;"],
        &context,
    );
    check_fragment_shader(&program, &[], &[], &context);
}

#[test]
fn test_add_varying_adds_varyings_to_both_shaders() {
    let mut builder = ShaderBuilder::new();
    builder.add_varying("vec2", "v_uv", None);

    let context = ShaderContextInfo::default();
    let program = builder.build_shader_program(&context);
    check_vertex_shader(&program, &[], &["out vec2 v_uv;"], &context);
    check_fragment_shader(&program, &[], &["in vec2 v_uv;"], &context);
}

#[test]
fn test_build_shader_program_creates_full_shader_program() {
    let mut builder = ShaderBuilder::new();
    builder.set_position_attribute("vec3", "a_position").unwrap();
    builder.add_attribute("vec3", "a_uv");
    builder.add_varying("vec2", "v_uv", None);
    builder.add_define("BLUE_TINT", Some("0.5"), Some(ShaderDestination::FRAGMENT));

    let vertex_lines = [
        "void main()",
        "{",
        "    v_uv = a_uv",
        "    gl_Position = vec4(a_position, 1.0);",
        "}",
    ];
    builder.add_vertex_lines(&vertex_lines);

    let fragment_lines = [
        "void main()",
        "{",
        "    out_FragColor = vec4(v_uv, BLUE_TINT, 1.0);",
        "}",
    ];
    builder.add_fragment_lines(&fragment_lines);

    let expected_vs = [
        "in vec3 a_position;",
        "in vec3 a_uv;",
        "out vec2 v_uv;",
        "void main()",
        "{",
        "    v_uv = a_uv",
        "    gl_Position = vec4(a_position, 1.0);",
        "}",
    ];

    let expected_fs = [
        "in vec2 v_uv;",
        "void main()",
        "{",
        "    out_FragColor = vec4(v_uv, BLUE_TINT, 1.0);",
        "}",
    ];

    let context = ShaderContextInfo::default();
    let program = builder.build_shader_program(&context);
    check_vertex_shader(&program, &[], &expected_vs, &context);
    check_fragment_shader(&program, &["BLUE_TINT 0.5"], &expected_fs, &context);
}
