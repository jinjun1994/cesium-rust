use cesium_renderer::demodernize_shader;

#[test]
fn test_replaces_version() {
    let input = "#version 300 es\nvoid main() {}\n";
    let output = demodernize_shader(input, false);
    assert!(output.starts_with("#version 100"));
}

#[test]
fn test_replaces_out_frag_data_with_gl_frag_data() {
    let input = "#version 300 es
    layout (location = 0) vec4 out_FragData_0;
    layout (location = 1) vec4 out_FragData_1;
    layout (location = 10) vec4 out_FragData_10;
    layout (location = 11) vec4 out_FragData_11;
    
    void main() {
      out_FragData_0 = vec4(0.0);
      out_FragData_1 = vec4(1.0);
      out_FragData_10 = vec4(0.0, 0.0, 0.0, 0.0);
      out_FragData_11 = vec4(1.0, 2.0, 3.0, 4.0);
    }
    ";

    let output = demodernize_shader(input, true);
    assert!(output.starts_with("#version 100"));
    assert!(!output.contains("layout (location = 0) vec4 out_FragData_0"));
    assert!(!output.contains("layout (location = 1) vec4 out_FragData_1"));
    assert!(!output.contains("layout (location = 10) vec4 out_FragData_10"));
    assert!(!output.contains("layout (location = 11) vec4 out_FragData_11"));
    assert!(output.contains("gl_FragData[0] = vec4(0.0);"));
    assert!(output.contains("gl_FragData[1] = vec4(1.0);"));
    assert!(output.contains("gl_FragData[10] = vec4(0.0, 0.0, 0.0, 0.0);"));
    assert!(output.contains("gl_FragData[11] = vec4(1.0, 2.0, 3.0, 4.0);"));
}

#[test]
fn test_replaces_out_frag_color_with_gl_frag_color() {
    let input = "#version 300 es
    layout (location = 0) vec4 out_FragColor;
    
    void main() {
      out_FragColor = vec4(1.0);
      out_FragColor[0] = 0.0;
      out_FragColor[2] = 2.0;
    }
    ";
    let output = demodernize_shader(input, true);
    assert!(output.starts_with("#version 100"));
    assert!(!output.contains("layout (location = 0) vec4 out_FragColor"));
    assert!(output.contains("gl_FragColor = vec4(1.0);"));
    assert!(output.contains("gl_FragColor[0] = 0.0;"));
    assert!(output.contains("gl_FragColor[2] = 2.0;"));
}

#[test]
fn test_replaces_texture_with_texture2d() {
    let input = "#version 300 es\nuniform sampler2D u_texture;\nvoid main() {\n  vec4 tex = texture(u_texture, vec2(0.0));\n}\n";
    let output = demodernize_shader(input, false);
    assert!(output.starts_with("#version 100"));
    assert!(output.contains("vec4 tex = texture2D(u_texture, vec2(0.0));"));
}

#[test]
fn test_replaces_in_with_attribute_in_vertex_shader() {
    let input = "#version 300 es\nin float a_float;\nin vec3 a_vec3;\nin mat2 a_mat2;\nvoid main() {}\n";
    let output = demodernize_shader(input, false);
    assert!(output.starts_with("#version 100"));
    assert!(output.contains("attribute float a_float;"));
    assert!(output.contains("attribute vec3 a_vec3;"));
    assert!(output.contains("attribute mat2 a_mat2;"));
}

#[test]
fn test_replaces_in_with_varying_in_fragment_shader() {
    let input = "#version 300 es\nin float a_float;\nin vec3 a_vec3;\nin mat2 a_mat2;\nvoid main() {}\n";
    let output = demodernize_shader(input, true);
    assert!(output.starts_with("#version 100"));
    assert!(output.contains("varying float a_float;"));
    assert!(output.contains("varying vec3 a_vec3;"));
    assert!(output.contains("varying mat2 a_mat2;"));
}

#[test]
fn test_replaces_out_with_varying_in_vertex_shader() {
    let input = "#version 300 es\nout float a_float;\nout vec3 a_vec3;\nout mat2 a_mat2;\nvoid main() {}\n";
    let output = demodernize_shader(input, false);
    assert!(output.starts_with("#version 100"));
    assert!(output.contains("varying float a_float;"));
    assert!(output.contains("varying vec3 a_vec3;"));
    assert!(output.contains("varying mat2 a_mat2;"));
}

#[test]
fn test_replaces_gl_frag_depth_with_gl_frag_depth_ext() {
    let input = "#version 300 es\nvoid main() {\n  gl_FragDepth = 1.0;\n  gl_FragDepth += 2.0;\n}\n";
    let output = demodernize_shader(input, true);
    assert!(output.starts_with("#version 100"));
    assert!(output.contains("#extension GL_EXT_frag_depth : enable"));
    assert!(output.contains("gl_FragDepthEXT = 1.0;"));
    assert!(output.contains("gl_FragDepthEXT += 2.0;"));
}
