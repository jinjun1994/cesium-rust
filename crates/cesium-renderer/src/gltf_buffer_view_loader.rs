//! GltfBufferViewLoader matching CesiumJS `Scene/GltfBufferViewLoader.js` (Apache-2.0).

use crate::buffer_loader::{BufferLoader, BufferLoaderOptions, ResourceLoaderState};
use cesium_math::{DeveloperError, RuntimeError};
use serde_json::Value;
use std::future::Future;
use std::sync::{Arc, Mutex};

/// Meshopt compression extension info.
#[derive(Debug, Clone, PartialEq)]
pub struct MeshoptInfo {
    pub buffer: usize,
    pub byte_offset: usize,
    pub byte_length: usize,
    pub byte_stride: usize,
    pub count: usize,
    pub mode: String,
    pub filter: String,
}

/// Options for creating a [`GltfBufferViewLoader`].
#[derive(Debug, Clone)]
pub struct GltfBufferViewLoaderOptions {
    /// The glTF JSON object. Required.
    pub gltf: Value,
    /// The bufferView index. Required.
    pub buffer_view_id: usize,
    /// The resource URI containing the glTF.
    pub gltf_resource: Option<String>,
    /// The base resource URI that relative paths are resolved against.
    pub base_resource: Option<String>,
    /// An optional pre-constructed BufferLoader.
    pub buffer_loader: Option<BufferLoader>,
    /// Cache key.
    pub cache_key: Option<String>,
}

#[derive(Debug)]
struct GltfBufferViewLoaderInner {
    buffer_loader: Option<BufferLoader>,
    byte_offset: usize,
    byte_length: usize,
    meshopt: Option<MeshoptInfo>,
    typed_array: Option<Vec<u8>>,
    cache_key: Option<String>,
    state: ResourceLoaderState,
    is_destroyed: bool,
}

/// Loads a glTF buffer view.
#[derive(Debug, Clone)]
pub struct GltfBufferViewLoader {
    inner: Arc<Mutex<GltfBufferViewLoaderInner>>,
}

impl GltfBufferViewLoader {
    /// Creates a new `GltfBufferViewLoader`.
    pub fn new(options: GltfBufferViewLoaderOptions) -> Result<Self, DeveloperError> {
        let buffer_views = options
            .gltf
            .get("bufferViews")
            .and_then(Value::as_array)
            .ok_or_else(|| DeveloperError::new("gltf.bufferViews is required."))?;

        let buffer_view = buffer_views
            .get(options.buffer_view_id)
            .ok_or_else(|| DeveloperError::new("bufferViewId out of range."))?;

        let mut buffer_id = buffer_view
            .get("buffer")
            .and_then(Value::as_u64)
            .map(|v| v as usize)
            .unwrap_or(0);

        let mut byte_offset = buffer_view
            .get("byteOffset")
            .and_then(Value::as_u64)
            .map(|v| v as usize)
            .unwrap_or(0);

        let mut byte_length = buffer_view
            .get("byteLength")
            .and_then(Value::as_u64)
            .map(|v| v as usize)
            .unwrap_or(0);

        // Check for meshopt extensions (KHR_meshopt_compression or EXT_meshopt_compression)
        let meshopt_ext = buffer_view
            .get("extensions")
            .and_then(|ext| {
                ext.get("KHR_meshopt_compression")
                    .or_else(|| ext.get("EXT_meshopt_compression"))
            });

        let meshopt = if let Some(ext) = meshopt_ext {
            if let Some(b) = ext.get("buffer").and_then(Value::as_u64) {
                buffer_id = b as usize;
            }
            if let Some(bo) = ext.get("byteOffset").and_then(Value::as_u64) {
                byte_offset = bo as usize;
            } else {
                byte_offset = 0;
            }
            if let Some(bl) = ext.get("byteLength").and_then(Value::as_u64) {
                byte_length = bl as usize;
            }

            Some(MeshoptInfo {
                buffer: buffer_id,
                byte_offset,
                byte_length,
                byte_stride: ext.get("byteStride").and_then(Value::as_u64).unwrap_or(0) as usize,
                count: ext.get("count").and_then(Value::as_u64).unwrap_or(0) as usize,
                mode: ext
                    .get("mode")
                    .and_then(Value::as_str)
                    .unwrap_or("ATTRIBUTES")
                    .to_string(),
                filter: ext
                    .get("filter")
                    .and_then(Value::as_str)
                    .unwrap_or("NONE")
                    .to_string(),
            })
        } else {
            None
        };

        let buffer_loader = if let Some(loader) = options.buffer_loader {
            Some(loader)
        } else {
            // Find buffer definition
            let buffers = options.gltf.get("buffers").and_then(Value::as_array);
            let buffer_def = buffers.and_then(|b| b.get(buffer_id));

            if let Some(b) = buffer_def {
                if let Some(uri) = b.get("uri").and_then(Value::as_str) {
                    let full_resource = if let Some(base) = &options.base_resource {
                        format!("{base}/{uri}")
                    } else {
                        uri.to_string()
                    };
                    Some(BufferLoader::new(BufferLoaderOptions {
                        resource: Some(full_resource),
                        ..Default::default()
                    })?)
                } else {
                    None
                }
            } else {
                None
            }
        };

        Ok(Self {
            inner: Arc::new(Mutex::new(GltfBufferViewLoaderInner {
                buffer_loader,
                byte_offset,
                byte_length,
                meshopt,
                typed_array: None,
                cache_key: options.cache_key,
                state: ResourceLoaderState::Unloaded,
                is_destroyed: false,
            })),
        })
    }

    /// The cache key of the resource.
    pub fn cache_key(&self) -> Option<String> {
        self.inner.lock().unwrap().cache_key.clone()
    }

    /// The typed array containing buffer view data.
    pub fn typed_array(&self) -> Option<Vec<u8>> {
        self.inner.lock().unwrap().typed_array.clone()
    }

    /// The loader's current state.
    pub fn state(&self) -> ResourceLoaderState {
        self.inner.lock().unwrap().state
    }

    /// The meshopt compression info if present.
    pub fn meshopt(&self) -> Option<MeshoptInfo> {
        self.inner.lock().unwrap().meshopt.clone()
    }

    /// Whether this loader has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.inner.lock().unwrap().is_destroyed
    }

    /// Unloads the resource.
    pub fn unload(&self) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(bl) = &inner.buffer_loader {
            bl.destroy();
        }
        inner.buffer_loader = None;
        inner.typed_array = None;
    }

    /// Destroys the loader.
    pub fn destroy(&self) {
        self.unload();
        self.inner.lock().unwrap().is_destroyed = true;
    }

    /// Loads the buffer view.
    pub async fn load<F, Fut>(&self, fetch_fn: Option<F>) -> Result<(), RuntimeError>
    where
        F: FnOnce(String) -> Fut,
        Fut: Future<Output = Result<Vec<u8>, String>>,
    {
        {
            let mut inner = self.inner.lock().unwrap();
            if inner.is_destroyed {
                return Ok(());
            }
            inner.state = ResourceLoaderState::Loading;
        }

        let (buffer_loader, byte_offset, byte_length) = {
            let inner = self.inner.lock().unwrap();
            (
                inner.buffer_loader.clone(),
                inner.byte_offset,
                inner.byte_length,
            )
        };

        let loader = buffer_loader.ok_or_else(|| {
            let mut inner = self.inner.lock().unwrap();
            inner.state = ResourceLoaderState::Failed;
            RuntimeError::new("Failed to load buffer view: BufferLoader is not available.")
        })?;

        if let Err(e) = loader.load(fetch_fn).await {
            let mut inner = self.inner.lock().unwrap();
            inner.state = ResourceLoaderState::Failed;
            return Err(RuntimeError::new(format!(
                "Failed to load buffer view:\n{e}"
            )));
        }

        let full_buffer = loader.typed_array().ok_or_else(|| {
            let mut inner = self.inner.lock().unwrap();
            inner.state = ResourceLoaderState::Failed;
            RuntimeError::new("Failed to load buffer view: Loaded buffer is empty.")
        })?;

        if byte_offset + byte_length > full_buffer.len() {
            let mut inner = self.inner.lock().unwrap();
            inner.state = ResourceLoaderState::Failed;
            return Err(RuntimeError::new(format!(
                "Buffer view slice [{}..{}] exceeds buffer length {}.",
                byte_offset,
                byte_offset + byte_length,
                full_buffer.len()
            )));
        }

        let view_bytes = full_buffer[byte_offset..byte_offset + byte_length].to_vec();

        {
            let mut inner = self.inner.lock().unwrap();
            if inner.is_destroyed {
                return Ok(());
            }
            inner.typed_array = Some(view_bytes);
            inner.state = ResourceLoaderState::Ready;
        }

        Ok(())
    }
}
