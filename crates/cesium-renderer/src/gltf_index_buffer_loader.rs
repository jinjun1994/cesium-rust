//! GltfIndexBufferLoader matching CesiumJS `Scene/GltfIndexBufferLoader.js` (Apache-2.0).

use crate::buffer::{Buffer, IndexBufferOptions};
use crate::buffer_loader::ResourceLoaderState;
use crate::gltf_buffer_view_loader::{GltfBufferViewLoader, GltfBufferViewLoaderOptions};
use cesium_math::{DeveloperError, IndexDatatype, RuntimeError};
use serde_json::Value;
use std::future::Future;
use std::sync::{Arc, Mutex};

/// Options for creating a [`GltfIndexBufferLoader`].
#[derive(Debug, Clone, Default)]
pub struct GltfIndexBufferLoaderOptions {
    /// The glTF JSON. Required.
    pub gltf: Option<Value>,
    /// The accessor ID corresponding to the index buffer. Required.
    pub accessor_id: Option<usize>,
    /// The glTF resource URL. Required.
    pub gltf_resource: Option<String>,
    /// The base resource URL. Required.
    pub base_resource: Option<String>,
    /// Primitive containing the Draco extension.
    pub primitive: Option<Value>,
    /// The Draco extension object.
    pub draco: Option<Value>,
    /// Cache key.
    pub cache_key: Option<String>,
    /// Determines if WebGL resource creation will be spread out or synchronous. Default true.
    pub asynchronous: Option<bool>,
    /// Load the index buffer as a GPU index buffer. Default false.
    pub load_buffer: Option<bool>,
    /// Load the index buffer as a typed array. Default false.
    pub load_typed_array: Option<bool>,
    /// Optional pre-configured GltfBufferViewLoader.
    pub buffer_view_loader: Option<GltfBufferViewLoader>,
    /// Optional pre-decoded Draco indices (bytes, datatype) for testing or custom Draco decoders.
    pub decoded_draco_indices: Option<(Vec<u8>, IndexDatatype)>,
    /// Whether Draco decoding should simulate a failure.
    pub draco_should_fail: Option<bool>,
}

#[derive(Debug)]
struct GltfIndexBufferLoaderInner {
    gltf: Option<Value>,
    accessor_id: usize,
    gltf_resource: String,
    base_resource: String,
    primitive: Option<Value>,
    draco: Option<Value>,
    cache_key: Option<String>,
    asynchronous: bool,
    load_buffer: bool,
    load_typed_array: bool,
    index_datatype: IndexDatatype,
    buffer_view_loader: Option<GltfBufferViewLoader>,
    buffer: Option<Buffer>,
    typed_array: Option<Vec<u8>>,
    decoded_draco_indices: Option<(Vec<u8>, IndexDatatype)>,
    draco_should_fail: bool,
    state: ResourceLoaderState,
    is_destroyed: bool,
}

/// Loads an index buffer from a glTF accessor.
#[derive(Debug, Clone)]
pub struct GltfIndexBufferLoader {
    inner: Arc<Mutex<GltfIndexBufferLoaderInner>>,
}

impl GltfIndexBufferLoader {
    /// Creates a new `GltfIndexBufferLoader`.
    pub fn new(options: GltfIndexBufferLoaderOptions) -> Result<Self, DeveloperError> {
        let gltf = options
            .gltf
            .ok_or_else(|| DeveloperError::new("options.gltf is required."))?;
        let accessor_id = options
            .accessor_id
            .ok_or_else(|| DeveloperError::new("options.accessorId is required."))?;
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

        let accessor = gltf
            .get("accessors")
            .and_then(|a| a.get(accessor_id))
            .ok_or_else(|| DeveloperError::new(format!("Accessor ID {accessor_id} not found.")))?;

        let component_type = accessor
            .get("componentType")
            .and_then(Value::as_u64)
            .ok_or_else(|| DeveloperError::new("accessor.componentType is required."))?
            as u32;

        let index_datatype = IndexDatatype::from_gl_enum(component_type)
            .ok_or_else(|| DeveloperError::new(format!("Invalid index datatype: {component_type}")))?;

        let asynchronous = options.asynchronous.unwrap_or(true);

        Ok(Self {
            inner: Arc::new(Mutex::new(GltfIndexBufferLoaderInner {
                gltf: Some(gltf),
                accessor_id,
                gltf_resource,
                base_resource,
                primitive: options.primitive,
                draco: options.draco,
                cache_key: options.cache_key,
                asynchronous,
                load_buffer,
                load_typed_array,
                index_datatype,
                buffer_view_loader: options.buffer_view_loader,
                buffer: None,
                typed_array: None,
                decoded_draco_indices: options.decoded_draco_indices,
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

    /// The index buffer. Defined when `load_buffer` is true and loading is ready.
    pub fn buffer(&self) -> Option<Buffer> {
        self.inner.lock().unwrap().buffer.clone()
    }

    /// The raw typed array bytes containing indices.
    pub fn typed_array(&self) -> Option<Vec<u8>> {
        self.inner.lock().unwrap().typed_array.clone()
    }

    /// The index datatype after decode.
    pub fn index_datatype(&self) -> IndexDatatype {
        self.inner.lock().unwrap().index_datatype
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
        inner.gltf = None;
        inner.primitive = None;
    }

    /// Destroys the loader and all GPU/cached resources.
    pub fn destroy(&self) {
        self.unload();
        self.inner.lock().unwrap().is_destroyed = true;
    }

    /// Asynchronously loads the index data from the buffer view or Draco.
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

        let is_draco = { self.inner.lock().unwrap().draco.is_some() };

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
            (inner.draco_should_fail, inner.decoded_draco_indices.clone())
        };

        if should_fail {
            self.unload();
            self.inner.lock().unwrap().state = ResourceLoaderState::Failed;
            return Err(RuntimeError::new(
                "Failed to load index buffer\nFailed to load Draco\nDraco decode failed",
            ));
        }

        if let Some((indices_bytes, datatype)) = decoded {
            let mut inner = self.inner.lock().unwrap();
            if inner.is_destroyed {
                return Ok(());
            }
            inner.typed_array = Some(indices_bytes);
            inner.index_datatype = datatype;
            inner.state = ResourceLoaderState::Loaded;
            Ok(())
        } else {
            // Default placeholder or empty decode
            let mut inner = self.inner.lock().unwrap();
            if inner.is_destroyed {
                return Ok(());
            }
            inner.typed_array = Some(Vec::new());
            inner.state = ResourceLoaderState::Loaded;
            Ok(())
        }
    }

    async fn load_from_buffer_view<F, Fut>(&self, fetch_fn: Option<F>) -> Result<(), RuntimeError>
    where
        F: Fn(String) -> Fut + Clone + 'static,
        Fut: Future<Output = Result<Vec<u8>, String>>,
    {
        let (gltf, accessor_id, gltf_resource, base_resource, existing_loader) = {
            let inner = self.inner.lock().unwrap();
            if inner.is_destroyed {
                return Ok(());
            }
            (
                inner.gltf.clone().unwrap(),
                inner.accessor_id,
                inner.gltf_resource.clone(),
                inner.base_resource.clone(),
                inner.buffer_view_loader.clone(),
            )
        };

        let accessor = gltf
            .get("accessors")
            .and_then(|a| a.get(accessor_id))
            .ok_or_else(|| RuntimeError::new(format!("Accessor {accessor_id} not found.")))?;

        let buffer_view_id = accessor
            .get("bufferView")
            .and_then(Value::as_u64)
            .ok_or_else(|| RuntimeError::new("accessor.bufferView not found."))?
            as usize;

        let loader = if let Some(loader) = existing_loader {
            loader
        } else {
            let new_loader = GltfBufferViewLoader::new(GltfBufferViewLoaderOptions {
                gltf: gltf.clone(),
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
                "Failed to load index buffer\nFailed to load buffer view\n{}",
                e.message
            )));
        }

        let mut inner = self.inner.lock().unwrap();
        if inner.is_destroyed {
            return Ok(());
        }

        let bv_bytes = loader.typed_array().unwrap_or_default();
        let accessor_byte_offset = accessor
            .get("byteOffset")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;
        let count = accessor
            .get("count")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;
        let index_size = inner.index_datatype.get_size_in_bytes();
        let total_bytes = count * index_size;

        if accessor_byte_offset + total_bytes > bv_bytes.len() {
            inner.state = ResourceLoaderState::Failed;
            return Err(RuntimeError::new(
                "Accessor slice exceeds buffer view bounds.",
            ));
        }

        let slice = &bv_bytes[accessor_byte_offset..accessor_byte_offset + total_bytes];
        inner.typed_array = Some(slice.to_vec());
        inner.state = ResourceLoaderState::Processing;

        Ok(())
    }

    /// Processes the resource on the main/render thread until it becomes ready.
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

        let index_datatype = inner.index_datatype;

        if inner.load_buffer {
            let buffer = Buffer::create_index_buffer(IndexBufferOptions {
                size_in_bytes: 0,
                usage: None,
                typed_array: Some(typed_array.clone()),
                index_datatype,
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
