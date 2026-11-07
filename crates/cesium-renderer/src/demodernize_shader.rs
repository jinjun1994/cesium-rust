//! Transpiler from GLSL 3.00 es to GLSL 1.00 matching CesiumJS `Renderer/demodernizeShader.js` (Apache-2.0).

/// Transpiles a GLSL 3.00 ES shader to a GLSL 1.00 shader.
pub fn demodernize_shader(input: &str, is_fragment_shader: bool) -> String {
    let mut output = input.to_string();

    // Remove version string for GLSL 3.00
    output = output.replace("#version 300 es\n", "");
    output = output.replace("#version 300 es", "");
    output = output.replace("version 300 es", "");

    // Replace texture( with texture2D(
    output = output.replace("texture(", "texture2D(");

    if is_fragment_shader {
        // Replace in with varying for types
        for glsl_type in ["float", "vec2", "vec3", "vec4", "mat2", "mat3", "mat4"] {
            let target_line = format!("\n    in {glsl_type} ");
            let repl_line = format!("\n    varying {glsl_type} ");
            output = output.replace(&target_line, &repl_line);

            let target_simple = format!("in {glsl_type} ");
            let repl_simple = format!("varying {glsl_type} ");
            output = output.replace(&target_simple, &repl_simple);
        }

        if output.contains("out_FragData_") {
            // Remove layout declarations
            let lines: Vec<&str> = output.lines().collect();
            let mut filtered = Vec::new();
            for line in lines {
                let trimmed = line.trim();
                if trimmed.starts_with("layout") && trimmed.contains("out_FragData_") {
                    continue;
                }
                filtered.push(line);
            }
            output = filtered.join("\n");

            // Replace out_FragData_X with gl_FragData[X] in reverse order so out_FragData_10 isn't matched by out_FragData_1
            for i in (0..=15).rev() {
                let old_name = format!("out_FragData_{i}");
                let new_name = format!("gl_FragData[{i}]");
                output = output.replace(&old_name, &new_name);
            }
        }

        // Remove layout declarations for out_FragColor
        let lines: Vec<&str> = output.lines().collect();
        let mut filtered = Vec::new();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("layout") && trimmed.contains("out_FragColor") {
                continue;
            }
            filtered.push(line);
        }
        output = filtered.join("\n");

        // Replace out_FragColor with gl_FragColor
        output = output.replace("out_FragColor", "gl_FragColor");

        let mut prefix = String::new();
        if output.contains("gl_FragDepth") {
            prefix.push_str("#extension GL_EXT_frag_depth : enable\n");
            output = output.replace("gl_FragDepth", "gl_FragDepthEXT");
        }

        output = format!("{prefix}{output}");
    } else {
        // Vertex shader: in -> attribute
        for glsl_type in ["float", "vec2", "vec3", "vec4", "mat2", "mat3", "mat4"] {
            let target = format!("in {glsl_type} ");
            let repl = format!("attribute {glsl_type} ");
            output = output.replace(&target, &repl);
        }

        // Vertex shader: out -> varying
        for glsl_type in ["float", "vec2", "vec3", "vec4", "mat2", "mat3", "mat4"] {
            let target = format!("out {glsl_type} ");
            let repl = format!("varying {glsl_type} ");
            output = output.replace(&target, &repl);
        }
    }

    format!("#version 100\n{output}")
}
