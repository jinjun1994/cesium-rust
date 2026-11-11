//! Checks if a specific extension is present on a 3D Tiles or glTF JSON object.
//! Adapted from CesiumJS `Scene/hasExtension.js` (Apache-2.0).

use serde_json::Value;

/// Checks if a specific extension is present on a JSON object.
/// This can be used for either 3D Tiles extensions or glTF extensions.
pub fn has_extension(json: &Value, extension_name: &str) -> bool {
    json.get("extensions")
        .and_then(|ext| ext.get(extension_name))
        .is_some()
}
