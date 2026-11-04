//! Draco feature and batch-table attributes for 3D Tiles 1.0 point clouds.
use crate::{checked_range, invalid, DecodeError, DecodeLimits};
use draco_core::{
    DataType, DecoderBuffer, PointAttribute, PointCloud, PointCloudDecoder, PointIndex,
};
use serde_json::{json, Value};

fn component(ty: DataType) -> Result<(&'static str, usize), DecodeError> {
    Ok(match ty {
        DataType::Int8 => ("BYTE", 1),
        DataType::Uint8 => ("UNSIGNED_BYTE", 1),
        DataType::Int16 => ("SHORT", 2),
        DataType::Uint16 => ("UNSIGNED_SHORT", 2),
        DataType::Int32 => ("INT", 4),
        DataType::Uint32 => ("UNSIGNED_INT", 4),
        DataType::Float32 => ("FLOAT", 4),
        DataType::Float64 => ("DOUBLE", 8),
        _ => {
            return Err(DecodeError::Unsupported(
                "Draco attribute component type".into(),
            ))
        }
    })
}

fn append_attribute(
    output: &mut Vec<u8>,
    attribute: &PointAttribute,
    count: usize,
    budget: usize,
) -> Result<usize, DecodeError> {
    let (_, width) = component(attribute.data_type())?;
    let row = width
        .checked_mul(attribute.num_components() as usize)
        .ok_or(DecodeError::Overflow)?;
    let stride =
        usize::try_from(attribute.byte_stride()).map_err(|_| invalid("Draco attribute stride"))?;
    if row == 0 || stride < row {
        return Err(invalid("Draco attribute stride"));
    }
    let offset = output
        .len()
        .checked_add(width - 1)
        .ok_or(DecodeError::Overflow)?
        / width
        * width;
    let end = count
        .checked_mul(row)
        .and_then(|n| n.checked_add(offset))
        .ok_or(DecodeError::Overflow)?;
    if end > budget {
        return Err(DecodeError::Limit("PNTS Draco materialization"));
    }
    output.resize(offset, 0);
    for i in 0..count {
        let mapped = attribute
            .mapped_index(PointIndex(
                u32::try_from(i).map_err(|_| DecodeError::Overflow)?,
            ))
            .0 as usize;
        if mapped >= attribute.size() {
            return Err(invalid("Draco attribute mapping"));
        }
        let start = mapped.checked_mul(stride).ok_or(DecodeError::Overflow)?;
        output.extend_from_slice(checked_range(attribute.buffer().data(), start, row)?);
    }
    Ok(offset)
}

fn by_id<'a>(pc: &'a PointCloud, id: &Value) -> Result<&'a PointAttribute, DecodeError> {
    let id = id
        .as_u64()
        .and_then(|v| u32::try_from(v).ok())
        .ok_or_else(|| invalid("Draco attribute ID"))?;
    (0..pc.num_attributes())
        .map(|i| pc.attribute(i))
        .find(|a| a.unique_id() == id)
        .ok_or_else(|| invalid("Draco missing attribute"))
}

/// Rewrites compressed properties to bounded uncompressed PNTS attributes.
pub(super) fn decompress(
    feature: &mut Value,
    feature_binary: &[u8],
    batch: &mut Value,
    batch_binary: &[u8],
    count: usize,
    limits: &DecodeLimits,
) -> Result<(Vec<u8>, Vec<u8>), DecodeError> {
    let ext = feature
        .pointer("/extensions/3DTILES_draco_point_compression")
        .cloned()
        .ok_or_else(|| invalid("Draco extension"))?;
    let offset = ext
        .get("byteOffset")
        .and_then(Value::as_u64)
        .and_then(|v| usize::try_from(v).ok())
        .ok_or_else(|| invalid("Draco byteOffset"))?;
    let length = ext
        .get("byteLength")
        .and_then(Value::as_u64)
        .and_then(|v| usize::try_from(v).ok())
        .ok_or_else(|| invalid("Draco byteLength"))?;
    let stream = checked_range(feature_binary, offset, length)?;
    let decoder_limits = draco_core::DecodeLimits::default()
        .with_max_points(count as u64)
        .with_max_decoded_bytes(limits.max_decoded_bytes as u64);
    let mut buffer = DecoderBuffer::new(stream).with_limits(decoder_limits);
    let mut pc = PointCloud::new();
    PointCloudDecoder::new()
        .decode(&mut buffer, &mut pc)
        .map_err(|_| invalid("Draco point cloud decode"))?;
    if pc.num_points() != count {
        return Err(invalid("Draco point count"));
    }
    let mut features = feature_binary.to_vec();
    let mut metadata = batch_binary.to_vec();
    if features
        .len()
        .checked_add(metadata.len())
        .ok_or(DecodeError::Overflow)?
        > limits.max_decoded_bytes
    {
        return Err(DecodeError::Limit("PNTS Draco materialization"));
    }
    let properties = ext
        .get("properties")
        .and_then(Value::as_object)
        .ok_or_else(|| invalid("Draco properties"))?;
    for (name, id) in properties {
        let attribute = by_id(&pc, id)?;
        let components = attribute.num_components();
        let data_type = attribute.data_type();
        let valid = match name.as_str() {
            "POSITION" | "NORMAL" => components == 3 && data_type == DataType::Float32,
            "RGB" => components == 3 && data_type == DataType::Uint8,
            "RGBA" => components == 4 && data_type == DataType::Uint8,
            "BATCH_ID" => {
                components == 1
                    && matches!(
                        data_type,
                        DataType::Uint8 | DataType::Uint16 | DataType::Uint32
                    )
            }
            _ => {
                return Err(DecodeError::Unsupported(format!(
                    "Draco PNTS feature {name}"
                )))
            }
        };
        if !valid {
            return Err(invalid("Draco feature attribute type"));
        }
        let offset = append_attribute(
            &mut features,
            attribute,
            count,
            limits.max_decoded_bytes.saturating_sub(metadata.len()),
        )?;
        let (ty, _) = component(data_type)?;
        feature[name] = if name == "BATCH_ID" {
            json!({"byteOffset": offset, "componentType": ty})
        } else {
            json!({"byteOffset": offset})
        };
    }
    if let Some(properties) = batch
        .pointer("/extensions/3DTILES_draco_point_compression/properties")
        .cloned()
    {
        if feature.get("BATCH_ID").is_some()
            && feature.get("BATCH_LENGTH").and_then(Value::as_u64) != Some(count as u64)
        {
            return Err(DecodeError::Unsupported(
                "Draco per-point batch metadata with batched features".into(),
            ));
        }
        for (name, id) in properties
            .as_object()
            .ok_or_else(|| invalid("Draco metadata properties"))?
        {
            let attribute = by_id(&pc, id)?;
            let (ty, _) = component(attribute.data_type())?;
            let shape = match attribute.num_components() {
                1 => "SCALAR",
                2 => "VEC2",
                3 => "VEC3",
                4 => "VEC4",
                _ => return Err(DecodeError::Unsupported("Draco metadata dimensions".into())),
            };
            let offset = append_attribute(
                &mut metadata,
                attribute,
                count,
                limits.max_decoded_bytes.saturating_sub(features.len()),
            )?;
            batch[name] = json!({"byteOffset":offset,"componentType":ty,"type":shape});
        }
    }
    if let Some(object) = batch.as_object_mut() {
        object.remove("extensions");
    }
    Ok((features, metadata))
}
