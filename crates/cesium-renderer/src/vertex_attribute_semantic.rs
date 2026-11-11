//! VertexAttributeSemantic matching CesiumJS `Scene/VertexAttributeSemantic.js` (Apache-2.0).

use cesium_math::DeveloperError;

/// An enum/namespace describing the built-in vertex attribute semantics.
pub struct VertexAttributeSemantic;

impl VertexAttributeSemantic {
    pub const POSITION: &'static str = "POSITION";
    pub const NORMAL: &'static str = "NORMAL";
    pub const TANGENT: &'static str = "TANGENT";
    pub const TEXCOORD: &'static str = "TEXCOORD";
    pub const COLOR: &'static str = "COLOR";
    pub const JOINTS: &'static str = "JOINTS";
    pub const WEIGHTS: &'static str = "WEIGHTS";
    pub const FEATURE_ID: &'static str = "_FEATURE_ID";
    pub const SCALE: &'static str = "KHR_gaussian_splatting:SCALE";
    pub const ROTATION: &'static str = "KHR_gaussian_splatting:ROTATION";
    pub const CUMULATIVE_DISTANCE: &'static str = "BENTLEY_materials_line_style:CUMULATIVE_DISTANCE";
    pub const OPACITY: &'static str = "KHR_gaussian_splatting:OPACITY";

    /// Returns whether the vertex attribute semantic can have a set index.
    pub fn has_set_index(semantic: &str) -> Result<bool, DeveloperError> {
        match semantic {
            Self::POSITION | Self::NORMAL | Self::TANGENT | Self::CUMULATIVE_DISTANCE => Ok(false),
            Self::TEXCOORD
            | Self::COLOR
            | Self::JOINTS
            | Self::WEIGHTS
            | Self::FEATURE_ID
            | Self::SCALE
            | Self::ROTATION => Ok(true),
            _ => Err(DeveloperError::new("semantic is not a valid value.")),
        }
    }

    /// Gets the vertex attribute semantic matching the glTF semantic.
    pub fn from_gltf_semantic(gltf_semantic: &str) -> Result<Option<&'static str>, DeveloperError> {
        let mut semantic = gltf_semantic;

        // Strip the set index from the semantic (e.g. TEXCOORD_0 -> TEXCOORD)
        if let Some(pos) = gltf_semantic.rfind('_') {
            let prefix = &gltf_semantic[..pos];
            let suffix = &gltf_semantic[pos + 1..];
            if !prefix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
                semantic = prefix;
            }
        }

        let result = match semantic {
            "POSITION" => Some(Self::POSITION),
            "NORMAL" => Some(Self::NORMAL),
            "TANGENT" => Some(Self::TANGENT),
            "TEXCOORD" => Some(Self::TEXCOORD),
            "COLOR" => Some(Self::COLOR),
            "JOINTS" => Some(Self::JOINTS),
            "WEIGHTS" => Some(Self::WEIGHTS),
            "_FEATURE_ID" => Some(Self::FEATURE_ID),
            "KHR_gaussian_splatting:SCALE" | "_SCALE" => Some(Self::SCALE),
            "KHR_gaussian_splatting:ROTATION" | "_ROTATION" => Some(Self::ROTATION),
            "BENTLEY_materials_line_style:CUMULATIVE_DISTANCE" => Some(Self::CUMULATIVE_DISTANCE),
            _ => None,
        };

        Ok(result)
    }

    /// Gets the vertex attribute semantic matching the pnts semantic.
    pub fn from_pnts_semantic(pnts_semantic: &str) -> Result<&'static str, DeveloperError> {
        match pnts_semantic {
            "POSITION" | "POSITION_QUANTIZED" => Ok(Self::POSITION),
            "RGBA" | "RGB" | "RGB565" => Ok(Self::COLOR),
            "NORMAL" | "NORMAL_OCT16P" => Ok(Self::NORMAL),
            "BATCH_ID" => Ok(Self::FEATURE_ID),
            _ => Err(DeveloperError::new("pntsSemantic is not a valid value.")),
        }
    }

    /// Gets the GLSL type (such as `vec3` or `int`) for the given vertex attribute.
    pub fn get_glsl_type(semantic: &str) -> Result<&'static str, DeveloperError> {
        match semantic {
            Self::POSITION | Self::NORMAL | Self::TANGENT | Self::SCALE => Ok("vec3"),
            Self::TEXCOORD => Ok("vec2"),
            Self::COLOR | Self::WEIGHTS | Self::ROTATION => Ok("vec4"),
            Self::JOINTS => Ok("ivec4"),
            Self::FEATURE_ID => Ok("int"),
            Self::CUMULATIVE_DISTANCE | Self::OPACITY => Ok("float"),
            _ => Err(DeveloperError::new("semantic is not a valid value.")),
        }
    }

    /// Gets the variable name for the given semantic and set index.
    pub fn get_variable_name(
        semantic: &str,
        set_index: Option<usize>,
    ) -> Result<String, DeveloperError> {
        let base_name = match semantic {
            Self::POSITION => "positionMC",
            Self::NORMAL => "normalMC",
            Self::TANGENT => "tangentMC",
            Self::TEXCOORD => "texCoord",
            Self::COLOR => "color",
            Self::JOINTS => "joints",
            Self::WEIGHTS => "weights",
            Self::FEATURE_ID => "featureId",
            Self::SCALE => "scale",
            Self::ROTATION => "rotation",
            Self::CUMULATIVE_DISTANCE => "cumulativeDistance",
            _ => return Err(DeveloperError::new("semantic is not a valid value.")),
        };

        if let Some(idx) = set_index {
            Ok(format!("{base_name}_{idx}"))
        } else {
            Ok(base_name.to_string())
        }
    }
}
