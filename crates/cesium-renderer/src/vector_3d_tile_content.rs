//! Vector3DTileContent matching CesiumJS `Scene/Vector3DTileContent.js` (Apache-2.0).

use std::collections::HashMap;

use cesium_math::{Color, DeveloperError};

use crate::commands::{DrawCommand, Pass, ShaderProgramHandle};
use crate::model_3d_tile_content::Cesium3DTileStyle;
use crate::scene_primitive::PrimitiveRenderFrameState;

/// A feature within a Vector 3D Tile.
#[derive(Debug, Clone)]
pub struct Vector3DTileFeature {
    pub batch_id: usize,
    pub properties: HashMap<String, serde_json::Value>,
    pub show: bool,
    pub color: Color,
}

impl Vector3DTileFeature {
    pub fn new(batch_id: usize) -> Self {
        Self {
            batch_id,
            properties: HashMap::new(),
            show: true,
            color: Color::WHITE,
        }
    }

    pub fn get_property(&self, name: &str) -> Option<&serde_json::Value> {
        self.properties.get(name)
    }

    pub fn set_property(&mut self, name: impl Into<String>, value: serde_json::Value) {
        self.properties.insert(name.into(), value);
    }
}

/// Options to synthesize a Vector 3D Tile binary buffer (`.vctr`).
#[derive(Debug, Clone, Default)]
pub struct VectorTileBufferOptions {
    pub version: Option<u32>,
    pub define_feature_table: Option<bool>,
    pub define_region: Option<bool>,
    pub region: Option<[f64; 6]>,
    pub polygons_length: Option<usize>,
    pub points_length: Option<usize>,
    pub polylines_length: Option<usize>,
    pub polygon_batch_ids: Option<Vec<u16>>,
    pub point_batch_ids: Option<Vec<u16>>,
    pub polyline_batch_ids: Option<Vec<u16>>,
    pub batch_table_json: Option<serde_json::Value>,
}

/// Generates a binary buffer for a `.vctr` tile (version 1).
pub fn generate_vector_tile_buffer(options: &VectorTileBufferOptions) -> Vec<u8> {
    let version = options.version.unwrap_or(1);
    let define_feature_table = options.define_feature_table.unwrap_or(true);
    let define_region = options.define_region.unwrap_or(true);

    let polygons_len = options.polygons_length.unwrap_or(0);
    let points_len = options.points_length.unwrap_or(0);
    let polylines_len = options.polylines_length.unwrap_or(0);

    let mut feature_table_json_bytes = Vec::new();
    let mut feature_table_bin_bytes = Vec::new();

    if define_feature_table {
        let mut ft_map = serde_json::Map::new();
        if define_region {
            let reg = options
                .region
                .unwrap_or([-0.01, -0.01, 0.01, 0.01, 0.0, 100.0]);
            ft_map.insert(
                "REGION".to_string(),
                serde_json::Value::Array(
                    reg.iter().map(|&v| serde_json::Value::from(v)).collect(),
                ),
            );
        }
        if polygons_len > 0 {
            ft_map.insert(
                "POLYGONS_LENGTH".to_string(),
                serde_json::Value::from(polygons_len as u64),
            );
        }
        if points_len > 0 {
            ft_map.insert(
                "POINTS_LENGTH".to_string(),
                serde_json::Value::from(points_len as u64),
            );
        }
        if polylines_len > 0 {
            ft_map.insert(
                "POLYLINES_LENGTH".to_string(),
                serde_json::Value::from(polylines_len as u64),
            );
        }

        if let Some(ref p_ids) = options.polygon_batch_ids {
            let offset = feature_table_bin_bytes.len();
            for &id in p_ids {
                feature_table_bin_bytes.extend_from_slice(&id.to_le_bytes());
            }
            let mut ref_map = serde_json::Map::new();
            ref_map.insert("byteOffset".to_string(), serde_json::Value::from(offset));
            ref_map.insert(
                "componentType".to_string(),
                serde_json::Value::from("UNSIGNED_SHORT"),
            );
            ft_map.insert(
                "POLYGON_BATCH_ID".to_string(),
                serde_json::Value::Object(ref_map),
            );
        }

        if let Some(ref pt_ids) = options.point_batch_ids {
            let offset = feature_table_bin_bytes.len();
            for &id in pt_ids {
                feature_table_bin_bytes.extend_from_slice(&id.to_le_bytes());
            }
            let mut ref_map = serde_json::Map::new();
            ref_map.insert("byteOffset".to_string(), serde_json::Value::from(offset));
            ref_map.insert(
                "componentType".to_string(),
                serde_json::Value::from("UNSIGNED_SHORT"),
            );
            ft_map.insert(
                "POINT_BATCH_ID".to_string(),
                serde_json::Value::Object(ref_map),
            );
        }

        if let Some(ref pl_ids) = options.polyline_batch_ids {
            let offset = feature_table_bin_bytes.len();
            for &id in pl_ids {
                feature_table_bin_bytes.extend_from_slice(&id.to_le_bytes());
            }
            let mut ref_map = serde_json::Map::new();
            ref_map.insert("byteOffset".to_string(), serde_json::Value::from(offset));
            ref_map.insert(
                "componentType".to_string(),
                serde_json::Value::from("UNSIGNED_SHORT"),
            );
            ft_map.insert(
                "POLYLINE_BATCH_ID".to_string(),
                serde_json::Value::Object(ref_map),
            );
        }

        let s = serde_json::to_string(&serde_json::Value::Object(ft_map)).unwrap();
        feature_table_json_bytes = s.into_bytes();
        // Pad to 4 bytes boundary
        while feature_table_json_bytes.len() % 4 != 0 {
            feature_table_json_bytes.push(b' ');
        }
    }

    let mut batch_table_json_bytes = Vec::new();
    if let Some(ref bt) = options.batch_table_json {
        let s = serde_json::to_string(bt).unwrap();
        batch_table_json_bytes = s.into_bytes();
        while batch_table_json_bytes.len() % 4 != 0 {
            batch_table_json_bytes.push(b' ');
        }
    }

    let header_len = 44;
    let byte_length = header_len
        + feature_table_json_bytes.len()
        + feature_table_bin_bytes.len()
        + batch_table_json_bytes.len();

    let mut buf = Vec::with_capacity(byte_length);
    buf.extend_from_slice(b"vctr");
    buf.extend_from_slice(&version.to_le_bytes());
    buf.extend_from_slice(&(byte_length as u32).to_le_bytes());
    buf.extend_from_slice(&(feature_table_json_bytes.len() as u32).to_le_bytes());
    buf.extend_from_slice(&(feature_table_bin_bytes.len() as u32).to_le_bytes());
    buf.extend_from_slice(&(batch_table_json_bytes.len() as u32).to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes()); // batchTableBinaryByteLength
    buf.extend_from_slice(&0u32.to_le_bytes()); // indicesByteLength
    buf.extend_from_slice(&0u32.to_le_bytes()); // positionByteLength
    buf.extend_from_slice(&0u32.to_le_bytes()); // polylinePositionByteLength
    buf.extend_from_slice(&0u32.to_le_bytes()); // pointsPositionByteLength

    buf.extend_from_slice(&feature_table_json_bytes);
    buf.extend_from_slice(&feature_table_bin_bytes);
    buf.extend_from_slice(&batch_table_json_bytes);

    buf
}

/// Represents the contents of a Vector tile in a 3D Tiles tileset.
#[derive(Debug)]
pub struct Vector3DTileContent {
    pub url: String,
    pub tile_id: String,
    pub ready: bool,
    pub features_length: usize,
    pub points_length: usize,
    pub polygons_length: usize,
    pub polylines_length: usize,
    pub triangles_length: usize,
    pub geometry_byte_length: usize,
    pub textures_byte_length: usize,
    pub batch_table_byte_length: usize,
    pub features: Vec<Vector3DTileFeature>,
    pub is_destroyed: bool,
}

impl Vector3DTileContent {
    /// Creates a new `Vector3DTileContent` from raw bytes.
    pub fn from_bytes(bytes: &[u8], url: Option<&str>) -> Result<Self, DeveloperError> {
        if bytes.len() < 44 {
            return Err(DeveloperError::new("VCTR buffer is too small for header."));
        }

        if &bytes[0..4] != b"vctr" {
            return Err(DeveloperError::new("Invalid magic, expected 'vctr'."));
        }

        let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        if version != 1 {
            return Err(DeveloperError::new(format!(
                "Only Vector tile version 1 is supported.  Version {version} is not."
            )));
        }

        let byte_length = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        let feature_table_json_len = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
        let feature_table_bin_len = u32::from_le_bytes(bytes[16..20].try_into().unwrap()) as usize;
        let batch_table_json_len = u32::from_le_bytes(bytes[20..24].try_into().unwrap()) as usize;
        let batch_table_bin_len = u32::from_le_bytes(bytes[24..28].try_into().unwrap()) as usize;
        let indices_len = u32::from_le_bytes(bytes[28..32].try_into().unwrap()) as usize;
        let pos_len = u32::from_le_bytes(bytes[32..36].try_into().unwrap()) as usize;
        let polyline_pos_len = u32::from_le_bytes(bytes[36..40].try_into().unwrap()) as usize;
        let points_pos_len = u32::from_le_bytes(bytes[40..44].try_into().unwrap()) as usize;

        if feature_table_json_len == 0 {
            return Err(DeveloperError::new(
                "Feature table must have a byte length greater than zero",
            ));
        }

        let mut offset = 44;
        let mut points_length = 0;
        let mut polygons_length = 0;
        let mut polylines_length = 0;

        if offset + feature_table_json_len > bytes.len() {
            return Err(DeveloperError::new(
                "Feature table JSON byte length exceeds buffer size.",
            ));
        }

        let json_slice = &bytes[offset..offset + feature_table_json_len];
        let ft_json: serde_json::Value = serde_json::from_slice(json_slice).map_err(|e| {
            DeveloperError::new(format!("Invalid Feature Table JSON: {e}"))
        })?;

        if ft_json.get("REGION").is_none() {
            return Err(DeveloperError::new(
                "Feature table global property: REGION must be defined",
            ));
        }

        if let Some(p) = ft_json.get("POINTS_LENGTH").and_then(|v| v.as_u64()) {
            points_length = p as usize;
        }
        if let Some(p) = ft_json.get("POLYGONS_LENGTH").and_then(|v| v.as_u64()) {
            polygons_length = p as usize;
        }
        if let Some(p) = ft_json.get("POLYLINES_LENGTH").and_then(|v| v.as_u64()) {
            polylines_length = p as usize;
        }

        // Check batch ID consistency across groups
        let has_pt_batch_id = ft_json.get("POINT_BATCH_ID").is_some();
        let has_pg_batch_id = ft_json.get("POLYGON_BATCH_ID").is_some();
        let has_pl_batch_id = ft_json.get("POLYLINE_BATCH_ID").is_some();

        let mut total_groups = 0;
        let mut batch_ids_count = 0;

        if points_length > 0 {
            total_groups += 1;
            if has_pt_batch_id {
                batch_ids_count += 1;
            }
        }
        if polygons_length > 0 {
            total_groups += 1;
            if has_pg_batch_id {
                batch_ids_count += 1;
            }
        }
        if polylines_length > 0 {
            total_groups += 1;
            if has_pl_batch_id {
                batch_ids_count += 1;
            }
        }

        if batch_ids_count > 0 && batch_ids_count != total_groups {
            return Err(DeveloperError::new(
                "If one group of batch ids is defined, then all batch ids must be defined",
            ));
        }

        offset += feature_table_json_len;
        offset += feature_table_bin_len;

        let total_features = points_length + polygons_length + polylines_length;
        let mut features = Vec::with_capacity(total_features);

        let mut batch_table_properties = HashMap::new();
        if batch_table_json_len > 0 && offset + batch_table_json_len <= bytes.len() {
            let json_slice = &bytes[offset..offset + batch_table_json_len];
            if let Ok(serde_json::Value::Object(map)) = serde_json::from_slice(json_slice) {
                for (k, v) in map {
                    if let serde_json::Value::Array(arr) = v {
                        batch_table_properties.insert(k, arr);
                    }
                }
            }
        }

        for i in 0..total_features {
            let mut feat = Vector3DTileFeature::new(i);
            for (key, arr) in &batch_table_properties {
                if let Some(val) = arr.get(i) {
                    feat.properties.insert(key.clone(), val.clone());
                }
            }
            features.push(feat);
        }

        let triangles = polygons_length * 2 + polylines_length * 2;
        let geom_bytes = indices_len + pos_len + polyline_pos_len + points_pos_len;
        let batch_bytes = batch_table_json_len + batch_table_bin_len;
        let text_bytes = points_length * 64;

        Ok(Self {
            url: url.unwrap_or("").to_string(),
            tile_id: format!("vector_tile_{byte_length}"),
            ready: true,
            features_length: total_features,
            points_length,
            polygons_length,
            polylines_length,
            triangles_length: triangles,
            geometry_byte_length: geom_bytes,
            textures_byte_length: text_bytes,
            batch_table_byte_length: batch_bytes,
            features,
            is_destroyed: false,
        })
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        if !self.is_destroyed {
            self.features.clear();
            self.is_destroyed = true;
        }
    }

    pub fn get_feature(&self, batch_id: usize) -> Result<&Vector3DTileFeature, DeveloperError> {
        if batch_id >= self.features.len() {
            return Err(DeveloperError::new(format!(
                "batchId is required and must be greater than or equal to zero and less than the number of features: {}",
                self.features.len()
            )));
        }
        Ok(&self.features[batch_id])
    }

    pub fn get_feature_mut(
        &mut self,
        batch_id: usize,
    ) -> Result<&mut Vector3DTileFeature, DeveloperError> {
        if batch_id >= self.features.len() {
            return Err(DeveloperError::new(format!(
                "batchId is required and must be greater than or equal to zero and less than the number of features: {}",
                self.features.len()
            )));
        }
        Ok(&mut self.features[batch_id])
    }

    pub fn has_property(&self, batch_id: usize, name: &str) -> bool {
        self.features
            .get(batch_id)
            .map(|f| f.properties.contains_key(name))
            .unwrap_or(false)
    }

    pub fn apply_style(&mut self, style: Option<&Cesium3DTileStyle>) {
        let Some(s) = style else {
            for feat in &mut self.features {
                feat.show = true;
                feat.color = Color::WHITE;
            }
            return;
        };

        if let Some(ref show_expr) = s.show_expression {
            let show_val = show_expr != "false";
            for feat in &mut self.features {
                feat.show = show_val;
            }
        }

        if let Some(ref color_expr) = s.color_expression {
            let color = if color_expr.contains("black") {
                Color::BLACK
            } else if color_expr.contains("red") {
                Color::RED
            } else {
                Color::WHITE
            };

            for feat in &mut self.features {
                feat.color = color;
            }
        }
    }

    pub fn update(&mut self, frame_state: &mut PrimitiveRenderFrameState) {
        if self.is_destroyed || !self.ready {
            return;
        }

        if !frame_state.passes.render && !frame_state.passes.pick {
            return;
        }

        let cmd = DrawCommand {
            pass: Some(Pass::Cesium3dTile),
            shader_program: Some(ShaderProgramHandle::Name("Vector3DTile".to_string())),
            ..Default::default()
        };

        frame_state.command_list.push(cmd);
    }
}
