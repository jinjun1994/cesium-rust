//! Tileset3DTileContent matching CesiumJS `Scene/Tileset3DTileContent.js` (Apache-2.0).

use crate::content_metadata::ContentMetadata;
use crate::multiple_3d_tile_content::Cesium3DContentGroup;
use serde_json::Value;

/// Represents content for a tile in a 3D Tiles tileset whose content points to another 3D Tiles tileset.
#[derive(Debug, Clone)]
pub struct Tileset3DTileContent {
    pub tileset_id: Option<String>,
    pub tile_id: Option<String>,
    pub url: Option<String>,
    pub ready: bool,
    pub metadata: Option<ContentMetadata>,
    pub group: Option<Cesium3DContentGroup>,
    is_destroyed: bool,
}

impl Tileset3DTileContent {
    /// Creates a new `Tileset3DTileContent`.
    pub fn new(tileset_id: Option<String>, tile_id: Option<String>, url: Option<String>) -> Self {
        Self {
            tileset_id,
            tile_id,
            url,
            ready: false,
            metadata: None,
            group: None,
            is_destroyed: false,
        }
    }

    /// Creates an instance of `Tileset3DTileContent` from parsed JSON.
    pub fn from_json(
        tileset_id: Option<String>,
        tile_id: Option<String>,
        url: Option<String>,
        _json: &Value,
    ) -> Self {
        let mut content = Self::new(tileset_id, tile_id, url);
        content.ready = true;
        content
    }

    pub fn features_length(&self) -> usize {
        0
    }

    pub fn points_length(&self) -> usize {
        0
    }

    pub fn triangles_length(&self) -> usize {
        0
    }

    pub fn geometry_byte_length(&self) -> usize {
        0
    }

    pub fn textures_byte_length(&self) -> usize {
        0
    }

    pub fn batch_table_byte_length(&self) -> usize {
        0
    }

    pub fn inner_contents(&self) -> Option<&[Self]> {
        None
    }

    pub fn ready(&self) -> bool {
        self.ready
    }

    pub fn tileset_id(&self) -> Option<&str> {
        self.tileset_id.as_deref()
    }

    pub fn tile_id(&self) -> Option<&str> {
        self.tile_id.as_deref()
    }

    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    pub fn batch_table(&self) -> Option<&Value> {
        None
    }

    pub fn metadata(&self) -> Option<&ContentMetadata> {
        self.metadata.as_ref()
    }

    pub fn set_metadata(&mut self, metadata: ContentMetadata) {
        self.metadata = Some(metadata);
    }

    pub fn group(&self) -> Option<&Cesium3DContentGroup> {
        self.group.as_ref()
    }

    pub fn set_group(&mut self, group: Cesium3DContentGroup) {
        self.group = Some(group);
    }

    pub fn has_property(&self, _batch_id: usize, _name: &str) -> bool {
        false
    }

    pub fn get_feature(&self, _batch_id: usize) -> Option<&Value> {
        None
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        self.metadata = None;
        self.group = None;
    }
}
