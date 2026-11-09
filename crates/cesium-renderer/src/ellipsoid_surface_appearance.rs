use crate::appearance::Appearance;
use crate::material::Material;
use crate::render_state::RenderState;
use cesium_math::VertexFormat;

/// Initialization options for `EllipsoidSurfaceAppearance`.
#[derive(Debug, Clone, Default)]
pub struct EllipsoidSurfaceAppearanceOptions {
    /// When true, flat shading is used in the fragment shader. Defaults to false.
    pub flat: Option<bool>,
    /// When true, surface normals are flipped to face the viewer. Defaults to above_ground.
    pub face_forward: Option<bool>,
    /// When true, the geometry is expected to appear translucent. Defaults to true.
    pub translucent: Option<bool>,
    /// When true, the geometry is expected to be on the ellipsoid surface. Defaults to false.
    pub above_ground: Option<bool>,
    /// The material used to determine the fragment color. Defaults to Color material.
    pub material: Option<Material>,
    /// Optional vertex shader source.
    pub vertex_shader_source: Option<String>,
    /// Optional fragment shader source.
    pub fragment_shader_source: Option<String>,
    /// Optional render state override.
    pub render_state: Option<RenderState>,
}

/// An appearance for geometry on the surface of the ellipsoid (such as PolygonGeometry and RectangleGeometry).
#[derive(Debug, Clone)]
pub struct EllipsoidSurfaceAppearance {
    /// The material used to determine the fragment color.
    pub material: Material,
    /// Whether the geometry appears translucent.
    pub translucent: bool,
    /// The GLSL vertex shader source.
    pub vertex_shader_source: String,
    /// The GLSL fragment shader source.
    pub fragment_shader_source: String,
    /// The render state.
    pub render_state: RenderState,
    /// Whether the geometry is expected to be closed (always false).
    pub closed: bool,
    /// Whether flat shading is used.
    pub flat: bool,
    /// Whether surface normals face forward.
    pub face_forward: bool,
    /// Whether the geometry is above ground.
    pub above_ground: bool,
}

impl Default for EllipsoidSurfaceAppearance {
    fn default() -> Self {
        Self::new(EllipsoidSurfaceAppearanceOptions::default())
    }
}

impl EllipsoidSurfaceAppearance {
    /// The vertex format compatible with all EllipsoidSurfaceAppearance instances (Position and ST).
    pub const VERTEX_FORMAT: VertexFormat = VertexFormat::POSITION_AND_ST;

    /// Creates a new `EllipsoidSurfaceAppearance`.
    pub fn new(options: EllipsoidSurfaceAppearanceOptions) -> Self {
        let translucent = options.translucent.unwrap_or(true);
        let above_ground = options.above_ground.unwrap_or(false);
        let flat = options.flat.unwrap_or(false);
        let face_forward = options.face_forward.unwrap_or(above_ground);

        let material = options
            .material
            .unwrap_or_else(|| Material::from_type(Material::COLOR_TYPE).unwrap());

        let vs = options.vertex_shader_source.unwrap_or_else(|| {
            "in vec3 position3DHigh;\nin vec3 position3DLow;\nin vec2 st;\nin float batchId;\nout vec3 v_positionEC;\nout vec2 v_st;\nvoid main()\n{\nvec4 p = czm_translateRelativeToEye(position3DHigh, position3DLow);\nv_positionEC = (czm_modelViewRelativeToEye * p).xyz;\nv_st = st;\ngl_Position = czm_modelViewProjectionRelativeToEye * p;\n}".to_string()
        });

        let fs = options.fragment_shader_source.unwrap_or_else(|| {
            "in vec3 v_positionEC;\nin vec2 v_st;\nvoid main()\n{\nczm_materialInput materialInput;\nmaterialInput.s = v_st.s;\nmaterialInput.st = v_st;\nmaterialInput.str = vec3(v_st, 0.0);\nczm_material material = czm_getMaterial(materialInput);\nout_FragColor = vec4(material.diffuse + material.emission, material.alpha);\n}".to_string()
        });

        let render_state = Appearance::get_default_render_state(
            translucent,
            !above_ground,
            options.render_state,
        );

        Self {
            material,
            translucent,
            vertex_shader_source: vs,
            fragment_shader_source: fs,
            render_state,
            closed: false,
            flat,
            face_forward,
            above_ground,
        }
    }

    /// Returns the vertex format compatible with this appearance.
    pub fn vertex_format(&self) -> VertexFormat {
        Self::VERTEX_FORMAT
    }

    /// Determines if the geometry is translucent.
    pub fn is_translucent(&self) -> bool {
        self.translucent && self.material.is_translucent()
    }

    /// Gets the render state.
    pub fn get_render_state(&self) -> &RenderState {
        &self.render_state
    }
}
