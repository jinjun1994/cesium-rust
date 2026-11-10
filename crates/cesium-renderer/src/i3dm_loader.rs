//! I3dmLoader matching CesiumJS `Scene/Model/I3dmLoader.js` (Apache-2.0).

use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use cesium_math::{
    AttributeCompression, AttributeType, Cartesian3, ComponentDatatype, DeveloperError,
    Ellipsoid, FixedFrameTransforms, Matrix3, Matrix4, Quaternion, RuntimeError, Sphere,
};
use glam::DVec3;
use serde_json::Value;

use crate::axis::Axis;
use crate::buffer::{Buffer, BufferOptions, BufferUsage};
use crate::buffer_loader::ResourceLoaderState;
use crate::cesium_3d_tile_feature_table::Cesium3DTileFeatureTable;
use crate::gltf_loader::{GltfLoader, GltfLoaderOptions};
use crate::i3dm_parser::{I3dmParser, I3dmParserError};
use crate::instance_attribute_semantic::InstanceAttributeSemantic;
use crate::model_components::{Attribute, Components, FeatureId, FeatureIdAttribute, Instances};

/// Options for creating an [`I3dmLoader`].
#[derive(Debug, Clone, Default)]
pub struct I3dmLoaderOptions {
    pub i3dm_resource: Option<String>,
    pub base_resource: Option<String>,
    pub array_buffer: Option<Vec<u8>>,
    pub byte_offset: Option<usize>,
    pub release_gltf_json: Option<bool>,
    pub asynchronous: Option<bool>,
    pub incrementally_load_textures: Option<bool>,
    pub up_axis: Option<Axis>,
    pub forward_axis: Option<Axis>,
    pub load_attributes_as_typed_array: Option<bool>,
    pub enable_pick: Option<bool>,
    pub load_indices_for_wireframe: Option<bool>,
    pub load_primitive_outline: Option<bool>,
}

#[derive(Debug)]
#[allow(dead_code)]
struct I3dmLoaderInner {
    i3dm_resource: String,
    base_resource: String,
    array_buffer: Option<Vec<u8>>,
    byte_offset: usize,
    release_gltf_json: bool,
    asynchronous: bool,
    incrementally_load_textures: bool,
    up_axis: Axis,
    forward_axis: Axis,
    load_attributes_as_typed_array: bool,
    enable_pick: bool,
    load_indices_for_wireframe: bool,
    load_primitive_outline: bool,

    state: ResourceLoaderState,
    is_destroyed: bool,

    gltf_loader: Option<GltfLoader>,
    transform: Matrix4,
    feature_table_json: Value,
    feature_table_binary: Vec<u8>,
    batch_table_json: Option<Value>,
    batch_table_binary: Option<Vec<u8>>,
    instances_length: usize,
    rtc_center: Option<Vec<f64>>,

    buffers: Vec<Buffer>,
    components: Option<Components>,
}

unsafe impl Send for I3dmLoaderInner {}

/// Loads an Instanced 3D Model (`.i3dm`).
#[derive(Debug, Clone)]
pub struct I3dmLoader {
    inner: Arc<Mutex<I3dmLoaderInner>>,
}

impl I3dmLoader {
    /// Creates a new `I3dmLoader`.
    pub fn new(options: I3dmLoaderOptions) -> Result<Self, DeveloperError> {
        let i3dm_resource = options
            .i3dm_resource
            .ok_or_else(|| DeveloperError::new("options.i3dmResource is required."))?;
        let array_buffer = options
            .array_buffer
            .ok_or_else(|| DeveloperError::new("options.arrayBuffer is required."))?;
        let base_resource = options
            .base_resource
            .unwrap_or_else(|| i3dm_resource.clone());

        let byte_offset = options.byte_offset.unwrap_or(0);
        let release_gltf_json = options.release_gltf_json.unwrap_or(false);
        let asynchronous = options.asynchronous.unwrap_or(true);
        let incrementally_load_textures = options.incrementally_load_textures.unwrap_or(true);
        let up_axis = options.up_axis.unwrap_or(Axis::Y);
        let forward_axis = options.forward_axis.unwrap_or(Axis::X);
        let load_attributes_as_typed_array =
            options.load_attributes_as_typed_array.unwrap_or(false);
        let enable_pick = options.enable_pick.unwrap_or(false);
        let load_indices_for_wireframe = options.load_indices_for_wireframe.unwrap_or(false);
        let load_primitive_outline = options.load_primitive_outline.unwrap_or(true);

        Ok(Self {
            inner: Arc::new(Mutex::new(I3dmLoaderInner {
                i3dm_resource,
                base_resource,
                array_buffer: Some(array_buffer),
                byte_offset,
                release_gltf_json,
                asynchronous,
                incrementally_load_textures,
                up_axis,
                forward_axis,
                load_attributes_as_typed_array,
                enable_pick,
                load_indices_for_wireframe,
                load_primitive_outline,
                state: ResourceLoaderState::Unloaded,
                is_destroyed: false,
                gltf_loader: None,
                transform: Matrix4::IDENTITY,
                feature_table_json: Value::Null,
                feature_table_binary: Vec::new(),
                batch_table_json: None,
                batch_table_binary: None,
                instances_length: 0,
                rtc_center: None,
                buffers: Vec::new(),
                components: None,
            })),
        })
    }

    /// The current loader state.
    pub fn state(&self) -> ResourceLoaderState {
        self.inner.lock().unwrap().state
    }

    /// Whether this loader has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.inner.lock().unwrap().is_destroyed
    }

    /// Returns the loaded model components once ready.
    pub fn components(&self) -> Option<Components> {
        self.inner.lock().unwrap().components.clone()
    }

    /// Returns the GPU buffers created for instancing.
    pub fn buffers(&self) -> Vec<Buffer> {
        self.inner.lock().unwrap().buffers.clone()
    }

    /// Whether the underlying array buffer has been released.
    pub fn has_array_buffer(&self) -> bool {
        self.inner.lock().unwrap().array_buffer.is_some()
    }

    /// Destroys this loader and releases resources.
    pub fn destroy(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        if inner.is_destroyed {
            return;
        }

        if let Some(ref mut loader) = inner.gltf_loader {
            loader.destroy();
        }

        for buffer in &mut inner.buffers {
            let _ = buffer.destroy();
        }
        inner.buffers.clear();

        inner.components = None;
        inner.array_buffer = None;
        inner.is_destroyed = true;
        inner.state = ResourceLoaderState::Unloaded;
    }

    /// Loads the i3dm contents and parses feature tables.
    pub async fn load(&mut self) -> Result<(), RuntimeError> {
        let (buf, byte_offset, base_res, i3dm_res, up_axis, forward_axis) = {
            let inner = self.inner.lock().unwrap();
            if inner.is_destroyed {
                return Ok(());
            }
            let buf = inner
                .array_buffer
                .as_ref()
                .ok_or_else(|| RuntimeError::new("No arrayBuffer available to load."))?
                .clone();
            (
                buf,
                inner.byte_offset,
                inner.base_resource.clone(),
                inner.i3dm_resource.clone(),
                inner.up_axis,
                inner.forward_axis,
            )
        };

        let parsed = match I3dmParser::parse(Some(&buf), Some(byte_offset)) {
            Ok(res) => res,
            Err(I3dmParserError::RuntimeError(e)) => {
                let mut inner = self.inner.lock().unwrap();
                inner.state = ResourceLoaderState::Failed;
                return Err(e);
            }
            Err(I3dmParserError::DeveloperError(e)) => {
                let mut inner = self.inner.lock().unwrap();
                inner.state = ResourceLoaderState::Failed;
                return Err(RuntimeError::new(e.to_string()));
            }
        };

        let feature_table = Cesium3DTileFeatureTable::new(
            parsed.feature_table_json.clone(),
            Some(parsed.feature_table_binary.clone()),
        );

        let instances_length = feature_table
            .get_global_property("INSTANCES_LENGTH", None, None)
            .and_then(|v| v.first().copied())
            .map(|f| f as usize)
            .or_else(|| {
                parsed
                    .feature_table_json
                    .get("INSTANCES_LENGTH")
                    .and_then(Value::as_u64)
                    .map(|v| v as usize)
            });

        let Some(instances_len) = instances_length else {
            let mut inner = self.inner.lock().unwrap();
            inner.state = ResourceLoaderState::Failed;
            return Err(RuntimeError::new(
                "Feature table global property: INSTANCES_LENGTH must be defined",
            ));
        };

        let rtc_center = feature_table
            .get_global_property("RTC_CENTER", Some(ComponentDatatype::Float), Some(3));

        let transform = if let Some(ref rtc) = rtc_center {
            Matrix4::from_translation(Cartesian3::new(rtc[0], rtc[1], rtc[2]))
        } else {
            Matrix4::IDENTITY
        };

        let glb_bytes = if parsed.gltf_format == 0 {
            let uri_str = String::from_utf8_lossy(&parsed.gltf)
                .trim_matches(['\0', ' ', '\r', '\n'])
                .to_string();

            let base_path = Path::new(&base_res);
            let dir = base_path.parent().unwrap_or_else(|| Path::new("."));
            let resolved_path = dir.join(&uri_str);

            fs::read(&resolved_path).map_err(|e| {
                RuntimeError::new(format!(
                    "Failed to read external glTF from {}: {e}",
                    resolved_path.display()
                ))
            })?
        } else {
            parsed.gltf
        };

        let gltf_options = GltfLoaderOptions {
            gltf_resource: Some(i3dm_res),
            base_resource: Some(base_res),
            typed_array: Some(glb_bytes),
            up_axis: Some(up_axis),
            forward_axis: Some(forward_axis),
            ..Default::default()
        };

        let gltf_loader = GltfLoader::new(gltf_options)
            .map_err(|e| RuntimeError::new(format!("GltfLoader creation failed: {e}")))?;

        if let Err(e) = gltf_loader
            .load::<fn(String) -> std::future::Ready<Result<Vec<u8>, String>>, _>(None)
            .await
        {
            let mut inner = self.inner.lock().unwrap();
            inner.state = ResourceLoaderState::Failed;
            return Err(e);
        }

        let mut inner = self.inner.lock().unwrap();
        if inner.is_destroyed {
            return Ok(());
        }

        inner.gltf_loader = Some(gltf_loader);
        inner.transform = transform;
        inner.feature_table_json = parsed.feature_table_json;
        inner.feature_table_binary = parsed.feature_table_binary;
        inner.batch_table_json = parsed.batch_table_json;
        inner.batch_table_binary = parsed.batch_table_binary;
        inner.instances_length = instances_len;
        inner.rtc_center = rtc_center;
        inner.state = ResourceLoaderState::Processing;

        Ok(())
    }

    /// Processes GPU resources and instancing attributes.
    pub fn process(&mut self) -> Result<bool, RuntimeError> {
        let inner_state = self.inner.lock().unwrap().state;
        if inner_state == ResourceLoaderState::Ready {
            return Ok(true);
        }
        if inner_state != ResourceLoaderState::Processing {
            return Ok(false);
        }

        let gltf_loader = self.inner.lock().unwrap().gltf_loader.clone().unwrap();
        let ready = gltf_loader.process()?;
        if !ready {
            return Ok(false);
        }

        let mut components = gltf_loader
            .components()
            .unwrap_or_else(|| Components {
                transform: Matrix4::IDENTITY,
                ..Default::default()
            });

        let mut inner = self.inner.lock().unwrap();
        if inner.is_destroyed {
            return Ok(true);
        }

        // Multiply RTC center transform with components transform
        components.transform = inner.transform.multiply(&components.transform);

        let instances_length = inner.instances_length;
        let mut feature_table = Cesium3DTileFeatureTable::new(
            inner.feature_table_json.clone(),
            Some(inner.feature_table_binary.clone()),
        );
        feature_table.features_length = instances_length;

        // Process instances
        let (instances, created_buffers, final_transform) = create_instances_data(
            &feature_table,
            instances_length,
            inner.rtc_center.as_deref(),
            &components.transform,
        )?;

        components.transform = final_transform;
        inner.buffers = created_buffers;

        // Apply instances to all nodes that have primitives
        for node in &mut components.nodes {
            if !node.primitives.is_empty() {
                node.instances = Some(instances.clone());
            }
        }

        // Structural metadata
        if let Some(ref bt_json) = inner.batch_table_json {
            components.structural_metadata = Some(bt_json.clone());
        } else {
            components.structural_metadata = Some(serde_json::json!({
                "schema": {},
                "propertyTables": [{
                    "name": "BatchTable",
                    "count": instances_length,
                }]
            }));
        }

        inner.components = Some(components);
        inner.array_buffer = None;
        inner.state = ResourceLoaderState::Ready;

        Ok(true)
    }
}

fn create_instances_data(
    feature_table: &Cesium3DTileFeatureTable,
    instances_length: usize,
    rtc_center: Option<&[f64]>,
    current_transform: &Matrix4,
) -> Result<(Instances, Vec<Buffer>, Matrix4), RuntimeError> {
    if instances_length == 0 {
        return Ok((Instances::default(), Vec::new(), *current_transform));
    }

    let east_north_up = feature_table
        .get_global_property("EAST_NORTH_UP", None, None)
        .is_some()
        || feature_table
            .has_property("EAST_NORTH_UP");

    let has_rotation = feature_table.has_property("NORMAL_UP")
        || feature_table.has_property("NORMAL_UP_OCT32P")
        || east_north_up;

    let has_scale = feature_table.has_property("SCALE")
        || feature_table.has_property("SCALE_NON_UNIFORM");

    let mut translation_typed_array = get_positions(feature_table, instances_length)?;

    let mut rotation_typed_array = if has_rotation {
        Some(vec![0.0f32; 4 * instances_length])
    } else {
        None
    };

    let mut scale_typed_array = if has_scale {
        Some(vec![0.0f32; 3 * instances_length])
    } else {
        None
    };

    let mut feature_id_array = vec![0.0f32; instances_length];

    let rtc_is_zero = rtc_center.is_none_or(|c| c[0] == 0.0 && c[1] == 0.0 && c[2] == 0.0);

    let mut final_transform = *current_transform;

    // Handle bounding sphere translation if RTC center is missing or zero
    if rtc_center.is_none() || rtc_is_zero {
        let points_dvec: Vec<DVec3> = (0..instances_length)
            .map(|i| {
                DVec3::new(
                    translation_typed_array[3 * i] as f64,
                    translation_typed_array[3 * i + 1] as f64,
                    translation_typed_array[3 * i + 2] as f64,
                )
            })
            .collect();

        let sphere =
            Sphere::from_points(&points_dvec).map_err(|e| RuntimeError::new(e.to_string()))?;
        let center = sphere.center;

        for i in 0..instances_length {
            translation_typed_array[3 * i] -= center.x as f32;
            translation_typed_array[3 * i + 1] -= center.y as f32;
            translation_typed_array[3 * i + 2] -= center.z as f32;
        }

        let center_transform =
            Matrix4::from_translation(Cartesian3::new(center.x, center.y, center.z));
        final_transform = center_transform.multiply(&final_transform);
    }

    for i in 0..instances_length {
        let mut pos = Cartesian3::new(
            translation_typed_array[3 * i] as f64,
            translation_typed_array[3 * i + 1] as f64,
            translation_typed_array[3 * i + 2] as f64,
        );

        if let Some(rtc) = rtc_center {
            pos.x += rtc[0];
            pos.y += rtc[1];
            pos.z += rtc[2];
        }

        if has_rotation {
            let quat = process_rotation(feature_table, east_north_up, i, pos)?;
            if let Some(ref mut rot) = rotation_typed_array {
                rot[4 * i] = quat[0];
                rot[4 * i + 1] = quat[1];
                rot[4 * i + 2] = quat[2];
                rot[4 * i + 3] = quat[3];
            }
        }

        if has_scale {
            let sc = process_scale(feature_table, i);
            if let Some(ref mut scale_arr) = scale_typed_array {
                scale_arr[3 * i] = sc[0];
                scale_arr[3 * i + 1] = sc[1];
                scale_arr[3 * i + 2] = sc[2];
            }
        }

        let mut batch_buf = [0.0; 1];
        let batch_id = if feature_table
            .get_property(
                "BATCH_ID",
                Some(ComponentDatatype::UnsignedShort),
                1,
                i,
                &mut batch_buf,
            )
            .is_some()
        {
            batch_buf[0] as f32
        } else {
            i as f32
        };
        feature_id_array[i] = batch_id;
    }

    let mut instances = Instances {
        transform_in_world_space: true,
        attributes: Vec::new(),
        feature_ids: Vec::new(),
    };
    let mut buffers = Vec::new();

    // 1. Translation Attribute
    let translation_bytes: Vec<u8> = translation_typed_array
        .iter()
        .flat_map(|f| f.to_le_bytes())
        .collect();

    let translation_buffer = if !has_rotation {
        let buf = Buffer::create_vertex_buffer(BufferOptions {
            size_in_bytes: 0,
            usage: Some(BufferUsage::StaticDraw),
            typed_array: Some(translation_bytes.clone()),
        })
        .map_err(|e| RuntimeError::new(e.to_string()))?;
        buffers.push(buf.clone());
        Some(buf)
    } else {
        None
    };

    let translation_attr = Attribute {
        name: Some("Instance Translation".to_string()),
        semantic: Some(InstanceAttributeSemantic::Translation.as_str().to_string()),
        component_datatype: Some(ComponentDatatype::Float),
        attribute_type: Some(AttributeType::Vec3),
        count: instances_length,
        typed_array: Some(translation_bytes),
        buffer: translation_buffer,
        ..Default::default()
    };
    instances.attributes.push(translation_attr);

    // 2. Rotation Attribute
    if let Some(rot_arr) = rotation_typed_array {
        let rotation_bytes: Vec<u8> = rot_arr.iter().flat_map(|f| f.to_le_bytes()).collect();
        let rotation_attr = Attribute {
            name: Some("Instance Rotation".to_string()),
            semantic: Some(InstanceAttributeSemantic::Rotation.as_str().to_string()),
            component_datatype: Some(ComponentDatatype::Float),
            attribute_type: Some(AttributeType::Vec4),
            count: instances_length,
            typed_array: Some(rotation_bytes),
            buffer: None,
            ..Default::default()
        };
        instances.attributes.push(rotation_attr);
    }

    // 3. Scale Attribute
    if let Some(scale_arr) = scale_typed_array {
        let scale_bytes: Vec<u8> = scale_arr.iter().flat_map(|f| f.to_le_bytes()).collect();
        let scale_buffer = if !has_rotation {
            let buf = Buffer::create_vertex_buffer(BufferOptions {
                size_in_bytes: 0,
                usage: Some(BufferUsage::StaticDraw),
                typed_array: Some(scale_bytes.clone()),
            })
            .map_err(|e| RuntimeError::new(e.to_string()))?;
            buffers.push(buf.clone());
            Some(buf)
        } else {
            None
        };

        let scale_attr = Attribute {
            name: Some("Instance Scale".to_string()),
            semantic: Some(InstanceAttributeSemantic::Scale.as_str().to_string()),
            component_datatype: Some(ComponentDatatype::Float),
            attribute_type: Some(AttributeType::Vec3),
            count: instances_length,
            typed_array: if has_rotation {
                Some(scale_bytes)
            } else {
                None
            },
            buffer: scale_buffer,
            ..Default::default()
        };
        instances.attributes.push(scale_attr);
    }

    // 4. Feature ID Attribute
    let feature_id_bytes: Vec<u8> = feature_id_array
        .iter()
        .flat_map(|f| f.to_le_bytes())
        .collect();
    let feature_id_buffer = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 0,
        usage: Some(BufferUsage::StaticDraw),
        typed_array: Some(feature_id_bytes),
    })
    .map_err(|e| RuntimeError::new(e.to_string()))?;
    buffers.push(feature_id_buffer.clone());

    let feature_id_attr = Attribute {
        name: Some("Instance Feature ID".to_string()),
        semantic: Some(InstanceAttributeSemantic::FeatureId.as_str().to_string()),
        set_index: Some(0),
        component_datatype: Some(ComponentDatatype::Float),
        attribute_type: Some(AttributeType::Scalar),
        count: instances_length,
        typed_array: None,
        buffer: Some(feature_id_buffer),
        ..Default::default()
    };
    instances.attributes.push(feature_id_attr);

    // Feature ID definition
    let feature_id_def = FeatureId::Attribute(FeatureIdAttribute {
        feature_count: instances_length,
        null_feature_id: None,
        property_table_id: Some(0),
        set_index: Some(0),
        label: None,
        positional_label: Some("instanceFeatureId_0".to_string()),
    });
    instances.feature_ids.push(feature_id_def);

    Ok((instances, buffers, final_transform))
}

fn get_positions(
    feature_table: &Cesium3DTileFeatureTable,
    _instances_length: usize,
) -> Result<Vec<f32>, RuntimeError> {
    if feature_table.has_property("POSITION") {
        let arr = feature_table
            .get_property_array("POSITION", Some(ComponentDatatype::Float), 3)
            .ok_or_else(|| RuntimeError::new("Failed to read POSITION property array"))?;
        Ok(arr.iter().map(|f| *f as f32).collect())
    } else if feature_table.has_property("POSITION_QUANTIZED") {
        let quantized = feature_table
            .get_property_array(
                "POSITION_QUANTIZED",
                Some(ComponentDatatype::UnsignedShort),
                3,
            )
            .ok_or_else(|| {
                RuntimeError::new("Failed to read POSITION_QUANTIZED property array")
            })?;

        let offset = feature_table
            .get_global_property("QUANTIZED_VOLUME_OFFSET", Some(ComponentDatatype::Float), Some(3))
            .ok_or_else(|| {
                RuntimeError::new(
                    "Global property: QUANTIZED_VOLUME_OFFSET must be defined for quantized positions.",
                )
            })?;

        let scale = feature_table
            .get_global_property("QUANTIZED_VOLUME_SCALE", Some(ComponentDatatype::Float), Some(3))
            .ok_or_else(|| {
                RuntimeError::new(
                    "Global property: QUANTIZED_VOLUME_SCALE must be defined for quantized positions.",
                )
            })?;

        let mut decoded = vec![0.0f32; quantized.len()];
        for i in 0..(quantized.len() / 3) {
            for j in 0..3 {
                let idx = 3 * i + j;
                decoded[idx] = ((quantized[idx] / 65535.0) * scale[j] + offset[j]) as f32;
            }
        }
        Ok(decoded)
    } else {
        Err(RuntimeError::new(
            "Either POSITION or POSITION_QUANTIZED must be defined for each instance.",
        ))
    }
}

fn process_rotation(
    feature_table: &Cesium3DTileFeatureTable,
    east_north_up: bool,
    i: usize,
    instance_position: Cartesian3,
) -> Result<[f32; 4], RuntimeError> {
    let mut scratch1 = [0.0; 3];
    let mut scratch2 = [0.0; 3];

    let has_norm_up = feature_table
        .get_property(
            "NORMAL_UP",
            Some(ComponentDatatype::Float),
            3,
            i,
            &mut scratch1,
        )
        .is_some();

    if has_norm_up {
        let has_norm_right = feature_table
            .get_property(
                "NORMAL_RIGHT",
                Some(ComponentDatatype::Float),
                3,
                i,
                &mut scratch2,
            )
            .is_some();
        if !has_norm_right {
            return Err(RuntimeError::new(
                "To define a custom orientation, both NORMAL_UP and NORMAL_RIGHT must be defined.",
            ));
        }

        let norm_up = Cartesian3::new(scratch1[0], scratch1[1], scratch1[2]);
        let norm_right = Cartesian3::new(scratch2[0], scratch2[1], scratch2[2]);
        let norm_fwd = norm_right.cross(&norm_up).normalize();

        let rot_mat = Matrix3::from_column_major_array([
            norm_right.x,
            norm_right.y,
            norm_right.z,
            norm_up.x,
            norm_up.y,
            norm_up.z,
            norm_fwd.x,
            norm_fwd.y,
            norm_fwd.z,
        ]);
        let q = Quaternion::from_rotation_matrix(&rot_mat);
        return Ok([q.x as f32, q.y as f32, q.z as f32, q.w as f32]);
    }

    let mut oct_scratch1 = [0.0; 2];
    let mut oct_scratch2 = [0.0; 2];
    let has_oct_up = feature_table
        .get_property(
            "NORMAL_UP_OCT32P",
            Some(ComponentDatatype::UnsignedShort),
            2,
            i,
            &mut oct_scratch1,
        )
        .is_some();

    if has_oct_up {
        let has_oct_right = feature_table
            .get_property(
                "NORMAL_RIGHT_OCT32P",
                Some(ComponentDatatype::UnsignedShort),
                2,
                i,
                &mut oct_scratch2,
            )
            .is_some();
        if !has_oct_right {
            return Err(RuntimeError::new(
                "To define a custom orientation with oct-encoded vectors, both NORMAL_UP_OCT32P and NORMAL_RIGHT_OCT32P must be defined.",
            ));
        }

        let norm_up =
            AttributeCompression::oct_decode_in_range(oct_scratch1[0], oct_scratch1[1], 65535.0)
                .map_err(|e| RuntimeError::new(e.to_string()))?;
        let norm_right = AttributeCompression::oct_decode_in_range(
            oct_scratch2[0],
            oct_scratch2[1],
            65535.0,
        )
        .map_err(|e| RuntimeError::new(e.to_string()))?;
        let norm_fwd = norm_right.cross(&norm_up).normalize();

        let rot_mat = Matrix3::from_column_major_array([
            norm_right.x,
            norm_right.y,
            norm_right.z,
            norm_up.x,
            norm_up.y,
            norm_up.z,
            norm_fwd.x,
            norm_fwd.y,
            norm_fwd.z,
        ]);
        let q = Quaternion::from_rotation_matrix(&rot_mat);
        return Ok([q.x as f32, q.y as f32, q.z as f32, q.w as f32]);
    }

    if east_north_up {
        let enu_mat = FixedFrameTransforms::east_north_up_to_fixed_frame(
            instance_position,
            Some(Ellipsoid::WGS84),
        )
        .map_err(|e| RuntimeError::new(e.to_string()))?;
        let rot_mat = enu_mat.get_matrix3();
        let q = Quaternion::from_rotation_matrix(&rot_mat);
        return Ok([q.x as f32, q.y as f32, q.z as f32, q.w as f32]);
    }

    let q = Quaternion::from_rotation_matrix(&Matrix3::IDENTITY);
    Ok([q.x as f32, q.y as f32, q.z as f32, q.w as f32])
}

fn process_scale(feature_table: &Cesium3DTileFeatureTable, i: usize) -> [f32; 3] {
    let mut scale = [1.0f32, 1.0f32, 1.0f32];

    let mut sc = [0.0; 1];
    if feature_table
        .get_property("SCALE", Some(ComponentDatatype::Float), 1, i, &mut sc)
        .is_some()
    {
        scale[0] *= sc[0] as f32;
        scale[1] *= sc[0] as f32;
        scale[2] *= sc[0] as f32;
    }

    let mut nus = [0.0; 3];
    if feature_table
        .get_property(
            "SCALE_NON_UNIFORM",
            Some(ComponentDatatype::Float),
            3,
            i,
            &mut nus,
        )
        .is_some()
    {
        scale[0] *= nus[0] as f32;
        scale[1] *= nus[1] as f32;
        scale[2] *= nus[2] as f32;
    }

    scale
}
