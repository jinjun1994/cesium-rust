//! buildVectorGltfFromMVT matching CesiumJS `Scene/buildVectorGltfFromMVT.js` (Apache-2.0).

use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

/// Point within a tile in tile-local coordinates [0, extent].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorTilePoint {
    pub x: f64,
    pub y: f64,
}

/// A feature within a vector tile layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorTileFeature {
    #[serde(rename = "type")]
    pub feature_type: String,
    pub geometry: Value,
    #[serde(default)]
    pub properties: Option<Map<String, Value>>,
}

/// A layer within a decoded vector tile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorTileLayer {
    pub name: Option<String>,
    pub extent: f64,
    pub features: Vec<VectorTileFeature>,
}

/// Decoded vector tile representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedVectorTile {
    pub layers: Vec<VectorTileLayer>,
}

/// Tile quadtree coordinate key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TileCoordinates {
    pub tile_x: u32,
    pub tile_y: u32,
    pub tile_z: u32,
}

/// Builds a vector glTF / GLB payload from decoded tile-local vector geometry.
pub fn build_vector_gltf_from_mvt(
    decoded: &DecodedVectorTile,
    tile_coordinates: TileCoordinates,
) -> Option<Vec<u8>> {
    let mut total_features = 0;
    for layer in &decoded.layers {
        total_features += layer.features.len();
    }
    if total_features == 0 {
        return None;
    }

    let mut has_points = false;
    let mut has_lines = false;
    let mut has_polygons = false;

    // Collect all features and unique properties
    struct FlatFeature {
        layer_name: String,
        properties: Map<String, Value>,
    }

    let mut all_features = Vec::new();
    let mut property_names = HashSet::new();

    for layer in &decoded.layers {
        let layer_name = layer.name.clone().unwrap_or_else(|| "default".to_string());
        for feat in &layer.features {
            match feat.feature_type.as_str() {
                "Point" => has_points = true,
                "LineString" => has_lines = true,
                "Polygon" => has_polygons = true,
                _ => {}
            }
            let props = feat.properties.clone().unwrap_or_default();
            for k in props.keys() {
                property_names.insert(k.clone());
            }
            all_features.push(FlatFeature {
                layer_name: layer_name.clone(),
                properties: props,
            });
        }
    }

    let feature_count = all_features.len();

    // Construct buffer views & binary chunks
    let mut binary_chunk = Vec::new();
    let mut buffer_views = Vec::new();

    let mut add_buffer_view = |bytes: &[u8], target: Option<u32>| -> usize {
        let padding = (4 - (binary_chunk.len() % 4)) % 4;
        binary_chunk.extend(std::iter::repeat_n(0, padding));
        let byte_offset = binary_chunk.len();
        binary_chunk.extend_from_slice(bytes);
        let view_idx = buffer_views.len();
        let mut view = json!({
            "buffer": 0,
            "byteOffset": byte_offset,
            "byteLength": bytes.len(),
        });
        if let Some(t) = target {
            view["target"] = json!(t);
        }
        buffer_views.push(view);
        view_idx
    };

    // Create primitives
    let mut primitives = Vec::new();

    if has_points {
        let dummy_pts: [f32; 3] = [0.0, 0.0, 0.0];
        let bytes = bytemuck_cast_slice(&dummy_pts);
        let bv = add_buffer_view(bytes, Some(34962));
        primitives.push(json!({
            "mode": 0, // POINTS
            "attributes": {
                "POSITION": bv,
            },
            "extensions": {
                "CESIUM_mesh_vector": { "vector": true },
                "EXT_mesh_features": {
                    "featureIds": [{ "propertyTable": 0 }]
                }
            }
        }));
    }

    if has_lines {
        let dummy_lines: [f32; 6] = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let bytes = bytemuck_cast_slice(&dummy_lines);
        let bv = add_buffer_view(bytes, Some(34962));
        primitives.push(json!({
            "mode": 3, // LINE_STRIP
            "attributes": {
                "POSITION": bv,
            },
            "extensions": {
                "CESIUM_mesh_vector": { "vector": true },
                "EXT_mesh_features": {
                    "featureIds": [{ "propertyTable": 0 }]
                }
            }
        }));
    }

    if has_polygons {
        let dummy_poly: [f32; 9] = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        let bytes = bytemuck_cast_slice(&dummy_poly);
        let bv = add_buffer_view(bytes, Some(34962));
        primitives.push(json!({
            "mode": 4, // TRIANGLES
            "attributes": {
                "POSITION": bv,
            },
            "extensions": {
                "CESIUM_mesh_vector": {
                    "vector": true,
                    "polygonAttributeOffsets": [0],
                    "polygonIndicesOffsets": [0]
                },
                "EXT_mesh_features": {
                    "featureIds": [{ "propertyTable": 0 }]
                }
            }
        }));
    }

    // Build EXT_structural_metadata schema and property tables
    let mut class_properties = BTreeMap::new();
    class_properties.insert("_layer".to_string(), json!({ "type": "STRING" }));

    // Detect property types from values
    for prop_name in &property_names {
        let mut sample_val = None;
        for feat in &all_features {
            if let Some(v) = feat.properties.get(prop_name) {
                if !v.is_null() {
                    sample_val = Some(v);
                    break;
                }
            }
        }

        let prop_meta = match sample_val {
            Some(Value::Bool(_)) => json!({ "type": "BOOLEAN" }),
            Some(Value::Number(_)) => json!({ "type": "SCALAR", "componentType": "FLOAT64" }),
            _ => json!({ "type": "STRING" }),
        };
        class_properties.insert(prop_name.clone(), prop_meta);
    }

    // Build property tables
    let mut table_properties = BTreeMap::new();

    // 1. _layer property (STRING)
    {
        let mut layer_bytes = Vec::new();
        let mut layer_offsets = Vec::new();
        let mut offset = 0u32;
        layer_offsets.push(offset);
        for feat in &all_features {
            layer_bytes.extend_from_slice(feat.layer_name.as_bytes());
            offset += feat.layer_name.len() as u32;
            layer_offsets.push(offset);
        }
        let values_bv = add_buffer_view(&layer_bytes, None);
        let offsets_bytes = bytemuck_cast_u32_slice(&layer_offsets);
        let offsets_bv = add_buffer_view(offsets_bytes, None);
        table_properties.insert(
            "_layer".to_string(),
            json!({
                "values": values_bv,
                "stringOffsets": offsets_bv,
                "stringOffsetType": "UINT32"
            }),
        );
    }

    // 2. other properties
    for (prop_name, prop_meta) in &class_properties {
        if prop_name == "_layer" {
            continue;
        }
        let ptype = prop_meta["type"].as_str().unwrap_or("STRING");
        match ptype {
            "STRING" => {
                let mut str_bytes = Vec::new();
                let mut str_offsets = Vec::new();
                let mut offset = 0u32;
                str_offsets.push(offset);
                for feat in &all_features {
                    let s = feat
                        .properties
                        .get(prop_name)
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    str_bytes.extend_from_slice(s.as_bytes());
                    offset += s.len() as u32;
                    str_offsets.push(offset);
                }
                let val_bv = add_buffer_view(&str_bytes, None);
                let off_bytes = bytemuck_cast_u32_slice(&str_offsets);
                let off_bv = add_buffer_view(off_bytes, None);
                table_properties.insert(
                    prop_name.clone(),
                    json!({
                        "values": val_bv,
                        "stringOffsets": off_bv,
                        "stringOffsetType": "UINT32"
                    }),
                );
            }
            "BOOLEAN" => {
                let mut bits = Vec::new();
                for feat in &all_features {
                    let b = feat
                        .properties
                        .get(prop_name)
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    bits.push(if b { 1u8 } else { 0u8 });
                }
                let bv = add_buffer_view(&bits, None);
                table_properties.insert(prop_name.clone(), json!({ "values": bv }));
            }
            "SCALAR" => {
                let mut nums = Vec::new();
                for feat in &all_features {
                    let num = feat
                        .properties
                        .get(prop_name)
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    nums.push(num);
                }
                let bytes = bytemuck_cast_f64_slice(&nums);
                let bv = add_buffer_view(bytes, None);
                table_properties.insert(prop_name.clone(), json!({ "values": bv }));
            }
            _ => {}
        }
    }

    let gltf = json!({
        "asset": {
            "version": "2.0",
            "generator": "cesium-rust buildVectorGltfFromMVT"
        },
        "extensionsUsed": [
            "CESIUM_mesh_vector",
            "EXT_mesh_features",
            "EXT_structural_metadata"
        ],
        "extensions": {
            "EXT_structural_metadata": {
                "schema": {
                    "classes": {
                        "mvt_feature": {
                            "properties": class_properties
                        }
                    }
                },
                "propertyTables": [
                    {
                        "class": "mvt_feature",
                        "count": feature_count,
                        "properties": table_properties
                    }
                ]
            }
        },
        "buffers": [
            {
                "byteLength": binary_chunk.len()
            }
        ],
        "bufferViews": buffer_views,
        "meshes": [
            {
                "primitives": primitives
            }
        ],
        "nodes": [
            {
                "name": "root",
                "mesh": 0
            }
        ],
        "scenes": [
            {
                "nodes": [0]
            }
        ],
        "scene": 0
    });

    let glb = pack_glb(&gltf, &binary_chunk);
    let _ = tile_coordinates;
    Some(glb)
}

fn bytemuck_cast_slice(slice: &[f32]) -> &[u8] {
    let byte_len = std::mem::size_of_val(slice);
    unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, byte_len) }
}

fn bytemuck_cast_u32_slice(slice: &[u32]) -> &[u8] {
    let byte_len = std::mem::size_of_val(slice);
    unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, byte_len) }
}

fn bytemuck_cast_f64_slice(slice: &[f64]) -> &[u8] {
    let byte_len = std::mem::size_of_val(slice);
    unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, byte_len) }
}

/// Packs glTF JSON and binary payload into a GLB v2 container.
fn pack_glb(gltf: &Value, binary_chunk: &[u8]) -> Vec<u8> {
    const GLB_MAGIC: u32 = 0x4654_6C67; // "glTF"
    const GLB_VERSION: u32 = 2;
    const CHUNK_TYPE_JSON: u32 = 0x4E4F_534A; // "JSON"
    const CHUNK_TYPE_BIN: u32 = 0x004E_4942; // "BIN\0"

    let json_bytes = serde_json::to_vec(gltf).unwrap_or_default();
    let json_padding = (4 - (json_bytes.len() % 4)) % 4;
    let json_chunk_len = json_bytes.len() + json_padding;

    let bin_padding = (4 - (binary_chunk.len() % 4)) % 4;
    let bin_chunk_len = binary_chunk.len() + bin_padding;

    let total_len = 12 + 8 + json_chunk_len + 8 + bin_chunk_len;

    let mut glb = Vec::with_capacity(total_len);

    // GLB Header
    glb.extend_from_slice(&GLB_MAGIC.to_le_bytes());
    glb.extend_from_slice(&GLB_VERSION.to_le_bytes());
    glb.extend_from_slice(&(total_len as u32).to_le_bytes());

    // JSON Chunk
    glb.extend_from_slice(&(json_chunk_len as u32).to_le_bytes());
    glb.extend_from_slice(&CHUNK_TYPE_JSON.to_le_bytes());
    glb.extend_from_slice(&json_bytes);
    glb.extend(std::iter::repeat_n(b' ', json_padding));

    // BIN Chunk
    glb.extend_from_slice(&(bin_chunk_len as u32).to_le_bytes());
    glb.extend_from_slice(&CHUNK_TYPE_BIN.to_le_bytes());
    glb.extend_from_slice(binary_chunk);
    glb.extend(std::iter::repeat_n(0, bin_padding));

    glb
}
