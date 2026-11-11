//! PerInstanceColorAppearance matching CesiumJS `Scene/PerInstanceColorAppearance.js` (Apache-2.0).

use cesium_math::VertexFormat;

use crate::appearance::Appearance;
use crate::material::Material;
use crate::render_state::RenderState;

pub const PER_INSTANCE_COLOR_APPEARANCE_VS: &str = "\
in vec3 position3DHigh;\n\
in vec3 position3DLow;\n\
in vec3 normal;\n\
in vec4 color;\n\
in float batchId;\n\
\n\
out vec3 v_positionEC;\n\
out vec3 v_normalEC;\n\
out vec4 v_color;\n\
\n\
void main()\n\
{\n\
    vec4 p = czm_computePosition();\n\
\n\
    v_positionEC = (czm_modelViewRelativeToEye * p).xyz;\n\
    v_normalEC = czm_normal * normal;\n\
    v_color = color;\n\
\n\
    gl_Position = czm_modelViewProjectionRelativeToEye * p;\n\
}\n";

pub const PER_INSTANCE_COLOR_APPEARANCE_FS: &str = "\
in vec3 v_positionEC;\n\
in vec3 v_normalEC;\n\
in vec4 v_color;\n\
\n\
void main()\n\
{\n\
    vec3 positionToEyeEC = -v_positionEC;\n\
\n\
    vec3 normalEC = normalize(v_normalEC);\n\
#ifdef FACE_FORWARD\n\
    normalEC = faceforward(normalEC, vec3(0.0, 0.0, 1.0), -normalEC);\n\
#endif\n\
\n\
    vec4 color = czm_gammaCorrect(v_color);\n\
\n\
    czm_materialInput materialInput;\n\
    materialInput.normalEC = normalEC;\n\
    materialInput.positionToEyeEC = positionToEyeEC;\n\
    czm_material material = czm_getDefaultMaterial(materialInput);\n\
    material.diffuse = color.rgb;\n\
    material.alpha = color.a;\n\
\n\
    out_FragColor = czm_phong(normalize(positionToEyeEC), material, czm_lightDirectionEC);\n\
}\n";

pub const PER_INSTANCE_FLAT_COLOR_APPEARANCE_VS: &str = "\
in vec3 position3DHigh;\n\
in vec3 position3DLow;\n\
in vec4 color;\n\
in float batchId;\n\
\n\
out vec4 v_color;\n\
\n\
void main()\n\
{\n\
    vec4 p = czm_computePosition();\n\
\n\
    v_color = color;\n\
\n\
    gl_Position = czm_modelViewProjectionRelativeToEye * p;\n\
}\n";

pub const PER_INSTANCE_FLAT_COLOR_APPEARANCE_FS: &str = "\
in vec4 v_color;\n\
\n\
void main()\n\
{\n\
    out_FragColor = czm_gammaCorrect(v_color);\n\
}\n";

/// Options for configuring a [`PerInstanceColorAppearance`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PerInstanceColorAppearanceOptions {
    pub flat: Option<bool>,
    pub face_forward: Option<bool>,
    pub translucent: Option<bool>,
    pub closed: Option<bool>,
    pub vertex_shader_source: Option<String>,
    pub fragment_shader_source: Option<String>,
    pub render_state: Option<RenderState>,
}

/// An appearance for geometry instances with per-instance color attributes.
#[derive(Clone, Debug, PartialEq)]
pub struct PerInstanceColorAppearance {
    pub material: Option<Material>,
    pub translucent: bool,
    pub closed: bool,
    pub flat: bool,
    pub face_forward: bool,
    pub vertex_format: VertexFormat,
    pub vertex_shader_source: String,
    pub fragment_shader_source: String,
    pub render_state: RenderState,
}

impl Default for PerInstanceColorAppearance {
    fn default() -> Self {
        Self::new(PerInstanceColorAppearanceOptions::default())
    }
}

impl PerInstanceColorAppearance {
    /// Vertex format compatible with standard PerInstanceColorAppearance (position + normal).
    pub const VERTEX_FORMAT: VertexFormat = VertexFormat::POSITION_AND_NORMAL;

    /// Vertex format compatible when flat shading is enabled (position only).
    pub const FLAT_VERTEX_FORMAT: VertexFormat = VertexFormat::POSITION_ONLY;

    /// Creates a new `PerInstanceColorAppearance` instance.
    pub fn new(options: PerInstanceColorAppearanceOptions) -> Self {
        let translucent = options.translucent.unwrap_or(true);
        let closed = options.closed.unwrap_or(false);
        let flat = options.flat.unwrap_or(false);
        let face_forward = options.face_forward.unwrap_or(!closed);

        let vs = options.vertex_shader_source.unwrap_or_else(|| {
            if flat {
                PER_INSTANCE_FLAT_COLOR_APPEARANCE_VS.to_string()
            } else {
                PER_INSTANCE_COLOR_APPEARANCE_VS.to_string()
            }
        });

        let fs = options.fragment_shader_source.unwrap_or_else(|| {
            if flat {
                PER_INSTANCE_FLAT_COLOR_APPEARANCE_FS.to_string()
            } else {
                PER_INSTANCE_COLOR_APPEARANCE_FS.to_string()
            }
        });

        let vertex_format = if flat {
            Self::FLAT_VERTEX_FORMAT
        } else {
            Self::VERTEX_FORMAT
        };

        let render_state =
            Appearance::get_default_render_state(translucent, closed, options.render_state);

        Self {
            material: None,
            translucent,
            closed,
            flat,
            face_forward,
            vertex_format,
            vertex_shader_source: vs,
            fragment_shader_source: fs,
            render_state,
        }
    }

    /// Procedurally creates the full GLSL fragment shader source.
    pub fn get_fragment_shader_source(&self) -> String {
        let mut parts = Vec::new();
        if self.flat {
            parts.push("#define FLAT".to_string());
        }
        if self.face_forward {
            parts.push("#define FACE_FORWARD".to_string());
        }
        parts.push(self.fragment_shader_source.clone());
        parts.join("\n")
    }

    /// Determines if the geometry is translucent based on `self.translucent`.
    #[inline]
    pub fn is_translucent(&self) -> bool {
        self.translucent
    }

    /// Creates or returns the WebGL render state.
    #[inline]
    pub fn get_render_state(&self) -> RenderState {
        self.render_state.clone()
    }
}
