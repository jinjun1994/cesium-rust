//! Cesium3DTile matching CesiumJS `Scene/Cesium3DTile.js` (Apache-2.0).

use cesium_math::{
    Cartesian3, Matrix3, Matrix4, Rectangle, RuntimeError,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cesium_3d_tileset_heatmap::Cesium3DTilesetHeatmap;
use crate::empty_3d_tile_content::Empty3DTileContent;
use crate::tile_bounding_region::{TileBoundingRegion, TileBoundingRegionOptions};
use crate::tile_bounding_sphere::TileBoundingSphere;
use crate::tile_oriented_bounding_box::TileOrientedBoundingBox;

/// Refinement strategy for hierarchical tileset traversal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Cesium3DTileRefine {
    #[default]
    Replace = 0,
    Add = 1,
}

impl Cesium3DTileRefine {
    pub fn parse(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("REPLACE") {
            Some(Self::Replace)
        } else if s.eq_ignore_ascii_case("ADD") {
            Some(Self::Add)
        } else {
            None
        }
    }
}

/// Concrete bounding volume representation for a 3D Tile.
#[derive(Clone, Debug, PartialEq)]
pub enum Cesium3DTileBoundingVolume {
    Sphere(TileBoundingSphere),
    Region(Box<TileBoundingRegion>),
    Box(TileOrientedBoundingBox),
}

impl Cesium3DTileBoundingVolume {
    pub fn from_json(bv_val: &Value) -> Result<Self, RuntimeError> {
        let obj = bv_val
            .as_object()
            .ok_or_else(|| RuntimeError::new("tile boundingVolume must be an object."))?;

        if let Some(sphere_val) = obj.get("sphere").and_then(|v| v.as_array()) {
            if sphere_val.len() == 4 {
                let x = sphere_val[0].as_f64().unwrap_or(0.0);
                let y = sphere_val[1].as_f64().unwrap_or(0.0);
                let z = sphere_val[2].as_f64().unwrap_or(0.0);
                let radius = sphere_val[3].as_f64().unwrap_or(0.0);
                return Ok(Self::Sphere(TileBoundingSphere::new(
                    Some(Cartesian3::new(x, y, z)),
                    Some(radius),
                )));
            }
        }

        if let Some(region_val) = obj.get("region").and_then(|v| v.as_array()) {
            if region_val.len() == 6 {
                let west = region_val[0].as_f64().unwrap_or(0.0);
                let south = region_val[1].as_f64().unwrap_or(0.0);
                let east = region_val[2].as_f64().unwrap_or(0.0);
                let north = region_val[3].as_f64().unwrap_or(0.0);
                let min_height = region_val[4].as_f64().unwrap_or(0.0);
                let max_height = region_val[5].as_f64().unwrap_or(0.0);

                let region = TileBoundingRegion::new(TileBoundingRegionOptions {
                    rectangle: Rectangle::new(west, south, east, north),
                    minimum_height: Some(min_height.min(max_height)),
                    maximum_height: Some(min_height.max(max_height)),
                    ellipsoid: None,
                    compute_bounding_volumes: None,
                })
                .map_err(|e| RuntimeError::new(e.message))?;
                return Ok(Self::Region(Box::new(region)));
            }
        }

        if let Some(box_val) = obj.get("box").and_then(|v| v.as_array()) {
            if box_val.len() == 12 {
                let center = Cartesian3::new(
                    box_val[0].as_f64().unwrap_or(0.0),
                    box_val[1].as_f64().unwrap_or(0.0),
                    box_val[2].as_f64().unwrap_or(0.0),
                );
                let half_axes_raw: Vec<f64> =
                    box_val[3..12].iter().map(|v| v.as_f64().unwrap_or(0.0)).collect();
                let half_axes = Matrix3::from_array(&half_axes_raw, 0)
                    .map_err(|e| RuntimeError::new(format!("{e:?}")))?;
                return Ok(Self::Box(TileOrientedBoundingBox::new(
                    Some(center),
                    Some(half_axes),
                )));
            }
        }

        Err(RuntimeError::new(
            "tile boundingVolume does not contain a valid sphere, region, or box.",
        ))
    }
}

/// Minimal tileset context passed during tile creation.
#[derive(Clone, Debug)]
pub struct MockTileset {
    pub debug_show_bounding_volume: bool,
    pub debug_show_viewer_request_volume: bool,
    pub model_matrix: Matrix4,
    pub geometric_error: f64,
    pub scaled_geometric_error: f64,
    pub heatmap: Cesium3DTilesetHeatmap,
}

impl Default for MockTileset {
    fn default() -> Self {
        Self {
            debug_show_bounding_volume: true,
            debug_show_viewer_request_volume: true,
            model_matrix: Matrix4::IDENTITY,
            geometric_error: 2.0,
            scaled_geometric_error: 2.0,
            heatmap: Cesium3DTilesetHeatmap::new("level"),
        }
    }
}

/// A tile in a 3D Tiles tileset matching CesiumJS `Scene/Cesium3DTile.js`.
#[derive(Clone, Debug)]
pub struct Cesium3DTile {
    pub base_resource: String,
    pub geometric_error: f64,
    pub refine: Cesium3DTileRefine,
    pub transform: Matrix4,
    pub computed_transform: Matrix4,
    pub bounding_volume: Cesium3DTileBoundingVolume,
    content_bounding_volume: Option<Cesium3DTileBoundingVolume>,
    pub viewer_request_volume: Option<Cesium3DTileBoundingVolume>,
    pub content_uri: Option<String>,
    pub empty_content: Option<Empty3DTileContent>,
    is_destroyed: bool,
}

impl Cesium3DTile {
    /// Creates a new `Cesium3DTile` from a tileset header JSON value.
    pub fn new(
        tileset: &MockTileset,
        base_resource: impl Into<String>,
        header: &Value,
        parent: Option<&Cesium3DTile>,
    ) -> Result<Self, RuntimeError> {
        let base_resource = base_resource.into();

        // 1. Bounding Volume
        let bv_val = header
            .get("boundingVolume")
            .ok_or_else(|| RuntimeError::new("tile.boundingVolume is required."))?;
        let bounding_volume = Cesium3DTileBoundingVolume::from_json(bv_val)?;

        // 2. Refine
        let refine = if let Some(refine_str) = header.get("refine").and_then(|v| v.as_str()) {
            Cesium3DTileRefine::parse(refine_str).unwrap_or(Cesium3DTileRefine::Replace)
        } else if let Some(p) = parent {
            p.refine
        } else {
            Cesium3DTileRefine::Replace
        };

        // 3. Geometric error
        let geometric_error = if let Some(ge) = header.get("geometricError").and_then(|v| v.as_f64()) {
            ge
        } else if let Some(p) = parent {
            p.geometric_error
        } else {
            tileset.geometric_error
        };

        // 4. Transform
        let transform = if let Some(arr) = header.get("transform").and_then(|v| v.as_array()) {
            if arr.len() == 16 {
                let raw: Vec<f64> = arr.iter().map(|v| v.as_f64().unwrap_or(0.0)).collect();
                Matrix4::from_array(&raw, 0).unwrap_or(Matrix4::IDENTITY)
            } else {
                Matrix4::IDENTITY
            }
        } else {
            Matrix4::IDENTITY
        };

        let parent_transform = parent
            .map(|p| p.computed_transform)
            .unwrap_or(tileset.model_matrix);
        let computed_transform = parent_transform.multiply(&transform);

        // 5. Content & Content Bounding Volume
        let mut content_bounding_volume = None;
        let mut content_uri = None;
        let mut empty_content = None;

        let content_header = header.get("content").or_else(|| {
            header
                .get("contents")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
        });

        if let Some(ch) = content_header {
            if let Some(cbv) = ch.get("boundingVolume") {
                if let Ok(vol) = Cesium3DTileBoundingVolume::from_json(cbv) {
                    content_bounding_volume = Some(vol);
                }
            }

            if let Some(uri) = ch
                .get("uri")
                .or_else(|| ch.get("url"))
                .and_then(|v| v.as_str())
            {
                if uri.is_empty() {
                    empty_content = Some(Empty3DTileContent::new(None, None));
                } else {
                    content_uri = Some(uri.to_string());
                }
            }
        }

        // 6. Viewer Request Volume
        let viewer_request_volume = header
            .get("viewerRequestVolume")
            .and_then(|v| Cesium3DTileBoundingVolume::from_json(v).ok());

        Ok(Self {
            base_resource,
            geometric_error,
            refine,
            transform,
            computed_transform,
            bounding_volume,
            content_bounding_volume,
            viewer_request_volume,
            content_uri,
            empty_content,
            is_destroyed: false,
        })
    }

    /// Returns the effective content bounding volume, falling back to tile bounding volume.
    pub fn content_bounding_volume(&self) -> &Cesium3DTileBoundingVolume {
        self.content_bounding_volume
            .as_ref()
            .unwrap_or(&self.bounding_volume)
    }

    /// Returns true if this tile has empty content.
    pub fn has_empty_content(&self) -> bool {
        self.empty_content.is_some()
    }

    /// Destroys this tile.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }

    /// Returns whether this tile has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }
}
