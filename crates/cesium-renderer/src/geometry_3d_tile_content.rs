//! Geometry3DTileContent matching CesiumJS `Scene/Geometry3DTileContent.js`.

use std::collections::HashMap;

use cesium_math::{Cartesian3, Color, Matrix4, RuntimeError};
use serde_json::Value;

use crate::cesium_3d_tile_style::Cesium3DTileStyleDocument;
use crate::content_metadata::ContentMetadata;
use crate::model_3d_tile_content::Cesium3DTileStyle;
use crate::multiple_3d_tile_content::Cesium3DContentGroup;
use crate::scene_primitive::PrimitiveRenderFrameState;
use crate::vector_3d_tile_content::Vector3DTileFeature;
use crate::vector_3d_tile_geometry::{
    PACKED_BOX_LENGTH, PACKED_CYLINDER_LENGTH, PACKED_ELLIPSOID_LENGTH, PACKED_SPHERE_LENGTH,
    Vector3DTileGeometry, Vector3DTileGeometryOptions,
};

/// Options to synthesize a Geometry 3D Tile binary buffer (`.geom`).
#[derive(Debug, Clone, Default)]
pub struct GeometryTileBufferOptions {
    pub version: Option<u32>,
    pub empty: bool,
    pub feature_table_json_override: Option<Vec<u8>>,
    pub boxes: Option<Vec<f32>>,
    pub cylinders: Option<Vec<f32>>,
    pub ellipsoids: Option<Vec<f32>>,
    pub spheres: Option<Vec<f32>>,
    pub box_batch_ids: Option<Vec<u16>>,
    pub cylinder_batch_ids: Option<Vec<u16>>,
    pub ellipsoid_batch_ids: Option<Vec<u16>>,
    pub sphere_batch_ids: Option<Vec<u16>>,
    pub rtc_center: Option<[f64; 3]>,
    pub batch_table_json: Option<Value>,
}

/// Generates a binary buffer for a `.geom` tile (version 1).
pub fn generate_geometry_tile_buffer(options: &GeometryTileBufferOptions) -> Vec<u8> {
    let version = options.version.unwrap_or(1);
    if options.empty {
        let mut buf = Vec::with_capacity(12);
        buf.extend_from_slice(b"geom");
        buf.extend_from_slice(&version.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        return buf;
    }

    let mut ft_binary = Vec::<u8>::new();
    let mut ft_map = serde_json::Map::new();

    if let Some(rtc) = options.rtc_center {
        ft_map.insert("RTC_CENTER".into(), serde_json::json!(rtc));
    }

    let mut push_f32_section =
        |len_key: &str, data_key: &str, packed: &Option<Vec<f32>>, stride: usize| {
            if let Some(vals) = packed {
                let count = vals.len() / stride;
                ft_map.insert(len_key.into(), serde_json::json!(count));
                let byte_offset = ft_binary.len();
                for &v in vals {
                    ft_binary.extend_from_slice(&v.to_le_bytes());
                }
                ft_map.insert(
                    data_key.into(),
                    serde_json::json!({ "byteOffset": byte_offset }),
                );
            }
        };

    push_f32_section("BOXES_LENGTH", "BOXES", &options.boxes, PACKED_BOX_LENGTH);
    push_f32_section(
        "CYLINDERS_LENGTH",
        "CYLINDERS",
        &options.cylinders,
        PACKED_CYLINDER_LENGTH,
    );
    push_f32_section(
        "ELLIPSOIDS_LENGTH",
        "ELLIPSOIDS",
        &options.ellipsoids,
        PACKED_ELLIPSOID_LENGTH,
    );
    push_f32_section(
        "SPHERES_LENGTH",
        "SPHERES",
        &options.spheres,
        PACKED_SPHERE_LENGTH,
    );

    let mut push_batch_ids = |key: &str, ids: &Option<Vec<u16>>| {
        if let Some(vals) = ids {
            let byte_offset = ft_binary.len();
            for &id in vals {
                ft_binary.extend_from_slice(&id.to_le_bytes());
            }
            ft_map.insert(key.into(), serde_json::json!({ "byteOffset": byte_offset }));
        }
    };
    push_batch_ids("BOX_BATCH_IDS", &options.box_batch_ids);
    push_batch_ids("CYLINDER_BATCH_IDS", &options.cylinder_batch_ids);
    push_batch_ids("ELLIPSOID_BATCH_IDS", &options.ellipsoid_batch_ids);
    push_batch_ids("SPHERE_BATCH_IDS", &options.sphere_batch_ids);

    let mut ft_json_bytes = options
        .feature_table_json_override
        .clone()
        .unwrap_or_else(|| Value::Object(ft_map).to_string().into_bytes());
    while !ft_json_bytes.len().is_multiple_of(8) {
        ft_json_bytes.push(b' ');
    }
    while !ft_binary.len().is_multiple_of(8) {
        ft_binary.push(0);
    }

    let mut bt_json_bytes = options
        .batch_table_json
        .as_ref()
        .map(|v| v.to_string().into_bytes())
        .unwrap_or_default();
    if !bt_json_bytes.is_empty() {
        while !bt_json_bytes.len().is_multiple_of(8) {
            bt_json_bytes.push(b' ');
        }
    }

    let total_len = 28 + ft_json_bytes.len() + ft_binary.len() + bt_json_bytes.len();
    let mut buf = Vec::with_capacity(total_len);
    buf.extend_from_slice(b"geom");
    buf.extend_from_slice(&version.to_le_bytes());
    buf.extend_from_slice(&(total_len as u32).to_le_bytes());
    buf.extend_from_slice(&(ft_json_bytes.len() as u32).to_le_bytes());
    buf.extend_from_slice(&(ft_binary.len() as u32).to_le_bytes());
    buf.extend_from_slice(&(bt_json_bytes.len() as u32).to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&ft_json_bytes);
    buf.extend_from_slice(&ft_binary);
    buf.extend_from_slice(&bt_json_bytes);
    buf
}

/// Represents the contents of a Geometry (`geom`) tile in a 3D Tiles tileset.
#[derive(Debug)]
pub struct Geometry3DTileContent {
    pub tileset_id: Option<String>,
    pub tile_id: Option<String>,
    pub resource_url: String,
    pub feature_properties_dirty: bool,
    geometries: Option<Vector3DTileGeometry>,
    features: Vec<Vector3DTileFeature>,
    batch_table_json: Option<Value>,
    batch_table_byte_length: usize,
    metadata: Option<ContentMetadata>,
    group: Option<Cesium3DContentGroup>,
    ready: bool,
    is_destroyed: bool,
}

impl Geometry3DTileContent {
    /// Parses a `.geom` binary buffer starting at `byte_offset`.
    pub fn from_bytes(
        tileset_id: Option<String>,
        tile_id: Option<String>,
        resource_url: impl Into<String>,
        bytes: &[u8],
        byte_offset: usize,
        tile_transform: Matrix4,
    ) -> Result<Self, RuntimeError> {
        let resource_url = resource_url.into();
        if byte_offset.saturating_add(12) > bytes.len() {
            return Err(RuntimeError::new(
                "Geometry tile buffer is too small for header.",
            ));
        }
        let magic = &bytes[byte_offset..byte_offset + 4];
        if magic != b"geom" {
            return Err(RuntimeError::new(format!(
                "Invalid Geometry tile magic: {}",
                String::from_utf8_lossy(magic)
            )));
        }

        let version = read_u32_le(bytes, byte_offset + 4)?;
        if version != 1 {
            return Err(RuntimeError::new(format!(
                "Only Geometry tile version 1 is supported.  Version {version} is not."
            )));
        }

        let byte_length = read_u32_le(bytes, byte_offset + 8)? as usize;
        if byte_length == 0 {
            return Ok(Self {
                tileset_id,
                tile_id,
                resource_url,
                feature_properties_dirty: false,
                geometries: None,
                features: Vec::new(),
                batch_table_json: None,
                batch_table_byte_length: 0,
                metadata: None,
                group: None,
                ready: true,
                is_destroyed: false,
            });
        }

        if byte_offset.saturating_add(28) > bytes.len() {
            return Err(RuntimeError::new(
                "Geometry tile buffer is too small for 28-byte header.",
            ));
        }

        let ft_json_len = read_u32_le(bytes, byte_offset + 12)? as usize;
        if ft_json_len == 0 {
            return Err(RuntimeError::new(
                "Feature table must have a byte length greater than zero",
            ));
        }
        let ft_bin_len = read_u32_le(bytes, byte_offset + 16)? as usize;
        let bt_json_len = read_u32_le(bytes, byte_offset + 20)? as usize;
        let bt_bin_len = read_u32_le(bytes, byte_offset + 24)? as usize;

        let mut cursor = byte_offset + 28;
        let ft_json_slice = bytes
            .get(cursor..cursor + ft_json_len)
            .ok_or_else(|| RuntimeError::new("Feature table JSON extends past buffer"))?;
        let feature_table_json: Value = serde_json::from_slice(ft_json_slice)
            .map_err(|e| RuntimeError::new(format!("Invalid feature table JSON: {e}")))?;
        cursor += ft_json_len;

        let feature_table_binary = bytes
            .get(cursor..cursor + ft_bin_len)
            .ok_or_else(|| RuntimeError::new("Feature table binary extends past buffer"))?;
        cursor += ft_bin_len;

        let batch_table_json = if bt_json_len > 0 {
            let bt_slice = bytes
                .get(cursor..cursor + bt_json_len)
                .ok_or_else(|| RuntimeError::new("Batch table JSON extends past buffer"))?;
            cursor += bt_json_len;
            Some(
                serde_json::from_slice::<Value>(bt_slice)
                    .map_err(|e| RuntimeError::new(format!("Invalid batch table JSON: {e}")))?,
            )
        } else {
            None
        };

        let _batch_table_binary = if bt_bin_len > 0 {
            bytes
                .get(cursor..cursor + bt_bin_len)
                .ok_or_else(|| RuntimeError::new("Batch table binary extends past buffer"))?
        } else {
            &[]
        };

        let number_of_boxes = feature_table_json
            .get("BOXES_LENGTH")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;
        let number_of_cylinders = feature_table_json
            .get("CYLINDERS_LENGTH")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;
        let number_of_ellipsoids = feature_table_json
            .get("ELLIPSOIDS_LENGTH")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;
        let number_of_spheres = feature_table_json
            .get("SPHERES_LENGTH")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;

        let total_primitives =
            number_of_boxes + number_of_cylinders + number_of_ellipsoids + number_of_spheres;

        let mut features = Vec::with_capacity(total_primitives);
        for batch_id in 0..total_primitives {
            let mut f = Vector3DTileFeature::new(batch_id);
            if let Some(Value::Object(bt_obj)) = &batch_table_json {
                for (prop_name, prop_vals) in bt_obj {
                    if let Some(arr) = prop_vals.as_array() {
                        if let Some(val) = arr.get(batch_id) {
                            f.set_property(prop_name.clone(), val.clone());
                        }
                    }
                }
            }
            features.push(f);
        }

        if total_primitives == 0 {
            return Ok(Self {
                tileset_id,
                tile_id,
                resource_url,
                feature_properties_dirty: false,
                geometries: None,
                features,
                batch_table_json,
                batch_table_byte_length: bt_json_len + bt_bin_len,
                metadata: None,
                group: None,
                ready: true,
                is_destroyed: false,
            });
        }

        let center = feature_table_json
            .get("RTC_CENTER")
            .and_then(Value::as_array)
            .filter(|a| a.len() >= 3)
            .map(|a| {
                let local = Cartesian3::new(
                    a[0].as_f64().unwrap_or(0.0),
                    a[1].as_f64().unwrap_or(0.0),
                    a[2].as_f64().unwrap_or(0.0),
                );
                tile_transform.multiply_by_point(local)
            });

        let (box_batch_ids, cylinder_batch_ids, ellipsoid_batch_ids, sphere_batch_ids) =
            get_batch_ids(
                &feature_table_json,
                feature_table_binary,
                number_of_boxes,
                number_of_cylinders,
                number_of_ellipsoids,
                number_of_spheres,
            )?;

        let boxes = read_f32_section(
            &feature_table_json,
            feature_table_binary,
            "BOXES",
            number_of_boxes * PACKED_BOX_LENGTH,
        )?;
        let cylinders = read_f32_section(
            &feature_table_json,
            feature_table_binary,
            "CYLINDERS",
            number_of_cylinders * PACKED_CYLINDER_LENGTH,
        )?;
        let ellipsoids = read_f32_section(
            &feature_table_json,
            feature_table_binary,
            "ELLIPSOIDS",
            number_of_ellipsoids * PACKED_ELLIPSOID_LENGTH,
        )?;
        let spheres = read_f32_section(
            &feature_table_json,
            feature_table_binary,
            "SPHERES",
            number_of_spheres * PACKED_SPHERE_LENGTH,
        )?;

        let mut geometries = Vector3DTileGeometry::new(Vector3DTileGeometryOptions {
            boxes,
            box_batch_ids,
            cylinders,
            cylinder_batch_ids,
            ellipsoids,
            ellipsoid_batch_ids,
            spheres,
            sphere_batch_ids,
            center,
            model_matrix: Some(tile_transform),
            bounding_volume: None,
            classification_type: None,
            batch_colors: Some(HashMap::new()),
        });
        let mut frame_state = PrimitiveRenderFrameState::default();
        geometries.update(&mut frame_state);

        Ok(Self {
            tileset_id,
            tile_id,
            resource_url,
            feature_properties_dirty: false,
            geometries: Some(geometries),
            features,
            batch_table_json,
            batch_table_byte_length: bt_json_len + bt_bin_len,
            metadata: None,
            group: None,
            ready: true,
            is_destroyed: false,
        })
    }

    pub fn features_length(&self) -> usize {
        self.features.len()
    }

    pub fn points_length(&self) -> usize {
        0
    }

    pub fn triangles_length(&self) -> usize {
        self.geometries
            .as_ref()
            .map(|g| g.triangles_length)
            .unwrap_or(0)
    }

    pub fn geometry_byte_length(&self) -> usize {
        self.geometries
            .as_ref()
            .map(|g| g.geometry_byte_length)
            .unwrap_or(0)
    }

    pub fn textures_byte_length(&self) -> usize {
        0
    }

    pub fn batch_table_byte_length(&self) -> usize {
        self.batch_table_byte_length
    }

    pub fn inner_contents(&self) -> Option<&[()]> {
        None
    }

    pub fn ready(&self) -> bool {
        self.ready
    }

    pub fn url(&self) -> &str {
        &self.resource_url
    }

    pub fn metadata(&self) -> Option<&ContentMetadata> {
        self.metadata.as_ref()
    }

    pub fn set_metadata(&mut self, metadata: Option<ContentMetadata>) {
        self.metadata = metadata;
    }

    pub fn batch_table(&self) -> Option<&Value> {
        self.batch_table_json.as_ref()
    }

    pub fn group(&self) -> Option<&Cesium3DContentGroup> {
        self.group.as_ref()
    }

    pub fn set_group(&mut self, group: Option<Cesium3DContentGroup>) {
        self.group = group;
    }

    pub fn has_property(&self, batch_id: usize, name: &str) -> bool {
        self.features
            .get(batch_id)
            .and_then(|f| f.get_property(name))
            .is_some()
    }

    pub fn get_feature(&self, batch_id: usize) -> Option<&Vector3DTileFeature> {
        self.features.get(batch_id)
    }

    pub fn get_feature_mut(&mut self, batch_id: usize) -> Option<&mut Vector3DTileFeature> {
        self.features.get_mut(batch_id)
    }

    pub fn apply_debug_settings(&mut self, enabled: bool, _color: Option<Color>) {
        if let Some(g) = &mut self.geometries {
            g.debug_wireframe = enabled;
        }
    }

    pub fn apply_style(&mut self, style: Option<&Cesium3DTileStyleDocument>) {
        if let Some(g) = &mut self.geometries {
            let legacy_style = style.map(|_| Cesium3DTileStyle::default());
            g.apply_style(legacy_style.as_ref(), &mut self.features);
        }
    }

    pub fn update(&mut self, frame_state: &mut PrimitiveRenderFrameState) -> bool {
        if let Some(g) = &mut self.geometries {
            g.update(frame_state);
            self.ready = g.ready;
        }
        self.ready
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        self.geometries = None;
        self.features.clear();
        self.batch_table_json = None;
        self.is_destroyed = true;
    }
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Result<u32, RuntimeError> {
    let slice: [u8; 4] = bytes
        .get(offset..offset + 4)
        .and_then(|s| s.try_into().ok())
        .ok_or_else(|| RuntimeError::new("Unexpected end of Geometry tile buffer."))?;
    Ok(u32::from_le_bytes(slice))
}

fn read_u16_slice(bytes: &[u8], offset: usize, count: usize) -> Result<Vec<u16>, RuntimeError> {
    let byte_len = count * 2;
    let slice = bytes
        .get(offset..offset + byte_len)
        .ok_or_else(|| RuntimeError::new("Geometry batch IDs extend past feature table binary"))?;
    Ok(slice
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&c| u16::from_le_bytes(c))
        .collect())
}

fn read_f32_section(
    ft_json: &Value,
    ft_binary: &[u8],
    key: &str,
    count: usize,
) -> Result<Option<Vec<f32>>, RuntimeError> {
    if count == 0 {
        return Ok(None);
    }
    let byte_offset = ft_json
        .get(key)
        .and_then(|v| v.get("byteOffset"))
        .and_then(Value::as_u64)
        .ok_or_else(|| RuntimeError::new(format!("Missing {key}.byteOffset in feature table")))?
        as usize;
    let byte_len = count * 4;
    let slice = ft_binary
        .get(byte_offset..byte_offset + byte_len)
        .ok_or_else(|| RuntimeError::new(format!("{key} extends past feature table binary")))?;
    Ok(Some(
        slice
            .as_chunks::<4>()
            .0
            .iter()
            .map(|&c| f32::from_le_bytes(c))
            .collect(),
    ))
}

type GeometryBatchIdsTuple = (
    Option<Vec<u16>>,
    Option<Vec<u16>>,
    Option<Vec<u16>>,
    Option<Vec<u16>>,
);

fn get_batch_ids(
    ft_json: &Value,
    ft_binary: &[u8],
    n_boxes: usize,
    n_cylinders: usize,
    n_ellipsoids: usize,
    n_spheres: usize,
) -> Result<GeometryBatchIdsTuple, RuntimeError> {
    let read_opt = |len: usize, key: &str| -> Result<Option<Vec<u16>>, RuntimeError> {
        if len > 0 {
            if let Some(off) = ft_json
                .get(key)
                .and_then(|v| v.get("byteOffset"))
                .and_then(Value::as_u64)
            {
                return Ok(Some(read_u16_slice(ft_binary, off as usize, len)?));
            }
        }
        Ok(None)
    };

    let mut box_ids = read_opt(n_boxes, "BOX_BATCH_IDS")?;
    let mut cyl_ids = read_opt(n_cylinders, "CYLINDER_BATCH_IDS")?;
    let mut ell_ids = read_opt(n_ellipsoids, "ELLIPSOID_BATCH_IDS")?;
    let mut sph_ids = read_opt(n_spheres, "SPHERE_BATCH_IDS")?;

    let at_least_one_defined =
        box_ids.is_some() || cyl_ids.is_some() || ell_ids.is_some() || sph_ids.is_some();
    let at_least_one_undefined = (n_boxes > 0 && box_ids.is_none())
        || (n_cylinders > 0 && cyl_ids.is_none())
        || (n_ellipsoids > 0 && ell_ids.is_none())
        || (n_spheres > 0 && sph_ids.is_none());

    if at_least_one_defined && at_least_one_undefined {
        return Err(RuntimeError::new(
            "If one group of batch ids is defined, then all batch ids must be defined",
        ));
    }

    if !at_least_one_defined {
        let mut id = 0u16;
        if n_boxes > 0 {
            box_ids = Some((0..n_boxes).map(|_| { let v = id; id += 1; v }).collect());
        }
        if n_cylinders > 0 {
            cyl_ids = Some((0..n_cylinders).map(|_| { let v = id; id += 1; v }).collect());
        }
        if n_ellipsoids > 0 {
            ell_ids = Some((0..n_ellipsoids).map(|_| { let v = id; id += 1; v }).collect());
        }
        if n_spheres > 0 {
            sph_ids = Some((0..n_spheres).map(|_| { let v = id; id += 1; v }).collect());
        }
    }

    Ok((box_ids, cyl_ids, ell_ids, sph_ids))
}
