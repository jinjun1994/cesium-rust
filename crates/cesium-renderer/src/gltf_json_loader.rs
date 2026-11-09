//! GltfJsonLoader matching CesiumJS `Scene/GltfJsonLoader.js` (Apache-2.0).

use crate::buffer_loader::ResourceLoaderState;
use cesium_math::{DeveloperError, RuntimeError};
use serde_json::Value;
use std::collections::HashSet;
use std::future::Future;
use std::sync::{Arc, Mutex};

pub const GLB_MAGIC: u32 = 0x46546c67; // "glTF"
pub const GLB_CHUNK_TYPE_JSON: u32 = 0x4e4f534a; // "JSON"
pub const GLB_CHUNK_TYPE_BIN: u32 = 0x004e4942; // "BIN\0"

/// Set of supported glTF extensions.
pub fn is_supported_extension(name: &str) -> bool {
    matches!(
        name,
        "KHR_meshopt_compression"
            | "EXT_meshopt_compression"
            | "KHR_materials_unlit"
            | "KHR_texture_transform"
            | "KHR_materials_common"
            | "KHR_techniques_webgl"
            | "CESIUM_primitive_outline"
            | "EXT_structural_metadata"
            | "EXT_instance_features"
            | "EXT_feature_metadata"
            | "KHR_draco_mesh_compression"
    )
}

/// Options for creating a [`GltfJsonLoader`].
#[derive(Debug, Clone, Default)]
pub struct GltfJsonLoaderOptions {
    /// URI or URL pointing to the glTF resource. Required.
    pub gltf_resource: Option<String>,
    /// Base URI for resolving relative paths. Required.
    pub base_resource: Option<String>,
    /// Optional in-memory typed array of the glTF (JSON text or GLB binary).
    pub typed_array: Option<Vec<u8>>,
    /// Optional pre-parsed glTF JSON.
    pub gltf_json: Option<Value>,
    /// Cache key.
    pub cache_key: Option<String>,
}

#[derive(Debug)]
struct GltfJsonLoaderInner {
    gltf_resource: String,
    base_resource: String,
    typed_array: Option<Vec<u8>>,
    gltf_json: Option<Value>,
    cache_key: Option<String>,
    gltf: Option<Value>,
    state: ResourceLoaderState,
    is_destroyed: bool,
}

/// Loads a glTF JSON from a glTF (JSON text) or GLB (binary).
#[derive(Debug, Clone)]
pub struct GltfJsonLoader {
    inner: Arc<Mutex<GltfJsonLoaderInner>>,
}

impl GltfJsonLoader {
    /// Creates a new `GltfJsonLoader`.
    pub fn new(options: GltfJsonLoaderOptions) -> Result<Self, DeveloperError> {
        let gltf_resource = options
            .gltf_resource
            .ok_or_else(|| DeveloperError::new("options.gltfResource is required."))?;
        let base_resource = options
            .base_resource
            .ok_or_else(|| DeveloperError::new("options.baseResource is required."))?;

        Ok(Self {
            inner: Arc::new(Mutex::new(GltfJsonLoaderInner {
                gltf_resource,
                base_resource,
                typed_array: options.typed_array,
                gltf_json: options.gltf_json,
                cache_key: options.cache_key,
                gltf: None,
                state: ResourceLoaderState::Unloaded,
                is_destroyed: false,
            })),
        })
    }

    /// The base resource for resolving relative paths.
    pub fn base_resource(&self) -> String {
        self.inner.lock().unwrap().base_resource.clone()
    }

    /// The glTF resource URL.
    pub fn gltf_resource(&self) -> String {
        self.inner.lock().unwrap().gltf_resource.clone()
    }

    /// The cache key of the resource.
    pub fn cache_key(&self) -> Option<String> {
        self.inner.lock().unwrap().cache_key.clone()
    }

    /// The parsed and loaded glTF JSON.
    pub fn gltf(&self) -> Option<Value> {
        self.inner.lock().unwrap().gltf.clone()
    }

    /// The loader's current state.
    pub fn state(&self) -> ResourceLoaderState {
        self.inner.lock().unwrap().state
    }

    /// Whether this loader has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.inner.lock().unwrap().is_destroyed
    }

    /// Unloads the glTF resource.
    pub fn unload(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.gltf = None;
    }

    /// Destroys the loader.
    pub fn destroy(&self) {
        self.unload();
        self.inner.lock().unwrap().is_destroyed = true;
    }

    /// Loads the glTF JSON.
    pub async fn load<F, Fut>(&self, fetch_fn: Option<F>) -> Result<(), RuntimeError>
    where
        F: FnOnce(String) -> Fut,
        Fut: Future<Output = Result<Vec<u8>, String>>,
    {
        let (resource_url, typed_array, gltf_json, is_destroyed) = {
            let mut inner = self.inner.lock().unwrap();
            if inner.is_destroyed {
                return Ok(());
            }
            inner.state = ResourceLoaderState::Loading;
            (
                inner.gltf_resource.clone(),
                inner.typed_array.clone(),
                inner.gltf_json.clone(),
                inner.is_destroyed,
            )
        };

        if is_destroyed {
            return Ok(());
        }

        let parsed_gltf = if let Some(json) = gltf_json {
            Ok(json)
        } else if let Some(bytes) = typed_array {
            Self::parse_bytes(&bytes)
        } else {
            let fetch = fetch_fn.ok_or_else(|| {
                RuntimeError::new(format!(
                    "Failed to load glTF: {resource_url}\nNo fetcher provided"
                ))
            })?;

            match fetch(resource_url.clone()).await {
                Ok(bytes) => Self::parse_bytes(&bytes),
                Err(err) => Err(RuntimeError::new(format!(
                    "Failed to load glTF: {resource_url}\n{err}"
                ))),
            }
        };

        match parsed_gltf {
            Ok(gltf) => {
                // Validate asset version
                let version = gltf
                    .get("asset")
                    .and_then(|a| a.get("version"))
                    .and_then(Value::as_str)
                    .unwrap_or("");

                if version != "1.0" && version != "2.0" && version != "2.1" {
                    let mut inner = self.inner.lock().unwrap();
                    inner.state = ResourceLoaderState::Failed;
                    return Err(RuntimeError::new(format!(
                        "Unsupported glTF version: {version}"
                    )));
                }

                // Check extensionsRequired
                if let Some(extensions) = gltf.get("extensionsRequired").and_then(Value::as_array) {
                    let mut unsupported = HashSet::new();
                    for ext in extensions {
                        if let Some(ext_name) = ext.as_str() {
                            if !is_supported_extension(ext_name) {
                                unsupported.insert(ext_name.to_string());
                            }
                        }
                    }

                    if !unsupported.is_empty() {
                        let mut inner = self.inner.lock().unwrap();
                        inner.state = ResourceLoaderState::Failed;
                        let mut names: Vec<_> = unsupported.into_iter().collect();
                        names.sort();
                        return Err(RuntimeError::new(format!(
                            "Unsupported glTF extension(s): {}",
                            names.join(", ")
                        )));
                    }
                }

                let mut inner = self.inner.lock().unwrap();
                if inner.is_destroyed {
                    return Ok(());
                }
                inner.gltf = Some(gltf);
                inner.state = ResourceLoaderState::Ready;
                Ok(())
            }
            Err(e) => {
                let mut inner = self.inner.lock().unwrap();
                inner.state = ResourceLoaderState::Failed;
                Err(e)
            }
        }
    }

    /// Parses glTF text JSON or binary GLB into `serde_json::Value`.
    pub fn parse_bytes(bytes: &[u8]) -> Result<Value, RuntimeError> {
        if bytes.len() >= 4 {
            let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            if magic == GLB_MAGIC {
                return Self::parse_glb(bytes);
            }
        }

        serde_json::from_slice(bytes)
            .map_err(|e| RuntimeError::new(format!("Failed to parse glTF JSON: {e}")))
    }

    /// Parses GLB binary format (versions 1 and 2).
    pub fn parse_glb(bytes: &[u8]) -> Result<Value, RuntimeError> {
        if bytes.len() < 12 {
            return Err(RuntimeError::new("GLB header too short."));
        }

        let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

        if version == 1 {
            if bytes.len() < 20 {
                return Err(RuntimeError::new("GLB v1 header too short."));
            }
            let json_length =
                u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
            let offset = 20;
            if offset + json_length > bytes.len() {
                return Err(RuntimeError::new("GLB v1 JSON chunk exceeds file bounds."));
            }

            let mut json: Value = serde_json::from_slice(&bytes[offset..offset + json_length])
                .map_err(|e| RuntimeError::new(format!("Failed to parse GLB v1 JSON: {e}")))?;

            // Store binary chunk in buffer 0 if present
            let bin_offset = offset + json_length;
            if bin_offset < bytes.len() {
                let bin_data = &bytes[bin_offset..];
                Self::attach_buffer_0_source(&mut json, bin_data);
            }

            Ok(json)
        } else if version == 2 {
            let mut offset = 12;
            let mut json_val = None;
            let mut bin_data = None;

            while offset + 8 <= bytes.len() {
                let chunk_length = u32::from_le_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ]) as usize;
                let chunk_type = u32::from_le_bytes([
                    bytes[offset + 4],
                    bytes[offset + 5],
                    bytes[offset + 6],
                    bytes[offset + 7],
                ]);
                offset += 8;

                if offset + chunk_length > bytes.len() {
                    return Err(RuntimeError::new("GLB chunk exceeds file bounds."));
                }

                let chunk_data = &bytes[offset..offset + chunk_length];
                offset += chunk_length;

                if chunk_type == GLB_CHUNK_TYPE_JSON && json_val.is_none() {
                    let v: Value = serde_json::from_slice(chunk_data).map_err(|e| {
                        RuntimeError::new(format!("Failed to parse GLB v2 JSON chunk: {e}"))
                    })?;
                    json_val = Some(v);
                } else if chunk_type == GLB_CHUNK_TYPE_BIN && bin_data.is_none() {
                    bin_data = Some(chunk_data);
                }
            }

            let mut json = json_val
                .ok_or_else(|| RuntimeError::new("GLB does not contain a JSON chunk."))?;

            if let Some(bin) = bin_data {
                Self::attach_buffer_0_source(&mut json, bin);
            }

            Ok(json)
        } else {
            Err(RuntimeError::new(format!(
                "Unsupported GLB version: {version}"
            )))
        }
    }

    fn attach_buffer_0_source(json: &mut Value, bin_data: &[u8]) {
        if let Some(buffers) = json.get_mut("buffers").and_then(Value::as_array_mut) {
            if let Some(b0) = buffers.get_mut(0) {
                let obj = b0.as_object_mut();
                if let Some(map) = obj {
                    let extras = map.entry("extras").or_insert_with(|| serde_json::json!({}));
                    if let Some(extras_map) = extras.as_object_mut() {
                        let pipeline = extras_map
                            .entry("_pipeline")
                            .or_insert_with(|| serde_json::json!({}));
                        if let Some(pipe_map) = pipeline.as_object_mut() {
                            pipe_map.insert(
                                "source_len".to_string(),
                                serde_json::json!(bin_data.len()),
                            );
                        }
                    }
                }
            }
        }
    }
}
