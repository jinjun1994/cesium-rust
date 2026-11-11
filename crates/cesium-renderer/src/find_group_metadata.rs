//! Finds group metadata for a 3D Tile content header.
//! Adapted from CesiumJS `Scene/findGroupMetadata.js` (Apache-2.0).

use serde_json::Value;

use crate::group_metadata::GroupMetadata;

/// Group metadata extension associated with a tileset.
#[derive(Debug, Clone, Default)]
pub struct TilesetMetadataExtension {
    pub groups: Vec<GroupMetadata>,
    pub group_ids: Vec<String>,
}

/// Query context containing an optional tileset metadata extension.
#[derive(Debug, Clone, Default)]
pub struct TilesetGroupMetadataQuery {
    pub metadata_extension: Option<TilesetMetadataExtension>,
}

impl TilesetGroupMetadataQuery {
    /// Creates a query context from groups and their corresponding IDs.
    pub fn new(groups: Vec<GroupMetadata>, group_ids: Vec<String>) -> Self {
        Self {
            metadata_extension: Some(TilesetMetadataExtension { groups, group_ids }),
        }
    }
}

/// Checks if a content has metadata, either defined in its `metadata` field
/// (3D Tiles 1.1) or in the `3DTILES_metadata` extension. If so, looks up
/// the group with the corresponding ID or index.
pub fn find_group_metadata(
    tileset: &TilesetGroupMetadataQuery,
    content_header: &Value,
) -> Option<GroupMetadata> {
    let metadata_ext = tileset.metadata_extension.as_ref()?;

    // 1. Extract group attribute from 3DTILES_metadata extension or direct group field
    let group_val = if let Some(ext) = content_header
        .get("extensions")
        .and_then(|e| e.get("3DTILES_metadata"))
    {
        ext.get("group")
    } else {
        content_header.get("group")
    }?;

    // 2. Numeric index lookup (3D Tiles 1.1)
    if let Some(idx) = group_val.as_u64() {
        return metadata_ext.groups.get(idx as usize).cloned();
    }
    if let Some(idx) = group_val.as_i64() {
        if idx >= 0 {
            return metadata_ext.groups.get(idx as usize).cloned();
        }
        return None;
    }

    // 3. String ID lookup (legacy 3DTILES_metadata)
    if let Some(target_id) = group_val.as_str() {
        if let Some(idx) = metadata_ext.group_ids.iter().position(|id| id == target_id) {
            return metadata_ext.groups.get(idx).cloned();
        }
        // Fallback: check group.id() directly
        for group in &metadata_ext.groups {
            if group.id() == Some(target_id) {
                return Some(group.clone());
            }
        }
    }

    None
}
