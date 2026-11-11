//! Preprocesses 3D Tile content to determine content type and parse payload.
//! Adapted from CesiumJS `Scene/preprocess3DTileContent.js` (Apache-2.0).

use cesium_math::RuntimeError;
use serde_json::Value;

use crate::cesium_3d_tile_content_type::Cesium3DTileContentType;
use crate::has_extension::has_extension;

const GLB_JSON_CHUNK_TYPE: u32 = 0x4e4f534a; // "JSON"

/// Results of the `preprocess_3d_tile_content()` function.
#[derive(Debug, Clone, PartialEq)]
pub struct PreprocessedContent {
    /// The content type identifier (e.g. "b3dm", "glb", "externalTileset", etc.)
    pub content_type: String,
    /// For binary files, the binary payload.
    pub binary_payload: Option<Vec<u8>>,
    /// For JSON files, the parsed JSON payload.
    pub json_payload: Option<Value>,
}

/// Preprocesses 3D tile content to determine the type of content
/// and to parse JSON files into structured objects.
pub fn preprocess_3d_tile_content(data: &[u8]) -> Result<PreprocessedContent, RuntimeError> {
    let magic = get_magic(data);
    let mut content_type = magic.clone();

    // Disambiguate binary glTF: "glTF" -> "glb"
    if content_type == "glTF" {
        content_type = Cesium3DTileContentType::GLTF_BINARY.to_string();
    }

    if content_type == Cesium3DTileContentType::GLTF_BINARY {
        if let Some(glb_json) = get_glb_json(data) {
            if has_extension(&glb_json, "3DTILES_subtree") {
                content_type = Cesium3DTileContentType::IMPLICIT_SUBTREE_GLB.to_string();
            }
        }
    }

    if Cesium3DTileContentType::is_binary_format(&content_type) {
        return Ok(PreprocessedContent {
            content_type,
            binary_payload: Some(data.to_vec()),
            json_payload: None,
        });
    }

    // Try parsing as JSON
    let json = get_json_content(data, 0, data.len())?;

    if json.get("root").is_some() {
        return Ok(PreprocessedContent {
            content_type: Cesium3DTileContentType::EXTERNAL_TILESET.to_string(),
            binary_payload: None,
            json_payload: Some(json),
        });
    }

    if json.get("asset").is_some() {
        if has_extension(&json, "3DTILES_subtree") {
            return Ok(PreprocessedContent {
                content_type: Cesium3DTileContentType::IMPLICIT_SUBTREE_GLTF.to_string(),
                binary_payload: None,
                json_payload: Some(json),
            });
        }
        return Ok(PreprocessedContent {
            content_type: Cesium3DTileContentType::GLTF.to_string(),
            binary_payload: None,
            json_payload: Some(json),
        });
    }

    if json.get("tileAvailability").is_some() {
        return Ok(PreprocessedContent {
            content_type: Cesium3DTileContentType::IMPLICIT_SUBTREE_JSON.to_string(),
            binary_payload: None,
            json_payload: Some(json),
        });
    }

    if json.get("type").is_some() {
        return Ok(PreprocessedContent {
            content_type: Cesium3DTileContentType::GEOJSON.to_string(),
            binary_payload: None,
            json_payload: Some(json),
        });
    }

    if json.get("voxelTable").is_some() {
        return Ok(PreprocessedContent {
            content_type: Cesium3DTileContentType::VOXEL_JSON.to_string(),
            binary_payload: None,
            json_payload: Some(json),
        });
    }

    Err(RuntimeError::new("Invalid tile content."))
}

/// Reads the magic number (first 4 bytes as ASCII) of the given buffer.
fn get_magic(data: &[u8]) -> String {
    if data.len() < 4 {
        return String::new();
    }
    String::from_utf8_lossy(&data[0..4]).to_string()
}

/// Reads the JSON chunk of a GLB file if present.
fn get_glb_json(data: &[u8]) -> Option<Value> {
    const GLB_HEADER_BYTE_LENGTH: usize = 12;
    const CHUNK_HEADER_BYTE_LENGTH: usize = 8;
    const JSON_BYTE_OFFSET: usize = GLB_HEADER_BYTE_LENGTH + CHUNK_HEADER_BYTE_LENGTH;

    if data.len() < JSON_BYTE_OFFSET {
        return None;
    }

    let json_byte_length = u32::from_le_bytes([
        data[GLB_HEADER_BYTE_LENGTH],
        data[GLB_HEADER_BYTE_LENGTH + 1],
        data[GLB_HEADER_BYTE_LENGTH + 2],
        data[GLB_HEADER_BYTE_LENGTH + 3],
    ]) as usize;

    let chunk_type = u32::from_le_bytes([
        data[GLB_HEADER_BYTE_LENGTH + 4],
        data[GLB_HEADER_BYTE_LENGTH + 5],
        data[GLB_HEADER_BYTE_LENGTH + 6],
        data[GLB_HEADER_BYTE_LENGTH + 7],
    ]);

    if chunk_type != GLB_JSON_CHUNK_TYPE || JSON_BYTE_OFFSET + json_byte_length > data.len() {
        return None;
    }

    get_json_content(data, JSON_BYTE_OFFSET, json_byte_length).ok()
}

/// Parses JSON from a subslice of bytes.
fn get_json_content(
    data: &[u8],
    byte_offset: usize,
    byte_length: usize,
) -> Result<Value, RuntimeError> {
    if byte_offset + byte_length > data.len() {
        return Err(RuntimeError::new("Invalid tile content."));
    }
    let slice = &data[byte_offset..byte_offset + byte_length];
    serde_json::from_slice(slice).map_err(|_| RuntimeError::new("Invalid tile content."))
}
