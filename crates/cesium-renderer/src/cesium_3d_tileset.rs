//! Cesium3DTileset implementation matching CesiumJS `Scene/Cesium3DTileset.js` (Apache-2.0).

use cesium_math::{DeveloperError, Matrix4, RuntimeError};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cesium_3d_tile::{Cesium3DTile, MockTileset};
use crate::cesium_3d_tileset_heatmap::Cesium3DTilesetHeatmap;
use crate::cesium_3d_tileset_metadata::Cesium3DTilesetMetadata;
use crate::cesium_3d_tileset_traversal::{
    Cesium3DTilesetBaseTraversal, Cesium3DTilesetSkipTraversal, Cesium3DTilesetTraversal,
    TraversalFrameState, TraversalTilesetContext,
};
use crate::clipping_plane_collection::ClippingPlaneCollection;
use crate::clipping_polygon_collection::ClippingPolygonCollection;
use crate::commands::DrawCommand;
use crate::create_vector_tile_buffers::BlendOption;
use crate::scene_primitive::PrimitiveRenderFrameState;

/// Height reference mode for positioning 3D tilesets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum TilesetHeightReference {
    #[default]
    None = 0,
    ClampToGround = 1,
    RelativeToGround = 2,
    ClampToTerrain = 3,
    RelativeToTerrain = 4,
}

/// Asset metadata header of a 3D Tiles tileset.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TilesetAsset {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tileset_version: Option<String>,
}

/// Construction options for [`Cesium3DTileset`].
#[derive(Clone, Debug)]
pub struct Cesium3DTilesetOptions {
    pub url: Option<String>,
    pub model_matrix: Option<Matrix4>,
    pub show: Option<bool>,
    pub maximum_screen_space_error: Option<f64>,
    pub cull_with_children_bounds: Option<bool>,
    pub cull_requests_while_moving: Option<bool>,
    pub cull_requests_while_moving_multiplier: Option<f64>,
    pub preload_when_hidden: Option<bool>,
    pub preload_flight_predictions: Option<bool>,
    pub prefer_leaves: Option<bool>,
    pub progressive_resolution_skip_levels: Option<u32>,
    pub foveated_screen_space_error: Option<bool>,
    pub skip_level_of_detail: Option<bool>,
    pub base_screen_space_error: Option<f64>,
    pub skip_screen_space_error_factor: Option<f64>,
    pub skip_levels: Option<u32>,
    pub immediately_load_desired_level_of_detail: Option<bool>,
    pub load_siblings: Option<bool>,
    pub clipping_planes: Option<ClippingPlaneCollection>,
    pub clipping_polygons: Option<ClippingPolygonCollection>,
    pub vector_blend_option: Option<BlendOption>,
    pub height_reference: Option<TilesetHeightReference>,
    pub has_scene: bool,
    pub debug_show_bounding_volume: Option<bool>,
    pub debug_show_viewer_request_volume: Option<bool>,
    pub debug_freeze_frame: Option<bool>,
}

impl Default for Cesium3DTilesetOptions {
    fn default() -> Self {
        Self {
            url: None,
            model_matrix: None,
            show: Some(true),
            maximum_screen_space_error: Some(16.0),
            cull_with_children_bounds: Some(true),
            cull_requests_while_moving: Some(true),
            cull_requests_while_moving_multiplier: Some(60.0),
            preload_when_hidden: Some(false),
            preload_flight_predictions: Some(true),
            prefer_leaves: Some(false),
            progressive_resolution_skip_levels: Some(1),
            foveated_screen_space_error: Some(true),
            skip_level_of_detail: Some(false),
            base_screen_space_error: Some(1024.0),
            skip_screen_space_error_factor: Some(16.0),
            skip_levels: Some(1),
            immediately_load_desired_level_of_detail: Some(false),
            load_siblings: Some(false),
            clipping_planes: None,
            clipping_polygons: None,
            vector_blend_option: Some(BlendOption::Translucent),
            height_reference: Some(TilesetHeightReference::None),
            has_scene: true,
            debug_show_bounding_volume: Some(false),
            debug_show_viewer_request_volume: Some(false),
            debug_freeze_frame: Some(false),
        }
    }
}

/// A 3D Tiles tileset primitive matching CesiumJS `Scene/Cesium3DTileset.js`.
#[derive(Clone, Debug)]
pub struct Cesium3DTileset {
    pub url: String,
    pub resource_url: String,
    pub model_matrix: Matrix4,
    pub show: bool,
    pub maximum_screen_space_error: f64,
    pub cull_with_children_bounds: bool,
    pub cull_requests_while_moving: bool,
    pub cull_requests_while_moving_multiplier: f64,
    pub preload_when_hidden: bool,
    pub preload_flight_predictions: bool,
    pub prefer_leaves: bool,
    pub progressive_resolution_skip_levels: u32,
    pub foveated_screen_space_error: bool,
    pub skip_level_of_detail: bool,
    pub base_screen_space_error: f64,
    pub skip_screen_space_error_factor: f64,
    pub skip_levels: u32,
    pub immediately_load_desired_level_of_detail: bool,
    pub load_siblings: bool,
    pub clipping_planes: Option<ClippingPlaneCollection>,
    pub clipping_polygons: Option<ClippingPolygonCollection>,
    pub vector_blend_option: BlendOption,
    pub height_reference: TilesetHeightReference,
    pub debug_show_bounding_volume: bool,
    pub debug_show_viewer_request_volume: bool,
    pub debug_freeze_frame: bool,
    pub asset: Option<TilesetAsset>,
    pub root: Option<Cesium3DTile>,
    pub metadata: Option<Cesium3DTilesetMetadata>,
    pub heatmap: Cesium3DTilesetHeatmap,
    pub tiles_loaded: bool,
    pub ready: bool,
    draw_commands: Vec<DrawCommand>,
    is_destroyed: bool,
}

impl Default for Cesium3DTileset {
    fn default() -> Self {
        Self::new(Cesium3DTilesetOptions::default()).expect("default tileset valid")
    }
}

impl Cesium3DTileset {
    /// Creates a new `Cesium3DTileset` from options.
    pub fn new(options: Cesium3DTilesetOptions) -> Result<Self, DeveloperError> {
        let height_ref = options.height_reference.unwrap_or(TilesetHeightReference::None);
        if height_ref != TilesetHeightReference::None && !options.has_scene {
            return Err(DeveloperError::new(
                "HeightReference requires scene to be defined.",
            ));
        }

        let url = options.url.unwrap_or_default();
        let resource_url = url.clone();

        Ok(Self {
            url,
            resource_url,
            model_matrix: options.model_matrix.unwrap_or(Matrix4::IDENTITY),
            show: options.show.unwrap_or(true),
            maximum_screen_space_error: options.maximum_screen_space_error.unwrap_or(16.0),
            cull_with_children_bounds: options.cull_with_children_bounds.unwrap_or(true),
            cull_requests_while_moving: options.cull_requests_while_moving.unwrap_or(true),
            cull_requests_while_moving_multiplier: options
                .cull_requests_while_moving_multiplier
                .unwrap_or(60.0),
            preload_when_hidden: options.preload_when_hidden.unwrap_or(false),
            preload_flight_predictions: options.preload_flight_predictions.unwrap_or(true),
            prefer_leaves: options.prefer_leaves.unwrap_or(false),
            progressive_resolution_skip_levels: options
                .progressive_resolution_skip_levels
                .unwrap_or(1),
            foveated_screen_space_error: options.foveated_screen_space_error.unwrap_or(true),
            skip_level_of_detail: options.skip_level_of_detail.unwrap_or(false),
            base_screen_space_error: options.base_screen_space_error.unwrap_or(1024.0),
            skip_screen_space_error_factor: options.skip_screen_space_error_factor.unwrap_or(16.0),
            skip_levels: options.skip_levels.unwrap_or(1),
            immediately_load_desired_level_of_detail: options
                .immediately_load_desired_level_of_detail
                .unwrap_or(false),
            load_siblings: options.load_siblings.unwrap_or(false),
            clipping_planes: options.clipping_planes,
            clipping_polygons: options.clipping_polygons,
            vector_blend_option: options
                .vector_blend_option
                .unwrap_or(BlendOption::Translucent),
            height_reference: height_ref,
            debug_show_bounding_volume: options.debug_show_bounding_volume.unwrap_or(false),
            debug_show_viewer_request_volume: options
                .debug_show_viewer_request_volume
                .unwrap_or(false),
            debug_freeze_frame: options.debug_freeze_frame.unwrap_or(false),
            asset: None,
            root: None,
            metadata: None,
            heatmap: Cesium3DTilesetHeatmap::new("level"),
            tiles_loaded: true,
            ready: true,
            draw_commands: Vec::new(),
            is_destroyed: false,
        })
    }

    /// Loads and parses a tileset JSON string or base64 data URI.
    pub fn load_json(uri_or_content: &str) -> Result<Value, RuntimeError> {
        let json_text = if let Some(encoded) = uri_or_content.strip_prefix("data:text/plain;base64,") {
            let bytes = base64_decode(encoded)
                .map_err(|e| RuntimeError::new(format!("failed to decode base64 URI: {e}")))?;
            String::from_utf8(bytes)
                .map_err(|e| RuntimeError::new(format!("invalid utf-8 in data URI: {e}")))?
        } else {
            uri_or_content.to_string()
        };

        serde_json::from_str(&json_text)
            .map_err(|e| RuntimeError::new(format!("invalid tileset JSON: {e}")))
    }

    /// Validates version and extensions of a parsed tileset JSON.
    pub fn validate_tileset_json(json: &Value) -> Result<(), RuntimeError> {
        if let Some(asset_val) = json.get("asset") {
            if let Some(ver_val) = asset_val.get("version") {
                let ver_str = ver_val.as_str().map(|s| s.to_string()).or_else(|| {
                    ver_val.as_f64().map(|f| format!("{f:.1}"))
                }).unwrap_or_default();

                if ver_str != "0.0" && ver_str != "1.0" && ver_str != "1.1" {
                    return Err(RuntimeError::new(
                        "The tileset must be 3D Tiles version 0.0, 1.0, or 1.1",
                    ));
                }
            }
        }

        if let Some(req_exts) = json.get("extensionsRequired").and_then(|v| v.as_array()) {
            for ext in req_exts {
                if let Some(ext_name) = ext.as_str() {
                    let supported = matches!(
                        ext_name,
                        "3DTILES_multiple_contents"
                            | "3DTILES_batch_table_hierarchy"
                            | "3DTILES_implicit_tiling"
                            | "KHR_texture_basisu"
                    );
                    if !supported {
                        return Err(RuntimeError::new(format!(
                            "Unsupported 3D Tiles Extension: {ext_name}"
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    /// Creates a tileset from a URL, path, or data URI.
    pub fn from_url(
        url: Option<&str>,
        options: Option<Cesium3DTilesetOptions>,
    ) -> Result<Self, Result<DeveloperError, RuntimeError>> {
        let Some(url_str) = url else {
            return Err(Ok(DeveloperError::new(
                "url is required, actual value was undefined",
            )));
        };

        let mut opts = options.unwrap_or_default();
        opts.url = Some(url_str.to_string());
        let mut tileset = Self::new(opts).map_err(Ok)?;

        // If it's a data URI or inline json, parse and configure immediately
        if url_str.starts_with("data:") || url_str.trim_start().starts_with('{') {
            let json = Self::load_json(url_str).map_err(Err)?;
            Self::validate_tileset_json(&json).map_err(Err)?;

            if let Some(asset_val) = json.get("asset") {
                if let Ok(asset) = serde_json::from_value::<TilesetAsset>(asset_val.clone()) {
                    tileset.asset = Some(asset);
                }
            }

            if let Some(root_val) = json.get("root") {
                let mock = MockTileset {
                    model_matrix: tileset.model_matrix,
                    geometric_error: tileset.maximum_screen_space_error,
                    ..Default::default()
                };
                let root_tile = Cesium3DTile::new(&mock, url_str, root_val, None).map_err(Err)?;
                tileset.root = Some(root_tile);
            }
        }

        Ok(tileset)
    }

    /// Creates a tileset from a Cesium ion asset ID.
    pub fn from_ion_asset_id(
        asset_id: Option<u64>,
        _options: Option<Cesium3DTilesetOptions>,
    ) -> Result<Self, DeveloperError> {
        let Some(_id) = asset_id else {
            return Err(DeveloperError::new(
                "assetId is required, actual value was undefined",
            ));
        };
        Err(DeveloperError::new(
            "ion asset loading requires an active ion access token and network context.",
        ))
    }

    /// Updates tileset traversal and emits draw commands for visible selected tiles.
    pub fn update(&mut self, frame_state: &mut PrimitiveRenderFrameState) {
        self.draw_commands.clear();
        if self.is_destroyed || !self.show {
            return;
        }

        // Run traversal if root tile exists
        if let Some(root) = &self.root {
            let mut ctx = TraversalTilesetContext::new(self.maximum_screen_space_error);
            let root_node = crate::cesium_3d_tileset_traversal::TraversalTileNode::new(
                0,
                root.geometric_error,
                root.refine,
            );
            ctx.add_node(root_node);

            let t_state = TraversalFrameState {
                camera_position: cesium_math::Cartesian3::ZERO,
                screen_space_error_threshold: self.maximum_screen_space_error,
                frame_number: frame_state.frame_number,
            };

            if self.skip_level_of_detail {
                Cesium3DTilesetSkipTraversal.select_tiles(&mut ctx, &t_state);
            } else {
                Cesium3DTilesetBaseTraversal.select_tiles(&mut ctx, &t_state);
            }

            if !ctx.selected_tiles.is_empty() {
                self.draw_commands.push(DrawCommand::default());
            }
        }

        frame_state.command_list.extend(self.draw_commands.clone());
    }

    /// Destroys this tileset.
    pub fn destroy(&mut self) {
        if let Some(mut root) = self.root.take() {
            root.destroy();
        }
        self.draw_commands.clear();
        self.is_destroyed = true;
    }

    /// Returns whether this tileset has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::new();
    let mut buf = 0_u32;
    let mut bits = 0_u32;

    for &b in input.as_bytes() {
        if b == b'=' || b.is_ascii_whitespace() {
            continue;
        }
        let val = TABLE.iter().position(|&t| t == b).ok_or("invalid base64 byte")? as u32;
        buf = (buf << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    Ok(out)
}
