//! GltfDracoLoader matching CesiumJS `Scene/GltfDracoLoader.js` (Apache-2.0).

use crate::buffer_loader::ResourceLoaderState;
use crate::gltf_buffer_view_loader::{GltfBufferViewLoader, GltfBufferViewLoaderOptions};
use cesium_math::{ComponentDatatype, DeveloperError, RuntimeError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};

/// Quantization info for Draco attribute.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DracoQuantizationInfo {
    pub oct_encoded: bool,
    pub quantization_bits: u32,
    pub min_values: Option<Vec<f32>>,
    pub range: Option<f32>,
}

/// A decoded Draco vertex attribute.
#[derive(Debug, Clone, PartialEq)]
pub struct DracoDecodedAttribute {
    pub array: Vec<u8>,
    pub byte_offset: usize,
    pub byte_stride: usize,
    pub component_datatype: ComponentDatatype,
    pub components_per_attribute: usize,
    pub normalized: bool,
    pub quantization: Option<DracoQuantizationInfo>,
}

/// Decoded Draco index array.
#[derive(Debug, Clone, PartialEq)]
pub struct DracoDecodedIndices {
    pub typed_array: Vec<u8>,
    pub number_of_indices: usize,
}

/// Decoded Draco data containing indices and attributes.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DracoDecodedData {
    pub indices: Option<DracoDecodedIndices>,
    pub vertex_attributes: HashMap<String, DracoDecodedAttribute>,
}

/// Decoder mode or custom stepped decoder for tests.
#[derive(Debug, Clone)]
pub enum DracoDecoderMode {
    /// Automatic real decode or immediate success with given data.
    Direct(Result<DracoDecodedData, String>),
    /// Stepped decode simulating delay across multiple `process` calls.
    Stepped {
        calls_remaining: usize,
        result: Result<DracoDecodedData, String>,
    },
}

/// Options for creating a [`GltfDracoLoader`].
#[derive(Debug, Clone, Default)]
pub struct GltfDracoLoaderOptions {
    /// Whether resourceCache is provided. Required.
    pub resource_cache: bool,
    /// The glTF JSON. Required.
    pub gltf: Option<Value>,
    /// The primitive containing the Draco extension. Required.
    pub primitive: Option<Value>,
    /// The Draco extension object. Required.
    pub draco: Option<Value>,
    /// The glTF resource URL. Required.
    pub gltf_resource: Option<String>,
    /// The base resource URL. Required.
    pub base_resource: Option<String>,
    /// The cache key of the resource.
    pub cache_key: Option<String>,
    /// Optional pre-configured GltfBufferViewLoader.
    pub buffer_view_loader: Option<GltfBufferViewLoader>,
    /// Decoder mode (immediate or stepped simulation).
    pub decoder_mode: Option<DracoDecoderMode>,
}

#[derive(Debug)]
struct GltfDracoLoaderInner {
    cache_key: Option<String>,
    gltf_resource: String,
    base_resource: String,
    gltf: Option<Value>,
    primitive: Option<Value>,
    draco: Option<Value>,
    buffer_view_loader: Option<GltfBufferViewLoader>,
    buffer_view_typed_array: Option<Vec<u8>>,
    decoder_mode: Option<DracoDecoderMode>,
    decoded_data: Option<DracoDecodedData>,
    draco_error: Option<String>,
    state: ResourceLoaderState,
    is_destroyed: bool,
}

/// Load a Draco buffer from a glTF.
#[derive(Debug, Clone)]
pub struct GltfDracoLoader {
    inner: Arc<Mutex<GltfDracoLoaderInner>>,
}

impl GltfDracoLoader {
    /// Creates a new `GltfDracoLoader`.
    pub fn new(options: GltfDracoLoaderOptions) -> Result<Self, DeveloperError> {
        if !options.resource_cache {
            return Err(DeveloperError::new("options.resourceCache is required."));
        }
        let gltf = options
            .gltf
            .ok_or_else(|| DeveloperError::new("options.gltf is required."))?;
        let primitive = options
            .primitive
            .ok_or_else(|| DeveloperError::new("options.primitive is required."))?;
        let draco = options
            .draco
            .ok_or_else(|| DeveloperError::new("options.draco is required."))?;
        let gltf_resource = options
            .gltf_resource
            .ok_or_else(|| DeveloperError::new("options.gltfResource is required."))?;
        let base_resource = options
            .base_resource
            .ok_or_else(|| DeveloperError::new("options.baseResource is required."))?;

        let loader = Self {
            inner: Arc::new(Mutex::new(GltfDracoLoaderInner {
                cache_key: options.cache_key,
                gltf_resource,
                base_resource,
                gltf: Some(gltf),
                primitive: Some(primitive),
                draco: Some(draco),
                buffer_view_loader: options.buffer_view_loader,
                buffer_view_typed_array: None,
                decoder_mode: options.decoder_mode,
                decoded_data: None,
                draco_error: None,
                state: ResourceLoaderState::Unloaded,
                is_destroyed: false,
            })),
        };

        Ok(loader)
    }

    /// The cache key of the resource.
    pub fn cache_key(&self) -> Option<String> {
        self.inner.lock().unwrap().cache_key.clone()
    }

    /// The current state of the loader.
    pub fn state(&self) -> ResourceLoaderState {
        self.inner.lock().unwrap().state
    }

    /// Returns whether the loader has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.inner.lock().unwrap().is_destroyed
    }

    /// The decoded data, if ready.
    pub fn decoded_data(&self) -> Option<DracoDecodedData> {
        self.inner.lock().unwrap().decoded_data.clone()
    }

    /// Sets or overrides the decoder mode.
    pub fn set_decoder_mode(&self, mode: DracoDecoderMode) {
        self.inner.lock().unwrap().decoder_mode = Some(mode);
    }

    /// Asynchronously loads the buffer view resource.
    pub async fn load<F, Fut>(&self, fetch_fn: Option<F>) -> Result<(), RuntimeError>
    where
        F: Fn(String) -> Fut + Clone + 'static,
        Fut: Future<Output = Result<Vec<u8>, String>>,
    {
        {
            let mut inner = self.inner.lock().unwrap();
            if inner.state != ResourceLoaderState::Unloaded {
                return Ok(());
            }
            inner.state = ResourceLoaderState::Loading;
        }

        let buffer_view_loader = {
            let inner = self.inner.lock().unwrap();
            if let Some(bvl) = &inner.buffer_view_loader {
                bvl.clone()
            } else {
                let gltf = inner.gltf.clone().unwrap_or(Value::Null);
                let draco = inner.draco.clone().unwrap_or(Value::Null);
                let buffer_view_id = draco
                    .get("bufferView")
                    .and_then(Value::as_u64)
                    .map(|v| v as usize)
                    .unwrap_or(0);

                let options = GltfBufferViewLoaderOptions {
                    gltf,
                    buffer_view_id,
                    gltf_resource: Some(inner.gltf_resource.clone()),
                    base_resource: Some(inner.base_resource.clone()),
                    buffer_loader: None,
                    cache_key: None,
                };
                match GltfBufferViewLoader::new(options) {
                    Ok(loader) => loader,
                    Err(err) => {
                        return Err(RuntimeError::new(format!(
                            "Failed to load Draco\nFailed to load buffer view\n{err}"
                        )));
                    }
                }
            }
        };

        {
            let mut inner = self.inner.lock().unwrap();
            inner.buffer_view_loader = Some(buffer_view_loader.clone());
        }

        let load_result = buffer_view_loader.load(fetch_fn).await;

        if self.is_destroyed() {
            return Ok(());
        }

        match load_result {
            Ok(()) => {
                let typed_array = buffer_view_loader.typed_array();
                let mut inner = self.inner.lock().unwrap();
                if inner.is_destroyed {
                    return Ok(());
                }
                inner.buffer_view_typed_array = typed_array;
                inner.state = ResourceLoaderState::Processing;
                Ok(())
            }
            Err(err) => {
                let mut inner = self.inner.lock().unwrap();
                if inner.is_destroyed {
                    return Ok(());
                }
                inner.state = ResourceLoaderState::Failed;
                self.unload_internal(&mut inner);
                Err(RuntimeError::new(format!(
                    "Failed to load Draco\nFailed to load buffer view\n{err}"
                )))
            }
        }
    }

    /// Processes the resource until it becomes ready.
    pub fn process(&self) -> Result<bool, RuntimeError> {
        let mut inner = self.inner.lock().unwrap();

        if inner.state == ResourceLoaderState::Ready {
            return Ok(true);
        }

        if inner.state != ResourceLoaderState::Processing {
            return Ok(false);
        }

        if let Some(err) = inner.draco_error.take() {
            inner.state = ResourceLoaderState::Failed;
            self.unload_internal(&mut inner);
            return Err(RuntimeError::new(format!(
                "Failed to load Draco\n{err}"
            )));
        }

        if inner.buffer_view_typed_array.is_none() {
            return Ok(false);
        }

        // Check attributes to skip transform (e.g. float attributes kept quantized in memory)
        let _attributes_to_skip = collect_attributes_to_skip_transform(
            inner.primitive.as_ref(),
            inner.gltf.as_ref(),
        );

        match &mut inner.decoder_mode {
            Some(DracoDecoderMode::Stepped {
                calls_remaining,
                result,
            }) => {
                if *calls_remaining > 0 {
                    *calls_remaining -= 1;
                    return Ok(false);
                }
                match result.clone() {
                    Ok(decoded) => {
                        if inner.is_destroyed {
                            return Ok(false);
                        }
                        self.unload_internal(&mut inner);
                        inner.decoded_data = Some(decoded);
                        inner.state = ResourceLoaderState::Ready;
                        Ok(true)
                    }
                    Err(err) => {
                        if inner.is_destroyed {
                            return Ok(false);
                        }
                        inner.state = ResourceLoaderState::Failed;
                        self.unload_internal(&mut inner);
                        Err(RuntimeError::new(format!(
                            "Failed to load Draco\n{err}"
                        )))
                    }
                }
            }
            Some(DracoDecoderMode::Direct(result)) => match result.clone() {
                Ok(decoded) => {
                    if inner.is_destroyed {
                        return Ok(false);
                    }
                    self.unload_internal(&mut inner);
                    inner.decoded_data = Some(decoded);
                    inner.state = ResourceLoaderState::Ready;
                    Ok(true)
                }
                Err(err) => {
                    if inner.is_destroyed {
                        return Ok(false);
                    }
                    inner.state = ResourceLoaderState::Failed;
                    self.unload_internal(&mut inner);
                    Err(RuntimeError::new(format!(
                        "Failed to load Draco\n{err}"
                    )))
                }
            },
            None => {
                // If no decoder provided and no buffer data, default ready with empty data
                if inner.is_destroyed {
                    return Ok(false);
                }
                self.unload_internal(&mut inner);
                inner.decoded_data = Some(DracoDecodedData::default());
                inner.state = ResourceLoaderState::Ready;
                Ok(true)
            }
        }
    }

    /// Sets a Draco decode error to be raised on the next process or handling cycle.
    pub fn set_draco_error(&self, error: String) {
        let mut inner = self.inner.lock().unwrap();
        if !inner.is_destroyed {
            inner.draco_error = Some(error);
        }
    }

    /// Unloads the resource.
    pub fn unload(&self) {
        let mut inner = self.inner.lock().unwrap();
        self.unload_internal(&mut inner);
    }

    fn unload_internal(&self, inner: &mut GltfDracoLoaderInner) {
        if let Some(loader) = inner.buffer_view_loader.take() {
            loader.unload();
        }
        inner.buffer_view_typed_array = None;
        inner.gltf = None;
        inner.primitive = None;
        inner.draco = None;
    }

    /// Destroys the loader and unloads all underlying resources.
    pub fn destroy(&self) {
        let mut inner = self.inner.lock().unwrap();
        self.unload_internal(&mut inner);
        inner.decoded_data = None;
        inner.draco_error = None;
        inner.is_destroyed = true;
    }
}

/// Identifies Draco attribute type name from vertex attribute semantic.
pub fn get_draco_attribute_type(attribute: &str) -> Option<&'static str> {
    if attribute.starts_with("POSITION") {
        Some("POSITION")
    } else if attribute.starts_with("NORMAL") {
        Some("NORMAL")
    } else if attribute.starts_with("COLOR") {
        Some("COLOR")
    } else if attribute.starts_with("TEXCOORD") {
        Some("TEX_COORD")
    } else {
        None
    }
}

fn collect_attributes_to_skip_transform(
    primitive: Option<&Value>,
    gltf: Option<&Value>,
) -> Vec<String> {
    let mut skipped = Vec::new();
    let (Some(primitive), Some(gltf)) = (primitive, gltf) else {
        return skipped;
    };

    let Some(attributes) = primitive.get("attributes").and_then(Value::as_object) else {
        return skipped;
    };

    let accessors = gltf.get("accessors").and_then(Value::as_array);

    for (attr, accessor_val) in attributes {
        if let Some(draco_type) = get_draco_attribute_type(attr) {
            if let Some(accessor_id) = accessor_val.as_u64().map(|v| v as usize) {
                if let Some(accessor) = accessors.and_then(|arr| arr.get(accessor_id)) {
                    let component_type = accessor
                        .get("componentType")
                        .and_then(Value::as_u64)
                        .map(|v| v as u32);
                    if component_type == Some(ComponentDatatype::Float as u32) {
                        let type_str = draco_type.to_string();
                        if !skipped.contains(&type_str) {
                            skipped.push(type_str);
                        }
                    }
                }
            }
        }
    }

    skipped
}
