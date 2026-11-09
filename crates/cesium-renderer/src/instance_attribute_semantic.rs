use serde::{Deserialize, Serialize};

/// Built-in instance attribute semantics in glTF EXT_mesh_gpu_instancing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InstanceAttributeSemantic {
    /// Per-instance translation.
    Translation,
    /// Per-instance rotation (quaternion).
    Rotation,
    /// Per-instance scale.
    Scale,
    /// Per-instance feature ID.
    FeatureId,
}

impl InstanceAttributeSemantic {
    /// String representation as used in glTF semantics.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Translation => "TRANSLATION",
            Self::Rotation => "ROTATION",
            Self::Scale => "SCALE",
            Self::FeatureId => "_FEATURE_ID",
        }
    }

    /// Gets the instance attribute semantic matching a glTF attribute semantic.
    ///
    /// If the semantic has a numeric set index suffix (e.g. `_FEATURE_ID_0`), the suffix is stripped.
    pub fn from_gltf_semantic(gltf_semantic: &str) -> Option<Self> {
        let base_semantic = if let Some(last_underscore_idx) = gltf_semantic.rfind('_') {
            let suffix = &gltf_semantic[last_underscore_idx + 1..];
            if !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
                &gltf_semantic[..last_underscore_idx]
            } else {
                gltf_semantic
            }
        } else {
            gltf_semantic
        };

        match base_semantic {
            "TRANSLATION" => Some(Self::Translation),
            "ROTATION" => Some(Self::Rotation),
            "SCALE" => Some(Self::Scale),
            "_FEATURE_ID" => Some(Self::FeatureId),
            _ => None,
        }
    }
}
