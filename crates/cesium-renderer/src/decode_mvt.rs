//! Decodes Mapbox Vector Tile (MVT / .pbf) binary buffers into layers and features.
//! Spec: https://github.com/mapbox/vector-tile-spec/tree/master/2.1
//! Adapted from CesiumJS `Scene/decodeMVT.js` (Apache-2.0).

use cesium_math::RuntimeError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

/// 2D Point in tile-local coordinates [0, extent].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MvtPoint {
    pub x: i64,
    pub y: i64,
}

/// A feature decoded from an MVT layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MvtFeature {
    #[serde(rename = "type")]
    pub feature_type: String,
    pub geometry: Value,
    pub properties: Map<String, Value>,
}

/// A layer within a decoded vector tile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MvtLayer {
    pub name: String,
    pub extent: u32,
    pub features: Vec<MvtFeature>,
}

/// Decoded MVT representation containing all layers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodedMvt {
    pub layers: Vec<MvtLayer>,
}

const GEOM_TYPE_UNKNOWN: u32 = 0;
const GEOM_TYPE_POINT: u32 = 1;
const GEOM_TYPE_LINESTRING: u32 = 2;
const GEOM_TYPE_POLYGON: u32 = 3;

const TILE_FIELD_LAYERS: u32 = 3;

const LAYER_FIELD_NAME: u32 = 1;
const LAYER_FIELD_FEATURES: u32 = 2;
const LAYER_FIELD_KEYS: u32 = 3;
const LAYER_FIELD_VALUES: u32 = 4;
const LAYER_FIELD_EXTENT: u32 = 5;

const FEATURE_FIELD_TAGS: u32 = 2;
const FEATURE_FIELD_TYPE: u32 = 3;
const FEATURE_FIELD_GEOMETRY: u32 = 4;

const VALUE_FIELD_STRING: u32 = 1;
const VALUE_FIELD_FLOAT: u32 = 2;
const VALUE_FIELD_DOUBLE: u32 = 3;
const VALUE_FIELD_INT64: u32 = 4;
const VALUE_FIELD_UINT64: u32 = 5;
const VALUE_FIELD_SINT64: u32 = 6;
const VALUE_FIELD_BOOL: u32 = 7;

/// Decodes an MVT binary buffer into layers and features.
pub fn decode_mvt(bytes: &[u8]) -> Result<DecodedMvt, RuntimeError> {
    let mut layers = Vec::new();
    let mut pos = 0;
    let limit = bytes.len();

    while pos < limit {
        let (field_number, wire_type, new_pos) = read_tag(bytes, pos, limit)?;
        pos = new_pos;

        if field_number == TILE_FIELD_LAYERS && wire_type == 2 {
            let (length, new_pos) = read_varint_length(bytes, pos, limit)?;
            pos = new_pos;
            let layer_end = advance_by_length(pos, length, limit, "layer")?;
            layers.push(decode_layer(bytes, pos, layer_end)?);
            pos = layer_end;
        } else {
            pos = skip_field(bytes, pos, wire_type, limit)?;
        }
    }

    Ok(DecodedMvt { layers })
}

fn decode_layer(bytes: &[u8], start: usize, end: usize) -> Result<MvtLayer, RuntimeError> {
    let mut pos = start;
    let mut name = String::new();
    let mut extent = 4096u32;
    let mut keys = Vec::new();
    let mut values = Vec::new();
    let mut raw_features = Vec::new();

    while pos < end {
        let (field_number, wire_type, new_pos) = read_tag(bytes, pos, end)?;
        pos = new_pos;

        if field_number == LAYER_FIELD_NAME && wire_type == 2 {
            let (length, new_pos) = read_varint_length(bytes, pos, end)?;
            pos = new_pos;
            let string_end = advance_by_length(pos, length, end, "layer name")?;
            name = read_string(bytes, pos, length)?;
            pos = string_end;
        } else if field_number == LAYER_FIELD_FEATURES && wire_type == 2 {
            let (length, new_pos) = read_varint_length(bytes, pos, end)?;
            pos = new_pos;
            let feature_end = advance_by_length(pos, length, end, "feature")?;
            raw_features.push((pos, feature_end));
            pos = feature_end;
        } else if field_number == LAYER_FIELD_KEYS && wire_type == 2 {
            let (length, new_pos) = read_varint_length(bytes, pos, end)?;
            pos = new_pos;
            let string_end = advance_by_length(pos, length, end, "key")?;
            keys.push(read_string(bytes, pos, length)?);
            pos = string_end;
        } else if field_number == LAYER_FIELD_VALUES && wire_type == 2 {
            let (length, new_pos) = read_varint_length(bytes, pos, end)?;
            pos = new_pos;
            let value_end = advance_by_length(pos, length, end, "value")?;
            let val = decode_value(bytes, pos, value_end)?;
            values.push(val);
            pos = value_end;
        } else if field_number == LAYER_FIELD_EXTENT && wire_type == 0 {
            let (val, new_pos) = read_varint32(bytes, pos, end)?;
            extent = val;
            pos = new_pos;
        } else {
            pos = skip_field(bytes, pos, wire_type, end)?;
        }
    }

    let mut features = Vec::with_capacity(raw_features.len());
    for (f_start, f_end) in raw_features {
        features.push(decode_feature(bytes, f_start, f_end, &keys, &values)?);
    }

    Ok(MvtLayer {
        name,
        extent,
        features,
    })
}

fn decode_feature(
    bytes: &[u8],
    start: usize,
    end: usize,
    keys: &[String],
    values: &[Value],
) -> Result<MvtFeature, RuntimeError> {
    let mut pos = start;
    let mut geom_type = GEOM_TYPE_UNKNOWN;
    let mut tags = Vec::new();
    let mut geometry_commands = Vec::new();

    while pos < end {
        let (field_number, wire_type, new_pos) = read_tag(bytes, pos, end)?;
        pos = new_pos;

        if field_number == FEATURE_FIELD_TYPE && wire_type == 0 {
            let (val, new_pos) = read_varint32(bytes, pos, end)?;
            geom_type = val;
            pos = new_pos;
        } else if field_number == FEATURE_FIELD_TAGS && wire_type == 2 {
            let (length, new_pos) = read_varint_length(bytes, pos, end)?;
            pos = new_pos;
            let tag_end = advance_by_length(pos, length, end, "feature tags")?;
            while pos < tag_end {
                let (val, new_pos) = read_varint32(bytes, pos, tag_end)?;
                tags.push(val);
                pos = new_pos;
            }
        } else if field_number == FEATURE_FIELD_GEOMETRY && wire_type == 2 {
            let (length, new_pos) = read_varint_length(bytes, pos, end)?;
            pos = new_pos;
            let geom_end = advance_by_length(pos, length, end, "feature geometry")?;
            while pos < geom_end {
                let (val, new_pos) = read_varint32(bytes, pos, geom_end)?;
                geometry_commands.push(val);
                pos = new_pos;
            }
        } else {
            pos = skip_field(bytes, pos, wire_type, end)?;
        }
    }

    let mut properties = Map::new();
    let mut i = 0;
    while i + 1 < tags.len() {
        let key_idx = tags[i] as usize;
        let val_idx = tags[i + 1] as usize;
        if key_idx < keys.len() && val_idx < values.len() {
            properties.insert(keys[key_idx].clone(), values[val_idx].clone());
        }
        i += 2;
    }

    let (type_str, geometry_json) = decode_geometry(geom_type, &geometry_commands);

    Ok(MvtFeature {
        feature_type: type_str,
        geometry: geometry_json,
        properties,
    })
}

fn decode_geometry(geom_type: u32, cmds: &[u32]) -> (String, Value) {
    let mut i = 0;
    let mut x: i64 = 0;
    let mut y: i64 = 0;

    match geom_type {
        GEOM_TYPE_POINT => {
            let mut points = Vec::new();
            while i < cmds.len() {
                let cmd = cmds[i];
                i += 1;
                let cmd_id = cmd & 0x7;
                let count = cmd >> 3;
                if cmd_id == 1 {
                    // MoveTo
                    for _ in 0..count {
                        if i + 1 < cmds.len() {
                            x += zigzag(cmds[i]);
                            y += zigzag(cmds[i + 1]);
                            i += 2;
                            points.push(json!({ "x": x, "y": y }));
                        }
                    }
                }
            }
            ("Point".to_string(), Value::Array(points))
        }
        GEOM_TYPE_LINESTRING => {
            let mut lines = Vec::new();
            let mut current: Option<Vec<Value>> = None;
            while i < cmds.len() {
                let cmd = cmds[i];
                i += 1;
                let cmd_id = cmd & 0x7;
                let count = cmd >> 3;
                if cmd_id == 1 {
                    // MoveTo - start new line
                    if let Some(cur) = current.take() {
                        lines.push(Value::Array(cur));
                    }
                    let mut cur = Vec::new();
                    if i + 1 < cmds.len() {
                        x += zigzag(cmds[i]);
                        y += zigzag(cmds[i + 1]);
                        i += 2;
                        cur.push(json!({ "x": x, "y": y }));
                    }
                    current = Some(cur);
                } else if cmd_id == 2 {
                    // LineTo
                    if let Some(ref mut cur) = current {
                        for _ in 0..count {
                            if i + 1 < cmds.len() {
                                x += zigzag(cmds[i]);
                                y += zigzag(cmds[i + 1]);
                                i += 2;
                                cur.push(json!({ "x": x, "y": y }));
                            }
                        }
                    }
                }
            }
            if let Some(cur) = current {
                lines.push(Value::Array(cur));
            }
            ("LineString".to_string(), Value::Array(lines))
        }
        GEOM_TYPE_POLYGON => {
            let mut rings = Vec::new();
            let mut current: Option<Vec<Value>> = None;
            while i < cmds.len() {
                let cmd = cmds[i];
                i += 1;
                let cmd_id = cmd & 0x7;
                let count = cmd >> 3;
                if cmd_id == 1 {
                    // MoveTo - start ring
                    if let Some(cur) = current.take() {
                        rings.push(Value::Array(cur));
                    }
                    let mut cur = Vec::new();
                    if i + 1 < cmds.len() {
                        x += zigzag(cmds[i]);
                        y += zigzag(cmds[i + 1]);
                        i += 2;
                        cur.push(json!({ "x": x, "y": y }));
                    }
                    current = Some(cur);
                } else if cmd_id == 2 {
                    // LineTo
                    if let Some(ref mut cur) = current {
                        for _ in 0..count {
                            if i + 1 < cmds.len() {
                                x += zigzag(cmds[i]);
                                y += zigzag(cmds[i + 1]);
                                i += 2;
                                cur.push(json!({ "x": x, "y": y }));
                            }
                        }
                    }
                } else if cmd_id == 7 {
                    // ClosePath
                    if let Some(ref mut cur) = current {
                        if !cur.is_empty() {
                            let first = cur[0].clone();
                            cur.push(first);
                            rings.push(Value::Array(cur.clone()));
                            current = None;
                        }
                    }
                }
            }
            if let Some(cur) = current {
                rings.push(Value::Array(cur));
            }
            ("Polygon".to_string(), Value::Array(rings))
        }
        _ => ("Unknown".to_string(), Value::Array(Vec::new())),
    }
}

fn decode_value(bytes: &[u8], start: usize, end: usize) -> Result<Value, RuntimeError> {
    let mut pos = start;
    while pos < end {
        let (field_number, wire_type, new_pos) = read_tag(bytes, pos, end)?;
        pos = new_pos;

        if field_number == VALUE_FIELD_STRING && wire_type == 2 {
            let (length, new_pos) = read_varint_length(bytes, pos, end)?;
            pos = new_pos;
            let _str_end = advance_by_length(pos, length, end, "string value")?;
            return Ok(Value::String(read_string(bytes, pos, length)?));
        } else if field_number == VALUE_FIELD_FLOAT && wire_type == 5 {
            advance_by_length(pos, 4, end, "float value")?;
            let slice: [u8; 4] = bytes[pos..pos + 4]
                .try_into()
                .map_err(|_| RuntimeError::new("Failed to read float"))?;
            let f = f32::from_le_bytes(slice);
            return Ok(json!(f));
        } else if field_number == VALUE_FIELD_DOUBLE && wire_type == 1 {
            advance_by_length(pos, 8, end, "double value")?;
            let slice: [u8; 8] = bytes[pos..pos + 8]
                .try_into()
                .map_err(|_| RuntimeError::new("Failed to read double"))?;
            let d = f64::from_le_bytes(slice);
            return Ok(json!(d));
        } else if field_number == VALUE_FIELD_INT64 && wire_type == 0 {
            let (v, _) = read_big_varint(bytes, pos, end, 10)?;
            return Ok(json!(v as i64));
        } else if field_number == VALUE_FIELD_UINT64 && wire_type == 0 {
            let (v, _) = read_big_varint(bytes, pos, end, 10)?;
            return Ok(json!(v));
        } else if field_number == VALUE_FIELD_SINT64 && wire_type == 0 {
            let (v, _) = read_big_varint(bytes, pos, end, 10)?;
            let decoded_sint = zigzag_u64(v);
            return Ok(json!(decoded_sint));
        } else if field_number == VALUE_FIELD_BOOL && wire_type == 0 {
            let (v, _) = read_varint32(bytes, pos, end)?;
            return Ok(Value::Bool(v != 0));
        }
        pos = skip_field(bytes, pos, wire_type, end)?;
    }
    Ok(Value::Null)
}

fn read_tag(bytes: &[u8], pos: usize, limit: usize) -> Result<(u32, u32, usize), RuntimeError> {
    let (value, new_pos) = read_varint32(bytes, pos, limit)?;
    Ok((value >> 3, value & 0x7, new_pos))
}

fn read_varint32(bytes: &[u8], pos: usize, limit: usize) -> Result<(u32, usize), RuntimeError> {
    let (value, new_pos) = read_big_varint(bytes, pos, limit, 5)?;
    if value > 0xffff_ffff {
        return Err(RuntimeError::new("Invalid MVT uint32 varint."));
    }
    Ok((value as u32, new_pos))
}

fn read_varint_length(bytes: &[u8], pos: usize, limit: usize) -> Result<(usize, usize), RuntimeError> {
    let (value, new_pos) = read_big_varint(bytes, pos, limit, 10)?;
    Ok((value as usize, new_pos))
}

fn read_big_varint(
    bytes: &[u8],
    mut pos: usize,
    limit: usize,
    max_bytes: usize,
) -> Result<(u64, usize), RuntimeError> {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;
    let mut byte_count = 0;

    loop {
        if pos >= limit || pos >= bytes.len() {
            return Err(RuntimeError::new("Invalid MVT: truncated varint."));
        }
        let byte = bytes[pos];
        pos += 1;
        result |= ((byte & 0x7f) as u64) << shift;
        byte_count += 1;
        if (byte & 0x80) == 0 {
            break;
        }
        if byte_count >= max_bytes {
            return Err(RuntimeError::new("Invalid MVT: varint is too long."));
        }
        shift += 7;
    }

    Ok((result, pos))
}

fn read_string(bytes: &[u8], pos: usize, len: usize) -> Result<String, RuntimeError> {
    let end = advance_by_length(pos, len, bytes.len(), "string")?;
    String::from_utf8(bytes[pos..end].to_vec())
        .map_err(|e| RuntimeError::new(format!("Invalid UTF-8 string: {e}")))
}

fn advance_by_length(
    pos: usize,
    length: usize,
    limit: usize,
    field_name: &str,
) -> Result<usize, RuntimeError> {
    let end = pos.checked_add(length).ok_or_else(|| {
        RuntimeError::new(format!("Invalid MVT {field_name}: length overflow."))
    })?;
    if end > limit {
        return Err(RuntimeError::new(format!(
            "Invalid MVT {field_name}: length exceeds available bytes."
        )));
    }
    Ok(end)
}

fn skip_field(
    bytes: &[u8],
    pos: usize,
    wire_type: u32,
    limit: usize,
) -> Result<usize, RuntimeError> {
    match wire_type {
        0 => {
            let (_, new_pos) = read_big_varint(bytes, pos, limit, 10)?;
            Ok(new_pos)
        }
        1 => advance_by_length(pos, 8, limit, "fixed64 field"),
        2 => {
            let (length, new_pos) = read_varint_length(bytes, pos, limit)?;
            advance_by_length(new_pos, length, limit, "length-delimited field")
        }
        5 => advance_by_length(pos, 4, limit, "fixed32 field"),
        _ => Err(RuntimeError::new(format!(
            "Unsupported protobuf wire type: {wire_type}"
        ))),
    }
}

fn zigzag(n: u32) -> i64 {
    ((n >> 1) as i64) ^ (-((n & 1) as i64))
}

fn zigzag_u64(n: u64) -> i64 {
    ((n >> 1) as i64) ^ (-((n & 1) as i64))
}
