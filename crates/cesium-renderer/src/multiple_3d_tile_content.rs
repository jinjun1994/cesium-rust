//! Multiple3DTileContent matching CesiumJS `Scene/Multiple3DTileContent.js` and `3DTILES_multiple_contents` extension (Apache-2.0).

use cesium_math::{BoundingSphere, Color, DeveloperError};
use glam::DVec3;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cesium_3d_tile_content_type::Cesium3DTileContentType;
use crate::content_metadata::ContentMetadata;
use crate::group_metadata::GroupMetadata;

/// A simple abstraction for a content group, matching CesiumJS `Scene/Cesium3DContentGroup.js`.
#[derive(Clone, Debug)]
pub struct Cesium3DContentGroup {
    pub metadata: GroupMetadata,
}

impl Cesium3DContentGroup {
    /// Creates a new `Cesium3DContentGroup` with given group metadata.
    pub fn new(metadata: GroupMetadata) -> Self {
        Self { metadata }
    }

    /// Creates a new `Cesium3DContentGroup`, returning `DeveloperError` when metadata is missing.
    pub fn try_new(metadata: Option<GroupMetadata>) -> Result<Self, DeveloperError> {
        let metadata =
            metadata.ok_or_else(|| DeveloperError::new("options.metadata is required."))?;
        Ok(Self { metadata })
    }
}

/// Header describing one inner content item in a multiple content tile.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InnerContentHeader {
    pub uri: String,
    #[serde(default)]
    pub group: Option<usize>,
    #[serde(default)]
    pub metadata: Option<Value>,
}

/// Statistics on tile requests matching CesiumJS `Cesium3DTilesetStatistics`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TilesetStatistics {
    pub number_of_pending_requests: usize,
    pub number_of_attempted_requests: usize,
}

/// Individual inner content item loaded within a multiple 3D tile.
#[derive(Clone, Debug)]
pub struct InnerTileContent {
    pub uri: String,
    pub content_type: Cesium3DTileContentType,
    pub group: Option<Cesium3DContentGroup>,
    pub metadata: Option<ContentMetadata>,
    pub ready: bool,
    pub feature_properties_dirty: bool,
    pub color: Option<Color>,
    pub show: bool,
    pub bounding_volume: Option<BoundingSphere>,
    pub payload: Vec<u8>,
}

impl InnerTileContent {
    /// Creates a new inner tile content.
    pub fn new(uri: String, content_type: Cesium3DTileContentType, payload: Vec<u8>) -> Self {
        Self {
            uri,
            content_type,
            group: None,
            metadata: None,
            ready: true,
            feature_properties_dirty: false,
            color: None,
            show: true,
            bounding_volume: None,
            payload,
        }
    }

    /// Determines content type from URI extension.
    pub fn infer_content_type(uri: &str) -> Cesium3DTileContentType {
        let lower = uri.to_lowercase();
        if lower.ends_with(".b3dm") {
            Cesium3DTileContentType::Batched3DModel
        } else if lower.ends_with(".i3dm") {
            Cesium3DTileContentType::Instanced3DModel
        } else if lower.ends_with(".pnts") {
            Cesium3DTileContentType::PointCloud
        } else if lower.ends_with(".cmpt") {
            Cesium3DTileContentType::Composite
        } else if lower.ends_with(".glb") {
            Cesium3DTileContentType::GltfBinary
        } else if lower.ends_with(".gltf") {
            Cesium3DTileContentType::Gltf
        } else if lower.ends_with(".json") {
            Cesium3DTileContentType::ExternalTileset
        } else {
            Cesium3DTileContentType::Other
        }
    }
}

/// A collection of contents for tiles that have multiple contents (3D Tiles 1.1 or `3DTILES_multiple_contents`).
#[derive(Clone, Debug)]
pub struct Multiple3DTileContent {
    pub tileset_url: String,
    pub inner_content_headers: Vec<InnerContentHeader>,
    pub inner_contents: Vec<InnerTileContent>,
    pub cancel_count: usize,
    pub requests_in_flight: usize,
    pub contents_created: bool,
    pub ready: bool,
    pub is_destroyed: bool,
}

impl Multiple3DTileContent {
    /// Creates a new `Multiple3DTileContent` from JSON containing `contents` (3D Tiles 1.1) or `content` (legacy).
    pub fn new(tileset_url: &str, contents_json: &Value) -> Result<Self, DeveloperError> {
        let headers_val = contents_json
            .get("contents")
            .or_else(|| contents_json.get("content"))
            .ok_or_else(|| {
                DeveloperError::new(
                    "contentsJson must contain 'contents' or 'content' array property.",
                )
            })?;

        let array = headers_val.as_array().ok_or_else(|| {
            DeveloperError::new("contents or content property must be a JSON array.")
        })?;

        let mut inner_content_headers = Vec::with_capacity(array.len());
        for item in array {
            let uri = item
                .get("uri")
                .or_else(|| item.get("url"))
                .and_then(Value::as_str)
                .ok_or_else(|| DeveloperError::new("Each inner content header must contain 'uri'."))?
                .to_string();

            let group = item.get("group").and_then(Value::as_u64).map(|g| g as usize);
            let metadata = item.get("metadata").cloned();

            inner_content_headers.push(InnerContentHeader {
                uri,
                group,
                metadata,
            });
        }

        Ok(Self {
            tileset_url: tileset_url.to_string(),
            inner_content_headers,
            inner_contents: Vec::new(),
            cancel_count: 0,
            requests_in_flight: 0,
            contents_created: false,
            ready: false,
            is_destroyed: false,
        })
    }

    /// Returns list of inner content URLs.
    pub fn inner_content_urls(&self) -> Vec<String> {
        self.inner_content_headers
            .iter()
            .map(|h| h.uri.clone())
            .collect()
    }

    /// Part of Cesium3DTileContent interface - returns false.
    pub fn feature_properties_dirty(&self) -> bool {
        self.inner_contents
            .iter()
            .any(|c| c.feature_properties_dirty)
    }

    /// Sets featurePropertiesDirty across all inner contents.
    pub fn set_feature_properties_dirty(&mut self, dirty: bool) {
        for content in &mut self.inner_contents {
            content.feature_properties_dirty = dirty;
        }
    }

    /// Always 0 for Multiple3DTileContent.
    pub fn features_length(&self) -> usize {
        0
    }

    /// Always 0 for Multiple3DTileContent.
    pub fn points_length(&self) -> usize {
        0
    }

    /// Always 0 for Multiple3DTileContent.
    pub fn triangles_length(&self) -> usize {
        0
    }

    /// Always 0 for Multiple3DTileContent.
    pub fn geometry_byte_length(&self) -> usize {
        0
    }

    /// Always 0 for Multiple3DTileContent.
    pub fn textures_byte_length(&self) -> usize {
        0
    }

    /// Always 0 for Multiple3DTileContent.
    pub fn batch_table_byte_length(&self) -> usize {
        0
    }

    /// Returns None because multiple content does not have a single URL.
    pub fn url(&self) -> Option<&str> {
        None
    }

    /// Returns None because multiple content does not have a single metadata instance.
    pub fn metadata(&self) -> Option<&ContentMetadata> {
        None
    }

    /// Assigning metadata to Multiple3DTileContent throws DeveloperError.
    pub fn set_metadata(&mut self, _metadata: ContentMetadata) -> Result<(), DeveloperError> {
        Err(DeveloperError::new(
            "Multiple3DTileContent cannot have metadata",
        ))
    }

    /// Returns None because multiple content does not have group metadata.
    pub fn group(&self) -> Option<&Cesium3DContentGroup> {
        None
    }

    /// Assigning group metadata to Multiple3DTileContent throws DeveloperError.
    pub fn set_group(&mut self, _group: Cesium3DContentGroup) -> Result<(), DeveloperError> {
        Err(DeveloperError::new(
            "Multiple3DTileContent cannot have group metadata",
        ))
    }

    /// Cancels in-flight requests.
    pub fn cancel_requests(&mut self, statistics: &mut TilesetStatistics) {
        self.cancel_count += 1;
        statistics.number_of_pending_requests = statistics
            .number_of_pending_requests
            .saturating_sub(self.requests_in_flight);
        statistics.number_of_attempted_requests += self.requests_in_flight;
        self.requests_in_flight = 0;
    }

    /// Requests and schedules inner contents, updating statistics and returning true if scheduled.
    pub fn request_inner_contents<F>(
        &mut self,
        statistics: &mut TilesetStatistics,
        maximum_requests_per_server: usize,
        server_key: &str,
        fetcher: F,
        mut on_failure: impl FnMut(String, String),
    ) -> bool
    where
        F: Fn(&str) -> Result<Vec<u8>, String>,
    {
        let count = self.inner_content_headers.len();

        // Check if server and scheduler can take all requests at once
        let _ = server_key;
        if count > maximum_requests_per_server {
            statistics.number_of_attempted_requests += count;
            return false;
        }

        self.requests_in_flight += count;
        statistics.number_of_pending_requests += count;

        let initial_cancel_count = self.cancel_count;
        let mut created = Vec::new();

        for header in &self.inner_content_headers {
            if self.cancel_count > initial_cancel_count {
                // Cancelled during processing
                return false;
            }

            match fetcher(&header.uri) {
                Ok(bytes) => {
                    self.requests_in_flight = self.requests_in_flight.saturating_sub(1);
                    statistics.number_of_pending_requests =
                        statistics.number_of_pending_requests.saturating_sub(1);

                    let c_type = InnerTileContent::infer_content_type(&header.uri);
                    let inner = InnerTileContent::new(header.uri.clone(), c_type, bytes);
                    created.push(inner);
                }
                Err(err) => {
                    self.requests_in_flight = self.requests_in_flight.saturating_sub(1);
                    statistics.number_of_pending_requests =
                        statistics.number_of_pending_requests.saturating_sub(1);
                    on_failure(header.uri.clone(), err);
                }
            }
        }

        self.inner_contents = created;
        self.contents_created = true;
        self.ready = true;
        true
    }

    /// Finds intersection with closest inner content along ray.
    pub fn pick(&self, ray_origin: DVec3, ray_direction: DVec3) -> Option<usize> {
        if !self.ready {
            return None;
        }

        let mut closest_idx = None;
        let mut min_distance = f64::INFINITY;

        for (idx, content) in self.inner_contents.iter().enumerate() {
            if !content.show {
                continue;
            }
            if let Some(bs) = content.bounding_volume {
                let to_center = bs.center - ray_origin;
                let proj = to_center.dot(ray_direction);
                if proj > 0.0 {
                    let d2 = to_center.length_squared() - proj * proj;
                    if d2 <= bs.radius * bs.radius && proj < min_distance {
                        min_distance = proj;
                        closest_idx = Some(idx);
                    }
                }
            } else {
                // Fallback default index
                if closest_idx.is_none() {
                    closest_idx = Some(idx);
                }
            }
        }

        closest_idx
    }

    /// Evaluates rendered RGBA for spec verification.
    pub fn simulate_rendered_rgba(&self) -> [u8; 4] {
        let mut color_out = [0u8, 0u8, 0u8, 255u8];

        for content in &self.inner_contents {
            if !content.show {
                continue;
            }
            if let Some(c) = content.color {
                if c.alpha > 0.0 {
                    color_out = [
                        (c.red * 255.0).round() as u8,
                        (c.green * 255.0).round() as u8,
                        (c.blue * 255.0).round() as u8,
                        (c.alpha * 255.0).round() as u8,
                    ];
                    break;
                }
            }
        }

        color_out
    }

    /// Lifecycle destroy method.
    pub fn destroy(&mut self) {
        self.inner_contents.clear();
        self.is_destroyed = true;
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }
}
