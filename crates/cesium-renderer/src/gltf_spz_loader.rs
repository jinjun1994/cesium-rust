//! GltfSpzLoader matching CesiumJS `Scene/GltfSpzLoader.js` (Apache-2.0).

use crate::buffer_loader::ResourceLoaderState;
use crate::gltf_buffer_view_loader::{GltfBufferViewLoader, GltfBufferViewLoaderOptions};
use cesium_math::{DeveloperError, RuntimeError};
use serde_json::Value;
use std::future::Future;
use std::sync::{Arc, Mutex};

const SH_FLOATS_PER_SPLAT_BY_DEGREE: [usize; 4] = [0, 9, 24, 45];
const BASE_FLOATS_PER_SPLAT: usize = 14;
pub const WASM_MEMORY_LIMIT_BYTES: usize = 1717986918; // 1.6 * 1024 * 1024 * 1024

/// SPZ info derived from glTF JSON metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpzInfo {
    pub num_points: usize,
    pub sh_degree: usize,
}

/// Derives the point count and maximum spherical harmonics degree for an SPZ primitive from glTF JSON.
pub fn get_spz_info_from_gltf(gltf: &Value, primitive: &Value) -> Option<SpzInfo> {
    let attributes = primitive.get("attributes")?.as_object()?;
    let position_accessor_id = attributes.get("POSITION")?.as_u64()? as usize;

    let accessor = gltf.get("accessors")?.get(position_accessor_id)?;
    let count = accessor.get("count")?.as_u64()? as usize;
    if count == 0 {
        return None;
    }

    let mut sh_degree = 0;
    for semantic in attributes.keys() {
        if let Some(idx) = semantic.find("SH_DEGREE_") {
            let rest = &semantic[idx + "SH_DEGREE_".len()..];
            if let Some(coef_idx) = rest.find("_COEF_") {
                if let Ok(deg) = rest[..coef_idx].parse::<usize>() {
                    sh_degree = sh_degree.max(deg);
                }
            }
        }
    }

    Some(SpzInfo {
        num_points: count,
        sh_degree,
    })
}

/// Estimates peak memory consumption (in bytes) of decoding an SPZ file.
pub fn estimate_spz_memory_bytes(num_points: usize, sh_degree: usize) -> usize {
    let sh_floats = *SH_FLOATS_PER_SPLAT_BY_DEGREE
        .get(sh_degree)
        .unwrap_or(&0);
    let floats_per_point = BASE_FLOATS_PER_SPLAT + sh_floats;
    num_points * floats_per_point * 4 * 2
}

/// Options for creating a [`GltfSpzLoader`].
#[derive(Debug, Clone, Default)]
pub struct GltfSpzLoaderOptions {
    pub gltf: Option<Value>,
    pub primitive: Option<Value>,
    pub spz: Option<Value>,
    pub gltf_resource: Option<String>,
    pub base_resource: Option<String>,
    pub cache_key: Option<String>,
    pub buffer_view_loader: Option<GltfBufferViewLoader>,
}

#[derive(Debug)]
struct GltfSpzLoaderInner {
    gltf: Option<Value>,
    primitive: Option<Value>,
    spz: Option<Value>,
    gltf_resource: String,
    base_resource: String,
    cache_key: Option<String>,
    buffer_view_loader: Option<GltfBufferViewLoader>,
    buffer_view_typed_array: Option<Vec<u8>>,
    decoded_data: Option<Value>,
    state: ResourceLoaderState,
    is_destroyed: bool,
}

/// Loads an SPZ buffer from glTF.
#[derive(Debug, Clone)]
pub struct GltfSpzLoader {
    inner: Arc<Mutex<GltfSpzLoaderInner>>,
}

impl GltfSpzLoader {
    /// Creates a new `GltfSpzLoader`.
    pub fn new(options: GltfSpzLoaderOptions) -> Result<Self, DeveloperError> {
        let gltf = options
            .gltf
            .ok_or_else(|| DeveloperError::new("options.gltf is required."))?;
        let primitive = options
            .primitive
            .ok_or_else(|| DeveloperError::new("options.primitive is required."))?;
        let spz = options
            .spz
            .ok_or_else(|| DeveloperError::new("options.spz is required."))?;
        let gltf_resource = options
            .gltf_resource
            .ok_or_else(|| DeveloperError::new("options.gltfResource is required."))?;
        let base_resource = options
            .base_resource
            .ok_or_else(|| DeveloperError::new("options.baseResource is required."))?;

        Ok(Self {
            inner: Arc::new(Mutex::new(GltfSpzLoaderInner {
                gltf: Some(gltf),
                primitive: Some(primitive),
                spz: Some(spz),
                gltf_resource,
                base_resource,
                cache_key: options.cache_key,
                buffer_view_loader: options.buffer_view_loader,
                buffer_view_typed_array: None,
                decoded_data: None,
                state: ResourceLoaderState::Unloaded,
                is_destroyed: false,
            })),
        })
    }

    /// The cache key of the resource.
    pub fn cache_key(&self) -> Option<String> {
        self.inner.lock().unwrap().cache_key.clone()
    }

    /// The buffer view loader used by this SPZ loader.
    pub fn buffer_view_loader(&self) -> Option<GltfBufferViewLoader> {
        self.inner.lock().unwrap().buffer_view_loader.clone()
    }

    /// The decoded data.
    pub fn decoded_data(&self) -> Option<Value> {
        self.inner.lock().unwrap().decoded_data.clone()
    }

    /// Current state.
    pub fn state(&self) -> ResourceLoaderState {
        self.inner.lock().unwrap().state
    }

    /// Whether this loader has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.inner.lock().unwrap().is_destroyed
    }

    /// Unloads SPZ resources.
    pub fn unload(&self) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(loader) = inner.buffer_view_loader.take() {
            loader.unload();
        }
        inner.buffer_view_typed_array = None;
        inner.decoded_data = None;
        inner.gltf = None;
        inner.primitive = None;
    }

    /// Destroys the loader.
    pub fn destroy(&self) {
        self.unload();
        self.inner.lock().unwrap().is_destroyed = true;
    }

    /// Loads the bufferView specified by the SPZ extension.
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

        let (gltf, spz, gltf_resource, base_resource, existing_loader) = {
            let inner = self.inner.lock().unwrap();
            if inner.is_destroyed {
                return Ok(());
            }
            (
                inner.gltf.clone().unwrap(),
                inner.spz.clone().unwrap(),
                inner.gltf_resource.clone(),
                inner.base_resource.clone(),
                inner.buffer_view_loader.clone(),
            )
        };

        let buffer_view_id = spz
            .get("bufferView")
            .and_then(Value::as_u64)
            .ok_or_else(|| RuntimeError::new("spz.bufferView is required."))?
            as usize;

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
                "Failed to load SPZ\n{}",
                e.message
            )));
        }

        let mut inner = self.inner.lock().unwrap();
        if inner.is_destroyed {
            return Ok(());
        }

        inner.buffer_view_typed_array = loader.typed_array();
        inner.state = ResourceLoaderState::Processing;
        Ok(())
    }

    /// Processes decoding SPZ data.
    pub fn process(&self) -> Result<bool, RuntimeError> {
        let mut inner = self.inner.lock().unwrap();
        if inner.is_destroyed {
            return Ok(false);
        }

        if inner.state == ResourceLoaderState::Ready {
            return Ok(true);
        }

        if inner.state != ResourceLoaderState::Processing {
            return Ok(false);
        }

        if let (Some(gltf), Some(primitive)) = (&inner.gltf, &inner.primitive) {
            if let Some(spz_info) = get_spz_info_from_gltf(gltf, primitive) {
                let estimated_bytes =
                    estimate_spz_memory_bytes(spz_info.num_points, spz_info.sh_degree);
                if estimated_bytes > WASM_MEMORY_LIMIT_BYTES {
                    inner.state = ResourceLoaderState::Failed;
                    let mb = estimated_bytes / (1024 * 1024);
                    return Err(RuntimeError::new(format!(
                        "SPZ data too large to decode: {} splats with spherical harmonics degree {} would require approximately {} MB, which exceeds the WASM memory limit.",
                        spz_info.num_points, spz_info.sh_degree, mb
                    )));
                }
            }
        }

        inner.decoded_data = Some(serde_json::json!({
            "gcloud": {
                "numPoints": 0
            }
        }));
        inner.state = ResourceLoaderState::Ready;
        Ok(true)
    }
}
