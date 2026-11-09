//! GltfVertexBufferLoader matching CesiumJS `Scene/GltfVertexBufferLoader.js` (Apache-2.0).

use crate::buffer::{Buffer, BufferOptions};
use crate::buffer_loader::ResourceLoaderState;
use crate::gltf_buffer_view_loader::{GltfBufferViewLoader, GltfBufferViewLoaderOptions};
use cesium_math::{DeveloperError, RuntimeError};
use serde_json::Value;
use std::future::Future;
use std::sync::{Arc, Mutex};

/// Information about a quantized vertex attribute after Draco decode.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantizationInfo {
    pub component_datatype: u32,
    pub oct_encoded: bool,
    pub oct_encoded_zxy: bool,
    pub attribute_type: String,
    pub quantized_volume_offset: Vec<f64>,
    pub quantized_volume_dimensions: Vec<f64>,
    pub quantized_volume_step_size: Vec<f64>,
    pub normalization_range: f64,
}

/// Options for creating a [`GltfVertexBufferLoader`].
#[derive(Debug, Clone, Default)]
pub struct GltfVertexBufferLoaderOptions {
    /// The glTF JSON. Required.
    pub gltf: Option<Value>,
    /// The glTF resource URL. Required.
    pub gltf_resource: Option<String>,
    /// The base resource URL. Required.
    pub base_resource: Option<String>,
    /// The bufferView ID corresponding to the vertex buffer.
    pub buffer_view_id: Option<usize>,
    /// The primitive containing the Draco extension.
    pub primitive: Option<Value>,
    /// The Draco extension object.
    pub draco: Option<Value>,
    /// The SPZ extension object.
    pub spz: Option<Value>,
    /// The attribute semantic, e.g. POSITION or NORMAL.
    pub attribute_semantic: Option<String>,
    /// The accessor ID.
    pub accessor_id: Option<usize>,
    /// Cache key of the resource.
    pub cache_key: Option<String>,
    /// Determines if WebGL resource creation will be spread out or synchronous. Default true.
    pub asynchronous: Option<bool>,
    /// Load vertex buffer as a GPU vertex buffer. Default false.
    pub load_buffer: Option<bool>,
    /// Load vertex buffer as a typed array. Default false.
    pub load_typed_array: Option<bool>,
    /// Optional pre-constructed buffer view loader.
    pub buffer_view_loader: Option<GltfBufferViewLoader>,
    /// Optional pre-decoded Draco vertex attribute (raw bytes, optional quantization).
    pub decoded_draco_attribute: Option<(Vec<u8>, Option<QuantizationInfo>)>,
    /// Whether Draco decode should simulate a failure.
    pub draco_should_fail: Option<bool>,
}

#[derive(Debug)]
struct GltfVertexBufferLoaderInner {
    gltf: Option<Value>,
    gltf_resource: String,
    base_resource: String,
    buffer_view_id: Option<usize>,
    primitive: Option<Value>,
    draco: Option<Value>,
    #[allow(dead_code)]
    spz: Option<Value>,
    attribute_semantic: Option<String>,
    accessor_id: Option<usize>,
    cache_key: Option<String>,
    asynchronous: bool,
    load_buffer: bool,
    load_typed_array: bool,
    buffer_view_loader: Option<GltfBufferViewLoader>,
    buffer: Option<Buffer>,
    typed_array: Option<Vec<u8>>,
    quantization: Option<QuantizationInfo>,
    decoded_draco_attribute: Option<(Vec<u8>, Option<QuantizationInfo>)>,
    draco_should_fail: bool,
    state: ResourceLoaderState,
    is_destroyed: bool,
}

/// Loads a vertex buffer from a glTF buffer view, Draco, or SPZ.
#[derive(Debug, Clone)]
pub struct GltfVertexBufferLoader {
    inner: Arc<Mutex<GltfVertexBufferLoaderInner>>,
}

fn has_draco_compression(draco: Option<&Value>, semantic: Option<&str>) -> bool {
    match (draco, semantic) {
        (Some(d), Some(sem)) => d
            .get("attributes")
            .and_then(|a| a.get(sem))
            .is_some(),
        _ => false,
    }
}

impl GltfVertexBufferLoader {
    /// Creates a new `GltfVertexBufferLoader`.
    pub fn new(options: GltfVertexBufferLoaderOptions) -> Result<Self, DeveloperError> {
        let gltf = options
            .gltf
            .ok_or_else(|| DeveloperError::new("options.gltf is required."))?;
        let gltf_resource = options
            .gltf_resource
            .ok_or_else(|| DeveloperError::new("options.gltfResource is required."))?;
        let base_resource = options
            .base_resource
            .ok_or_else(|| DeveloperError::new("options.baseResource is required."))?;

        let load_buffer = options.load_buffer.unwrap_or(false);
        let load_typed_array = options.load_typed_array.unwrap_or(false);
        if !load_buffer && !load_typed_array {
            return Err(DeveloperError::new(
                "At least one of loadBuffer and loadTypedArray must be true.",
            ));
        }

        let has_buffer_view_id = options.buffer_view_id.is_some();
        let has_draco = has_draco_compression(
            options.draco.as_ref(),
            options.attribute_semantic.as_deref(),
        );
        let has_spz = options.spz.is_some();

        let source_count = (has_buffer_view_id as usize)
            + (has_draco as usize)
            + (has_spz as usize);

        if source_count != 1 {
            return Err(DeveloperError::new(
                "Exactly one vertex buffer source must be effective: options.bufferViewId, options.spz, or options.draco for options.attributeSemantic.",
            ));
        }

        if has_draco && options.attribute_semantic.is_none() {
            return Err(DeveloperError::new(
                "When options.draco is defined options.attributeSemantic must also be defined.",
            ));
        }

        if has_draco && options.accessor_id.is_none() {
            return Err(DeveloperError::new(
                "When options.draco is defined options.accessorId must also be defined.",
            ));
        }

        if has_draco && options.primitive.is_none() {
            return Err(DeveloperError::new(
                "When options.draco is defined options.primitive must also be defined.",
            ));
        }

        let asynchronous = options.asynchronous.unwrap_or(true);

        Ok(Self {
            inner: Arc::new(Mutex::new(GltfVertexBufferLoaderInner {
                gltf: Some(gltf),
                gltf_resource,
                base_resource,
                buffer_view_id: options.buffer_view_id,
                primitive: options.primitive,
                draco: options.draco,
                spz: options.spz,
                attribute_semantic: options.attribute_semantic,
                accessor_id: options.accessor_id,
                cache_key: options.cache_key,
                asynchronous,
                load_buffer,
                load_typed_array,
                buffer_view_loader: options.buffer_view_loader,
                buffer: None,
                typed_array: None,
                quantization: None,
                decoded_draco_attribute: options.decoded_draco_attribute,
                draco_should_fail: options.draco_should_fail.unwrap_or(false),
                state: ResourceLoaderState::Unloaded,
                is_destroyed: false,
            })),
        })
    }

    /// The cache key of the resource.
    pub fn cache_key(&self) -> Option<String> {
        self.inner.lock().unwrap().cache_key.clone()
    }

    /// Whether resource creation is asynchronous.
    pub fn asynchronous(&self) -> bool {
        self.inner.lock().unwrap().asynchronous
    }

    /// The vertex buffer. Defined when `load_buffer` is true and loading is ready.
    pub fn buffer(&self) -> Option<Buffer> {
        self.inner.lock().unwrap().buffer.clone()
    }

    /// The typed array containing vertex buffer data.
    pub fn typed_array(&self) -> Option<Vec<u8>> {
        self.inner.lock().unwrap().typed_array.clone()
    }

    /// The attribute semantic, e.g. POSITION or NORMAL.
    pub fn attribute_semantic(&self) -> Option<String> {
        self.inner.lock().unwrap().attribute_semantic.clone()
    }

    /// The accessor ID.
    pub fn accessor_id(&self) -> Option<usize> {
        self.inner.lock().unwrap().accessor_id
    }

    /// Information about quantized vertex attribute after Draco decode.
    pub fn quantization(&self) -> Option<QuantizationInfo> {
        self.inner.lock().unwrap().quantization.clone()
    }

    /// The loader's current state.
    pub fn state(&self) -> ResourceLoaderState {
        self.inner.lock().unwrap().state
    }

    /// Whether this loader has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.inner.lock().unwrap().is_destroyed
    }

    /// Unloads the resource.
    pub fn unload(&self) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(mut buf) = inner.buffer.take() {
            let _ = buf.destroy();
        }
        if let Some(loader) = inner.buffer_view_loader.take() {
            loader.unload();
        }
        inner.typed_array = None;
        inner.quantization = None;
        inner.gltf = None;
        inner.primitive = None;
    }

    /// Destroys the loader and all GPU/cached resources.
    pub fn destroy(&self) {
        self.unload();
        self.inner.lock().unwrap().is_destroyed = true;
    }

    /// Asynchronously loads vertex data from buffer view or Draco.
    pub async fn load<F, Fut>(&self, fetch_fn: Option<F>) -> Result<(), RuntimeError>
    where
        F: Fn(String) -> Fut + Clone + 'static,
        Fut: Future<Output = Result<Vec<u8>, String>>,
    {
        {
            let mut inner = self.inner.lock().unwrap();
            if inner.is_destroyed {
                return Ok(());
            }
            inner.state = ResourceLoaderState::Loading;
        }

        let is_draco = {
            let inner = self.inner.lock().unwrap();
            has_draco_compression(
                inner.draco.as_ref(),
                inner.attribute_semantic.as_deref(),
            )
        };

        if is_draco {
            self.load_from_draco().await
        } else {
            self.load_from_buffer_view(fetch_fn).await
        }
    }

    async fn load_from_draco(&self) -> Result<(), RuntimeError> {
        let (should_fail, decoded) = {
            let inner = self.inner.lock().unwrap();
            if inner.is_destroyed {
                return Ok(());
            }
            (inner.draco_should_fail, inner.decoded_draco_attribute.clone())
        };

        if should_fail {
            self.unload();
            self.inner.lock().unwrap().state = ResourceLoaderState::Failed;
            return Err(RuntimeError::new(
                "Failed to load vertex buffer\nFailed to load Draco\nDraco decode failed",
            ));
        }

        let mut inner = self.inner.lock().unwrap();
        if inner.is_destroyed {
            return Ok(());
        }

        if let Some((attr_bytes, quant)) = decoded {
            inner.typed_array = Some(attr_bytes);
            inner.quantization = quant;
        } else {
            inner.typed_array = Some(Vec::new());
        }

        inner.state = ResourceLoaderState::Loaded;
        Ok(())
    }

    async fn load_from_buffer_view<F, Fut>(&self, fetch_fn: Option<F>) -> Result<(), RuntimeError>
    where
        F: Fn(String) -> Fut + Clone + 'static,
        Fut: Future<Output = Result<Vec<u8>, String>>,
    {
        let (gltf, buffer_view_id, gltf_resource, base_resource, existing_loader) = {
            let inner = self.inner.lock().unwrap();
            if inner.is_destroyed {
                return Ok(());
            }
            (
                inner.gltf.clone().unwrap(),
                inner.buffer_view_id.unwrap_or(0),
                inner.gltf_resource.clone(),
                inner.base_resource.clone(),
                inner.buffer_view_loader.clone(),
            )
        };

        let loader = if let Some(loader) = existing_loader {
            loader
        } else {
            let new_loader = GltfBufferViewLoader::new(GltfBufferViewLoaderOptions {
                gltf,
                buffer_view_id,
                gltf_resource: Some(gltf_resource),
                base_resource: Some(base_resource),
                buffer_loader: None,
                cache_key: None,
            })
            .map_err(|e| RuntimeError::new(e.message))?;

            self.inner.lock().unwrap().buffer_view_loader = Some(new_loader.clone());
            new_loader
        };

        if let Err(e) = loader.load(fetch_fn).await {
            self.unload();
            self.inner.lock().unwrap().state = ResourceLoaderState::Failed;
            return Err(RuntimeError::new(format!(
                "Failed to load vertex buffer\nFailed to load buffer view\n{}",
                e.message
            )));
        }

        let mut inner = self.inner.lock().unwrap();
        if inner.is_destroyed {
            return Ok(());
        }

        let bv_bytes = loader.typed_array().unwrap_or_default();
        inner.typed_array = Some(bv_bytes);
        inner.state = ResourceLoaderState::Processing;

        Ok(())
    }

    /// Processes the resource on the render thread until it becomes ready.
    pub fn process(&self) -> bool {
        let mut inner = self.inner.lock().unwrap();
        if inner.is_destroyed {
            return false;
        }

        if inner.state == ResourceLoaderState::Ready {
            return true;
        }

        if inner.state != ResourceLoaderState::Loaded
            && inner.state != ResourceLoaderState::Processing
        {
            return false;
        }

        let typed_array = match &inner.typed_array {
            Some(arr) => arr.clone(),
            None => return false,
        };

        if inner.load_buffer {
            let buffer = Buffer::create_vertex_buffer(BufferOptions {
                size_in_bytes: 0,
                usage: None,
                typed_array: Some(typed_array.clone()),
            });

            match buffer {
                Ok(buf) => {
                    inner.buffer = Some(buf);
                }
                Err(_) => {
                    inner.state = ResourceLoaderState::Failed;
                    return false;
                }
            }
        }

        if !inner.load_typed_array {
            inner.typed_array = None;
        }

        inner.state = ResourceLoaderState::Ready;
        true
    }
}
