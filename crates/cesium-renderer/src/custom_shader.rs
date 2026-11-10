//! CustomShader matching CesiumJS `Scene/Model/CustomShader.js` (Apache-2.0).

use cesium_math::{
    Cartesian2, Cartesian3, Cartesian4, DeveloperError, Matrix2, Matrix3, Matrix4,
};
use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, RwLock};

/// Custom shader mode determining how code interacts with materials.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CustomShaderMode {
    #[default]
    ModifyMaterial,
    ReplaceMaterial,
}

impl CustomShaderMode {
    pub fn get_define_name(self) -> &'static str {
        match self {
            Self::ModifyMaterial => "CUSTOM_SHADER_MODIFY_MATERIAL",
            Self::ReplaceMaterial => "CUSTOM_SHADER_REPLACE_MATERIAL",
        }
    }
}

/// Translucency mode for custom shader.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CustomShaderTranslucencyMode {
    #[default]
    Inherit = 0,
    Opaque = 1,
    Translucent = 2,
}

/// Lighting model for custom shader.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LightingModel {
    Unlit = 0,
    #[default]
    Pbr = 1,
}

/// Supported GLSL uniform types for custom shaders.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CustomShaderUniformType {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Int,
    IntVec2,
    IntVec3,
    IntVec4,
    Bool,
    BoolVec2,
    BoolVec3,
    BoolVec4,
    Mat2,
    Mat3,
    Mat4,
    Sampler2D,
    SamplerCube,
}

impl CustomShaderUniformType {
    pub fn glsl_type(self) -> &'static str {
        match self {
            Self::Float => "float",
            Self::Vec2 => "vec2",
            Self::Vec3 => "vec3",
            Self::Vec4 => "vec4",
            Self::Int => "int",
            Self::IntVec2 => "ivec2",
            Self::IntVec3 => "ivec3",
            Self::IntVec4 => "ivec4",
            Self::Bool => "bool",
            Self::BoolVec2 => "bvec2",
            Self::BoolVec3 => "bvec3",
            Self::BoolVec4 => "bvec4",
            Self::Mat2 => "mat2",
            Self::Mat3 => "mat3",
            Self::Mat4 => "mat4",
            Self::Sampler2D => "sampler2D",
            Self::SamplerCube => "samplerCube",
        }
    }
}

/// Supported GLSL varying types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VaryingType {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Mat2,
    Mat3,
    Mat4,
}

impl VaryingType {
    pub fn glsl_type(self) -> &'static str {
        match self {
            Self::Float => "float",
            Self::Vec2 => "vec2",
            Self::Vec3 => "vec3",
            Self::Vec4 => "vec4",
            Self::Mat2 => "mat2",
            Self::Mat3 => "mat3",
            Self::Mat4 => "mat4",
        }
    }
}

/// Struct representing a texture uniform value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureUniform {
    pub url: Option<String>,
    pub typed_array: Option<Vec<u8>>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub repeat: bool,
}

impl TextureUniform {
    pub fn from_url(url: impl Into<String>) -> Self {
        Self {
            url: Some(url.into()),
            typed_array: None,
            width: None,
            height: None,
            repeat: true,
        }
    }

    pub fn from_bytes(bytes: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            url: None,
            typed_array: Some(bytes),
            width: Some(width),
            height: Some(height),
            repeat: true,
        }
    }
}

/// Value of a custom shader uniform.
#[derive(Clone, Debug, PartialEq)]
pub enum CustomShaderValue {
    Float(f64),
    Vec2(Cartesian2),
    Vec3(Cartesian3),
    Vec4(Cartesian4),
    Int(i32),
    IntVec2([i32; 2]),
    IntVec3([i32; 3]),
    IntVec4([i32; 4]),
    Bool(bool),
    BoolVec2([bool; 2]),
    BoolVec3([bool; 3]),
    BoolVec4([bool; 4]),
    Mat2(Matrix2),
    Mat3(Matrix3),
    Mat4(Matrix4),
    Texture(TextureUniform),
}

/// Uniform specification for CustomShader.
#[derive(Clone, Debug, PartialEq)]
pub struct UniformSpecifier {
    pub uniform_type: CustomShaderUniformType,
    pub value: CustomShaderValue,
}

/// Variable sets detected in vertex shader code.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VertexVariableSets {
    pub attribute_set: BTreeSet<String>,
    pub feature_id_set: BTreeSet<String>,
    pub metadata_set: BTreeSet<String>,
}

/// Variable sets detected in fragment shader code.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FragmentVariableSets {
    pub attribute_set: BTreeSet<String>,
    pub feature_id_set: BTreeSet<String>,
    pub metadata_set: BTreeSet<String>,
    pub material_set: BTreeSet<String>,
}

/// Options to instantiate a CustomShader.
#[derive(Clone, Debug, Default)]
pub struct CustomShaderOptions {
    pub mode: Option<CustomShaderMode>,
    pub lighting_model: Option<LightingModel>,
    pub translucency_mode: Option<CustomShaderTranslucencyMode>,
    pub uniforms: HashMap<String, UniformSpecifier>,
    pub varyings: HashMap<String, VaryingType>,
    pub vertex_shader_text: Option<String>,
    pub fragment_shader_text: Option<String>,
}

pub type CustomShaderUniformCallback = Arc<dyn Fn() -> CustomShaderValue + Send + Sync>;

/// User defined GLSL shader used with Model and Cesium3DTileset.
#[derive(Clone)]
pub struct CustomShader {
    pub mode: CustomShaderMode,
    pub lighting_model: Option<LightingModel>,
    pub translucency_mode: CustomShaderTranslucencyMode,
    pub uniforms: Arc<RwLock<HashMap<String, UniformSpecifier>>>,
    pub varyings: HashMap<String, VaryingType>,
    pub vertex_shader_text: Option<String>,
    pub fragment_shader_text: Option<String>,
    pub used_variables_vertex: VertexVariableSets,
    pub used_variables_fragment: FragmentVariableSets,
    pub uniform_map: HashMap<String, CustomShaderUniformCallback>,
    is_destroyed: Arc<RwLock<bool>>,
}

impl std::fmt::Debug for CustomShader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomShader")
            .field("mode", &self.mode)
            .field("lighting_model", &self.lighting_model)
            .field("translucency_mode", &self.translucency_mode)
            .field("varyings", &self.varyings)
            .field("vertex_shader_text", &self.vertex_shader_text)
            .field("fragment_shader_text", &self.fragment_shader_text)
            .finish()
    }
}

fn expand_coordinate_abbreviations(name: &str) -> String {
    if name.ends_with("MC") {
        format!("{name} (model coordinates)")
    } else if name.ends_with("WC") {
        format!("{name} (Cartesian world coordinates)")
    } else if name.ends_with("EC") {
        format!("{name} (eye coordinates)")
    } else {
        name.to_string()
    }
}

fn validate_variable_usage(
    variable_set: &BTreeSet<String>,
    incorrect: &str,
    correct: &str,
    stage: &str,
) -> Result<(), DeveloperError> {
    if variable_set.contains(incorrect) {
        let msg = format!(
            "{} is not available in the {} shader. Did you mean {} instead?",
            expand_coordinate_abbreviations(incorrect),
            stage,
            expand_coordinate_abbreviations(correct)
        );
        return Err(DeveloperError::new(msg));
    }
    Ok(())
}

fn extract_identifiers_after_prefixes(text: &str, prefixes: &[&str], output: &mut BTreeSet<String>) {
    let bytes = text.as_bytes();
    for prefix in prefixes {
        let p_bytes = prefix.as_bytes();
        let mut idx = 0;
        while let Some(pos) = text[idx..].find(prefix) {
            let start = idx + pos + p_bytes.len();
            let mut end = start;
            while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                end += 1;
            }
            if end > start {
                output.insert(text[start..end].to_string());
            }
            idx = end.max(idx + pos + 1);
        }
    }
}

fn find_used_variables(
    vs_text: Option<&str>,
    fs_text: Option<&str>,
) -> (VertexVariableSets, FragmentVariableSets) {
    let attr_prefixes = ["vsInput.attributes.", "fsInput.attributes."];
    let feat_prefixes = ["vsInput.featureIds.", "fsInput.featureIds."];
    let meta_prefixes = [
        "vsInput.metadata.",
        "fsInput.metadata.",
        "vsInput.metadataClass.",
        "fsInput.metadataClass.",
        "vsInput.metadataStatistics.",
        "fsInput.metadataStatistics.",
    ];
    let mat_prefixes = ["material."];

    let mut vs_vars = VertexVariableSets::default();
    if let Some(text) = vs_text {
        extract_identifiers_after_prefixes(text, &attr_prefixes, &mut vs_vars.attribute_set);
        extract_identifiers_after_prefixes(text, &feat_prefixes, &mut vs_vars.feature_id_set);
        extract_identifiers_after_prefixes(text, &meta_prefixes, &mut vs_vars.metadata_set);
    }

    let mut fs_vars = FragmentVariableSets::default();
    if let Some(text) = fs_text {
        extract_identifiers_after_prefixes(text, &attr_prefixes, &mut fs_vars.attribute_set);
        extract_identifiers_after_prefixes(text, &feat_prefixes, &mut fs_vars.feature_id_set);
        extract_identifiers_after_prefixes(text, &meta_prefixes, &mut fs_vars.metadata_set);
        extract_identifiers_after_prefixes(text, &mat_prefixes, &mut fs_vars.material_set);
    }

    (vs_vars, fs_vars)
}

fn validate_builtin_variables(
    vs_vars: &VertexVariableSets,
    fs_vars: &FragmentVariableSets,
) -> Result<(), DeveloperError> {
    let vs = &vs_vars.attribute_set;
    validate_variable_usage(vs, "position", "positionMC", "vertex")?;
    validate_variable_usage(vs, "normal", "normalMC", "vertex")?;
    validate_variable_usage(vs, "tangent", "tangentMC", "vertex")?;
    validate_variable_usage(vs, "bitangent", "bitangentMC", "vertex")?;
    validate_variable_usage(vs, "positionWC", "positionMC", "vertex")?;
    validate_variable_usage(vs, "positionEC", "positionMC", "vertex")?;
    validate_variable_usage(vs, "normalEC", "normalMC", "vertex")?;
    validate_variable_usage(vs, "tangentEC", "tangentMC", "vertex")?;
    validate_variable_usage(vs, "bitangentEC", "bitangentMC", "vertex")?;

    let fs = &fs_vars.attribute_set;
    validate_variable_usage(fs, "position", "positionEC", "fragment")?;
    validate_variable_usage(fs, "normal", "normalEC", "fragment")?;
    validate_variable_usage(fs, "tangent", "tangentEC", "fragment")?;
    validate_variable_usage(fs, "bitangent", "bitangentEC", "fragment")?;
    validate_variable_usage(fs, "normalMC", "normalEC", "fragment")?;
    validate_variable_usage(fs, "tangentMC", "tangentEC", "fragment")?;
    validate_variable_usage(fs, "bitangentMC", "bitangentEC", "fragment")?;

    Ok(())
}

impl CustomShader {
    /// Constructs a new CustomShader.
    pub fn new(options: CustomShaderOptions) -> Result<Self, DeveloperError> {
        let mode = options.mode.unwrap_or(CustomShaderMode::ModifyMaterial);
        let lighting_model = options.lighting_model;
        let translucency_mode = options
            .translucency_mode
            .unwrap_or(CustomShaderTranslucencyMode::Inherit);

        for (name, spec) in &options.uniforms {
            if spec.uniform_type == CustomShaderUniformType::SamplerCube {
                return Err(DeveloperError::new(
                    "CustomShader does not support samplerCube uniforms",
                ));
            }
            let _ = name;
        }

        let (used_variables_vertex, used_variables_fragment) = find_used_variables(
            options.vertex_shader_text.as_deref(),
            options.fragment_shader_text.as_deref(),
        );

        validate_builtin_variables(&used_variables_vertex, &used_variables_fragment)?;

        let uniforms_arc = Arc::new(RwLock::new(options.uniforms));

        let mut uniform_map: HashMap<String, CustomShaderUniformCallback> = HashMap::new();
        {
            let uniforms_guard = uniforms_arc.read().unwrap();
            for uniform_name in uniforms_guard.keys() {
                let name = uniform_name.clone();
                let uniforms_ref = Arc::clone(&uniforms_arc);
                let cb: CustomShaderUniformCallback = Arc::new(move || {
                    let guard = uniforms_ref.read().unwrap();
                    guard.get(&name).unwrap().value.clone()
                });
                uniform_map.insert(uniform_name.clone(), cb);
            }
        }

        Ok(Self {
            mode,
            lighting_model,
            translucency_mode,
            uniforms: uniforms_arc,
            varyings: options.varyings,
            vertex_shader_text: options.vertex_shader_text,
            fragment_shader_text: options.fragment_shader_text,
            used_variables_vertex,
            used_variables_fragment,
            uniform_map,
            is_destroyed: Arc::new(RwLock::new(false)),
        })
    }

    /// Updates the value of a declared uniform.
    pub fn set_uniform(
        &self,
        uniform_name: &str,
        value: CustomShaderValue,
    ) -> Result<(), DeveloperError> {
        if *self.is_destroyed.read().unwrap() {
            return Err(DeveloperError::new("This object was destroyed."));
        }

        let mut uniforms_guard = self.uniforms.write().unwrap();
        let uniform = uniforms_guard.get_mut(uniform_name).ok_or_else(|| {
            DeveloperError::new(format!(
                "Uniform {uniform_name} must be declared in the CustomShader constructor."
            ))
        })?;

        uniform.value = value;
        Ok(())
    }

    pub fn is_destroyed(&self) -> bool {
        *self.is_destroyed.read().unwrap()
    }

    pub fn destroy(&mut self) {
        *self.is_destroyed.write().unwrap() = true;
    }
}
