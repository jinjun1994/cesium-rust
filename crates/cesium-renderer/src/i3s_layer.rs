//! I3SLayer matching CesiumJS `Scene/I3SLayer.js` (Apache-2.0).

use cesium_math::{Color, GeoRectangle, Resource, RuntimeError};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Attribute filter for an I3S Layer or Node.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct I3SAttributeFilter {
    pub name: String,
    pub values: Vec<Value>,
}

/// Geometry buffer information within an I3S geometry definition.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct I3SGeometryBufferInfo {
    pub compressed: bool,
    pub attributes: Vec<String>,
    pub index: usize,
}

/// A node page entry.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct I3SNodePageEntry {
    pub index: usize,
    pub children: Vec<usize>,
    pub obb: Option<Value>,
    pub mbs: Option<Vec<f64>>,
    pub lod_threshold: Option<f64>,
    pub parent_index: Option<usize>,
}

/// Node representation in I3SLayer.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct I3SRootNode {
    pub index: usize,
    pub data: Value,
}

/// Simulated Cesium3DTileset options for I3SLayer.
#[derive(Clone, Debug, Default)]
pub struct I3STilesetOptions {
    pub debug_show_bounding_volume: Option<bool>,
    pub maximum_screen_space_error: Option<f64>,
    pub outline_color: Option<Color>,
}

/// Simulated Cesium3DTileset created for an I3SLayer.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct I3STileset {
    pub show: bool,
    pub outline_color: Color,
    pub maximum_screen_space_error: f64,
    pub debug_show_bounding_volume: bool,
}

/// Implementation of an I3S Layer.
#[derive(Clone, Debug, Default)]
pub struct I3SLayer {
    pub url: String,
    pub query_params: Option<String>,
    pub data: Value,
    pub version: String,
    pub major_version: u32,
    pub minor_version: u32,
    pub legacy_version_16: bool,
    pub extent: Option<GeoRectangle>,
    pub geometry_definitions: Vec<Vec<I3SGeometryBufferInfo>>,
    pub root_node: Option<I3SRootNode>,
    pub tileset: Option<I3STileset>,
    pub visibility: bool,
    pub resource: Option<Resource>,
}

impl I3SLayer {
    /// Constructs an `I3SLayer` from JSON data and optional base URL.
    pub fn new(base_url: &str, layer_data: &Value) -> Self {
        let url_base = base_url.split('?').next().unwrap_or(base_url);
        let mut tileset_url = url_base.trim_end_matches('/').to_string();

        let mut href = layer_data
            .get("href")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if href.is_empty() {
            if let Some(id) = layer_data.get("id").and_then(Value::as_i64) {
                href = format!("layers/{id}/");
            }
        }

        if !tileset_url.contains("layers/") && !href.is_empty() {
            if !tileset_url.ends_with('/') {
                tileset_url.push('/');
            }
            tileset_url.push_str(&href);
        }

        if !tileset_url.ends_with('/') {
            tileset_url.push('/');
        }

        let query_params = if base_url.contains('?') {
            base_url.split('?').nth(1).map(|s| s.to_string())
        } else {
            None
        };

        // Extract version
        let version = layer_data
            .get("store")
            .and_then(|s| s.get("version"))
            .and_then(Value::as_str)
            .unwrap_or("1.7")
            .to_string();

        let mut major = 1;
        let mut minor = 7;
        let parts: Vec<&str> = version.split('.').collect();
        if let Some(p0) = parts.first() {
            major = p0.parse().unwrap_or(1);
        }
        if parts.len() > 1 {
            minor = parts[1].parse().unwrap_or(0);
        }

        let legacy_version_16 = major < 1 || (major == 1 && minor <= 6);

        let mut full_layer_url = tileset_url.clone();
        if let Some(qp) = &query_params {
            full_layer_url.push('?');
            full_layer_url.push_str(qp);
        }
        let resource = Some(Resource::from_url(&full_layer_url));

        let mut layer = Self {
            url: tileset_url,
            query_params,
            data: layer_data.clone(),
            version,
            major_version: major,
            minor_version: minor,
            legacy_version_16,
            extent: None,
            geometry_definitions: Vec::new(),
            root_node: None,
            tileset: None,
            visibility: true,
            resource,
        };

        layer.compute_extent();
        layer.compute_geometry_definitions(true);
        layer
    }

    /// Computes geodetic bounding extent in radians from layer fullExtent or store.extent.
    pub fn compute_extent(&mut self) {
        if let Some(full_ext) = self.data.get("fullExtent") {
            if let (Some(xmin), Some(ymin), Some(xmax), Some(ymax)) = (
                full_ext.get("xmin").and_then(Value::as_f64),
                full_ext.get("ymin").and_then(Value::as_f64),
                full_ext.get("xmax").and_then(Value::as_f64),
                full_ext.get("ymax").and_then(Value::as_f64),
            ) {
                self.extent = Some(GeoRectangle::from_degrees(xmin, ymin, xmax, ymax));
                return;
            }
        }

        if let Some(store_ext) = self
            .data
            .get("store")
            .and_then(|s| s.get("extent"))
            .and_then(Value::as_array)
        {
            if store_ext.len() >= 4 {
                let xmin = store_ext[0].as_f64().unwrap_or(0.0);
                let ymin = store_ext[1].as_f64().unwrap_or(0.0);
                let xmax = store_ext[2].as_f64().unwrap_or(0.0);
                let ymax = store_ext[3].as_f64().unwrap_or(0.0);
                self.extent = Some(GeoRectangle::from_degrees(xmin, ymin, xmax, ymax));
            }
        }
    }

    /// Computes geometry definitions ranked by compression and attribute count.
    pub fn compute_geometry_definitions(&mut self, use_compression: bool) {
        self.geometry_definitions.clear();

        if let Some(defs_arr) = self.data.get("geometryDefinitions").and_then(Value::as_array) {
            for def in defs_arr {
                let mut buffer_infos = Vec::new();
                if let Some(bufs_arr) = def.get("geometryBuffers").and_then(Value::as_array) {
                    for (buf_idx, buf) in bufs_arr.iter().enumerate() {
                        let mut attributes = Vec::new();
                        let mut compressed = false;

                        if use_compression && buf.get("compressedAttributes").is_some() {
                            compressed = true;
                            if let Some(attr_arr) = buf
                                .get("compressedAttributes")
                                .and_then(|c| c.get("attributes"))
                                .and_then(Value::as_array)
                            {
                                for a in attr_arr {
                                    if let Some(s) = a.as_str() {
                                        attributes.push(s.to_string());
                                    }
                                }
                            }
                        } else if let Some(buf_obj) = buf.as_object() {
                            for k in buf_obj.keys() {
                                if k != "offset" && k != "compressedAttributes" {
                                    attributes.push(k.clone());
                                }
                            }
                        }

                        buffer_infos.push(I3SGeometryBufferInfo {
                            compressed,
                            attributes,
                            index: buf_idx,
                        });
                    }
                }

                // Rank: compressed first, then by attribute count
                buffer_infos.sort_by(|a, b| {
                    if a.compressed != b.compressed {
                        b.compressed.cmp(&a.compressed)
                    } else {
                        a.attributes.len().cmp(&b.attributes.len())
                    }
                });

                self.geometry_definitions.push(buffer_infos);
            }
        }
    }

    /// Finds the best geometry buffer index matching given attributes.
    pub fn find_best_geometry_buffers(
        &self,
        def_index: usize,
        required_attributes: &[&str],
    ) -> Option<usize> {
        let def = self.geometry_definitions.get(def_index)?;
        for info in def {
            let mut all_found = true;
            for &req in required_attributes {
                if !info.attributes.iter().any(|a| a == req) {
                    all_found = false;
                    break;
                }
            }
            if all_found {
                return Some(info.index);
            }
        }

        def.first().map(|info| info.index)
    }

    /// Loads this layer, creating tileset and validating spatial reference.
    pub fn load(&mut self, options: Option<I3STilesetOptions>) -> Result<(), RuntimeError> {
        let wkid = self
            .data
            .get("spatialReference")
            .and_then(|s| s.get("wkid"))
            .and_then(Value::as_i64)
            .unwrap_or(4326);

        if wkid != 4326 {
            return Err(RuntimeError::new(format!(
                "Unsupported spatial reference: {wkid}"
            )));
        }

        // Extract outline color from symbology or options
        let mut outline_color = Color::new(1.0, 1.0, 1.0, 1.0);
        if let Some(sym_edges) = self
            .data
            .get("drawingInfo")
            .and_then(|d| d.get("renderer"))
            .and_then(|r| r.get("symbol"))
            .and_then(|s| s.get("symbolLayers"))
            .and_then(Value::as_array)
            .and_then(|layers| layers.first())
            .and_then(|l| l.get("edges"))
        {
            if let Some(rgb) = sym_edges.get("color").and_then(Value::as_array) {
                if rgb.len() >= 3 {
                    let r = rgb[0].as_f64().unwrap_or(255.0) as f32 / 255.0;
                    let g = rgb[1].as_f64().unwrap_or(255.0) as f32 / 255.0;
                    let b = rgb[2].as_f64().unwrap_or(255.0) as f32 / 255.0;
                    outline_color = Color::new(r, g, b, 1.0);
                }
            }
        }

        let opts = options.unwrap_or_default();
        if let Some(col) = opts.outline_color {
            outline_color = col;
        }

        self.tileset = Some(I3STileset {
            show: self.visibility,
            outline_color,
            maximum_screen_space_error: opts.maximum_screen_space_error.unwrap_or(16.0),
            debug_show_bounding_volume: opts.debug_show_bounding_volume.unwrap_or(false),
        });

        Ok(())
    }

    /// Sets the root node data for this layer.
    pub fn set_root_node(&mut self, index: usize, data: Value) {
        self.root_node = Some(I3SRootNode { index, data });
    }

    /// Updates visibility across the layer and internal tileset.
    pub fn update_visibility(&mut self, visible: bool) {
        self.visibility = visible;
        if let Some(ts) = &mut self.tileset {
            ts.show = visible;
        }
    }

    /// Filters features by attributes.
    pub fn filter_by_attributes(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }
}
