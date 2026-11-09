//! MaterialAppearance matching CesiumJS `Scene/MaterialAppearance.js` (Apache-2.0).

use crate::appearance::Appearance;
use crate::material::Material;
use crate::render_state::RenderState;
use cesium_math::VertexFormat;

/// Determines the type of material supported by a `MaterialAppearance` instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaterialSupport {
    /// Only basic materials requiring `position` and `normal` vertex attributes.
    Basic,
    /// Materials with textures requiring `position`, `normal`, and `st` vertex attributes.
    Textured,
    /// All materials, including tangent-space materials, requiring `position`, `normal`,
    /// `st`, `tangent`, and `bitangent` attributes.
    All,
}

impl MaterialSupport {
    /// Returns the vertex format required by this material support mode.
    pub const fn vertex_format(&self) -> VertexFormat {
        match self {
            Self::Basic => VertexFormat::POSITION_AND_NORMAL,
            Self::Textured => VertexFormat::POSITION_NORMAL_AND_ST,
            Self::All => VertexFormat::ALL,
        }
    }

    /// Returns the default GLSL vertex shader source.
    pub const fn vertex_shader_source(&self) -> &'static str {
        match self {
            Self::Basic => {
                "in vec3 position3DHigh;\n\
                 in vec3 position3DLow;\n\
                 in vec3 normal;\n\
                 in float batchId;\n\
                 \n\
                 out vec3 v_positionEC;\n\
                 out vec3 v_normalEC;\n\
                 \n\
                 void main()\n\
                 {\n\
                     vec4 p = czm_computePosition();\n\
                     \n\
                     v_positionEC = (czm_modelViewRelativeToEye * p).xyz;\n\
                     v_normalEC = czm_normal * normal;\n\
                     \n\
                     gl_Position = czm_modelViewProjectionRelativeToEye * p;\n\
                 }\n"
            }
            Self::Textured => {
                "in vec3 position3DHigh;\n\
                 in vec3 position3DLow;\n\
                 in vec3 normal;\n\
                 in vec2 st;\n\
                 in float batchId;\n\
                 \n\
                 out vec3 v_positionEC;\n\
                 out vec3 v_normalEC;\n\
                 out vec2 v_st;\n\
                 \n\
                 void main()\n\
                 {\n\
                     vec4 p = czm_computePosition();\n\
                     \n\
                     v_positionEC = (czm_modelViewRelativeToEye * p).xyz;\n\
                     v_normalEC = czm_normal * normal;\n\
                     v_st = st;\n\
                     \n\
                     gl_Position = czm_modelViewProjectionRelativeToEye * p;\n\
                 }\n"
            }
            Self::All => {
                "in vec3 position3DHigh;\n\
                 in vec3 position3DLow;\n\
                 in vec3 normal;\n\
                 in vec3 tangent;\n\
                 in vec3 bitangent;\n\
                 in vec2 st;\n\
                 in float batchId;\n\
                 \n\
                 out vec3 v_positionEC;\n\
                 out vec3 v_normalEC;\n\
                 out vec3 v_tangentEC;\n\
                 out vec3 v_bitangentEC;\n\
                 out vec2 v_st;\n\
                 \n\
                 void main()\n\
                 {\n\
                     vec4 p = czm_computePosition();\n\
                     \n\
                     v_positionEC = (czm_modelViewRelativeToEye * p).xyz;\n\
                     v_normalEC = czm_normal * normal;\n\
                     v_tangentEC = czm_normal * tangent;\n\
                     v_bitangentEC = czm_normal * bitangent;\n\
                     v_st = st;\n\
                     \n\
                     gl_Position = czm_modelViewProjectionRelativeToEye * p;\n\
                 }\n"
            }
        }
    }

    /// Returns the default GLSL fragment shader source.
    pub const fn fragment_shader_source(&self) -> &'static str {
        match self {
            Self::Basic => {
                "in vec3 v_positionEC;\n\
                 in vec3 v_normalEC;\n\
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
                     czm_materialInput materialInput;\n\
                     materialInput.normalEC = normalEC;\n\
                     materialInput.positionToEyeEC = positionToEyeEC;\n\
                     czm_material material = czm_getMaterial(materialInput);\n\
                     \n\
                 #ifdef FLAT\n\
                     out_FragColor = vec4(material.diffuse + material.emission, material.alpha);\n\
                 #else\n\
                     out_FragColor = czm_phong(normalize(positionToEyeEC), material, czm_lightDirectionEC);\n\
                 #endif\n\
                 }\n"
            }
            Self::Textured => {
                "in vec3 v_positionEC;\n\
                 in vec3 v_normalEC;\n\
                 in vec2 v_st;\n\
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
                     czm_materialInput materialInput;\n\
                     materialInput.normalEC = normalEC;\n\
                     materialInput.positionToEyeEC = positionToEyeEC;\n\
                     materialInput.st = v_st;\n\
                     czm_material material = czm_getMaterial(materialInput);\n\
                     \n\
                 #ifdef FLAT\n\
                     out_FragColor = vec4(material.diffuse + material.emission, material.alpha);\n\
                 #else\n\
                     out_FragColor = czm_phong(normalize(positionToEyeEC), material, czm_lightDirectionEC);\n\
                 #endif\n\
                 }\n"
            }
            Self::All => {
                "in vec3 v_positionEC;\n\
                 in vec3 v_normalEC;\n\
                 in vec3 v_tangentEC;\n\
                 in vec3 v_bitangentEC;\n\
                 in vec2 v_st;\n\
                 \n\
                 void main()\n\
                 {\n\
                     vec3 positionToEyeEC = -v_positionEC;\n\
                     mat3 tangentToEyeMatrix = czm_tangentToEyeSpaceMatrix(v_normalEC, v_tangentEC, v_bitangentEC);\n\
                     \n\
                     vec3 normalEC = normalize(v_normalEC);\n\
                 #ifdef FACE_FORWARD\n\
                     normalEC = faceforward(normalEC, vec3(0.0, 0.0, 1.0), -normalEC);\n\
                 #endif\n\
                     \n\
                     czm_materialInput materialInput;\n\
                     materialInput.normalEC = normalEC;\n\
                     materialInput.tangentToEyeMatrix = tangentToEyeMatrix;\n\
                     materialInput.positionToEyeEC = positionToEyeEC;\n\
                     materialInput.st = v_st;\n\
                     czm_material material = czm_getMaterial(materialInput);\n\
                     \n\
                 #ifdef FLAT\n\
                     out_FragColor = vec4(material.diffuse + material.emission, material.alpha);\n\
                 #else\n\
                     out_FragColor = czm_phong(normalize(positionToEyeEC), material, czm_lightDirectionEC);\n\
                 #endif\n\
                 }\n"
            }
        }
    }
}

/// Options for configuring a `MaterialAppearance`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MaterialAppearanceOptions {
    /// When true, flat shading is used in the fragment shader. Defaults to false.
    pub flat: Option<bool>,
    /// When true, surface normals are flipped to face the viewer. Defaults to `!closed`.
    pub face_forward: Option<bool>,
    /// When true, geometry is expected to appear translucent. Defaults to true.
    pub translucent: Option<bool>,
    /// When true, geometry is expected to be closed so backface culling is enabled. Defaults to false.
    pub closed: Option<bool>,
    /// The material support type. Defaults to `MaterialSupport::Textured`.
    pub material_support: Option<MaterialSupport>,
    /// The material used to determine fragment color. Defaults to Color material.
    pub material: Option<Material>,
    /// Optional GLSL vertex shader source.
    pub vertex_shader_source: Option<String>,
    /// Optional GLSL fragment shader source.
    pub fragment_shader_source: Option<String>,
    /// Optional render state override.
    pub render_state: Option<RenderState>,
}

/// An appearance for arbitrary geometry that supports shading with materials.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialAppearance {
    /// The material used to determine fragment color.
    pub material: Material,
    /// When true, geometry is expected to appear translucent.
    pub translucent: bool,
    /// The vertex shader source.
    pub vertex_shader_source: String,
    /// The fragment shader source.
    pub fragment_shader_source: String,
    /// The render state.
    pub render_state: RenderState,
    /// When true, geometry is expected to be closed.
    pub closed: bool,
    /// The material support type.
    pub material_support: MaterialSupport,
    /// The vertex format compatible with this appearance.
    pub vertex_format: VertexFormat,
    /// When true, flat shading is used.
    pub flat: bool,
    /// When true, fragment shader flips surface normal to face viewer.
    pub face_forward: bool,
}

impl Default for MaterialAppearance {
    fn default() -> Self {
        Self::new(MaterialAppearanceOptions::default())
    }
}

impl MaterialAppearance {
    /// Creates a new `MaterialAppearance`.
    pub fn new(options: MaterialAppearanceOptions) -> Self {
        let translucent = options.translucent.unwrap_or(true);
        let closed = options.closed.unwrap_or(false);
        let material_support = options
            .material_support
            .unwrap_or(MaterialSupport::Textured);
        let flat = options.flat.unwrap_or(false);
        let face_forward = options.face_forward.unwrap_or(!closed);

        let material = options
            .material
            .unwrap_or_else(|| Material::from_type(Material::COLOR_TYPE).unwrap());

        let vertex_shader_source = options
            .vertex_shader_source
            .unwrap_or_else(|| material_support.vertex_shader_source().to_string());
        let fragment_shader_source = options
            .fragment_shader_source
            .unwrap_or_else(|| material_support.fragment_shader_source().to_string());

        let render_state = Appearance::get_default_render_state(
            translucent,
            closed,
            options.render_state,
        );

        let vertex_format = material_support.vertex_format();

        Self {
            material,
            translucent,
            vertex_shader_source,
            fragment_shader_source,
            render_state,
            closed,
            material_support,
            vertex_format,
            flat,
            face_forward,
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
        parts.push(self.material.shader_source.clone());
        parts.push(self.fragment_shader_source.clone());
        parts.join("\n")
    }

    /// Determines if the geometry is translucent based on translucency and material.
    pub fn is_translucent(&self) -> bool {
        self.material.is_translucent()
    }

    /// Creates or retrieves the active render state.
    pub fn get_render_state(&self) -> RenderState {
        let mut rs = self.render_state.clone();
        if self.is_translucent() {
            rs.depth_mask = false;
            rs.blending = crate::render_state::BlendingState::ALPHA_BLEND;
        } else {
            rs.depth_mask = true;
            rs.blending = crate::render_state::BlendingState::DISABLED;
        }
        rs
    }

    /// Returns the vertex format compatible with this appearance.
    pub const fn vertex_format(&self) -> VertexFormat {
        self.vertex_format
    }
}
