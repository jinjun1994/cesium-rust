//! Bounded, CPU-side PNTS container and feature-table parser.
use crate::{
    check_input, checked_range, decode_batch_table, finite_vec3, invalid, json_section,
    metadata_footprint, u32_at, DecodeError, DecodeLimits,
};
use glam::DVec3;
use serde_json::Value;
use std::borrow::Cow;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PositionEncoding {
    Float32,
    QuantizedU16,
}

#[derive(Debug)]
pub struct Pnts<'a> {
    pub points_length: usize,
    pub feature_table: Value,
    pub feature_binary: Cow<'a, [u8]>,
    pub metadata: Value,
    pub rtc_center: DVec3,
    pub batch_length: Option<usize>,
    pub position_encoding: PositionEncoding,
    pub quantized_volume_scale: Option<DVec3>,
    pub quantized_volume_offset: Option<DVec3>,
    /// Estimated retained bytes excluding borrowed input data.
    pub decoded_bytes: usize,
}

/// CPU point data ready for upload by a point-capable renderer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub normal: [f32; 3],
    pub feature_id: u32,
}

#[derive(Debug)]
pub struct PointCloud {
    /// Tile-local origin in double precision, before any tileset transform.
    pub origin: DVec3,
    pub points: Vec<PointVertex>,
    pub metadata: Value,
    pub has_colors: bool,
    pub has_normals: bool,
    pub decoded_bytes: usize,
}

// Convert JSON typed-array semantics once so all accessors use one bounded
// binary representation. Original binary input remains borrowed when possible.
fn normalize_inline<'a>(
    table: &mut Value,
    binary: &'a [u8],
    count: usize,
    limits: &DecodeLimits,
) -> Result<Cow<'a, [u8]>, DecodeError> {
    let mut output = Cow::Borrowed(binary);
    for (name, components, size) in [
        ("POSITION", 3, 4),
        ("POSITION_QUANTIZED", 3, 2),
        ("NORMAL", 3, 4),
        ("NORMAL_OCT16P", 2, 1),
        ("RGB", 3, 1),
        ("RGBA", 4, 1),
        ("RGB565", 1, 2),
        ("BATCH_ID", 1, 2),
    ] {
        let Some(values) = table.get(name).and_then(Value::as_array) else {
            continue;
        };
        let expected = count.checked_mul(components).ok_or(DecodeError::Overflow)?;
        if values.len() != expected {
            return Err(invalid("PNTS inline attribute count"));
        }
        let start = output
            .len()
            .checked_add(size - 1)
            .ok_or(DecodeError::Overflow)?
            / size
            * size;
        let end = expected
            .checked_mul(size)
            .and_then(|n| start.checked_add(n))
            .ok_or(DecodeError::Overflow)?;
        if end > limits.max_decoded_bytes {
            return Err(DecodeError::Limit("PNTS inline attributes"));
        }
        let buffer = output.to_mut();
        buffer.resize(start, 0);
        for value in values {
            if size == 4 {
                let v = value.as_f64().ok_or_else(|| invalid("PNTS inline float"))? as f32;
                if !v.is_finite() {
                    return Err(invalid("PNTS inline float"));
                }
                buffer.extend_from_slice(&v.to_le_bytes());
            } else {
                let v = value
                    .as_u64()
                    .ok_or_else(|| invalid("PNTS inline integer"))?;
                if size == 1 {
                    buffer.push(u8::try_from(v).map_err(|_| invalid("PNTS inline integer"))?);
                } else {
                    buffer.extend_from_slice(
                        &u16::try_from(v)
                            .map_err(|_| invalid("PNTS inline integer"))?
                            .to_le_bytes(),
                    );
                }
            }
        }
        table[name] = serde_json::json!({"byteOffset":start});
    }
    Ok(output)
}

fn attribute<'a>(
    table: &Value,
    binary: &'a [u8],
    name: &str,
    count: usize,
    components: usize,
    size: usize,
) -> Result<Option<&'a [u8]>, DecodeError> {
    let Some(value) = table.get(name) else {
        return Ok(None);
    };
    let offset = value
        .get("byteOffset")
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid("PNTS attribute byteOffset"))?;
    let offset = usize::try_from(offset).map_err(|_| DecodeError::Overflow)?;
    if !offset.is_multiple_of(size) {
        return Err(invalid("PNTS attribute alignment"));
    }
    let len = count
        .checked_mul(components)
        .and_then(|n| n.checked_mul(size))
        .ok_or(DecodeError::Overflow)?;
    Ok(Some(checked_range(binary, offset, len)?))
}

fn global_vec3(table: &Value, binary: &[u8], name: &str) -> Result<Option<DVec3>, DecodeError> {
    let Some(value) = table.get(name) else {
        return Ok(None);
    };
    if value.is_array() {
        return Ok(Some(finite_vec3(value)?));
    }
    let bytes =
        attribute(table, binary, name, 1, 3, 4)?.ok_or_else(|| invalid("PNTS global property"))?;
    let mut v = [0.0; 3];
    for (i, item) in v.iter_mut().enumerate() {
        *item = f32::from_le_bytes(
            bytes[i * 4..i * 4 + 4]
                .try_into()
                .map_err(|_| DecodeError::Overflow)?,
        ) as f64;
    }
    let value = DVec3::from_array(v);
    if !value.is_finite() {
        return Err(invalid("PNTS non-finite global property"));
    }
    Ok(Some(value))
}

pub fn parse_pnts<'a>(data: &'a [u8], limits: &DecodeLimits) -> Result<Pnts<'a>, DecodeError> {
    check_input(data, limits)?;
    if checked_range(data, 0, 4)? != b"pnts" || u32_at(data, 4)? != 1 {
        return Err(invalid("PNTS magic/version"));
    }
    if u32_at(data, 8)? as usize != data.len() {
        return Err(invalid("PNTS byteLength"));
    }
    let mut cursor = 28usize;
    let mut sections = Vec::with_capacity(4);
    for offset in [12, 16, 20, 24] {
        let len = u32_at(data, offset)? as usize;
        sections.push(checked_range(data, cursor, len)?);
        cursor = cursor.checked_add(len).ok_or(DecodeError::Overflow)?;
    }
    if cursor != data.len() || sections[0].is_empty() {
        return Err(invalid("PNTS section lengths"));
    }
    let mut feature_table = json_section(sections[0])?;
    let count = feature_table
        .get("POINTS_LENGTH")
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid("PNTS POINTS_LENGTH"))?;
    let count = usize::try_from(count).map_err(|_| DecodeError::Overflow)?;
    if count > limits.max_vertices {
        return Err(DecodeError::Limit("point count"));
    }
    let mut table = json_section(sections[2])?;
    let decompressed = if feature_table
        .pointer("/extensions/3DTILES_draco_point_compression")
        .is_some()
    {
        Some(crate::pnts_draco::decompress(
            &mut feature_table,
            sections[1],
            &mut table,
            sections[3],
            count,
            limits,
        )?)
    } else {
        None
    };
    let (feature_binary, batch_storage) = if let Some((features, batch)) = decompressed {
        let features = match normalize_inline(&mut feature_table, &features, count, limits)? {
            Cow::Borrowed(_) => features,
            Cow::Owned(v) => v,
        };
        (Cow::Owned(features), Some(batch))
    } else {
        (
            normalize_inline(&mut feature_table, sections[1], count, limits)?,
            None,
        )
    };
    let position_encoding = if feature_table.get("POSITION").is_some() {
        attribute(
            &feature_table,
            feature_binary.as_ref(),
            "POSITION",
            count,
            3,
            4,
        )?;
        PositionEncoding::Float32
    } else if feature_table.get("POSITION_QUANTIZED").is_some() {
        attribute(
            &feature_table,
            feature_binary.as_ref(),
            "POSITION_QUANTIZED",
            count,
            3,
            2,
        )?;
        PositionEncoding::QuantizedU16
    } else {
        return Err(invalid("PNTS missing positions"));
    };
    let scale = global_vec3(
        &feature_table,
        feature_binary.as_ref(),
        "QUANTIZED_VOLUME_SCALE",
    )?;
    let offset = global_vec3(
        &feature_table,
        feature_binary.as_ref(),
        "QUANTIZED_VOLUME_OFFSET",
    )?;
    if position_encoding == PositionEncoding::QuantizedU16 && (scale.is_none() || offset.is_none())
    {
        return Err(invalid("PNTS missing quantized volume"));
    }
    let rtc_center =
        global_vec3(&feature_table, feature_binary.as_ref(), "RTC_CENTER")?.unwrap_or(DVec3::ZERO);
    for (name, components, size) in [
        ("RGBA", 4, 1),
        ("RGB", 3, 1),
        ("RGB565", 1, 2),
        ("NORMAL", 3, 4),
        ("NORMAL_OCT16P", 2, 1),
    ] {
        attribute(
            &feature_table,
            feature_binary.as_ref(),
            name,
            count,
            components,
            size,
        )?;
    }
    let batch_length = if let Some(batch) = feature_table.get("BATCH_ID") {
        let length = feature_table
            .get("BATCH_LENGTH")
            .and_then(Value::as_u64)
            .ok_or_else(|| invalid("PNTS BATCH_LENGTH"))?;
        let length = usize::try_from(length).map_err(|_| DecodeError::Overflow)?;
        if length > limits.max_vertices {
            return Err(DecodeError::Limit("batch length"));
        }
        let size = match batch
            .get("componentType")
            .and_then(Value::as_str)
            .unwrap_or("UNSIGNED_SHORT")
        {
            "UNSIGNED_BYTE" => 1,
            "UNSIGNED_SHORT" => 2,
            "UNSIGNED_INT" => 4,
            _ => return Err(invalid("PNTS BATCH_ID componentType")),
        };
        let ids = attribute(
            &feature_table,
            feature_binary.as_ref(),
            "BATCH_ID",
            count,
            1,
            size,
        )?
        .ok_or_else(|| invalid("PNTS BATCH_ID"))?;
        for item in ids.chunks_exact(size) {
            let id = match size {
                1 => item[0] as u32,
                2 => u16::from_le_bytes([item[0], item[1]]) as u32,
                4 => u32::from_le_bytes(item.try_into().map_err(|_| DecodeError::Overflow)?),
                _ => unreachable!(),
            };
            if id as usize >= length {
                return Err(invalid("PNTS BATCH_ID out of range"));
            }
        }
        Some(length)
    } else {
        None
    };
    let feature_bytes = metadata_footprint(&feature_table)?
        .checked_add(if matches!(feature_binary, Cow::Owned(_)) {
            feature_binary.len()
        } else {
            0
        })
        .ok_or(DecodeError::Overflow)?;
    let mut metadata_limits = limits.clone();
    metadata_limits.max_decoded_bytes = limits
        .max_decoded_bytes
        .checked_sub(feature_bytes)
        .ok_or(DecodeError::Limit("PNTS decoded bytes"))?;
    let batch_binary = batch_storage.as_deref().unwrap_or(sections[3]);
    if batch_storage.is_some() {
        metadata_limits.max_decoded_bytes = metadata_limits
            .max_decoded_bytes
            .checked_sub(batch_binary.len())
            .ok_or(DecodeError::Limit("PNTS Draco decoded bytes"))?;
    }
    let metadata = decode_batch_table(
        table,
        batch_binary,
        batch_length.unwrap_or(count),
        &metadata_limits,
    )?;
    let decoded_bytes = feature_bytes
        .checked_add(metadata_footprint(&metadata)?)
        .ok_or(DecodeError::Overflow)?;
    if decoded_bytes > limits.max_decoded_bytes {
        return Err(DecodeError::Limit("PNTS metadata"));
    }
    Ok(Pnts {
        points_length: count,
        feature_table,
        feature_binary,
        metadata,
        rtc_center,
        batch_length,
        position_encoding,
        quantized_volume_scale: scale,
        quantized_volume_offset: offset,
        decoded_bytes,
    })
}

/// Materializes validated PNTS points while retaining the precise local origin.
pub fn decode_point_cloud(data: &[u8], limits: &DecodeLimits) -> Result<PointCloud, DecodeError> {
    let pnts = parse_pnts(data, limits)?;
    let vertex_bytes = pnts
        .points_length
        .checked_mul(std::mem::size_of::<PointVertex>())
        .ok_or(DecodeError::Overflow)?;
    let peak_estimate = pnts
        .decoded_bytes
        .checked_add(vertex_bytes)
        .ok_or(DecodeError::Overflow)?;
    if peak_estimate > limits.max_decoded_bytes {
        return Err(DecodeError::Limit("PNTS decoded bytes"));
    }
    let quant_offset = pnts
        .quantized_volume_offset
        .filter(|_| pnts.position_encoding == PositionEncoding::QuantizedU16)
        .unwrap_or(DVec3::ZERO);
    let origin = pnts.rtc_center + quant_offset;
    if !origin.is_finite() {
        return Err(invalid("PNTS origin"));
    }
    let has_colors = ["RGBA", "RGB", "RGB565", "CONSTANT_RGBA"]
        .iter()
        .any(|name| pnts.feature_table.get(*name).is_some());
    let has_normals = ["NORMAL", "NORMAL_OCT16P"]
        .iter()
        .any(|name| pnts.feature_table.get(*name).is_some());
    let mut points = Vec::with_capacity(pnts.points_length);
    for index in 0..pnts.points_length {
        let local = pnts.local_position(index)?;
        let position = local.as_vec3();
        if !position.is_finite() {
            return Err(invalid("PNTS position range"));
        }
        let normal = pnts.normal(index)?.unwrap_or(DVec3::Z).as_vec3();
        let feature_id = pnts
            .batch_id(index)?
            .unwrap_or(u32::try_from(index).map_err(|_| DecodeError::Overflow)?);
        points.push(PointVertex {
            position: position.to_array(),
            color: pnts.color(index)?.unwrap_or([1.0; 4]),
            normal: normal.to_array(),
            feature_id,
        });
    }
    let metadata_bytes = metadata_footprint(&pnts.metadata)?;
    Ok(PointCloud {
        origin,
        points,
        metadata: pnts.metadata,
        has_colors,
        has_normals,
        decoded_bytes: vertex_bytes
            .checked_add(metadata_bytes)
            .ok_or(DecodeError::Overflow)?,
    })
}

impl Pnts<'_> {
    fn point_attribute(
        &self,
        name: &str,
        components: usize,
        size: usize,
        index: usize,
    ) -> Result<Option<&[u8]>, DecodeError> {
        if index >= self.points_length {
            return Err(DecodeError::Range(index));
        }
        let Some(bytes) = attribute(
            &self.feature_table,
            self.feature_binary.as_ref(),
            name,
            self.points_length,
            components,
            size,
        )?
        else {
            return Ok(None);
        };
        let stride = components * size;
        Ok(Some(checked_range(bytes, index * stride, stride)?))
    }

    /// Color as normalized RGBA, or None when the tile has no color property.
    pub fn color(&self, index: usize) -> Result<Option<[f32; 4]>, DecodeError> {
        if let Some(v) = self.point_attribute("RGBA", 4, 1, index)? {
            return Ok(Some([
                v[0] as f32 / 255.0,
                v[1] as f32 / 255.0,
                v[2] as f32 / 255.0,
                v[3] as f32 / 255.0,
            ]));
        }
        if let Some(v) = self.point_attribute("RGB", 3, 1, index)? {
            return Ok(Some([
                v[0] as f32 / 255.0,
                v[1] as f32 / 255.0,
                v[2] as f32 / 255.0,
                1.0,
            ]));
        }
        if let Some(v) = self.point_attribute("RGB565", 1, 2, index)? {
            let value = u16::from_le_bytes([v[0], v[1]]);
            return Ok(Some([
                ((value >> 11) & 31) as f32 / 31.0,
                ((value >> 5) & 63) as f32 / 63.0,
                (value & 31) as f32 / 31.0,
                1.0,
            ]));
        }
        if let Some(value) = self.feature_table.get("CONSTANT_RGBA") {
            let mut rgba = [0u8; 4];
            if let Some(array) = value.as_array() {
                if array.len() != 4 {
                    return Err(invalid("PNTS CONSTANT_RGBA"));
                }
                for (out, v) in rgba.iter_mut().zip(array) {
                    *out = u8::try_from(v.as_u64().ok_or_else(|| invalid("PNTS CONSTANT_RGBA"))?)
                        .map_err(|_| invalid("PNTS CONSTANT_RGBA"))?;
                }
            } else {
                let v = attribute(
                    &self.feature_table,
                    self.feature_binary.as_ref(),
                    "CONSTANT_RGBA",
                    1,
                    4,
                    1,
                )?
                .ok_or_else(|| invalid("PNTS CONSTANT_RGBA"))?;
                rgba.copy_from_slice(v);
            }
            return Ok(Some(rgba.map(|x| x as f32 / 255.0)));
        }
        Ok(None)
    }

    /// Returns the float normal or decodes an octahedrally encoded normal.
    pub fn normal(&self, index: usize) -> Result<Option<DVec3>, DecodeError> {
        if let Some(v) = self.point_attribute("NORMAL", 3, 4, index)? {
            let mut n = [0.0; 3];
            for (i, value) in n.iter_mut().enumerate() {
                *value = f32::from_le_bytes(
                    v[i * 4..i * 4 + 4]
                        .try_into()
                        .map_err(|_| DecodeError::Overflow)?,
                ) as f64;
            }
            let normal = DVec3::from_array(n);
            if !normal.is_finite() {
                return Err(invalid("PNTS non-finite normal"));
            }
            return Ok(Some(normal));
        }
        if let Some(v) = self.point_attribute("NORMAL_OCT16P", 2, 1, index)? {
            let mut x = v[0] as f64 / 255.0 * 2.0 - 1.0;
            let mut y = v[1] as f64 / 255.0 * 2.0 - 1.0;
            let z = 1.0 - x.abs() - y.abs();
            if z < 0.0 {
                let old_x = x;
                x = (1.0 - y.abs()) * if old_x < 0.0 { -1.0 } else { 1.0 };
                y = (1.0 - old_x.abs()) * if y < 0.0 { -1.0 } else { 1.0 };
            }
            return Ok(Some(DVec3::new(x, y, z).normalize()));
        }
        Ok(None)
    }

    pub fn batch_id(&self, index: usize) -> Result<Option<u32>, DecodeError> {
        if index >= self.points_length {
            return Err(DecodeError::Range(index));
        }
        let Some(batch) = self.feature_table.get("BATCH_ID") else {
            return Ok(None);
        };
        let size = match batch
            .get("componentType")
            .and_then(Value::as_str)
            .unwrap_or("UNSIGNED_SHORT")
        {
            "UNSIGNED_BYTE" => 1,
            "UNSIGNED_SHORT" => 2,
            "UNSIGNED_INT" => 4,
            _ => return Err(invalid("PNTS BATCH_ID componentType")),
        };
        let bytes = self
            .point_attribute("BATCH_ID", 1, size, index)?
            .ok_or_else(|| invalid("PNTS BATCH_ID"))?;
        Ok(Some(match size {
            1 => bytes[0] as u32,
            2 => u16::from_le_bytes([bytes[0], bytes[1]]) as u32,
            4 => u32::from_le_bytes(bytes.try_into().map_err(|_| DecodeError::Overflow)?),
            _ => unreachable!(),
        }))
    }
    // Decode before adding the quantization origin, so large offsets cannot erase
    // sub-meter local detail through floating-point cancellation.
    fn local_position(&self, index: usize) -> Result<DVec3, DecodeError> {
        if index >= self.points_length {
            return Err(DecodeError::Range(index));
        }
        let (name, size) = match self.position_encoding {
            PositionEncoding::Float32 => ("POSITION", 4),
            PositionEncoding::QuantizedU16 => ("POSITION_QUANTIZED", 2),
        };
        let bytes = attribute(
            &self.feature_table,
            self.feature_binary.as_ref(),
            name,
            self.points_length,
            3,
            size,
        )?
        .ok_or_else(|| invalid("PNTS missing positions"))?;
        let mut v = [0.0; 3];
        for (i, result) in v.iter_mut().enumerate() {
            let start = (index * 3 + i) * size;
            *result = if size == 4 {
                f32::from_le_bytes(
                    bytes[start..start + 4]
                        .try_into()
                        .map_err(|_| DecodeError::Overflow)?,
                ) as f64
            } else {
                u16::from_le_bytes([bytes[start], bytes[start + 1]]) as f64 / 65535.0
            };
        }
        let value = DVec3::from_array(v);
        let value = match self.position_encoding {
            PositionEncoding::Float32 => value,
            PositionEncoding::QuantizedU16 => {
                value
                    * self
                        .quantized_volume_scale
                        .ok_or_else(|| invalid("PNTS missing scale"))?
            }
        };
        if !value.is_finite() {
            return Err(invalid("PNTS non-finite position"));
        }
        Ok(value)
    }

    /// Returns the point in the tile-local coordinate system (before RTC_CENTER).
    pub fn position(&self, index: usize) -> Result<DVec3, DecodeError> {
        let offset = if self.position_encoding == PositionEncoding::QuantizedU16 {
            self.quantized_volume_offset
                .ok_or_else(|| invalid("PNTS missing offset"))?
        } else {
            DVec3::ZERO
        };
        let value = self.local_position(index)? + offset;
        if !value.is_finite() {
            return Err(invalid("PNTS non-finite position"));
        }
        Ok(value)
    }
}
