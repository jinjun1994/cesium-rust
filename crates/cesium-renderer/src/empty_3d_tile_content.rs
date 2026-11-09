use cesium_math::DeveloperError;
use serde_json::Value;

/// Represents empty content for tiles in a 3D Tiles tileset that do not have content
/// (e.g. used to optimize hierarchical culling).
#[derive(Debug, Clone, Default)]
pub struct Empty3DTileContent {
    /// Tileset identifier or reference.
    pub tileset_id: Option<String>,
    /// Tile identifier or reference.
    pub tile_id: Option<String>,
    is_destroyed: bool,
}

impl Empty3DTileContent {
    /// Creates a new `Empty3DTileContent`.
    pub fn new(tileset_id: Option<String>, tile_id: Option<String>) -> Self {
        Self {
            tileset_id,
            tile_id,
            is_destroyed: false,
        }
    }

    /// Number of features (always 0).
    pub fn features_length(&self) -> usize {
        0
    }

    /// Number of points (always 0).
    pub fn points_length(&self) -> usize {
        0
    }

    /// Number of triangles (always 0).
    pub fn triangles_length(&self) -> usize {
        0
    }

    /// Geometry byte length (always 0).
    pub fn geometry_byte_length(&self) -> usize {
        0
    }

    /// Textures byte length (always 0).
    pub fn textures_byte_length(&self) -> usize {
        0
    }

    /// Batch table byte length (always 0).
    pub fn batch_table_byte_length(&self) -> usize {
        0
    }

    /// Inner contents (always None).
    pub fn inner_contents(&self) -> Option<&[Empty3DTileContent]> {
        None
    }

    /// Content URL (always None).
    pub fn url(&self) -> Option<&str> {
        None
    }

    /// Content metadata (always None).
    pub fn metadata(&self) -> Option<&Value> {
        None
    }

    /// Setting metadata on Empty3DTileContent is forbidden and throws DeveloperError.
    pub fn set_metadata(&mut self, _metadata: Value) -> Result<(), DeveloperError> {
        Err(DeveloperError::new(
            "Empty3DTileContent cannot have content metadata",
        ))
    }

    /// Group metadata (always None).
    pub fn group(&self) -> Option<&Value> {
        None
    }

    /// Setting group on Empty3DTileContent is forbidden and throws DeveloperError.
    pub fn set_group(&mut self, _group: Value) -> Result<(), DeveloperError> {
        Err(DeveloperError::new(
            "Empty3DTileContent cannot have group metadata",
        ))
    }

    /// Checks if a property exists on a feature (always false).
    pub fn has_property(&self, _batch_id: usize, _name: &str) -> bool {
        false
    }

    /// Returns true if destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys resources.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }
}
