//! Strict, bounded binary containers and an explicitly limited static GLB decoder.
use base64::Engine;
use glam::{DMat4, DVec3, Vec3};
use serde_json::Value;
use std::{io::Cursor, sync::Arc};
use thiserror::Error;

mod pnts;
mod pnts_draco;
pub mod gltf_for_each;
pub mod gltf_pipeline;
pub use gltf_for_each::*;
pub use gltf_pipeline::*;
pub use pnts::{decode_point_cloud, parse_pnts, Pnts, PointCloud, PointVertex, PositionEncoding};

#[derive(Clone, Debug)]
pub struct DecodeLimits {
    pub max_input_bytes: usize,
    pub max_decoded_bytes: usize,
    pub max_vertices: usize,
    pub max_indices: usize,
    pub max_nodes: usize,
    pub max_depth: usize,
    pub max_images: usize,
    pub max_image_dimension: u32,
}
impl Default for DecodeLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 64 * 1024 * 1024,
            max_decoded_bytes: 256 * 1024 * 1024,
            max_vertices: 4_000_000,
            max_indices: 12_000_000,
            max_nodes: 100_000,
            max_depth: 64,
            max_images: 256,
            max_image_dimension: 4096,
        }
    }
}
#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("truncated or invalid range at byte {0}")]
    Range(usize),
    #[error("integer overflow")]
    Overflow,
    #[error("resource limit exceeded: {0}")]
    Limit(&'static str),
    #[error("invalid asset: {0}")]
    Invalid(String),
    #[error("unsupported capability: {0}")]
    Unsupported(String),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Gltf(#[from] gltf::Error),
}
fn invalid(s: &str) -> DecodeError {
    DecodeError::Invalid(s.into())
}
pub fn checked_range(data: &[u8], offset: usize, length: usize) -> Result<&[u8], DecodeError> {
    let end = offset.checked_add(length).ok_or(DecodeError::Overflow)?;
    data.get(offset..end).ok_or(DecodeError::Range(offset))
}
fn u32_at(data: &[u8], offset: usize) -> Result<u32, DecodeError> {
    let b = checked_range(data, offset, 4)?;
    Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}
pub fn check_input(data: &[u8], limits: &DecodeLimits) -> Result<(), DecodeError> {
    if data.len() > limits.max_input_bytes {
        return Err(DecodeError::Limit("input bytes"));
    }
    Ok(())
}
fn json_section(bytes: &[u8]) -> Result<Value, DecodeError> {
    if bytes.is_empty() {
        Ok(Value::Object(Default::default()))
    } else {
        Ok(serde_json::from_slice(bytes)?)
    }
}
fn finite_vec3(value: &Value) -> Result<DVec3, DecodeError> {
    let a = value
        .as_array()
        .filter(|a| a.len() == 3)
        .ok_or_else(|| invalid("expected vec3"))?;
    let mut out = [0.0; 3];
    for (o, v) in out.iter_mut().zip(a) {
        *o = v
            .as_f64()
            .filter(|v| v.is_finite())
            .ok_or_else(|| invalid("non-finite vec3"))?;
    }
    Ok(DVec3::from_array(out))
}

// Estimate the retained heap footprint of the decoded metadata tree. This
// covers values, array elements, object keys and strings, but not allocator or
// transient parser overhead.
fn metadata_footprint(value: &Value) -> Result<usize, DecodeError> {
    fn add(total: &mut usize, n: usize) -> Result<(), DecodeError> {
        *total = total.checked_add(n).ok_or(DecodeError::Overflow)?;
        Ok(())
    }
    let mut total = 0;
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        add(&mut total, std::mem::size_of::<Value>())?;
        match value {
            Value::String(s) => add(&mut total, s.len())?,
            Value::Array(values) => pending.extend(values),
            Value::Object(values) => {
                for (key, value) in values {
                    add(&mut total, std::mem::size_of::<String>())?;
                    add(&mut total, key.len())?;
                    pending.push(value);
                }
            }
            _ => {}
        }
    }
    Ok(total)
}

/// Materialize binary Batch Table properties into per-feature JSON values.
fn decode_batch_table(
    mut table: Value,
    binary: &[u8],
    count: usize,
    limits: &DecodeLimits,
) -> Result<Value, DecodeError> {
    let properties = table
        .as_object_mut()
        .ok_or_else(|| invalid("batch table object"))?;
    let mut decoded_size = 0usize;
    for property in properties.values_mut() {
        if let Some(values) = property.as_array() {
            if values.len() != count {
                return Err(invalid("batch property count"));
            }
            continue;
        }
        let descriptor = property
            .as_object()
            .ok_or_else(|| invalid("batch property descriptor"))?;
        let offset = descriptor
            .get("byteOffset")
            .and_then(Value::as_u64)
            .ok_or_else(|| invalid("batch property byteOffset"))?;
        let offset = usize::try_from(offset).map_err(|_| DecodeError::Overflow)?;
        let component = descriptor
            .get("componentType")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("batch property componentType"))?;
        let size = match component {
            "BYTE" | "UNSIGNED_BYTE" => 1,
            "SHORT" | "UNSIGNED_SHORT" => 2,
            "INT" | "UNSIGNED_INT" | "FLOAT" => 4,
            "DOUBLE" => 8,
            _ => return Err(invalid("batch property componentType")),
        };
        let components = match descriptor.get("type").and_then(Value::as_str) {
            Some("SCALAR") => 1,
            Some("VEC2") => 2,
            Some("VEC3") => 3,
            Some("VEC4" | "MAT2") => 4,
            Some("MAT3") => 9,
            Some("MAT4") => 16,
            _ => return Err(invalid("batch property type")),
        };
        if !offset.is_multiple_of(size) {
            return Err(invalid("batch property alignment"));
        }
        let n = count.checked_mul(components).ok_or(DecodeError::Overflow)?;
        let length = n.checked_mul(size).ok_or(DecodeError::Overflow)?;
        let bytes = checked_range(binary, offset, length)?;
        let estimated = n
            .checked_add(count)
            .and_then(|n| n.checked_mul(std::mem::size_of::<Value>()))
            .ok_or(DecodeError::Overflow)?;
        decoded_size = decoded_size
            .checked_add(estimated)
            .ok_or(DecodeError::Overflow)?;
        if decoded_size > limits.max_decoded_bytes {
            return Err(DecodeError::Limit("batch metadata"));
        }
        let mut values = Vec::with_capacity(count);
        for feature in 0..count {
            let mut row = Vec::with_capacity(components);
            for index in 0..components {
                let start = (feature * components + index) * size;
                let b = &bytes[start..start + size];
                let v = match component {
                    "BYTE" => Value::from(i8::from_le_bytes([b[0]])),
                    "UNSIGNED_BYTE" => Value::from(b[0]),
                    "SHORT" => Value::from(i16::from_le_bytes([b[0], b[1]])),
                    "UNSIGNED_SHORT" => Value::from(u16::from_le_bytes([b[0], b[1]])),
                    "INT" => Value::from(i32::from_le_bytes(
                        b.try_into().map_err(|_| DecodeError::Overflow)?,
                    )),
                    "UNSIGNED_INT" => Value::from(u32::from_le_bytes(
                        b.try_into().map_err(|_| DecodeError::Overflow)?,
                    )),
                    "FLOAT" => {
                        let v =
                            f32::from_le_bytes(b.try_into().map_err(|_| DecodeError::Overflow)?);
                        if !v.is_finite() {
                            return Err(invalid("non-finite batch property"));
                        }
                        Value::from(v)
                    }
                    "DOUBLE" => {
                        let v =
                            f64::from_le_bytes(b.try_into().map_err(|_| DecodeError::Overflow)?);
                        if !v.is_finite() {
                            return Err(invalid("non-finite batch property"));
                        }
                        Value::from(v)
                    }
                    _ => unreachable!(),
                };
                row.push(v);
            }
            values.push(if components == 1 {
                row.pop().ok_or_else(|| invalid("batch property"))?
            } else {
                Value::Array(row)
            });
        }
        *property = Value::Array(values);
    }
    Ok(table)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum B3dmHeader {
    Current,
    Legacy20,
    Legacy24,
}

#[derive(Debug)]
pub struct B3dm<'a> {
    pub header: B3dmHeader,
    pub batch_length: u32,
    pub rtc_center: DVec3,
    pub feature_table: Value,
    pub batch_table: Value,
    /// Estimated retained bytes for decoded batch metadata.
    pub batch_metadata_bytes: usize,
    pub feature_binary: &'a [u8],
    pub batch_binary: &'a [u8],
    pub glb: &'a [u8],
}
pub fn parse_b3dm<'a>(data: &'a [u8], limits: &DecodeLimits) -> Result<B3dm<'a>, DecodeError> {
    check_input(data, limits)?;
    if checked_range(data, 0, 4)? != b"b3dm" || u32_at(data, 4)? != 1 {
        return Err(invalid("b3dm magic/version"));
    }
    if u32_at(data, 8)? as usize != data.len() {
        return Err(invalid("b3dm byteLength"));
    }
    // Cesium detects the two historical header layouts using the first
    // bytes of the following JSON section or embedded glTF as a sentinel.
    const LEGACY_SENTINEL: u32 = 570_425_344;
    let a = u32_at(data, 12)?;
    let b = u32_at(data, 16)?;
    let c = u32_at(data, 20)?;
    let d = u32_at(data, 24)?;
    let (header, mut cursor, lengths, legacy_batch_length) = if c >= LEGACY_SENTINEL {
        (B3dmHeader::Legacy20, 20usize, [0, 0, b, 0], Some(a))
    } else if d >= LEGACY_SENTINEL {
        (B3dmHeader::Legacy24, 24usize, [0, 0, a, b], Some(c))
    } else {
        (B3dmHeader::Current, 28usize, [a, b, c, d], None)
    };
    let mut sections = Vec::new();
    for length in lengths {
        let length = length as usize;
        sections.push(checked_range(data, cursor, length)?);
        cursor = cursor.checked_add(length).ok_or(DecodeError::Overflow)?;
    }
    let feature_table = if sections[0].is_empty() {
        serde_json::json!({"BATCH_LENGTH": legacy_batch_length.unwrap_or(0)})
    } else {
        json_section(sections[0])?
    };
    let batch_table = json_section(sections[2])?;
    let count = feature_table
        .get("BATCH_LENGTH")
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid("missing/integer BATCH_LENGTH"))?;
    let batch_length = u32::try_from(count).map_err(|_| DecodeError::Overflow)?;
    if count > limits.max_vertices as u64 {
        return Err(DecodeError::Limit("batch length"));
    }
    let rtc_center = match feature_table.get("RTC_CENTER") {
        None => DVec3::ZERO,
        Some(v) if v.is_array() => finite_vec3(v)?,
        Some(v) => {
            let offset = v
                .get("byteOffset")
                .and_then(Value::as_u64)
                .ok_or_else(|| invalid("RTC_CENTER offset"))?;
            let bytes = checked_range(
                sections[1],
                usize::try_from(offset).map_err(|_| DecodeError::Overflow)?,
                12,
            )?;
            let mut xyz = [0.0; 3];
            for i in 0..3 {
                xyz[i] = f32::from_le_bytes(
                    bytes[i * 4..i * 4 + 4]
                        .try_into()
                        .map_err(|_| invalid("RTC"))?,
                ) as f64;
            }
            if xyz.iter().any(|v| !v.is_finite()) {
                return Err(invalid("RTC_CENTER finite"));
            }
            DVec3::from_array(xyz)
        }
    };
    let glb = checked_range(data, cursor, data.len().saturating_sub(cursor))?;
    parse_glb(glb, limits)?;
    let batch_table = decode_batch_table(batch_table, sections[3], batch_length as usize, limits)?;
    let batch_metadata_bytes = metadata_footprint(&batch_table)?;
    if batch_metadata_bytes > limits.max_decoded_bytes {
        return Err(DecodeError::Limit("batch metadata"));
    }
    Ok(B3dm {
        header,
        batch_length,
        rtc_center,
        feature_table,
        batch_table,
        batch_metadata_bytes,
        feature_binary: sections[1],
        batch_binary: sections[3],
        glb,
    })
}
#[derive(Debug)]
pub struct Glb<'a> {
    pub json: Value,
    pub binary: Option<&'a [u8]>,
}
pub fn parse_glb<'a>(data: &'a [u8], limits: &DecodeLimits) -> Result<Glb<'a>, DecodeError> {
    check_input(data, limits)?;
    if checked_range(data, 0, 4)? != b"glTF" || u32_at(data, 4)? != 2 {
        return Err(invalid("GLB magic/version"));
    }
    if u32_at(data, 8)? as usize != data.len() {
        return Err(invalid("GLB byteLength"));
    }
    let mut cursor = 12usize;
    let mut json = None;
    let mut binary = None;
    let mut chunk = 0;
    while cursor < data.len() {
        let len = u32_at(data, cursor)? as usize;
        let kind = u32_at(data, cursor + 4)?;
        if !len.is_multiple_of(4) {
            return Err(invalid("GLB chunk alignment"));
        }
        cursor = cursor.checked_add(8).ok_or(DecodeError::Overflow)?;
        let bytes = checked_range(data, cursor, len)?;
        match kind {
            0x4e4f534a if chunk == 0 => json = Some(json_section(bytes)?),
            0x004e4942 if chunk == 1 => binary = Some(bytes),
            0x4e4f534a | 0x004e4942 => return Err(invalid("GLB chunk order/duplicates")),
            _ if chunk == 0 => return Err(invalid("first GLB chunk must be JSON")),
            _ => {} // Unknown optional chunks are bounded and ignored per GLB spec.
        }
        chunk += 1;
        cursor = cursor.checked_add(len).ok_or(DecodeError::Overflow)?;
    }
    Ok(Glb {
        json: json.ok_or_else(|| invalid("missing GLB JSON"))?,
        binary,
    })
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub feature_id: u32,
    pub uv1: [f32; 2],
    pub color: [f32; 4],
    pub tangent: [f32; 4],
}
#[derive(Clone, Debug)]
pub struct TextureImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}
#[derive(Clone, Copy, Debug)]
pub struct TextureSampler {
    pub wrap_s: u32,
    pub wrap_t: u32,
    pub mag: u32,
    pub min: u32,
}
impl Default for TextureSampler {
    fn default() -> Self {
        Self {
            wrap_s: 10497,
            wrap_t: 10497,
            mag: 9729,
            min: 9987,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Texture {
    pub image: Arc<TextureImage>,
    pub sampler: TextureSampler,
    pub tex_coord: u32,
}
impl Texture {
    pub fn gpu_bytes(&self) -> usize {
        let (mut w, mut h) = (self.image.width, self.image.height);
        let mut total = 0;
        while w > 0 && h > 0 {
            total += w as usize * h as usize * 4;
            if w == 1 && h == 1 {
                break;
            }
            w = (w / 2).max(1);
            h = (h / 2).max(1);
        }
        total
    }
}
impl Primitive {
    pub fn gpu_bytes(&self) -> usize {
        self.vertices.len() * std::mem::size_of::<Vertex>()
            + self.indices.len() * 4
            + 128
            + self
                .material
                .textures
                .iter()
                .flatten()
                .map(Texture::gpu_bytes)
                .sum::<usize>()
    }
}
#[derive(Clone, Debug)]
pub struct Pattern {
    pub kind: u32,
    pub secondary: [f32; 4],
    pub repeat: [f32; 2],
    pub offset: f32,
    pub axis: u32,
    pub thickness: [f32; 2],
    pub cell_alpha: f32,
}
#[derive(Clone, Debug)]
pub struct Material {
    pub alpha_blend: bool,
    pub pattern: Option<Pattern>,
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub double_sided: bool,
    pub face_forward: bool,
    pub two_pass: bool,
    pub lambert: bool,
    pub alpha_mask: Option<f32>,
    pub emissive: [f32; 3],
    pub normal_scale: f32,
    pub occlusion_strength: f32,
    pub unlit: bool,
    pub textures: [Option<Texture>; 5], // baseColor, metallicRoughness, normal, occlusion, emissive
}
impl Default for Material {
    fn default() -> Self {
        Self {
            alpha_blend: false,
            pattern: None,
            base_color: [1.0; 4],
            metallic: 1.0,
            roughness: 1.0,
            double_sided: false,
            face_forward: true,
            two_pass: false,
            lambert: false,
            alpha_mask: None,
            emissive: [0.0; 3],
            normal_scale: 1.0,
            occlusion_strength: 1.0,
            unlit: false,
            textures: std::array::from_fn(|_| None),
        }
    }
}
#[derive(Clone, Debug)]
pub struct Primitive {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub transform: DMat4,
    pub material: Material,
}
#[derive(Debug)]
pub struct Content {
    pub tile_frame: bool,
    pub primitives: Vec<Primitive>,
    pub rtc_center: DVec3,
    pub metadata: Value,
    pub decoded_bytes: usize,
}

fn decode_images(
    g: &gltf::Gltf,
    binary: Option<&[u8]>,
    limits: &DecodeLimits,
) -> Result<(Vec<Arc<TextureImage>>, usize), DecodeError> {
    if g.images().len() > limits.max_images {
        return Err(DecodeError::Limit("image count"));
    }
    let mut images = Vec::new();
    let mut total = 0usize;
    for img in g.images() {
        let owned;
        let (bytes, mime) = match img.source() {
            gltf::image::Source::View { view, mime_type } => (
                checked_range(
                    binary.ok_or_else(|| invalid("image missing BIN"))?,
                    view.offset(),
                    view.length(),
                )?,
                mime_type,
            ),
            gltf::image::Source::Uri { uri, .. } => {
                let (prefix, data) = uri
                    .split_once(',')
                    .ok_or_else(|| DecodeError::Unsupported("external glTF image URI".into()))?;
                let mime = prefix
                    .strip_prefix("data:")
                    .and_then(|v| v.strip_suffix(";base64"))
                    .ok_or_else(|| {
                        DecodeError::Unsupported(
                            "image URI must be PNG/JPEG base64 data URI".into(),
                        )
                    })?;
                owned = base64::engine::general_purpose::STANDARD
                    .decode(data)
                    .map_err(|_| invalid("invalid image base64"))?;
                (owned.as_slice(), mime)
            }
        };
        let format = match mime {
            "image/png" => image::ImageFormat::Png,
            "image/jpeg" => image::ImageFormat::Jpeg,
            _ => return Err(DecodeError::Unsupported("image must be PNG/JPEG".into())),
        };
        let remaining = limits
            .max_decoded_bytes
            .checked_sub(total)
            .ok_or(DecodeError::Limit("decoded image bytes"))?;
        let mut reader = image::ImageReader::with_format(Cursor::new(bytes), format);
        let mut bounded = image::Limits::default();
        bounded.max_image_width = Some(limits.max_image_dimension);
        bounded.max_image_height = Some(limits.max_image_dimension);
        bounded.max_alloc = Some(remaining as u64);
        reader.limits(bounded);
        let rgba = reader
            .decode()
            .map_err(|_| invalid("image decode failed or exceeds limits"))?
            .to_rgba8();
        total = total
            .checked_add(rgba.as_raw().len())
            .ok_or(DecodeError::Overflow)?;
        if total > limits.max_decoded_bytes {
            return Err(DecodeError::Limit("decoded image bytes"));
        }
        images.push(Arc::new(TextureImage {
            width: rgba.width(),
            height: rgba.height(),
            rgba: rgba.into_raw(),
        }));
    }
    Ok((images, total))
}
fn resolve_texture(
    texture: gltf::Texture<'_>,
    tex_coord: u32,
    images: &[Arc<TextureImage>],
) -> Result<Texture, DecodeError> {
    if tex_coord > 1 {
        return Err(DecodeError::Unsupported(
            "only TEXCOORD_0/1 are supported".into(),
        ));
    }
    let s = texture.sampler();
    Ok(Texture {
        image: images
            .get(texture.source().index())
            .ok_or_else(|| invalid("image index"))?
            .clone(),
        tex_coord,
        sampler: TextureSampler {
            wrap_s: s.wrap_s().as_gl_enum(),
            wrap_t: s.wrap_t().as_gl_enum(),
            mag: s.mag_filter().map_or(9729, |v| v.as_gl_enum()),
            min: s.min_filter().map_or(9987, |v| v.as_gl_enum()),
        },
    })
}
pub fn decode_content(data: &[u8], limits: &DecodeLimits) -> Result<Content, DecodeError> {
    if [b"pnts", b"i3dm", b"cmpt"]
        .iter()
        .any(|magic| data.starts_with(*magic))
    {
        return Err(DecodeError::Unsupported(format!(
            "{} tile content",
            String::from_utf8_lossy(&data[..4])
        )));
    }
    let (glb, rtc_center, metadata, batch_length, metadata_bytes) = if data.starts_with(b"b3dm") {
        let b = parse_b3dm(data, limits)?;
        (
            b.glb,
            b.rtc_center,
            b.batch_table,
            Some(b.batch_length),
            b.batch_metadata_bytes,
        )
    } else {
        (
            data,
            DVec3::ZERO,
            Value::Object(Default::default()),
            None,
            0,
        )
    };
    let owned_bin: Vec<u8>;
    let mut json_gltf = false;
    let mut container = if glb.iter().find(|b| !b.is_ascii_whitespace()) == Some(&b'{') {
        check_input(glb, limits)?;
        let mut json: Value = serde_json::from_slice(glb).map_err(|_| invalid("glTF JSON"))?;
        let uri = json
            .get("buffers")
            .and_then(|b| b.get(0))
            .and_then(|b| b.get("uri"))
            .and_then(Value::as_str)
            .map(str::to_owned);
        owned_bin = if let Some(b64) = uri
            .as_deref()
            .and_then(|u| u.strip_prefix("data:"))
            .and_then(|s| s.split_once(";base64,"))
            .map(|(_, b)| b)
        {
            use base64::Engine;
            if let Some(buf0) = json.get_mut("buffers").and_then(|b| b.get_mut(0)).and_then(Value::as_object_mut) {
                buf0.remove("uri");
            }
            base64::engine::general_purpose::STANDARD
                .decode(b64.trim())
                .map_err(|_| invalid("glTF data URI"))?
        } else {
            Vec::new()
        };
        json_gltf = true;
        Glb {
            json,
            binary: (!owned_bin.is_empty()).then_some(owned_bin.as_slice()),
        }
    } else {
        parse_glb(glb, limits)?
    };
    // Historical b3dm used BATCHID before glTF custom attributes required an
    // underscore. Normalize only that legacy semantic before glTF validation.
    let mut normalized = json_gltf;
    if batch_length.is_some() {
        if let Some(meshes) = container
            .json
            .get_mut("meshes")
            .and_then(Value::as_array_mut)
        {
            for mesh in meshes {
                if let Some(primitives) = mesh.get_mut("primitives").and_then(Value::as_array_mut) {
                    for primitive in primitives {
                        if let Some(attributes) = primitive
                            .get_mut("attributes")
                            .and_then(Value::as_object_mut)
                        {
                            if let Some(value) = attributes.remove("BATCHID") {
                                if attributes.contains_key("_BATCHID") {
                                    return Err(invalid("duplicate batch ID semantics"));
                                }
                                attributes.insert("_BATCHID".into(), value);
                                normalized = true;
                            }
                        }
                    }
                }
            }
        }
    }
    let normalized_json = if normalized {
        Some(serde_json::to_vec(&container.json)?)
    } else {
        None
    };
    let g = gltf::Gltf::from_slice(normalized_json.as_deref().unwrap_or(glb))?;
    if g.extensions_required().next().is_some() {
        return Err(DecodeError::Unsupported("required glTF extensions".into()));
    }
    if g.buffers()
        .any(|b| matches!(b.source(), gltf::buffer::Source::Uri(_)))
    {
        return Err(DecodeError::Unsupported("external/data URI buffers".into()));
    }
    validate_accessors(&g, container.binary, limits)?;
    let mut image_limits = limits.clone();
    image_limits.max_decoded_bytes = limits
        .max_decoded_bytes
        .checked_sub(metadata_bytes)
        .ok_or(DecodeError::Limit("decoded bytes"))?;
    let (images, image_bytes) = decode_images(&g, container.binary, &image_limits)?;
    if g.nodes().len() > limits.max_nodes {
        return Err(DecodeError::Limit("nodes"));
    }
    let scene = g
        .default_scene()
        .or_else(|| g.scenes().next())
        .ok_or_else(|| invalid("no glTF scene"))?;
    let mut stack: Vec<_> = scene
        .nodes()
        .map(|n| (n, DMat4::IDENTITY, 0usize, Vec::<usize>::new()))
        .collect();
    let mut primitives = Vec::new();
    let mut total_vertices = 0usize;
    let mut total_indices = 0usize;
    let mut decoded_bytes = 0usize;
    let mut visits = 0usize;
    while let Some((node, parent, depth, mut ancestors)) = stack.pop() {
        visits += 1;
        if visits > limits.max_nodes {
            return Err(DecodeError::Limit("node visits"));
        }
        if depth > limits.max_depth {
            return Err(DecodeError::Limit("node depth"));
        }
        if ancestors.contains(&node.index()) {
            return Err(invalid("glTF node cycle"));
        }
        ancestors.push(node.index());
        let m = node.transform().matrix();
        let local = DMat4::from_cols_array(
            &m.concat()
                .iter()
                .map(|x| *x as f64)
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| invalid("matrix"))?,
        );
        let transform = parent * local;
        if !transform.is_finite() || transform.determinant().abs() < 1e-20 {
            return Err(invalid("singular/non-finite node transform"));
        }
        for child in node.children() {
            stack.push((child, transform, depth + 1, ancestors.clone()));
        }
        if let Some(mesh) = node.mesh() {
            for primitive in mesh.primitives() {
                if primitive.mode() != gltf::mesh::Mode::Triangles {
                    return Err(DecodeError::Unsupported("non-triangle primitive".into()));
                }
                if primitive.morph_targets().next().is_some() {
                    return Err(DecodeError::Unsupported("morph targets".into()));
                }
                let accessor = primitive
                    .get(&gltf::Semantic::Positions)
                    .ok_or_else(|| invalid("missing POSITION"))?;
                total_vertices = total_vertices
                    .checked_add(accessor.count())
                    .ok_or(DecodeError::Overflow)?;
                let index_count = primitive.indices().map_or(accessor.count(), |a| a.count());
                total_indices = total_indices
                    .checked_add(index_count)
                    .ok_or(DecodeError::Overflow)?;
                if total_vertices > limits.max_vertices || total_indices > limits.max_indices {
                    return Err(DecodeError::Limit("geometry count"));
                }
                decoded_bytes = total_vertices
                    .checked_mul(std::mem::size_of::<Vertex>())
                    .and_then(|v| total_indices.checked_mul(4).and_then(|i| v.checked_add(i)))
                    .and_then(|n| n.checked_add(image_bytes))
                    .and_then(|n| n.checked_add(metadata_bytes))
                    .ok_or(DecodeError::Overflow)?;
                if decoded_bytes > limits.max_decoded_bytes {
                    return Err(DecodeError::Limit("decoded bytes"));
                }
                let reader = primitive.reader(|b| match b.source() {
                    gltf::buffer::Source::Bin => container.binary,
                    _ => None,
                });
                let positions: Vec<_> = reader
                    .read_positions()
                    .ok_or_else(|| invalid("POSITION buffer"))?
                    .collect();
                if positions.len() != accessor.count()
                    || positions.iter().flatten().any(|v| !v.is_finite())
                {
                    return Err(invalid("positions"));
                }
                let indices: Vec<u32> = match reader.read_indices() {
                    Some(i) => i.into_u32().collect(),
                    None => (0..positions.len() as u32).collect(),
                };
                if indices.len() != index_count
                    || !indices.len().is_multiple_of(3)
                    || indices.iter().any(|i| *i as usize >= positions.len())
                {
                    return Err(invalid("triangle indices"));
                }
                let mut normals: Vec<[f32; 3]> = reader
                    .read_normals()
                    .map(|n| n.collect())
                    .unwrap_or_default();
                if normals.is_empty() {
                    let mut generated = vec![Vec3::ZERO; positions.len()];
                    for t in indices.as_chunks::<3>().0 {
                        let [a, b, c] = [t[0] as usize, t[1] as usize, t[2] as usize];
                        let n = (Vec3::from(positions[b]) - Vec3::from(positions[a]))
                            .cross(Vec3::from(positions[c]) - Vec3::from(positions[a]));
                        for i in [a, b, c] {
                            generated[i] += n;
                        }
                    }
                    normals = generated
                        .into_iter()
                        .map(|n| n.try_normalize().unwrap_or(Vec3::Z).to_array())
                        .collect();
                }
                if normals.len() != positions.len()
                    || normals.iter().flatten().any(|v| !v.is_finite())
                {
                    return Err(invalid("normals"));
                }
                let uvs: Vec<_> = reader
                    .read_tex_coords(0)
                    .map(|v| v.into_f32().collect())
                    .unwrap_or_else(|| vec![[0.0; 2]; positions.len()]);
                if uvs.len() != positions.len() || uvs.iter().flatten().any(|v| !v.is_finite()) {
                    return Err(invalid("UV count"));
                }
                let ids = if let Some(a) = primitive.get(&gltf::Semantic::Extras("BATCHID".into()))
                {
                    read_batch_ids(a, container.binary, positions.len(), batch_length)?
                } else {
                    if batch_length.is_some_and(|n| n > 1) {
                        return Err(invalid("missing _BATCHID for batched content"));
                    }
                    vec![0; positions.len()]
                };
                let mat = primitive.material();
                let pbr = mat.pbr_metallic_roughness();
                let mut textures: [Option<Texture>; 5] = std::array::from_fn(|_| None);
                for (slot, info) in [pbr.base_color_texture(), pbr.metallic_roughness_texture()]
                    .into_iter()
                    .enumerate()
                {
                    if let Some(info) = info {
                        textures[slot] =
                            Some(resolve_texture(info.texture(), info.tex_coord(), &images)?);
                    }
                }
                if let Some(info) = mat.normal_texture() {
                    textures[2] = Some(resolve_texture(info.texture(), info.tex_coord(), &images)?);
                }
                if let Some(info) = mat.occlusion_texture() {
                    textures[3] = Some(resolve_texture(info.texture(), info.tex_coord(), &images)?);
                }
                if let Some(info) = mat.emissive_texture() {
                    textures[4] = Some(resolve_texture(info.texture(), info.tex_coord(), &images)?);
                }
                for t in textures.iter().flatten() {
                    if primitive
                        .get(&gltf::Semantic::TexCoords(t.tex_coord))
                        .is_none()
                    {
                        return Err(invalid("texture coordinate attribute missing"));
                    }
                }
                let uv1: Vec<_> = reader
                    .read_tex_coords(1)
                    .map(|v| v.into_f32().collect())
                    .unwrap_or_else(|| vec![[0.0; 2]; positions.len()]);
                let colors: Vec<_> = reader
                    .read_colors(0)
                    .map(|v| v.into_rgba_f32().collect())
                    .unwrap_or_else(|| vec![[1.0; 4]; positions.len()]);
                let tangents: Vec<_> = reader
                    .read_tangents()
                    .map(|v| v.collect())
                    .unwrap_or_else(|| vec![[0.0; 4]; positions.len()]);
                if uv1.len() != positions.len()
                    || colors.len() != positions.len()
                    || tangents.len() != positions.len()
                    || uv1
                        .iter()
                        .flatten()
                        .chain(colors.iter().flatten())
                        .chain(tangents.iter().flatten())
                        .any(|v| !v.is_finite())
                {
                    return Err(invalid("UV1/color/tangent attributes"));
                }
                let material = Material {
                    alpha_blend: mat.alpha_mode() == gltf::material::AlphaMode::Blend,
                    pattern: None,
                    base_color: pbr.base_color_factor(),
                    metallic: pbr.metallic_factor(),
                    roughness: pbr.roughness_factor(),
                    double_sided: mat.double_sided(),
                    face_forward: true,
                    two_pass: false,
                    lambert: false,
                    emissive: mat.emissive_factor(),
                    normal_scale: mat.normal_texture().map_or(1.0, |i| i.scale()),
                    occlusion_strength: mat.occlusion_texture().map_or(1.0, |i| i.strength()),
                    textures,
                    unlit: false,
                    alpha_mask: (mat.alpha_mode() == gltf::material::AlphaMode::Mask)
                        .then(|| mat.alpha_cutoff().unwrap_or(0.5)),
                };
                let vertices = positions
                    .into_iter()
                    .enumerate()
                    .map(|(i, position)| Vertex {
                        position,
                        normal: normals[i],
                        uv: uvs[i],
                        feature_id: ids[i],
                        uv1: uv1[i],
                        color: colors[i],
                        tangent: tangents[i],
                    })
                    .collect();
                primitives.push(Primitive {
                    vertices,
                    indices,
                    transform,
                    material,
                });
            }
        }
        if primitives.len() > limits.max_nodes {
            return Err(DecodeError::Limit("primitive count"));
        }
    }
    if primitives.is_empty() {
        return Err(invalid("no renderable triangle primitives"));
    }
    Ok(Content {
        tile_frame: data.starts_with(b"b3dm"),
        primitives,
        rtc_center,
        metadata,
        decoded_bytes,
    })
}
fn validate_accessors(
    g: &gltf::Gltf,
    bin: Option<&[u8]>,
    limits: &DecodeLimits,
) -> Result<(), DecodeError> {
    let bin = bin.unwrap_or(&[]);
    for buffer in g.buffers() {
        if buffer.length() > bin.len() || bin.len().saturating_sub(buffer.length()) > 3 {
            return Err(invalid("BIN buffer length/padding"));
        }
    }
    for view in g.views() {
        let end = view
            .offset()
            .checked_add(view.length())
            .ok_or(DecodeError::Overflow)?;
        if end > view.buffer().length() {
            return Err(invalid("bufferView exceeds declared buffer"));
        }
        checked_range(bin, view.offset(), view.length())?;
    }
    for accessor in g.accessors() {
        let count = accessor.count();
        let size = accessor.size();
        if count == 0 || count > limits.max_indices || count > u32::MAX as usize {
            return Err(DecodeError::Limit("accessor count"));
        }
        let check = |view: gltf::buffer::View<'_>,
                     offset: usize,
                     count: usize,
                     size: usize|
         -> Result<(), DecodeError> {
            let stride = view.stride().unwrap_or(size);
            if stride < size || !stride.is_multiple_of(accessor.data_type().size()) {
                return Err(invalid("accessor stride"));
            }
            let len = (count - 1)
                .checked_mul(stride)
                .and_then(|n| n.checked_add(size))
                .ok_or(DecodeError::Overflow)?;
            let bytes = checked_range(bin, view.offset(), view.length())?;
            checked_range(bytes, offset, len)?;
            Ok(())
        };
        if let Some(view) = accessor.view() {
            check(view, accessor.offset(), count, size)?;
        } else if accessor.sparse().is_none() {
            return Err(invalid("accessor has no storage"));
        }
        if let Some(sparse) = accessor.sparse() {
            let n = sparse.count();
            if n == 0 || n > count {
                return Err(invalid("sparse count"));
            }
            let indices = sparse.indices();
            let values = sparse.values();
            if indices.view().stride().is_some() || values.view().stride().is_some() {
                return Err(invalid("sparse views cannot have stride"));
            }
            let index_size = indices.index_type().size();
            let index_bytes = checked_range(bin, indices.view().offset(), indices.view().length())?;
            let index_bytes = checked_range(
                index_bytes,
                indices.offset(),
                n.checked_mul(index_size).ok_or(DecodeError::Overflow)?,
            )?;
            let value_bytes = checked_range(bin, values.view().offset(), values.view().length())?;
            checked_range(
                value_bytes,
                values.offset(),
                n.checked_mul(size).ok_or(DecodeError::Overflow)?,
            )?;
            let mut previous = None;
            for b in index_bytes.chunks_exact(index_size) {
                let id = match index_size {
                    1 => b[0] as u32,
                    2 => u16::from_le_bytes([b[0], b[1]]) as u32,
                    4 => u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
                    _ => return Err(invalid("sparse index type")),
                };
                if id as usize >= count || previous.is_some_and(|p| id <= p) {
                    return Err(invalid("sparse indices must increase and be in range"));
                }
                previous = Some(id);
            }
        }
    }
    Ok(())
}
fn read_batch_ids(
    a: gltf::Accessor<'_>,
    bin: Option<&[u8]>,
    count: usize,
    batch: Option<u32>,
) -> Result<Vec<u32>, DecodeError> {
    if a.count() != count
        || a.dimensions() != gltf::accessor::Dimensions::Scalar
        || a.normalized()
        || a.sparse().is_some()
    {
        return Err(DecodeError::Unsupported("batch accessor layout".into()));
    }
    let view = a.view().ok_or_else(|| invalid("batch buffer view"))?;
    let data = checked_range(
        bin.ok_or_else(|| invalid("batch BIN"))?,
        view.offset(),
        view.length(),
    )?;
    let size = a.data_type().size();
    let stride = view.stride().unwrap_or(size);
    if stride < size {
        return Err(invalid("batch stride"));
    }
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let off = i
            .checked_mul(stride)
            .and_then(|v| v.checked_add(a.offset()))
            .ok_or(DecodeError::Overflow)?;
        let b = checked_range(data, off, size)?;
        let id = match a.data_type() {
            gltf::accessor::DataType::U8 => b[0] as u32,
            gltf::accessor::DataType::U16 => u16::from_le_bytes([b[0], b[1]]) as u32,
            gltf::accessor::DataType::U32 => u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            gltf::accessor::DataType::F32 => {
                let v = f32::from_le_bytes([b[0], b[1], b[2], b[3]]);
                if !v.is_finite() || v < 0.0 || v.fract() != 0.0 || v as f64 > u32::MAX as f64 {
                    return Err(invalid("batch ID integer"));
                }
                v as u32
            }
            _ => return Err(invalid("batch ID type")),
        };
        if batch.is_some_and(|n| id >= n) {
            return Err(invalid("batch ID exceeds BATCH_LENGTH"));
        }
        out.push(id);
    }
    Ok(out)
}
/// Y-up glTF to Z-up tile frame; used exactly once at the load boundary.
pub fn gltf_y_up_to_z_up() -> DMat4 {
    DMat4::from_rotation_x(std::f64::consts::FRAC_PI_2)
}

/// Low-resolution WGS84 background. It is not a terrain provider.
pub fn globe_mesh(longitudes: u32, latitudes: u32) -> Primitive {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for y in 0..=latitudes {
        let lat = std::f64::consts::FRAC_PI_2 - y as f64 * std::f64::consts::PI / latitudes as f64;
        for x in 0..=longitudes {
            let lon = -std::f64::consts::PI + x as f64 * std::f64::consts::TAU / longitudes as f64;
            let c =
                cesium_math::Cartographic::radians(lon, lat, 0.0).expect("generated coordinates");
            let p = c.to_ecef();
            let n = DVec3::new(lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin());
            vertices.push(Vertex {
                position: p.as_vec3().to_array(),
                normal: n.as_vec3().to_array(),
                uv: [x as f32 / longitudes as f32, y as f32 / latitudes as f32],
                feature_id: 0,
                uv1: [0.0; 2],
                color: [1.0; 4],
                tangent: [0.0; 4],
            });
        }
    }
    for y in 0..latitudes {
        for x in 0..longitudes {
            let a = y * (longitudes + 1) + x;
            let b = a + longitudes + 1;
            indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    Primitive {
        vertices,
        indices,
        transform: DMat4::IDENTITY,
        material: Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.85,
            double_sided: true,
            alpha_mask: None,
            ..Default::default()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const GLB: &[u8] = include_bytes!("../../../tests/fixtures/triangle.glb");
    const B3DM: &[u8] = include_bytes!("../../../tests/fixtures/triangle.b3dm");
    #[test]
    fn cesium_binary_batch_table_reference_asset() {
        let bytes = include_bytes!("../../../tests/fixtures/cesium-batch-table-binary.b3dm");
        let content = decode_content(bytes, &DecodeLimits::default()).unwrap();
        let metadata = &content.metadata;
        for i in 0..10 {
            let values = metadata["cartographic"][i].as_array().unwrap();
            assert_eq!(values[0], metadata["Longitude"][i]);
            assert_eq!(values[1], metadata["Latitude"][i]);
            assert_eq!(values[2], metadata["Height"][i]);
            assert_eq!(metadata["code"][i], 255);
        }
        let ids: std::collections::BTreeSet<_> = content
            .primitives
            .iter()
            .flat_map(|p| p.vertices.iter().map(|v| v.feature_id))
            .collect();
        assert_eq!(ids, (0..10).collect());
    }

    #[test]
    fn metadata_and_geometry_share_decoded_budget() {
        let bytes = include_bytes!("../../../tests/fixtures/cesium-batch-table-binary.b3dm");
        let limits = DecodeLimits::default();
        let metadata_bytes = parse_b3dm(bytes, &limits).unwrap().batch_metadata_bytes;
        let complete = decode_content(bytes, &limits).unwrap();
        assert!(metadata_bytes > 0);
        assert!(complete.decoded_bytes > metadata_bytes + 1);
        let bounded = DecodeLimits {
            max_decoded_bytes: complete.decoded_bytes - 1,
            ..limits
        };
        // Each portion fits alone, while their combined retained size does not.
        assert!(metadata_bytes < bounded.max_decoded_bytes);
        assert!(complete.decoded_bytes - metadata_bytes < bounded.max_decoded_bytes);
        assert!(parse_b3dm(bytes, &bounded).is_ok());
        assert!(matches!(
            decode_content(bytes, &bounded),
            Err(DecodeError::Limit(_))
        ));
    }

    #[test]
    fn binary_batch_property_types_and_bounds() {
        let mut bytes = Vec::new();
        for v in [0.0_f64, 1.0] {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        for v in [2.0_f32, 3.0, 4.0, 5.0] {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        for v in [6_i32, 7, 8, 9, 10, 11] {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        for v in 12_u32..20 {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        for v in 20_i16..28 {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        for v in 28_u16..46 {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        bytes.extend(46_u8..78);
        let table = serde_json::json!({
            "scalar":{"byteOffset":0,"componentType":"DOUBLE","type":"SCALAR"},
            "vec2":{"byteOffset":16,"componentType":"FLOAT","type":"VEC2"},
            "vec3":{"byteOffset":32,"componentType":"INT","type":"VEC3"},
            "vec4":{"byteOffset":56,"componentType":"UNSIGNED_INT","type":"VEC4"},
            "mat2":{"byteOffset":88,"componentType":"SHORT","type":"MAT2"},
            "mat3":{"byteOffset":104,"componentType":"UNSIGNED_SHORT","type":"MAT3"},
            "mat4":{"byteOffset":140,"componentType":"UNSIGNED_BYTE","type":"MAT4"}
        });
        let decoded =
            decode_batch_table(table.clone(), &bytes, 2, &DecodeLimits::default()).unwrap();
        assert_eq!(decoded["scalar"][1], 1.0);
        assert_eq!(decoded["vec2"][1], serde_json::json!([4.0, 5.0]));
        assert_eq!(decoded["vec3"][1], serde_json::json!([9, 10, 11]));
        assert_eq!(decoded["vec4"][1], serde_json::json!([16, 17, 18, 19]));
        assert_eq!(decoded["mat2"][1], serde_json::json!([24, 25, 26, 27]));
        assert_eq!(
            decoded["mat3"][1],
            serde_json::json!([37, 38, 39, 40, 41, 42, 43, 44, 45])
        );
        assert_eq!(
            decoded["mat4"][1],
            serde_json::json!((62..78).collect::<Vec<_>>())
        );
        assert!(decode_batch_table(
            table.clone(),
            &bytes[..bytes.len() - 1],
            2,
            &DecodeLimits::default()
        )
        .is_err());
        let mut bad = table.clone();
        bad["vec3"]["byteOffset"] = 33.into();
        assert!(decode_batch_table(bad, &bytes, 2, &DecodeLimits::default()).is_err());
        let mut bad = table.clone();
        bad["vec3"].as_object_mut().unwrap().remove("componentType");
        assert!(decode_batch_table(bad, &bytes, 2, &DecodeLimits::default()).is_err());
        let limits = DecodeLimits {
            max_decoded_bytes: 1,
            ..Default::default()
        };
        assert!(decode_batch_table(table, &bytes, 2, &limits).is_err());
        let signed =
            serde_json::json!({"x":{"byteOffset":0,"componentType":"BYTE","type":"SCALAR"}});
        assert_eq!(
            decode_batch_table(signed, &[255, 127], 2, &DecodeLimits::default()).unwrap()["x"],
            serde_json::json!([-1, 127])
        );
        let floating =
            serde_json::json!({"x":{"byteOffset":0,"componentType":"FLOAT","type":"SCALAR"}});
        assert!(decode_batch_table(
            floating,
            &f32::NAN.to_le_bytes(),
            1,
            &DecodeLimits::default()
        )
        .is_err());
    }
    #[test]
    fn cesium_legacy_b3dm_headers_and_reference_content() {
        let cases: [(&[u8], B3dmHeader); 2] = [
            (
                include_bytes!("../../../tests/fixtures/cesium-batched-deprecated-20.b3dm"),
                B3dmHeader::Legacy20,
            ),
            (
                include_bytes!("../../../tests/fixtures/cesium-batched-deprecated-24.b3dm"),
                B3dmHeader::Legacy24,
            ),
        ];
        for (data, expected) in cases {
            let parsed = parse_b3dm(data, &DecodeLimits::default()).unwrap();
            assert_eq!(parsed.header, expected);
            assert_eq!(parsed.batch_length, 10);
            assert_eq!(parsed.feature_table["BATCH_LENGTH"], 10);
            assert_eq!(parsed.batch_table["id"].as_array().unwrap().len(), 10);
            assert!(parsed.glb.starts_with(b"glTF"));
            let content = decode_content(data, &DecodeLimits::default())
                .expect("decode legacy reference content");
            assert_eq!(content.metadata["id"].as_array().unwrap().len(), 10);
            let ids: std::collections::BTreeSet<_> = content
                .primitives
                .iter()
                .flat_map(|p| p.vertices.iter().map(|v| v.feature_id))
                .collect();
            assert_eq!(ids, (0..10).collect());
        }
    }

    #[test]
    fn cesium_b3dm_invalid_version_empty_payload_and_truncated_legacy() {
        let limits = DecodeLimits::default();
        let mut bad = B3DM.to_vec();
        bad[4..8].copy_from_slice(&2u32.to_le_bytes());
        assert!(parse_b3dm(&bad, &limits).is_err());
        let mut empty = b"b3dm".to_vec();
        for value in [1u32, 28, 0, 0, 0, 0] {
            empty.extend_from_slice(&value.to_le_bytes());
        }
        assert!(parse_b3dm(&empty, &limits).is_err());
        for data in [
            include_bytes!("../../../tests/fixtures/cesium-batched-deprecated-20.b3dm").as_slice(),
            include_bytes!("../../../tests/fixtures/cesium-batched-deprecated-24.b3dm").as_slice(),
        ] {
            for n in 0..data.len() {
                assert!(parse_b3dm(&data[..n], &limits).is_err());
            }
        }
    }
    #[test]
    fn blend_material_decode() {
        let c = decode_content(
            include_bytes!("../../../tests/fixtures/transparent.glb"),
            &Default::default(),
        )
        .unwrap();
        let m = &c.primitives[0].material;
        assert!(m.alpha_blend);
        assert_eq!(m.alpha_mask, None);
        assert_eq!(m.base_color[3], 0.5);
        assert!(m.textures[0].is_some());
    }
    #[test]
    fn containers_and_decode() {
        let c = decode_content(B3DM, &DecodeLimits::default()).unwrap();
        assert_eq!(c.primitives.len(), 1);
        assert_eq!(c.primitives[0].indices, vec![0, 1, 2]);
        assert_eq!(c.metadata["name"][0], "fixture triangle");
    }
    #[test]
    fn every_truncation_rejected_without_panic() {
        for asset in [GLB, B3DM] {
            for n in 0..asset.len() {
                assert!(decode_content(&asset[..n], &DecodeLimits::default()).is_err());
            }
        }
    }
    #[test]
    fn range_overflow_and_limits() {
        assert!(checked_range(&[], usize::MAX, 1).is_err());
        let limits = DecodeLimits {
            max_vertices: 2,
            ..Default::default()
        };
        assert!(decode_content(GLB, &limits).is_err());
        let mut bad = B3DM.to_vec();
        bad[12..16].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(parse_b3dm(&bad, &DecodeLimits::default()).is_err());
    }
    fn encode_glb(json: &Value, bin: &[u8]) -> Vec<u8> {
        let mut j = serde_json::to_vec(json).unwrap();
        while !j.len().is_multiple_of(4) {
            j.push(b' ');
        }
        let mut b = bin.to_vec();
        while !b.len().is_multiple_of(4) {
            b.push(0);
        }
        let mut out = b"glTF".to_vec();
        out.extend_from_slice(&2u32.to_le_bytes());
        out.extend_from_slice(&((28 + j.len() + b.len()) as u32).to_le_bytes());
        out.extend_from_slice(&(j.len() as u32).to_le_bytes());
        out.extend_from_slice(b"JSON");
        out.extend(j);
        out.extend_from_slice(&(b.len() as u32).to_le_bytes());
        out.extend_from_slice(b"BIN\0");
        out.extend(b);
        out
    }
    #[test]
    fn sparse_and_stride() {
        for asset in [
            include_bytes!("../../../tests/fixtures/sparse.glb").as_slice(),
            include_bytes!("../../../tests/fixtures/stride.glb").as_slice(),
        ] {
            let c = decode_content(asset, &Default::default()).unwrap();
            assert_eq!(c.primitives[0].vertices[2].position, [0.0, 40.0, 0.0]);
        }
        let asset = include_bytes!("../../../tests/fixtures/sparse.glb");
        let parsed = parse_glb(asset, &Default::default()).unwrap();
        let mut bin = parsed.binary.unwrap().to_vec();
        bin[1] = 0;
        assert!(decode_content(&encode_glb(&parsed.json, &bin), &Default::default()).is_err());
    }
    #[test]
    fn accessor_and_cycles_rejected() {
        let parsed = parse_glb(GLB, &Default::default()).unwrap();
        for field in ["count", "byteOffset"] {
            let mut j = parsed.json.clone();
            j["accessors"][0][field] = Value::from(u32::MAX);
            assert!(
                decode_content(&encode_glb(&j, parsed.binary.unwrap()), &Default::default())
                    .is_err()
            );
        }
        let mut j = parsed.json.clone();
        j["nodes"][0]["children"] = serde_json::json!([0]);
        assert!(
            decode_content(&encode_glb(&j, parsed.binary.unwrap()), &Default::default()).is_err()
        );
        j = parsed.json.clone();
        j["materials"][0]["emissiveFactor"] = serde_json::json!([1.0, 0.0, 0.0]);
        assert_eq!(
            decode_content(&encode_glb(&j, parsed.binary.unwrap()), &Default::default())
                .unwrap()
                .primitives[0]
                .material
                .emissive,
            [1.0, 0.0, 0.0]
        );
    }
    #[test]
    fn deterministic_corruption_smoke() {
        let mut seed = 0x12345678u64;
        for _ in 0..512 {
            let mut data = B3DM.to_vec();
            for _ in 0..3 {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let i = (seed as usize) % data.len();
                data[i] ^= (seed >> 32) as u8;
            }
            let _ = decode_content(
                &data,
                &DecodeLimits {
                    max_vertices: 1000,
                    max_indices: 3000,
                    ..Default::default()
                },
            );
        }
    }
    #[test]
    fn globe_seam_and_axes_match_equirectangular_uvs() {
        let mesh = globe_mesh(8, 4);
        let stride = 9usize;
        let equator = 2 * stride;
        let west = mesh.vertices[equator];
        let middle = mesh.vertices[equator + 4];
        let east = mesh.vertices[equator + 8];
        assert_eq!(west.uv, [0.0, 0.5]);
        assert_eq!(middle.uv, [0.5, 0.5]);
        assert_eq!(east.uv, [1.0, 0.5]);
        assert!(west.position[0] < 0.0);
        assert!(middle.position[0] > 0.0);
        assert!((west.position[0] - east.position[0]).abs() < 1.0);
    }
    #[test]
    fn embedded_and_data_uri_material_channels() {
        for data in [
            include_bytes!("../../../tests/fixtures/textured.glb").as_slice(),
            include_bytes!("../../../tests/fixtures/textured-data-uri.glb").as_slice(),
        ] {
            let content = decode_content(data, &Default::default()).unwrap();
            let p = &content.primitives[0];
            assert!(p.material.textures.iter().all(Option::is_some));
            assert_eq!(p.material.textures[0].as_ref().unwrap().tex_coord, 1);
            assert_eq!(
                p.material.textures[0].as_ref().unwrap().sampler.wrap_s,
                33648
            );
            assert_eq!(p.material.normal_scale, 0.6);
            assert_eq!(p.vertices[0].color, [1.0; 4]);
            assert_eq!(p.vertices[0].tangent, [1.0, 0.0, 0.0, 1.0]);
            let limit = DecodeLimits {
                max_decoded_bytes: 100,
                ..Default::default()
            };
            assert!(decode_content(data, &limit).is_err());
            let limit = DecodeLimits {
                max_image_dimension: 2,
                ..Default::default()
            };
            assert!(decode_content(data, &limit).is_err());
            let parsed = parse_glb(data, &Default::default()).unwrap();
            let mut json = parsed.json.clone();
            json["materials"][0]["pbrMetallicRoughness"]["baseColorTexture"]["texCoord"] =
                serde_json::json!(2);
            assert!(decode_content(
                &encode_glb(&json, parsed.binary.unwrap()),
                &Default::default()
            )
            .is_err());
        }
    }
    #[test]
    fn axis_rotation() {
        let q = gltf_y_up_to_z_up().transform_vector3(DVec3::Y);
        assert!(q.distance(DVec3::Z) < 1e-14);
    }
}
