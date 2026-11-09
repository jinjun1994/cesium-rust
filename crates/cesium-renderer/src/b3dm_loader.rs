//! B3dmLoader matching CesiumJS `Scene/Model/B3dmLoader.js` and `Scene/B3dmParser.js` (Apache-2.0).

use crate::axis::Axis;
use crate::buffer_loader::ResourceLoaderState;
use crate::gltf_loader::{GltfLoader, GltfLoaderOptions};
use crate::model_components::Components;
use cesium_formats::{parse_b3dm, DecodeLimits};
use cesium_math::{Cartesian3, DeveloperError, Matrix4, RuntimeError};
use serde_json::Value;
use std::sync::{Arc, Mutex};

/// Parsed payload from a B3DM binary.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedB3dm {
    pub version: u32,
    pub byte_length: u32,
    pub batch_length: u32,
    pub feature_table_json: Value,
    pub feature_table_binary: Vec<u8>,
    pub batch_table_json: Option<Value>,
    pub batch_table_binary: Vec<u8>,
    pub rtc_center: Option<Cartesian3>,
    pub glb: Vec<u8>,
}

/// Parser for B3DM byte buffers.
pub struct B3dmParser;

impl B3dmParser {
    /// Parses a B3DM byte array.
    pub fn parse(data: &[u8], byte_offset: usize) -> Result<ParsedB3dm, RuntimeError> {
        let slice = if byte_offset > 0 {
            if byte_offset >= data.len() {
                return Err(RuntimeError::new("byteOffset out of range"));
            }
            &data[byte_offset..]
        } else {
            data
        };

        if slice.len() < 12 {
            return Err(RuntimeError::new("B3DM buffer too small"));
        }

        if &slice[0..4] != b"b3dm" {
            return Err(RuntimeError::new("Invalid B3DM magic"));
        }

        let version = u32::from_le_bytes([slice[4], slice[5], slice[6], slice[7]]);
        if version != 1 {
            return Err(RuntimeError::new(format!(
                "Only Batched 3D Model version 1 is supported. Version {version} is not."
            )));
        }

        let byte_length = u32::from_le_bytes([slice[8], slice[9], slice[10], slice[11]]);
        if (byte_length as usize) > slice.len() {
            return Err(RuntimeError::new("B3DM byteLength exceeds buffer size"));
        }

        let limits = DecodeLimits::default();
        let parsed = parse_b3dm(slice, &limits)
            .map_err(|e| RuntimeError::new(format!("Failed to parse B3DM: {e}")))?;

        let rtc_center = if parsed.rtc_center != glam::DVec3::ZERO {
            Some(Cartesian3::new(
                parsed.rtc_center.x,
                parsed.rtc_center.y,
                parsed.rtc_center.z,
            ))
        } else {
            None
        };

        let batch_table_json = if parsed.batch_table.is_null()
            || parsed.batch_table == serde_json::json!({})
        {
            None
        } else {
            Some(parsed.batch_table)
        };

        Ok(ParsedB3dm {
            version,
            byte_length,
            batch_length: parsed.batch_length,
            feature_table_json: serde_json::json!({ "BATCH_LENGTH": parsed.batch_length }),
            feature_table_binary: parsed.feature_binary.to_vec(),
            batch_table_json,
            batch_table_binary: parsed.batch_binary.to_vec(),
            rtc_center,
            glb: parsed.glb.to_vec(),
        })
    }
}

/// Options for creating a [`B3dmLoader`].
#[derive(Debug, Clone, Default)]
pub struct B3dmLoaderOptions {
    pub b3dm_resource: Option<String>,
    pub base_resource: Option<String>,
    pub array_buffer: Option<Vec<u8>>,
    pub byte_offset: Option<usize>,
    pub release_gltf_json: Option<bool>,
    pub asynchronous: Option<bool>,
    pub incrementally_load_textures: Option<bool>,
    pub up_axis: Option<Axis>,
    pub forward_axis: Option<Axis>,
}

#[derive(Debug)]
struct B3dmLoaderInner {
    b3dm_resource: String,
    base_resource: String,
    array_buffer: Option<Vec<u8>>,
    byte_offset: usize,
    release_gltf_json: bool,
    asynchronous: bool,
    incrementally_load_textures: bool,
    up_axis: Axis,
    forward_axis: Axis,

    state: ResourceLoaderState,
    is_destroyed: bool,

    gltf_loader: Option<GltfLoader>,
    batch_length: u32,
    batch_table: Option<Value>,
    transform: Matrix4,
    components: Option<Components>,
}

unsafe impl Send for B3dmLoaderInner {}

/// Loads a Batched 3D Model (`.b3dm`).
#[derive(Debug, Clone)]
pub struct B3dmLoader {
    inner: Arc<Mutex<B3dmLoaderInner>>,
}

impl B3dmLoader {
    /// Creates a new `B3dmLoader`.
    pub fn new(options: B3dmLoaderOptions) -> Result<Self, DeveloperError> {
        let b3dm_resource = options
            .b3dm_resource
            .ok_or_else(|| DeveloperError::new("options.b3dmResource is required."))?;
        let array_buffer = options
            .array_buffer
            .ok_or_else(|| DeveloperError::new("options.arrayBuffer is required."))?;
        let base_resource = options
            .base_resource
            .unwrap_or_else(|| b3dm_resource.clone());

        let byte_offset = options.byte_offset.unwrap_or(0);
        let release_gltf_json = options.release_gltf_json.unwrap_or(false);
        let asynchronous = options.asynchronous.unwrap_or(true);
        let incrementally_load_textures = options.incrementally_load_textures.unwrap_or(true);
        let up_axis = options.up_axis.unwrap_or(Axis::Y);
        let forward_axis = options.forward_axis.unwrap_or(Axis::X);

        Ok(Self {
            inner: Arc::new(Mutex::new(B3dmLoaderInner {
                b3dm_resource,
                base_resource,
                array_buffer: Some(array_buffer),
                byte_offset,
                release_gltf_json,
                asynchronous,
                incrementally_load_textures,
                up_axis,
                forward_axis,
                state: ResourceLoaderState::Unloaded,
                is_destroyed: false,
                gltf_loader: None,
                batch_length: 0,
                batch_table: None,
                transform: Matrix4::IDENTITY,
                components: None,
            })),
        })
    }

    /// Returns the loader state.
    pub fn state(&self) -> ResourceLoaderState {
        self.inner.lock().unwrap().state
    }

    /// Whether this loader has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.inner.lock().unwrap().is_destroyed
    }

    /// Returns the loaded model components.
    pub fn components(&self) -> Option<Components> {
        self.inner.lock().unwrap().components.clone()
    }

    /// Whether the underlying array buffer has been released.
    pub fn has_array_buffer(&self) -> bool {
        self.inner.lock().unwrap().array_buffer.is_some()
    }

    /// Unloads and destroys this loader.
    pub fn destroy(&self) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(ref gltf) = inner.gltf_loader {
            gltf.destroy();
        }
        inner.gltf_loader = None;
        inner.components = None;
        inner.array_buffer = None;
        inner.is_destroyed = true;
    }

    /// Loads the B3DM resource and begins glTF decoding.
    pub async fn load(&self) -> Result<(), RuntimeError> {
        let (buf, byte_offset, b3dm_res, base_res, release_json, async_val, incr_tex, up, fwd) = {
            let mut inner = self.inner.lock().unwrap();
            if inner.is_destroyed {
                return Ok(());
            }
            if inner.state != ResourceLoaderState::Unloaded {
                return Ok(());
            }
            inner.state = ResourceLoaderState::Loading;
            let buf = inner
                .array_buffer
                .as_ref()
                .ok_or_else(|| RuntimeError::new("No arrayBuffer available"))?
                .clone();
            (
                buf,
                inner.byte_offset,
                inner.b3dm_resource.clone(),
                inner.base_resource.clone(),
                inner.release_gltf_json,
                inner.asynchronous,
                inner.incrementally_load_textures,
                inner.up_axis,
                inner.forward_axis,
            )
        };

        let parsed = match B3dmParser::parse(&buf, byte_offset) {
            Ok(p) => p,
            Err(e) => {
                let mut inner = self.inner.lock().unwrap();
                inner.state = ResourceLoaderState::Failed;
                return Err(e);
            }
        };

        let transform = if let Some(rtc) = parsed.rtc_center {
            Matrix4::from_translation(rtc)
        } else {
            Matrix4::IDENTITY
        };

        let gltf_options = GltfLoaderOptions {
            gltf_resource: Some(b3dm_res),
            base_resource: Some(base_res),
            typed_array: Some(parsed.glb),
            release_gltf_json: release_json,
            asynchronous: Some(async_val),
            incrementally_load_textures: Some(incr_tex),
            up_axis: Some(up),
            forward_axis: Some(fwd),
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
        inner.batch_length = parsed.batch_length;
        inner.batch_table = parsed.batch_table_json;
        inner.transform = transform;
        inner.state = ResourceLoaderState::Processing;

        Ok(())
    }

    /// Processes GPU resources and finalizes components.
    pub fn process(&self) -> Result<bool, RuntimeError> {
        let inner = self.inner.lock().unwrap();
        if inner.state == ResourceLoaderState::Ready {
            return Ok(true);
        }
        if inner.state != ResourceLoaderState::Processing {
            return Ok(false);
        }

        let gltf_loader = inner.gltf_loader.clone().unwrap();
        drop(inner);

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
        components.transform = inner.transform.multiply(&components.transform);

        if inner.batch_length > 0 {
            if let Some(ref bt) = inner.batch_table {
                components.structural_metadata = Some(bt.clone());
            } else {
                components.structural_metadata = Some(serde_json::json!({
                    "propertyTables": [{
                        "name": "BatchTable",
                        "count": inner.batch_length
                    }]
                }));
            }
        }

        inner.components = Some(components);
        inner.array_buffer = None;
        inner.state = ResourceLoaderState::Ready;

        Ok(true)
    }
}
