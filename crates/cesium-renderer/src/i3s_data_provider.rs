//! I3SDataProvider matching CesiumJS `Scene/I3SDataProvider.js` (Apache-2.0).

use cesium_math::{GeoRectangle, RuntimeError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;

/// Statistics about attributes in an I3S building or scene layer.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct I3SStatistics {
    pub url: String,
    pub names: Vec<String>,
    pub values: HashMap<String, Vec<Value>>,
    pub data: Value,
}

impl I3SStatistics {
    /// Creates a new `I3SStatistics` instance.
    pub fn new(url: String) -> Self {
        Self {
            url,
            names: Vec::new(),
            values: HashMap::new(),
            data: Value::Null,
        }
    }

    /// Loads statistics data from a JSON value.
    pub fn load_from_json(&mut self, json: Value) {
        self.data = json.clone();
        self.names.clear();
        self.values.clear();

        if let Some(stats_arr) = json.get("stats").and_then(Value::as_array) {
            for stat in stats_arr {
                if let Some(name) = stat.get("name").and_then(Value::as_str) {
                    let name_str = name.to_string();
                    if !self.names.contains(&name_str) {
                        self.names.push(name_str.clone());
                    }
                    let mut vals = Vec::new();
                    if let Some(values_arr) = stat.get("values").and_then(Value::as_array) {
                        vals.extend(values_arr.iter().cloned());
                    } else if let Some(most_frequent) = stat.get("mostFrequentValues").and_then(Value::as_array) {
                        vals.extend(most_frequent.iter().cloned());
                    }
                    self.values.insert(name_str, vals);
                }
            }
        }
    }

    /// Returns the values collection for the given attribute name.
    pub fn get_values(&self, name: &str) -> Option<&Vec<Value>> {
        self.values.get(name)
    }
}

/// An I3S sublayer representing a node in a Building Scene Layer hierarchy.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct I3SSublayer {
    pub id: i64,
    pub name: String,
    pub model_name: String,
    pub layer_type: String,
    pub visibility: bool,
    pub sublayers: Vec<I3SSublayer>,
    pub data: Value,
}

impl I3SSublayer {
    /// Constructs a sublayer hierarchy from JSON data.
    pub fn from_data(data: &Value) -> Self {
        let id = data.get("id").and_then(Value::as_i64).unwrap_or(0);
        let name = data.get("name").and_then(Value::as_str).unwrap_or("").to_string();
        let model_name = data.get("modelName").and_then(Value::as_str).unwrap_or("").to_string();
        let layer_type = data.get("layerType").and_then(Value::as_str).unwrap_or("").to_string();
        let visibility = data.get("visibility").and_then(Value::as_bool).unwrap_or(true);

        let mut sublayers = Vec::new();
        if let Some(children) = data.get("sublayers").and_then(Value::as_array) {
            for child in children {
                sublayers.push(Self::from_data(child));
            }
        }

        Self {
            id,
            name,
            model_name,
            layer_type,
            visibility,
            sublayers,
            data: data.clone(),
        }
    }

    /// Sets the visibility of this sublayer and updates descendants.
    pub fn set_visibility(&mut self, visibility: bool) {
        self.visibility = visibility;
    }
}

/// An I3S layer in a dataset.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct I3SLayer {
    pub id: i64,
    pub href: String,
    pub layer_type: String,
    pub name: String,
    pub extent: Option<GeoRectangle>,
    pub visibility: bool,
    pub data: Value,
}

/// Initialization options for [`I3SDataProvider`].
#[derive(Clone, Debug, Default)]
pub struct I3SDataProviderOptions {
    /// Human-readable dataset name.
    pub name: Option<String>,
    /// Whether the dataset will be shown. Defaults to true.
    pub show: Option<bool>,
    /// Determines if features will be shown.
    pub show_features: Option<bool>,
    /// Adjusts alpha mode of materials based on vertex transparency.
    pub adjust_material_alpha_mode: Option<bool>,
    /// Parses and applies I3S symbology.
    pub apply_symbology: Option<bool>,
    /// Calculates flat normals for geometry without normals.
    pub calculate_normals: Option<bool>,
}

/// Main data provider for I3S (Indexed 3D Scene Layers) datasets.
#[derive(Clone, Debug, Default)]
pub struct I3SDataProvider {
    name: Option<String>,
    show: bool,
    show_features: bool,
    adjust_material_alpha_mode: bool,
    apply_symbology: bool,
    calculate_normals: bool,
    url: String,
    data: Value,
    extent: Option<GeoRectangle>,
    layers: Vec<I3SLayer>,
    sublayers: Vec<I3SSublayer>,
    attribute_statistics: Vec<I3SStatistics>,
    is_destroyed: bool,
}

impl I3SDataProvider {
    /// Creates a new `I3SDataProvider` instance.
    pub fn new(options: I3SDataProviderOptions) -> Self {
        Self {
            name: options.name,
            show: options.show.unwrap_or(true),
            show_features: options.show_features.unwrap_or(false),
            adjust_material_alpha_mode: options.adjust_material_alpha_mode.unwrap_or(false),
            apply_symbology: options.apply_symbology.unwrap_or(false),
            calculate_normals: options.calculate_normals.unwrap_or(false),
            url: String::new(),
            data: Value::Null,
            extent: None,
            layers: Vec::new(),
            sublayers: Vec::new(),
            attribute_statistics: Vec::new(),
            is_destroyed: false,
        }
    }

    /// Asynchronously creates an I3SDataProvider from URL and options.
    pub async fn from_url<F, Fut>(
        url: &str,
        options: I3SDataProviderOptions,
        fetch_json: F,
    ) -> Result<Self, RuntimeError>
    where
        F: Fn(String) -> Fut + Clone,
        Fut: Future<Output = Result<Value, String>>,
    {
        if url.is_empty() {
            return Err(RuntimeError::new("url is required."));
        }

        let req_url = url.trim_end_matches('/').to_string();
        let sep = if req_url.contains('?') { '&' } else { '?' };
        let full_url = format!("{req_url}{sep}f=pjson");

        let data = fetch_json(full_url.clone())
            .await
            .map_err(|e| RuntimeError::new(format!("Failed to load I3S: {e}")))?;

        if let Some(err) = data.get("error") {
            let msg = err
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("Failed to fetch I3S");
            return Err(RuntimeError::new(msg));
        }

        let mut provider = Self::new(options.clone());
        provider.url = url.to_string();
        provider.data = data.clone();

        let mut layer_extents = Vec::new();

        // Process layers
        let process_single_layer = |l_data: &Value, prov: &mut I3SDataProvider, exts: &mut Vec<GeoRectangle>| {
            let l_type = l_data.get("layerType").and_then(Value::as_str).unwrap_or("");
            if l_type == "Building" {
                if options.show_features.is_none() {
                    prov.show_features = true;
                }
                if options.adjust_material_alpha_mode.is_none() {
                    prov.adjust_material_alpha_mode = true;
                }
                if options.apply_symbology.is_none() {
                    prov.apply_symbology = true;
                }
                if options.calculate_normals.is_none() {
                    prov.calculate_normals = true;
                }

                if let Some(sub_arr) = l_data.get("sublayers").and_then(Value::as_array) {
                    for sub in sub_arr {
                        prov.sublayers.push(I3SSublayer::from_data(sub));
                    }
                }

                if let Some(extent_obj) = l_data.get("fullExtent") {
                    if let (Some(xmin), Some(ymin), Some(xmax), Some(ymax)) = (
                        extent_obj.get("xmin").and_then(Value::as_f64),
                        extent_obj.get("ymin").and_then(Value::as_f64),
                        extent_obj.get("xmax").and_then(Value::as_f64),
                        extent_obj.get("ymax").and_then(Value::as_f64),
                    ) {
                        let rect = GeoRectangle::from_degrees(xmin, ymin, xmax, ymax);
                        exts.push(rect);
                    }
                }
            } else if l_type == "3DObject" || l_type == "IntegratedMesh" {
                let has_texture = l_data.get("textureSetDefinitions").is_some();
                if options.calculate_normals.is_none() && !has_texture {
                    prov.calculate_normals = true;
                }

                let id = l_data.get("id").and_then(Value::as_i64).unwrap_or(0);
                let href = l_data.get("href").and_then(Value::as_str).unwrap_or("").to_string();
                let name = l_data.get("name").and_then(Value::as_str).unwrap_or("").to_string();

                let mut layer_ext = None;
                if let Some(extent_obj) = l_data.get("fullExtent") {
                    if let (Some(xmin), Some(ymin), Some(xmax), Some(ymax)) = (
                        extent_obj.get("xmin").and_then(Value::as_f64),
                        extent_obj.get("ymin").and_then(Value::as_f64),
                        extent_obj.get("xmax").and_then(Value::as_f64),
                        extent_obj.get("ymax").and_then(Value::as_f64),
                    ) {
                        let rect = GeoRectangle::from_degrees(xmin, ymin, xmax, ymax);
                        exts.push(rect);
                        layer_ext = Some(rect);
                    }
                }

                prov.layers.push(I3SLayer {
                    id,
                    href,
                    layer_type: l_type.to_string(),
                    name,
                    extent: layer_ext,
                    visibility: prov.show,
                    data: l_data.clone(),
                });
            }
        };

        if let Some(layers_arr) = data.get("layers").and_then(Value::as_array) {
            for l in layers_arr {
                process_single_layer(l, &mut provider, &mut layer_extents);
            }
        } else {
            process_single_layer(&data, &mut provider, &mut layer_extents);
        }

        // Fetch statistics if defined in building layer
        if let Some(stat_href) = data.get("statisticsHRef").and_then(Value::as_str) {
            let stat_url = format!("{req_url}/{stat_href}");
            if let Ok(stat_json) = fetch_json(stat_url.clone()).await {
                let mut stat = I3SStatistics::new(stat_url);
                stat.load_from_json(stat_json);
                provider.attribute_statistics.push(stat);
            }
        }

        // Compute merged extent
        if !layer_extents.is_empty() {
            let mut min_w = f64::INFINITY;
            let mut min_s = f64::INFINITY;
            let mut max_e = f64::NEG_INFINITY;
            let mut max_n = f64::NEG_INFINITY;
            for ext in &layer_extents {
                min_w = min_w.min(ext.west);
                min_s = min_s.min(ext.south);
                max_e = max_e.max(ext.east);
                max_n = max_n.max(ext.north);
            }
            provider.extent = Some(GeoRectangle {
                west: min_w,
                south: min_s,
                east: max_e,
                north: max_n,
            });
        }

        Ok(provider)
    }

    /// Gets human-readable name.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Gets whether the dataset is shown.
    pub fn show(&self) -> bool {
        self.show
    }

    /// Sets whether the dataset is shown and updates layers.
    pub fn set_show(&mut self, show: bool) {
        if self.show != show {
            self.show = show;
            for l in &mut self.layers {
                l.visibility = show;
            }
        }
    }

    /// Gets whether features are shown.
    pub fn show_features(&self) -> bool {
        self.show_features
    }

    /// Gets whether material alpha mode is adjusted.
    pub fn adjust_material_alpha_mode(&self) -> bool {
        self.adjust_material_alpha_mode
    }

    /// Gets whether symbology is applied.
    pub fn apply_symbology(&self) -> bool {
        self.apply_symbology
    }

    /// Gets whether normals are calculated.
    pub fn calculate_normals(&self) -> bool {
        self.calculate_normals
    }

    /// Gets the collection of layers.
    pub fn layers(&self) -> &[I3SLayer] {
        &self.layers
    }

    /// Gets the collection of building sublayers.
    pub fn sublayers(&self) -> &[I3SSublayer] {
        &self.sublayers
    }

    /// Gets mutable sublayers.
    pub fn sublayers_mut(&mut self) -> &mut [I3SSublayer] {
        &mut self.sublayers
    }

    /// Gets raw I3S JSON data.
    pub fn data(&self) -> &Value {
        &self.data
    }

    /// Gets the bounding extent.
    pub fn extent(&self) -> Option<GeoRectangle> {
        self.extent
    }

    /// Gets all attribute names from attribute statistics.
    pub fn get_attribute_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        for stat in &self.attribute_statistics {
            for n in &stat.names {
                if !names.contains(n) {
                    names.push(n.clone());
                }
            }
        }
        names
    }

    /// Gets collection of values for attribute with given name.
    pub fn get_attribute_values(&self, name: &str) -> Vec<Value> {
        for stat in &self.attribute_statistics {
            if let Some(vals) = stat.get_values(name) {
                return vals.clone();
            }
        }
        Vec::new()
    }

    /// Packs raw JSON glTF into a binary GLB container (version 2).
    pub fn binarize_gltf(raw_gltf: &Value) -> Vec<u8> {
        let json_bytes = serde_json::to_vec(raw_gltf).unwrap_or_default();
        let json_len = json_bytes.len();
        // 4-byte align JSON chunk
        let padding = (4 - (json_len % 4)) % 4;
        let padded_json_len = json_len + padding;
        let total_length = 12 + 8 + padded_json_len;

        let mut glb = Vec::with_capacity(total_length);
        // Header
        glb.extend_from_slice(b"glTF");
        glb.extend_from_slice(&2u32.to_le_bytes()); // version
        glb.extend_from_slice(&(total_length as u32).to_le_bytes());

        // Chunk 0 (JSON)
        glb.extend_from_slice(&(padded_json_len as u32).to_le_bytes());
        glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // "JSON"
        glb.extend_from_slice(&json_bytes);
        glb.resize(glb.len() + padding, b' ');

        glb
    }

    /// Returns true if this object was destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys this provider and releases resources.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        self.layers.clear();
        self.sublayers.clear();
        self.attribute_statistics.clear();
    }
}
