/// Rendering context flags needed for shader source combination.
#[derive(Debug, Clone, Copy)]
pub struct ShaderContextInfo {
    pub webgl2: bool,
    pub texture_float_linear: bool,
    pub floating_point_texture: bool,
}

impl Default for ShaderContextInfo {
    fn default() -> Self {
        Self {
            webgl2: true,
            texture_float_linear: false,
            floating_point_texture: false,
        }
    }
}

/// A container for GLSL shader source code, defines, and pick qualifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderSource {
    pub defines: Vec<String>,
    pub sources: Vec<String>,
    pub pick_color_qualifier: Option<String>,
    pub include_built_ins: bool,
}

impl Default for ShaderSource {
    fn default() -> Self {
        Self {
            defines: Vec::new(),
            sources: Vec::new(),
            pick_color_qualifier: None,
            include_built_ins: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ShaderSourceOptions {
    pub defines: Vec<String>,
    pub sources: Vec<String>,
    pub pick_color_qualifier: Option<String>,
    pub include_built_ins: Option<bool>,
}

impl ShaderSource {
    pub fn new(options: ShaderSourceOptions) -> Result<Self, String> {
        if let Some(ref qualifier) = options.pick_color_qualifier {
            if qualifier != "uniform" && qualifier != "in" {
                return Err(format!(
                    "pickColorQualifier must be 'uniform' or 'in', got '{qualifier}'"
                ));
            }
        }

        Ok(Self {
            defines: options.defines,
            sources: options.sources,
            pick_color_qualifier: options.pick_color_qualifier,
            include_built_ins: options.include_built_ins.unwrap_or(true),
        })
    }

    /// Replaces `void main()` or `void main(void)` with `void {renamed_main}()`.
    pub fn replace_main(source: &str, renamed_main: &str) -> String {
        let regex_patterns = [
            "void main()",
            "void main( )",
            "void main(void)",
            "void main( void )",
        ];
        let replacement = format!("void {renamed_main}()");
        let mut result = source.to_string();
        for pattern in &regex_patterns {
            result = result.replace(pattern, &replacement);
        }
        result
    }

    /// Generates pick fragment shader source.
    pub fn create_pick_fragment_shader_source(
        fragment_shader_source: &str,
        pick_color_qualifier: &str,
    ) -> String {
        let renamed_fs = Self::replace_main(fragment_shader_source, "czm_old_main");
        let pick_main = format!(
            "{pick_color_qualifier} vec4 czm_pickColor;\nvoid main()\n{{\n    czm_old_main();\n    if (out_FragColor.a == 0.0) {{\n       discard;\n    }}\n    out_FragColor = czm_pickColor;\n}}"
        );
        format!("{renamed_fs}\n{pick_main}")
    }

    /// Generates pick vertex shader source.
    pub fn create_pick_vertex_shader_source(vertex_shader_source: &str) -> String {
        let renamed_vs = Self::replace_main(vertex_shader_source, "czm_old_main");
        let pick_main = "in vec4 pickColor;\nout vec4 czm_pickColor;\nvoid main()\n{\n    czm_old_main();\n    czm_pickColor = pickColor;\n}";
        format!("{renamed_vs}\n{pick_main}")
    }

    /// Generates a cache key for shader program caching.
    pub fn get_cache_key(&self) -> String {
        let mut sorted_defines = self.defines.clone();
        sorted_defines.sort();
        let defines_key = sorted_defines.join(",");
        let pick_key = self
            .pick_color_qualifier
            .as_deref()
            .unwrap_or("undefined");
        let builtins_key = self.include_built_ins.to_string();
        let sources_key = self.sources.join("\n");

        format!("{defines_key}:{pick_key}:{builtins_key}:{sources_key}")
    }

    /// Finds normal varying variable name if present.
    pub fn find_normal_varying(&self) -> Option<String> {
        let has_has_normals = self
            .sources
            .iter()
            .any(|s| s.contains("#ifdef HAS_NORMALS"));
        if has_has_normals {
            if self.defines.iter().any(|d| d == "HAS_NORMALS") {
                return Some("v_normalEC".to_string());
            }
            return None;
        }

        let normal_varying_names = ["v_normalEC", "v_normal"];
        for name in &normal_varying_names {
            if self.sources.iter().any(|s| s.contains(name)) {
                return Some((*name).to_string());
            }
        }
        None
    }

    /// Finds position varying variable name if present.
    pub fn find_position_varying(&self) -> Option<String> {
        let position_varying_names = ["v_positionEC"];
        for name in &position_varying_names {
            if self.sources.iter().any(|s| s.contains(name)) {
                return Some((*name).to_string());
            }
        }
        None
    }

    /// Combines vertex shader source with defines and line directives.
    pub fn create_combined_vertex_shader(&self, context: &ShaderContextInfo) -> String {
        self.combine_shader(false, context)
    }

    /// Combines fragment shader source with defines, precision qualifiers, and line directives.
    pub fn create_combined_fragment_shader(&self, context: &ShaderContextInfo) -> String {
        self.combine_shader(true, context)
    }

    fn combine_shader(&self, is_fragment_shader: bool, context: &ShaderContextInfo) -> String {
        let mut combined_sources = String::new();
        for source in &self.sources {
            combined_sources.push_str("\n#line 0\n");
            combined_sources.push_str(source);
        }

        // Remove comments
        combined_sources = remove_comments(&combined_sources);

        // Extract version
        let mut version: Option<String> = None;
        if let Some(pos) = combined_sources.find("#version ") {
            if let Some(newline_pos) = combined_sources[pos..].find('\n') {
                let end = pos + newline_pos;
                let version_line = &combined_sources[pos..end];
                let ver = version_line.trim_start_matches("#version ").trim();
                version = Some(ver.to_string());
                combined_sources.replace_range(pos..end, "\n");
            }
        }

        // Extract extensions
        let mut extensions = Vec::new();
        while let Some(pos) = combined_sources.find("#extension") {
            if let Some(newline_pos) = combined_sources[pos..].find('\n') {
                let end = pos + newline_pos + 1;
                let ext_line = combined_sources[pos..end].to_string();
                extensions.push(ext_line);
                combined_sources.replace_range(pos..end, "\n");
            } else {
                break;
            }
        }

        // Pick shader replacement if qualifier is provided
        if is_fragment_shader {
            if let Some(ref qualifier) = self.pick_color_qualifier {
                combined_sources =
                    Self::create_pick_fragment_shader_source(&combined_sources, qualifier);
            }
        } else if let Some(ref _qualifier) = self.pick_color_qualifier {
            // Vertex shader pick color replacement
            let renamed_vs = Self::replace_main(&combined_sources, "czm_old_main");
            let pick_main = "in vec4 pickColor;\nout vec4 czm_pickColor;\nvoid main()\n{\n    czm_old_main();\n    czm_pickColor = pickColor;\n}";
            combined_sources = format!("{renamed_vs}\n{pick_main}");
        }

        let mut result = String::new();

        for ext in extensions {
            result.push_str(&ext);
        }

        if is_fragment_shader {
            result.push_str("\n#ifdef GL_FRAGMENT_PRECISION_HIGH\n    precision highp float;\n    precision highp int;\n#else\n    precision mediump float;\n    precision mediump int;\n    #define highp mediump\n#endif\n");
        }

        if context.webgl2 {
            result.push_str("precision highp sampler3D;\n\n");
        }

        for define in &self.defines {
            if !define.is_empty() {
                result.push_str(&format!("#define {define}\n"));
            }
        }

        if context.texture_float_linear {
            result.push_str("#define OES_texture_float_linear\n\n");
        }

        if context.floating_point_texture {
            result.push_str("#define OES_texture_float\n\n");
        }

        result.push_str("\n#line 0\n");

        if context.webgl2
            && is_fragment_shader
            && !combined_sources.contains("layout(location = 0) out vec4 out_FragColor;")
            && !combined_sources.contains("layout (location = 0) out vec4 out_FragColor;")
            && !combined_sources.contains("czm_out_FragColor")
            && combined_sources.contains("out_FragColor")
        {
            result.push_str("layout(location = 0) out vec4 out_FragColor;\n\n");
        }

        result.push_str(&combined_sources);

        if context.webgl2 {
            if let Some(ver) = version {
                format!("#version {ver}\n{result}")
            } else {
                format!("#version 300 es\n{result}")
            }
        } else {
            result
        }
    }
}

fn remove_comments(source: &str) -> String {
    let mut result = String::new();
    let mut chars = source.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '/' {
            if let Some(&next) = chars.peek() {
                if next == '/' {
                    // Line comment: skip until newline
                    chars.next();
                    for lc in chars.by_ref() {
                        if lc == '\n' {
                            result.push('\n');
                            break;
                        }
                    }
                    continue;
                } else if next == '*' {
                    // Block comment: skip until */, preserve newlines
                    chars.next();
                    let mut prev = ' ';
                    for bc in chars.by_ref() {
                        if bc == '\n' {
                            result.push('\n');
                        }
                        if prev == '*' && bc == '/' {
                            break;
                        }
                        prev = bc;
                    }
                    continue;
                }
            }
        }
        result.push(c);
    }

    result
}
