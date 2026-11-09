use cesium_math::{DeveloperError, RuntimeError};
use crate::implicit_subdivision_scheme::ImplicitSubdivisionScheme;

/// Specification and templates for an implicit tileset.
#[derive(Debug, Clone, PartialEq)]
pub struct ImplicitTileset {
    pub base_resource: Option<String>,
    pub subdivision_scheme: ImplicitSubdivisionScheme,
    pub branching_factor: usize,
    pub subtree_levels: u32,
    pub available_levels: u32,
    pub geometric_error: f64,
    pub refine: String,
    pub bounding_volume: serde_json::Value,
    pub bounding_box: Option<[f64; 12]>,
    pub content_uri_templates: Vec<String>,
    pub content_headers: Vec<serde_json::Value>,
    pub content_count: usize,
    pub subtree_uri_template: String,
    pub tile_header: serde_json::Value,
    pub metadata_schema: Option<serde_json::Value>,
}

impl ImplicitTileset {
    /// Creates an `ImplicitTileset` with base resource, root tile JSON and optional metadata schema.
    pub fn new(
        base_resource: Option<String>,
        tile_json: &serde_json::Value,
        metadata_schema: Option<serde_json::Value>,
    ) -> Result<Self, RuntimeError> {
        let geometric_error = tile_json
            .get("geometricError")
            .and_then(|g| g.as_f64())
            .unwrap_or(0.0);
        let refine = tile_json
            .get("refine")
            .and_then(|r| r.as_str())
            .unwrap_or("REPLACE")
            .to_string();

        let bounding_volume = tile_json
            .get("boundingVolume")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        // Bounding spheres are rejected for implicit tiling
        let is_sphere = bounding_volume.get("sphere").is_some();
        let has_box = bounding_volume.get("box").is_some();
        let has_region = bounding_volume.get("region").is_some();
        let has_s2 = bounding_volume
            .get("extensions")
            .and_then(|ext| ext.get("3DTILES_bounding_volume_S2"))
            .is_some();
        let has_cylinder = bounding_volume
            .get("extensions")
            .and_then(|ext| ext.get("3DTILES_bounding_volume_cylinder"))
            .is_some();

        if is_sphere || (!has_box && !has_region && !has_s2 && !has_cylinder) {
            return Err(RuntimeError::new(
                "Only box, region, 3DTILES_bounding_volume_S2, and 3DTILES_bounding_volume_cylinder are supported for implicit tiling",
            ));
        }

        let bounding_box = bounding_volume
            .get("box")
            .and_then(|b| b.as_array())
            .and_then(|arr| {
                if arr.len() == 12 {
                    let mut box_arr = [0.0f64; 12];
                    for (i, v) in arr.iter().enumerate() {
                        box_arr[i] = v.as_f64().unwrap_or(0.0);
                    }
                    Some(box_arr)
                } else {
                    None
                }
            });

        let implicit_obj = tile_json
            .get("implicitTiling")
            .or_else(|| {
                tile_json
                    .get("extensions")
                    .and_then(|e| e.get("3DTILES_implicit_tiling"))
            })
            .ok_or_else(|| RuntimeError::new("Missing implicitTiling property in tile JSON"))?;

        let scheme_str = implicit_obj
            .get("subdivisionScheme")
            .and_then(|s| s.as_str())
            .unwrap_or("QUADTREE");
        let subdivision_scheme = ImplicitSubdivisionScheme::from_scheme_str(scheme_str)
            .map_err(|e| RuntimeError::new(e.message))?;

        let branching_factor = subdivision_scheme.branching_factor();

        let subtree_levels = implicit_obj
            .get("subtreeLevels")
            .and_then(|l| l.as_u64())
            .unwrap_or(1) as u32;

        let available_levels = if let Some(avail) = implicit_obj.get("availableLevels").and_then(|l| l.as_u64()) {
            avail as u32
        } else if let Some(max_lvl) = implicit_obj.get("maximumLevel").and_then(|l| l.as_u64()) {
            (max_lvl + 1) as u32
        } else {
            subtree_levels
        };

        let subtree_uri_template = implicit_obj
            .get("subtrees")
            .and_then(|s| s.get("uri"))
            .and_then(|u| u.as_str())
            .unwrap_or("{level}/{x}/{y}.subtree")
            .to_string();

        let content_headers = gather_content_headers(tile_json);
        let mut content_uri_templates = Vec::new();
        for content in &content_headers {
            if let Some(uri) = content.get("uri").and_then(|u| u.as_str()) {
                content_uri_templates.push(uri.to_string());
            }
        }
        let content_count = content_headers.len();

        let tile_header = make_tile_header_template(tile_json);

        Ok(Self {
            base_resource,
            subdivision_scheme,
            branching_factor,
            subtree_levels,
            available_levels,
            geometric_error,
            refine,
            bounding_volume,
            bounding_box,
            content_uri_templates,
            content_headers,
            content_count,
            subtree_uri_template,
            tile_header,
            metadata_schema,
        })
    }

    /// Creates an `ImplicitTileset` from root tile JSON without metadata schema.
    pub fn from_tile_json(json: &serde_json::Value) -> Result<Self, DeveloperError> {
        Self::new(None, json, None).map_err(|e| DeveloperError::new(e.message))
    }

    /// Formats a URI template replacing `{level}`, `{x}`, `{y}`, `{z}`.
    pub fn format_uri(&self, template: &str, level: u32, x: u32, y: u32, z: u32) -> String {
        template
            .replace("{level}", &level.to_string())
            .replace("{x}", &x.to_string())
            .replace("{y}", &y.to_string())
            .replace("{z}", &z.to_string())
    }
}

fn gather_content_headers(tile_json: &serde_json::Value) -> Vec<serde_json::Value> {
    if let Some(multi_ext) = tile_json
        .get("extensions")
        .and_then(|ext| ext.get("3DTILES_multiple_contents"))
    {
        if let Some(contents) = multi_ext.get("contents").and_then(|c| c.as_array()) {
            return contents.clone();
        }
        if let Some(content) = multi_ext.get("content") {
            return vec![content.clone()];
        }
    }

    if let Some(contents) = tile_json.get("contents").and_then(|c| c.as_array()) {
        return contents.clone();
    }

    if let Some(content) = tile_json.get("content") {
        return vec![content.clone()];
    }

    Vec::new()
}

fn make_tile_header_template(tile_json: &serde_json::Value) -> serde_json::Value {
    let mut template = tile_json.clone();

    if let Some(ext_val) = template.get_mut("extensions") {
        if let Some(ext_map) = ext_val.as_object_mut() {
            ext_map.remove("3DTILES_implicit_tiling");
            ext_map.remove("3DTILES_multiple_contents");
            if ext_map.is_empty() {
                if let Some(obj) = template.as_object_mut() {
                    obj.remove("extensions");
                }
            }
        }
    }

    if let Some(obj) = template.as_object_mut() {
        obj.remove("implicitTiling");
        obj.remove("contents");
        obj.remove("content");
    }

    template
}
