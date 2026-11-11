//! PolylineColorAppearance matching CesiumJS `Scene/PolylineColorAppearance.js` (Apache-2.0).

use cesium_math::VertexFormat;

use crate::appearance::Appearance;
use crate::material::Material;
use crate::render_state::RenderState;

pub const POLYLINE_COLOR_APPEARANCE_VS: &str = "\
#define CLIP_POLYLINE\n\
in vec3 position3DHigh;\n\
in vec3 position3DLow;\n\
in vec3 prevPosition3DHigh;\n\
in vec3 prevPosition3DLow;\n\
in vec3 nextPosition3DHigh;\n\
in vec3 nextPosition3DLow;\n\
in vec2 expandAndWidth;\n\
in vec4 color;\n\
\n\
out vec4 v_color;\n\
\n\
void main()\n\
{\n\
    v_color = color;\n\
    gl_Position = czm_modelViewProjectionRelativeToEye * czm_computePosition();\n\
}\n";

pub const POLYLINE_COLOR_APPEARANCE_FS: &str = "\
in vec4 v_color;\n\
\n\
void main()\n\
{\n\
    out_FragColor = czm_gammaCorrect(v_color);\n\
}\n";

/// Options for configuring a [`PolylineColorAppearance`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PolylineColorAppearanceOptions {
    pub translucent: Option<bool>,
    pub vertex_shader_source: Option<String>,
    pub fragment_shader_source: Option<String>,
    pub render_state: Option<RenderState>,
}

/// An appearance for polylines with per-instance color attributes.
#[derive(Clone, Debug, PartialEq)]
pub struct PolylineColorAppearance {
    pub material: Option<Material>,
    pub translucent: bool,
    pub closed: bool,
    pub vertex_format: VertexFormat,
    pub vertex_shader_source: String,
    pub fragment_shader_source: String,
    pub render_state: RenderState,
}

impl Default for PolylineColorAppearance {
    fn default() -> Self {
        Self::new(PolylineColorAppearanceOptions::default())
    }
}

impl PolylineColorAppearance {
    /// The compatible vertex format requiring only position.
    pub const VERTEX_FORMAT: VertexFormat = VertexFormat::POSITION_ONLY;

    /// Creates a new `PolylineColorAppearance`.
    pub fn new(options: PolylineColorAppearanceOptions) -> Self {
        let translucent = options.translucent.unwrap_or(true);
        let closed = false;

        let render_state = options
            .render_state
            .unwrap_or_else(|| Appearance::get_default_render_state(translucent, closed, None));

        let vertex_shader_source = options
            .vertex_shader_source
            .unwrap_or_else(|| POLYLINE_COLOR_APPEARANCE_VS.to_string());

        let fragment_shader_source = options
            .fragment_shader_source
            .unwrap_or_else(|| POLYLINE_COLOR_APPEARANCE_FS.to_string());

        Self {
            material: None,
            translucent,
            closed,
            vertex_format: Self::VERTEX_FORMAT,
            vertex_shader_source,
            fragment_shader_source,
            render_state,
        }
    }

    /// Determines if the geometry is translucent.
    pub fn is_translucent(&self) -> bool {
        self.translucent
    }

    /// Gets the render state.
    pub fn get_render_state(&self) -> &RenderState {
        &self.render_state
    }

    /// Gets the procedural fragment shader source.
    pub fn get_fragment_shader_source(&self) -> &str {
        &self.fragment_shader_source
    }
}
