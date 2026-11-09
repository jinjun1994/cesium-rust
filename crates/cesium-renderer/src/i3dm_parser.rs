//! I3dmParser matching CesiumJS `Scene/I3dmParser.js` (Apache-2.0).

use std::fmt;
use cesium_math::{DeveloperError, RuntimeError};
use serde_json::Value;

/// Error type returned by `I3dmParser::parse`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum I3dmParserError {
    DeveloperError(DeveloperError),
    RuntimeError(RuntimeError),
}

impl From<DeveloperError> for I3dmParserError {
    fn from(e: DeveloperError) -> Self {
        Self::DeveloperError(e)
    }
}

impl From<RuntimeError> for I3dmParserError {
    fn from(e: RuntimeError) -> Self {
        Self::RuntimeError(e)
    }
}

impl fmt::Display for I3dmParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeveloperError(e) => write!(f, "{e}"),
            Self::RuntimeError(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for I3dmParserError {}

/// Parsed results of an Instanced 3D Model (.i3dm) tile.
#[derive(Clone, Debug, PartialEq)]
pub struct I3dmParseResult {
    pub gltf_format: u32,
    pub feature_table_json: Value,
    pub feature_table_binary: Vec<u8>,
    pub batch_table_json: Option<Value>,
    pub batch_table_binary: Option<Vec<u8>>,
    pub gltf: Vec<u8>,
    pub is_unaligned: bool,
}

/// Parser for Instanced 3D Model (.i3dm) tiles.
pub struct I3dmParser;

impl I3dmParser {
    /// Parses the contents of an Instanced 3D Model.
    pub fn parse(
        array_buffer: Option<&[u8]>,
        byte_offset: Option<usize>,
    ) -> Result<I3dmParseResult, I3dmParserError> {
        let buffer = array_buffer.ok_or_else(|| {
            I3dmParserError::DeveloperError(DeveloperError::new("arrayBuffer is required."))
        })?;

        let byte_start = byte_offset.unwrap_or(0);
        let mut offset = byte_start;

        if buffer.len() < offset + 32 {
            return Err(RuntimeError::new("Buffer is too small to contain i3dm header.").into());
        }

        // Magic (bytes 0..4)
        offset += 4;

        // Version (bytes 4..8)
        let version = u32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap());
        if version != 1 {
            return Err(RuntimeError::new(format!(
                "Only Instanced 3D Model version 1 is supported. Version {version} is not."
            )).into());
        }
        offset += 4;

        // byteLength (bytes 8..12)
        let byte_length = u32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        // featureTableJsonByteLength (bytes 12..16)
        let feature_table_json_byte_length =
            u32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap()) as usize;
        if feature_table_json_byte_length == 0 {
            return Err(RuntimeError::new(
                "featureTableJsonByteLength is zero, the feature table must be defined.",
            ).into());
        }
        offset += 4;

        // featureTableBinaryByteLength (bytes 16..20)
        let feature_table_binary_byte_length =
            u32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        // batchTableJsonByteLength (bytes 20..24)
        let batch_table_json_byte_length =
            u32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        // batchTableBinaryByteLength (bytes 24..28)
        let batch_table_binary_byte_length =
            u32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        // gltfFormat (bytes 28..32)
        let gltf_format = u32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap());
        if gltf_format != 0 && gltf_format != 1 {
            return Err(RuntimeError::new(format!(
                "Only glTF format 0 (uri) or 1 (embedded) are supported. Format {gltf_format} is not."
            )).into());
        }
        offset += 4;

        // Feature table JSON
        if buffer.len() < offset + feature_table_json_byte_length {
            return Err(RuntimeError::new("Buffer truncated before featureTableJson.").into());
        }
        let feature_table_json_slice = &buffer[offset..offset + feature_table_json_byte_length];
        let feature_table_json_str = std::str::from_utf8(feature_table_json_slice)
            .map_err(|e| RuntimeError::new(format!("Invalid UTF-8 in featureTableJson: {e}")))?;
        let feature_table_json: Value = serde_json::from_str(feature_table_json_str.trim())
            .map_err(|e| RuntimeError::new(format!("Invalid JSON in featureTableJson: {e}")))?;
        offset += feature_table_json_byte_length;

        // Feature table binary
        if buffer.len() < offset + feature_table_binary_byte_length {
            return Err(RuntimeError::new("Buffer truncated before featureTableBinary.").into());
        }
        let feature_table_binary = buffer[offset..offset + feature_table_binary_byte_length].to_vec();
        offset += feature_table_binary_byte_length;

        // Batch table JSON & binary
        let mut batch_table_json = None;
        let mut batch_table_binary = None;

        if batch_table_json_byte_length > 0 {
            if buffer.len() < offset + batch_table_json_byte_length {
                return Err(RuntimeError::new("Buffer truncated before batchTableJson.").into());
            }
            let bt_json_slice = &buffer[offset..offset + batch_table_json_byte_length];
            let bt_json_str = std::str::from_utf8(bt_json_slice)
                .map_err(|e| RuntimeError::new(format!("Invalid UTF-8 in batchTableJson: {e}")))?;
            batch_table_json = Some(serde_json::from_str(bt_json_str.trim())
                .map_err(|e| RuntimeError::new(format!("Invalid JSON in batchTableJson: {e}")))?);
            offset += batch_table_json_byte_length;

            if batch_table_binary_byte_length > 0 {
                if buffer.len() < offset + batch_table_binary_byte_length {
                    return Err(RuntimeError::new("Buffer truncated before batchTableBinary.").into());
                }
                batch_table_binary = Some(buffer[offset..offset + batch_table_binary_byte_length].to_vec());
                offset += batch_table_binary_byte_length;
            }
        }

        let total_expected = byte_start + byte_length;
        if total_expected < offset {
            return Err(RuntimeError::new("Invalid byteLength in i3dm header.").into());
        }
        let gltf_byte_length = total_expected - offset;
        if gltf_byte_length == 0 {
            return Err(RuntimeError::new("glTF byte length must be greater than 0.").into());
        }
        if buffer.len() < offset + gltf_byte_length {
            return Err(RuntimeError::new("Buffer truncated before glTF payload.").into());
        }

        let is_unaligned = !offset.is_multiple_of(4);
        let gltf = buffer[offset..offset + gltf_byte_length].to_vec();

        Ok(I3dmParseResult {
            gltf_format,
            feature_table_json,
            feature_table_binary,
            batch_table_json,
            batch_table_binary,
            gltf,
            is_unaligned,
        })
    }
}
