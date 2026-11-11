//! Resolves content metadata from a 3D Tile content header and a tileset's schema.
//! Adapted from CesiumJS `Scene/findContentMetadata.js` (Apache-2.0).

use std::sync::atomic::{AtomicBool, Ordering};

use serde_json::Value;

use crate::content_metadata::{ContentMetadata, ContentMetadataOptions};
use crate::metadata_schema::MetadataSchema;

static WARNED_MISSING_ROOT_SCHEMA: AtomicBool = AtomicBool::new(false);

/// Query context representing a tileset's metadata schema.
#[derive(Debug, Clone, Default)]
pub struct TilesetMetadataQuery {
    pub schema: Option<MetadataSchema>,
}

impl TilesetMetadataQuery {
    /// Creates a query context from a `MetadataSchema`.
    pub fn new(schema: Option<MetadataSchema>) -> Self {
        Self { schema }
    }

    /// Creates a query context from a tileset JSON value.
    pub fn from_json(tileset_json: &Value) -> Self {
        let schema = tileset_json
            .get("schema")
            .and_then(|s| MetadataSchema::from_json(Some(s)).ok());
        Self { schema }
    }
}

/// Resets the one-time warning flag (for testing).
pub fn reset_find_content_metadata_warning() {
    WARNED_MISSING_ROOT_SCHEMA.store(false, Ordering::Relaxed);
}

/// Checks whether the one-time missing schema warning was triggered.
pub fn has_find_content_metadata_warned() -> bool {
    WARNED_MISSING_ROOT_SCHEMA.load(Ordering::Relaxed)
}

/// Check if a content header has metadata, either defined in its `metadata` field
/// (3D Tiles 1.1) or in the `3DTILES_metadata` extension. If defined, gets the content
/// metadata with the corresponding class.
pub fn find_content_metadata(
    tileset: &TilesetMetadataQuery,
    content_header: &Value,
) -> Option<ContentMetadata> {
    // 1. Extract metadata JSON: extensions["3DTILES_metadata"] takes precedence / legacy
    let metadata_json = if let Some(ext) = content_header
        .get("extensions")
        .and_then(|e| e.get("3DTILES_metadata"))
    {
        ext
    } else {
        content_header.get("metadata")?
    };

    // 2. Validate schema presence
    let Some(schema) = &tileset.schema else {
        WARNED_MISSING_ROOT_SCHEMA.store(true, Ordering::Relaxed);
        return None;
    };

    // 3. Find matching class
    let class_name = metadata_json.get("class").and_then(Value::as_str)?;
    let content_class = schema.classes.get(class_name)?;

    ContentMetadata::new(ContentMetadataOptions {
        content: Some(metadata_json.clone()),
        class_def: Some(content_class.clone()),
    })
    .ok()
}
