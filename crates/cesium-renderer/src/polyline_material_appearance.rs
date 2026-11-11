//! PolylineMaterialAppearance matching CesiumJS `Scene/PolylineMaterialAppearance.js` (Apache-2.0).

use cesium_math::VertexFormat;

use crate::appearance::Appearance;
use crate::material::Material;
use crate::render_state::RenderState;

pub const POLYLINE_MATERIAL_APPEARANCE_VS: &str = "\
#define CLIP_POLYLINE\n\
in vec3 position3DHigh;\n\
in vec3 position3DLow;\n\
in vec3 prevPosition3DHigh;\n\
in vec3 prevPosition3DLow;\n\
in vec3 nextPosition3DHigh;\n\
in vec3 nextPosition3DLow;\n\
in vec2 expandAndWidth;\n\
in vec2 st;\n\
in float batchId;\n\
\n\
out vec2 v_st;\n\
out float v_width;\n\
\n\
void main()\n\
{\n\
    v_st = st;\n\
    v_width = expandAndWidth.y;\n\
    gl_Position = czm_modelViewProjectionRelativeToEye * czm_computePosition();\n\
}\n";

pub const POLYLINE_MATERIAL_APPEARANCE_FS: &str = "\
in vec2 v_st;\n\
in float v_width;\n\
\n\
void main()\n\
{\n\
    czm_materialInput materialInput;\n\
    materialInput.s = v_st.s;\n\
    materialInput.st = v_st;\n\
    materialInput.str = vec3(v_st, 0.0);\n\
    materialInput.width = v_width;\n\
    czm_material material = czm_getMaterial(materialInput);\n\
    out_FragColor = vec4(material.diffuse + material.emission, material.alpha);\n\
}\n";

/// Options for configuring a [`PolylineMaterialAppearance`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PolylineMaterialAppearanceOptions {
    pub translucent: Option<bool>,
    pub material: Option<Material>,
    pub vertex_shader_source: Option<String>,
    pub fragment_shader_source: Option<String>,
    pub render_state: Option<RenderState>,
}

/// An appearance for polylines shaded with materials.
#[derive(Clone, Debug, PartialEq)]
pub struct PolylineMaterialAppearance {
    pub material: Material,
    pub translucent: bool,
    pub closed: bool,
    pub vertex_format: VertexFormat,
    base_vertex_shader_source: String,
    pub fragment_shader_source: String,
    pub render_state: RenderState,
}

impl Default for PolylineMaterialAppearance {
    fn default() -> Self {
        Self::new(PolylineMaterialAppearanceOptions::default())
    }
}

impl PolylineMaterialAppearance {
    /// The compatible vertex format requiring position and st texture coordinates.
    pub const VERTEX_FORMAT: VertexFormat = VertexFormat::POSITION_AND_ST;

    /// Creates a new `PolylineMaterialAppearance`.
    pub fn new(options: PolylineMaterialAppearanceOptions) -> Self {
        let translucent = options.translucent.unwrap_or(true);
        let closed = false;

        let material = options
            .material
            .unwrap_or_else(|| Material::from_type(Material::COLOR_TYPE).unwrap_or_default());

        let render_state = options
            .render_state
            .unwrap_or_else(|| Appearance::get_default_render_state(translucent, closed, None));

        let base_vertex_shader_source = options
            .vertex_shader_source
            .unwrap_or_else(|| POLYLINE_MATERIAL_APPEARANCE_VS.to_string());

        let fragment_shader_source = options
            .fragment_shader_source
            .unwrap_or_else(|| POLYLINE_MATERIAL_APPEARANCE_FS.to_string());

        Self {
            material,
            translucent,
            closed,
            vertex_format: Self::VERTEX_FORMAT,
            base_vertex_shader_source,
            fragment_shader_source,
            render_state,
        }
    }

    /// The GLSL source code for the vertex shader, prepending #define POLYLINE_DASH if dashed material is used.
    pub fn vertex_shader_source(&self) -> String {
        if self.material.shader_source.contains("v_polylineAngle") {
            format!("#define POLYLINE_DASH\n{}", self.base_vertex_shader_source)
        } else {
            self.base_vertex_shader_source.clone()
        }
    }

    /// Determines if the geometry is translucent based on appearance and material translucency.
    pub fn is_translucent(&self) -> bool {
        self.translucent || self.material.translucent
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
