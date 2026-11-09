use crate::appearance::Appearance;
use crate::material::Material;
use crate::render_state::RenderState;
use cesium_math::DeveloperError;

/// Initialization options for `DebugAppearance`.
#[derive(Debug, Clone, Default)]
pub struct DebugAppearanceOptions {
    /// The name of the attribute to visualize.
    pub attribute_name: Option<String>,
    /// Whether this attribute is a per-instance geometry attribute.
    pub per_instance_attribute: Option<bool>,
    /// The GLSL datatype of the attribute ('float', 'vec2', 'vec3', 'vec4').
    pub glsl_datatype: Option<String>,
    /// Optional custom vertex shader source.
    pub vertex_shader_source: Option<String>,
    /// Optional custom fragment shader source.
    pub fragment_shader_source: Option<String>,
    /// Optional render state.
    pub render_state: Option<RenderState>,
    /// When true, geometry is expected to appear translucent.
    pub translucent: Option<bool>,
    /// When true, geometry is expected to be closed.
    pub closed: Option<bool>,
}

/// Visualizes a vertex attribute by displaying it as a color for debugging.
#[derive(Debug, Clone)]
pub struct DebugAppearance {
    /// The name of the attribute being visualized.
    pub attribute_name: String,
    /// The GLSL datatype of the attribute being visualized.
    pub glsl_datatype: String,
    /// The GLSL source code for the vertex shader.
    pub vertex_shader_source: String,
    /// The GLSL source code for the fragment shader.
    pub fragment_shader_source: String,
    /// The render state.
    pub render_state: RenderState,
    /// Whether the geometry is expected to be translucent.
    pub translucent: bool,
    /// Whether the geometry is expected to be closed.
    pub closed: bool,
    /// Material, unused for DebugAppearance.
    pub material: Option<Material>,
}

impl DebugAppearance {
    /// Creates a new `DebugAppearance`.
    pub fn new(options: DebugAppearanceOptions) -> Result<Self, DeveloperError> {
        let attribute_name = options
            .attribute_name
            .ok_or_else(|| DeveloperError::new("options.attributeName is required."))?;

        let per_instance_attribute = options.per_instance_attribute.unwrap_or(false);

        let has_custom_datatype = options.glsl_datatype.is_some();
        let mut glsl_datatype = options
            .glsl_datatype
            .unwrap_or_else(|| "vec3".to_string());

        let varying_name = format!("v_{attribute_name}");
        let get_color;

        if attribute_name == "normal" || attribute_name == "tangent" || attribute_name == "bitangent" {
            get_color = format!(
                "vec4 getColor() {{ return vec4(({varying_name} + vec3(1.0)) * 0.5, 1.0); }}\n"
            );
        } else {
            if attribute_name == "st" && !has_custom_datatype {
                glsl_datatype = "vec2".to_string();
            }

            match glsl_datatype.as_str() {
                "float" => {
                    get_color = format!("vec4 getColor() {{ return vec4(vec3({varying_name}), 1.0); }}\n");
                }
                "vec2" => {
                    get_color = format!("vec4 getColor() {{ return vec4({varying_name}, 0.0, 1.0); }}\n");
                }
                "vec3" => {
                    get_color = format!("vec4 getColor() {{ return vec4({varying_name}, 1.0); }}\n");
                }
                "vec4" => {
                    get_color = format!("vec4 getColor() {{ return {varying_name}; }}\n");
                }
                _ => {
                    return Err(DeveloperError::new(
                        "options.glslDatatype must be float, vec2, vec3, or vec4.",
                    ));
                }
            }
        }

        let vs = if let Some(custom_vs) = options.vertex_shader_source {
            custom_vs
        } else {
            let in_attr = if per_instance_attribute {
                String::new()
            } else {
                format!("in {glsl_datatype} {attribute_name};\n")
            };
            let assign_varying = if per_instance_attribute {
                format!("{varying_name} = czm_batchTable_{attribute_name}(batchId);\n")
            } else {
                format!("{varying_name} = {attribute_name};\n")
            };

            format!(
                "in vec3 position3DHigh;\nin vec3 position3DLow;\nin float batchId;\n{in_attr}out {glsl_datatype} {varying_name};\nvoid main()\n{{\nvec4 p = czm_translateRelativeToEye(position3DHigh, position3DLow);\n{assign_varying}gl_Position = czm_modelViewProjectionRelativeToEye * p;\n}}"
            )
        };

        let fs = if let Some(custom_fs) = options.fragment_shader_source {
            custom_fs
        } else {
            format!(
                "in {glsl_datatype} {varying_name};\n{get_color}void main()\n{{\nout_FragColor = getColor();\n}}"
            )
        };

        let translucent = options.translucent.unwrap_or(false);
        let closed = options.closed.unwrap_or(false);
        let render_state = Appearance::get_default_render_state(translucent, closed, options.render_state);

        Ok(Self {
            attribute_name,
            glsl_datatype,
            vertex_shader_source: vs,
            fragment_shader_source: fs,
            render_state,
            translucent,
            closed,
            material: None,
        })
    }

    /// Returns the fragment shader source.
    pub fn get_fragment_shader_source(&self) -> &str {
        &self.fragment_shader_source
    }

    /// Determines if the appearance is translucent.
    pub fn is_translucent(&self) -> bool {
        self.translucent
    }

    /// Gets the render state.
    pub fn get_render_state(&self) -> &RenderState {
        &self.render_state
    }
}
