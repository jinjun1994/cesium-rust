//! Resolves tile metadata from a 3D Tile header and a tileset's schema.
//! Adapted from CesiumJS `Scene/findTileMetadata.js` (Apache-2.0).

use std::sync::atomic::{AtomicBool, Ordering};

use serde_json::Value;

use crate::find_content_metadata::TilesetMetadataQuery;
use crate::tile_metadata::{TileMetadata, TileMetadataOptions};

static WARNED_MISSING_ROOT_SCHEMA: AtomicBool = AtomicBool::new(false);

/// Resets the one-time warning flag (for testing).
pub fn reset_find_tile_metadata_warning() {
    WARNED_MISSING_ROOT_SCHEMA.store(false, Ordering::Relaxed);
}

/// Checks whether the one-time missing schema warning was triggered.
pub fn has_find_tile_metadata_warned() -> bool {
    WARNED_MISSING_ROOT_SCHEMA.load(Ordering::Relaxed)
}

/// Checks if a tile has metadata, either defined in its `metadata` field
/// (3D Tiles 1.1) or in the `3DTILES_metadata` extension. If defined, gets
/// the tile metadata with the corresponding class.
pub fn find_tile_metadata(
    tileset: &TilesetMetadataQuery,
    tile_header: &Value,
) -> Option<TileMetadata> {
    // 1. Extract metadata JSON: extensions["3DTILES_metadata"] takes precedence / legacy
    let metadata_json = if let Some(ext) = tile_header
        .get("extensions")
        .and_then(|e| e.get("3DTILES_metadata"))
    {
        ext
    } else {
        tile_header.get("metadata")?
    };

    // 2. Validate schema presence
    let Some(schema) = &tileset.schema else {
        WARNED_MISSING_ROOT_SCHEMA.store(true, Ordering::Relaxed);
        return None;
    };

    // 3. Find matching class
    let class_name = metadata_json.get("class").and_then(Value::as_str)?;
    let tile_class = schema.classes.get(class_name)?;

    TileMetadata::from_options(TileMetadataOptions {
        tile: Some(metadata_json.clone()),
        class_def: Some(tile_class.clone()),
    })
    .ok()
}
