//! ModelUtility functions matching CesiumJS `Scene/Model/ModelUtility.js` (Apache-2.0).

use cesium_math::{Cartesian3, DeveloperError, Matrix4, PrimitiveType, Quaternion};

use crate::axis::Axis;
use crate::render_state::CullFace;

/// Supported glTF / Cesium extensions.
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "AGI_articulations",
    "CESIUM_primitive_outline",
    "CESIUM_mesh_vector",
    "EXT_mesh_features",
    "EXT_structural_metadata",
    "EXT_primitive_outline",
    "EXT_mesh_gpu_instancing",
    "KHR_materials_unlit",
    "KHR_materials_pbrSpecularGlossiness",
    "KHR_texture_transform",
    "KHR_mesh_quantization",
    "KHR_draco_mesh_compression",
];

/// Attribute types for glTF and model pipeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttributeType {
    Scalar,
    Vec2,
    Vec3,
    Vec4,
    Mat2,
    Mat3,
    Mat4,
}

impl AttributeType {
    pub fn glsl_type(&self) -> &'static str {
        match self {
            Self::Scalar => "float",
            Self::Vec2 => "vec2",
            Self::Vec3 => "vec3",
            Self::Vec4 => "vec4",
            Self::Mat2 => "mat2",
            Self::Mat3 => "mat3",
            Self::Mat4 => "mat4",
        }
    }
}

/// Quantization information for an attribute.
#[derive(Clone, Debug, PartialEq)]
pub struct AttributeQuantization {
    pub attribute_type: AttributeType,
}

/// Description of a model vertex or instance attribute.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelAttributeDescription {
    pub name: Option<String>,
    pub semantic: Option<String>,
    pub set_index: Option<usize>,
    pub attribute_type: AttributeType,
    pub min: Option<Cartesian3>,
    pub max: Option<Cartesian3>,
    pub quantization: Option<AttributeQuantization>,
}

/// Output info about an attribute for shader generation.
#[derive(Clone, Debug, PartialEq)]
pub struct AttributeInfo {
    pub is_quantized: bool,
    pub variable_name: String,
    pub has_semantic: bool,
    pub glsl_type: String,
    pub quantized_glsl_type: Option<String>,
}

/// Feature ID description for label lookups.
#[derive(Clone, Debug, PartialEq)]
pub struct FeatureIdSetInfo {
    pub label: Option<String>,
    pub positional_label: Option<String>,
}

/// Utility functions for model pipelines and scene graph.
pub struct ModelUtility;

impl ModelUtility {
    /// Computes the node transform from either an explicit matrix or TRS components.
    pub fn get_node_transform(
        matrix: Option<Matrix4>,
        translation: Option<Cartesian3>,
        rotation: Option<Quaternion>,
        scale: Option<Cartesian3>,
    ) -> Matrix4 {
        if let Some(m) = matrix {
            return m;
        }
        let t = translation.unwrap_or(Cartesian3::ZERO);
        let r = rotation.unwrap_or(Quaternion::IDENTITY);
        let s = scale.unwrap_or(Cartesian3::new(1.0, 1.0, 1.0));
        Matrix4::from_translation_quaternion_rotation_scale(t, r, s)
    }

    /// Checks if any attributes are quantized.
    pub fn has_quantized_attributes(attributes: Option<&[ModelAttributeDescription]>) -> bool {
        match attributes {
            Some(attrs) => attrs.iter().any(|a| a.quantization.is_some()),
            None => false,
        }
    }

    /// Derives GLSL attribute information for an attribute.
    pub fn get_attribute_info(attribute: &ModelAttributeDescription) -> AttributeInfo {
        let is_quantized = attribute.quantization.is_some();
        let (variable_name, has_semantic, base_glsl_type) = if let Some(semantic) = &attribute.semantic {
            match semantic.as_str() {
                "POSITION" => ("positionMC".to_string(), true, "vec3"),
                "NORMAL" => ("normalMC".to_string(), true, "vec3"),
                "TANGENT" => ("tangentMC".to_string(), true, "vec4"),
                "TEXCOORD" => {
                    let idx = attribute.set_index.unwrap_or(0);
                    (format!("texCoord_{}", idx), true, "vec2")
                }
                "COLOR" => {
                    let idx = attribute.set_index.unwrap_or(0);
                    // Vertex colors are promoted to vec4 in GLSL
                    (format!("color_{}", idx), true, "vec4")
                }
                _ => {
                    let name = semantic.to_lowercase();
                    (name, true, attribute.attribute_type.glsl_type())
                }
            }
        } else {
            let raw_name = attribute.name.as_deref().unwrap_or("attribute");
            let clean_name = raw_name.trim_start_matches('_').to_lowercase();
            (clean_name, false, attribute.attribute_type.glsl_type())
        };

        let quantized_glsl_type = attribute.quantization.as_ref().map(|q| {
            if attribute.semantic.as_deref() == Some("COLOR") {
                "vec4".to_string()
            } else {
                q.attribute_type.glsl_type().to_string()
            }
        });

        AttributeInfo {
            is_quantized,
            variable_name,
            has_semantic,
            glsl_type: base_glsl_type.to_string(),
            quantized_glsl_type,
        }
    }

    /// Gets min and max position bounds, adding instancing min/max offsets if present.
    pub fn get_position_min_max(
        min: Cartesian3,
        max: Cartesian3,
        instancing_min: Option<Cartesian3>,
        instancing_max: Option<Cartesian3>,
    ) -> (Cartesian3, Cartesian3) {
        if let (Some(i_min), Some(i_max)) = (instancing_min, instancing_max) {
            (min + i_min, max + i_max)
        } else {
            (min, max)
        }
    }

    /// Computes the axis correction matrix given upAxis and forwardAxis.
    pub fn get_axis_correction_matrix(up_axis: Axis, forward_axis: Axis) -> Matrix4 {
        let mut result = Matrix4::IDENTITY;

        if up_axis == Axis::Y {
            result = Axis::Y_UP_TO_Z_UP;
        } else if up_axis == Axis::X {
            result = Axis::X_UP_TO_Z_UP;
        }

        if forward_axis == Axis::Z {
            result = result.multiply(&Axis::Z_UP_TO_X_UP);
        }

        result
    }

    /// Searches attributes for one matching semantic and optional set_index.
    pub fn get_attribute_by_semantic<'a>(
        attributes: &'a [ModelAttributeDescription],
        semantic: &str,
        set_index: Option<usize>,
    ) -> Option<&'a ModelAttributeDescription> {
        attributes.iter().find(|a| {
            if a.semantic.as_deref() != Some(semantic) {
                return false;
            }
            if let Some(expected_set) = set_index {
                a.set_index == Some(expected_set)
            } else {
                true
            }
        })
    }

    /// Gets a feature ID set by label or positional label.
    pub fn get_feature_ids_by_label<'a>(
        feature_ids: &'a [FeatureIdSetInfo],
        label: &str,
    ) -> Option<&'a FeatureIdSetInfo> {
        feature_ids.iter().find(|f| {
            if let Some(l) = &f.label {
                if l == label {
                    return true;
                }
            }
            if let Some(pl) = &f.positional_label {
                if pl == label {
                    return true;
                }
            }
            false
        })
    }

    /// Determines cull face based on the model matrix and primitive type.
    pub fn get_cull_face(model_matrix: &Matrix4, primitive_type: PrimitiveType) -> CullFace {
        if !primitive_type.is_triangles() {
            return CullFace::Back;
        }
        let m3 = model_matrix.get_matrix3();
        if m3.determinant() < 0.0 {
            CullFace::Front
        } else {
            CullFace::Back
        }
    }

    /// Sanitize identifier to be a valid GLSL identifier:
    /// - Non-alphanumeric character sequences are replaced with a single `_`.
    /// - Remove `gl_` prefix if present (reserved in GLSL).
    /// - If the identifier begins with a digit, prefix with `_`.
    pub fn sanitize_glsl_identifier(identifier: &str) -> String {
        let mut cleaned = String::with_capacity(identifier.len());
        let mut in_underscore = false;

        for ch in identifier.chars() {
            if ch.is_ascii_alphanumeric() {
                cleaned.push(ch);
                in_underscore = false;
            } else if !in_underscore {
                cleaned.push('_');
                in_underscore = true;
            }
        }

        let stripped = cleaned.strip_prefix("gl_").unwrap_or(&cleaned);
        if let Some(first) = stripped.chars().next() {
            if first.is_ascii_digit() {
                return format!("_{}", stripped);
            }
        }
        stripped.to_string()
    }

    /// Checks that all required extensions are supported by Cesium.
    pub fn check_supported_extensions(extensions_required: &[&str]) -> Result<(), DeveloperError> {
        for ext in extensions_required {
            if !SUPPORTED_EXTENSIONS.contains(ext) {
                return Err(DeveloperError::new(format!("Unsupported extension: {}", ext)));
            }
        }
        Ok(())
    }
}
