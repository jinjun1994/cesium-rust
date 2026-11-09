use cesium_math::DeveloperError;
use crate::implicit_subdivision_scheme::ImplicitSubdivisionScheme;

/// Specification and templates for an implicit tileset.
#[derive(Debug, Clone)]
pub struct ImplicitTileset {
    pub subdivision_scheme: ImplicitSubdivisionScheme,
    pub subtree_levels: u32,
    pub available_levels: u32,
    pub geometric_error: f64,
    pub refine: String,
    pub bounding_box: Option<[f64; 12]>,
    pub content_uri_templates: Vec<String>,
    pub subtree_uri_template: String,
}

impl ImplicitTileset {
    /// Creates an `ImplicitTileset` from root tile JSON.
    pub fn from_tile_json(json: &serde_json::Value) -> Result<Self, DeveloperError> {
        let geometric_error = json
            .get("geometricError")
            .and_then(|g| g.as_f64())
            .unwrap_or(0.0);
        let refine = json
            .get("refine")
            .and_then(|r| r.as_str())
            .unwrap_or("REPLACE")
            .to_string();

        let bounding_box = json
            .get("boundingVolume")
            .and_then(|bv| bv.get("box"))
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

        let implicit_obj = json
            .get("implicitTiling")
            .or_else(|| {
                json.get("extensions")
                    .and_then(|e| e.get("3DTILES_implicit_tiling"))
            })
            .ok_or_else(|| DeveloperError::new("Missing implicitTiling property in tile JSON"))?;

        let scheme_str = implicit_obj
            .get("subdivisionScheme")
            .and_then(|s| s.as_str())
            .unwrap_or("QUADTREE");
        let subdivision_scheme = ImplicitSubdivisionScheme::from_scheme_str(scheme_str)?;

        let subtree_levels = implicit_obj
            .get("subtreeLevels")
            .and_then(|l| l.as_u64())
            .unwrap_or(1) as u32;

        let available_levels = implicit_obj
            .get("availableLevels")
            .and_then(|l| l.as_u64())
            .unwrap_or(subtree_levels as u64) as u32;

        let subtree_uri_template = implicit_obj
            .get("subtrees")
            .and_then(|s| s.get("uri"))
            .and_then(|u| u.as_str())
            .unwrap_or("{level}/{x}/{y}.subtree")
            .to_string();

        let mut content_uri_templates = Vec::new();
        if let Some(content) = json.get("content") {
            if let Some(uri) = content.get("uri").and_then(|u| u.as_str()) {
                content_uri_templates.push(uri.to_string());
            }
        } else if let Some(contents) = json.get("contents").and_then(|c| c.as_array()) {
            for c in contents {
                if let Some(uri) = c.get("uri").and_then(|u| u.as_str()) {
                    content_uri_templates.push(uri.to_string());
                }
            }
        }

        Ok(Self {
            subdivision_scheme,
            subtree_levels,
            available_levels,
            geometric_error,
            refine,
            bounding_box,
            content_uri_templates,
            subtree_uri_template,
        })
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
