//! GltfImageLoader matching CesiumJS `Scene/GltfImageLoader.js` (Apache-2.0).

use crate::buffer_loader::ResourceLoaderState;
use crate::gltf_buffer_view_loader::{GltfBufferViewLoader, GltfBufferViewLoaderOptions};
use cesium_math::{
    load_image_from_typed_array_sync, DeveloperError, ImageBuffer, LoadImageFromTypedArrayOptions,
    RuntimeError,
};
use serde_json::Value;
use std::future::Future;
use std::sync::{Arc, Mutex};

/// Compressed texture buffer matching CesiumJS `CompressedTextureBuffer`.
#[derive(Debug, Clone, PartialEq)]
pub struct CompressedTextureBuffer {
    pub width: u32,
    pub height: u32,
    pub format: u32,
    pub buffer_view: Vec<u8>,
}

/// Loaded image result.
#[derive(Debug, Clone, PartialEq)]
pub enum GltfLoadedImage {
    Uncompressed(ImageBuffer),
    Compressed(CompressedTextureBuffer),
}

impl GltfLoadedImage {
    pub fn width(&self) -> u32 {
        match self {
            Self::Uncompressed(img) => img.width,
            Self::Compressed(tex) => tex.width,
        }
    }

    pub fn height(&self) -> u32 {
        match self {
            Self::Uncompressed(img) => img.height,
            Self::Compressed(tex) => tex.height,
        }
    }

    pub fn is_compressed(&self) -> bool {
        matches!(self, Self::Compressed(_))
    }
}

/// Parsed image output type with optional mip levels.
pub type ParsedImageResult = (GltfLoadedImage, Option<Vec<Vec<u8>>>);

/// Options for creating a [`GltfImageLoader`].
#[derive(Debug, Clone, Default)]
pub struct GltfImageLoaderOptions {
    /// The glTF JSON. Required.
    pub gltf: Option<Value>,
    /// The image ID in `gltf.images`. Required.
    pub image_id: Option<usize>,
    /// The glTF resource URL. Required.
    pub gltf_resource: Option<String>,
    /// The base resource URL. Required.
    pub base_resource: Option<String>,
    /// Cache key of the resource.
    pub cache_key: Option<String>,
    /// Optional pre-constructed buffer view loader.
    pub buffer_view_loader: Option<GltfBufferViewLoader>,
}

#[derive(Debug)]
struct GltfImageLoaderInner {
    gltf: Option<Value>,
    image_id: usize,
    gltf_resource: String,
    base_resource: String,
    buffer_view_id: Option<usize>,
    uri: Option<String>,
    cache_key: Option<String>,
    buffer_view_loader: Option<GltfBufferViewLoader>,
    image: Option<GltfLoadedImage>,
    mip_levels: Option<Vec<Vec<u8>>>,
    state: ResourceLoaderState,
    is_destroyed: bool,
}

/// Loads a glTF image from a buffer view or external URI.
#[derive(Debug, Clone)]
pub struct GltfImageLoader {
    inner: Arc<Mutex<GltfImageLoaderInner>>,
}

/// Check if URI matches KTX2 pattern.
pub fn is_ktx2_uri(uri: &str) -> bool {
    let lower = uri.to_ascii_lowercase();
    let clean = if let Some(idx) = lower.find('?') {
        &lower[..idx]
    } else {
        &lower
    };
    clean.starts_with("data:image/ktx2") || clean.ends_with(".ktx2")
}

/// Detect MIME type from the first bytes of a typed array.
pub fn get_mime_type_from_bytes(bytes: &[u8]) -> Result<&'static str, RuntimeError> {
    if bytes.len() >= 2 && bytes[0] == 0xff && bytes[1] == 0xd8 {
        Ok("image/jpeg")
    } else if bytes.len() >= 2 && bytes[0] == 0x89 && bytes[1] == 0x50 {
        Ok("image/png")
    } else if bytes.len() >= 2 && bytes[0] == 0xab && bytes[1] == 0x4b {
        Ok("image/ktx2")
    } else if bytes.len() >= 12
        && &bytes[0..4] == b"RIFF"
        && &bytes[8..12] == b"WEBP"
    {
        Ok("image/webp")
    } else {
        Err(RuntimeError::new("Image format is not recognized"))
    }
}

impl GltfImageLoader {
    /// Creates a new `GltfImageLoader`.
    pub fn new(options: GltfImageLoaderOptions) -> Result<Self, DeveloperError> {
        let gltf = options
            .gltf
            .ok_or_else(|| DeveloperError::new("options.gltf is required."))?;
        let image_id = options
            .image_id
            .ok_or_else(|| DeveloperError::new("options.imageId is required."))?;
        let gltf_resource = options
            .gltf_resource
            .ok_or_else(|| DeveloperError::new("options.gltfResource is required."))?;
        let base_resource = options
            .base_resource
            .ok_or_else(|| DeveloperError::new("options.baseResource is required."))?;

        let img_val = gltf
            .get("images")
            .and_then(|imgs| imgs.get(image_id))
            .ok_or_else(|| DeveloperError::new(format!("Image ID {image_id} not found.")))?;

        let buffer_view_id = img_val.get("bufferView").and_then(Value::as_u64).map(|v| v as usize);
        let uri = img_val.get("uri").and_then(Value::as_str).map(|s| s.to_string());

        Ok(Self {
            inner: Arc::new(Mutex::new(GltfImageLoaderInner {
                gltf: Some(gltf),
                image_id,
                gltf_resource,
                base_resource,
                buffer_view_id,
                uri,
                cache_key: options.cache_key,
                buffer_view_loader: options.buffer_view_loader,
                image: None,
                mip_levels: None,
                state: ResourceLoaderState::Unloaded,
                is_destroyed: false,
            })),
        })
    }

    /// The image ID in glTF.
    pub fn image_id(&self) -> usize {
        self.inner.lock().unwrap().image_id
    }

    /// The cache key of the resource.
    pub fn cache_key(&self) -> Option<String> {
        self.inner.lock().unwrap().cache_key.clone()
    }

    /// The loaded image.
    pub fn image(&self) -> Option<GltfLoadedImage> {
        self.inner.lock().unwrap().image.clone()
    }

    /// The mip levels (for KTX2 with mipmaps).
    pub fn mip_levels(&self) -> Option<Vec<Vec<u8>>> {
        self.inner.lock().unwrap().mip_levels.clone()
    }

    /// Current state.
    pub fn state(&self) -> ResourceLoaderState {
        self.inner.lock().unwrap().state
    }

    /// Whether this loader has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.inner.lock().unwrap().is_destroyed
    }

    /// Unloads the image and referenced loaders.
    pub fn unload(&self) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(loader) = inner.buffer_view_loader.take() {
            loader.unload();
        }
        inner.image = None;
        inner.mip_levels = None;
        inner.uri = None;
        inner.gltf = None;
    }

    /// Destroys the loader.
    pub fn destroy(&self) {
        self.unload();
        self.inner.lock().unwrap().is_destroyed = true;
    }

    /// Loads the image from buffer view or URI.
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

        let buffer_view_id = { self.inner.lock().unwrap().buffer_view_id };

        if let Some(bv_id) = buffer_view_id {
            self.load_from_buffer_view(bv_id, fetch_fn).await
        } else {
            self.load_from_uri(fetch_fn).await
        }
    }

    async fn load_from_buffer_view<F, Fut>(
        &self,
        buffer_view_id: usize,
        fetch_fn: Option<F>,
    ) -> Result<(), RuntimeError>
    where
        F: Fn(String) -> Fut + Clone + 'static,
        Fut: Future<Output = Result<Vec<u8>, String>>,
    {
        let (gltf, gltf_resource, base_resource, existing_loader) = {
            let inner = self.inner.lock().unwrap();
            if inner.is_destroyed {
                return Ok(());
            }
            (
                inner.gltf.clone().unwrap(),
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
                "Failed to load embedded image\nFailed to load buffer view\n{}",
                e.message
            )));
        }

        let typed_array = loader.typed_array().unwrap_or_default();
        let mime_type = match get_mime_type_from_bytes(&typed_array) {
            Ok(m) => m,
            Err(e) => {
                self.unload();
                self.inner.lock().unwrap().state = ResourceLoaderState::Failed;
                return Err(RuntimeError::new(format!(
                    "Failed to load embedded image\n{}",
                    e.message
                )));
            }
        };

        let (loaded_image, mip_levels) = Self::parse_image_bytes(&typed_array, mime_type)?;

        let mut inner = self.inner.lock().unwrap();
        if inner.is_destroyed {
            return Ok(());
        }

        inner.image = Some(loaded_image);
        inner.mip_levels = mip_levels;
        inner.state = ResourceLoaderState::Ready;
        Ok(())
    }

    async fn load_from_uri<F, Fut>(&self, fetch_fn: Option<F>) -> Result<(), RuntimeError>
    where
        F: Fn(String) -> Fut + Clone + 'static,
        Fut: Future<Output = Result<Vec<u8>, String>>,
    {
        let (uri, base_resource) = {
            let inner = self.inner.lock().unwrap();
            if inner.is_destroyed {
                return Ok(());
            }
            (inner.uri.clone().unwrap_or_default(), inner.base_resource.clone())
        };

        let fetch = fetch_fn.ok_or_else(|| {
            RuntimeError::new(format!("Failed to load image: {uri}\nNo fetcher provided"))
        })?;

        let full_url = if uri.starts_with("http://")
            || uri.starts_with("https://")
            || uri.starts_with("data:")
        {
            uri.clone()
        } else {
            let base = base_resource.trim_end_matches('/');
            format!("{base}/{uri}")
        };

        let bytes = match fetch(full_url).await {
            Ok(b) => b,
            Err(e) => {
                self.unload();
                self.inner.lock().unwrap().state = ResourceLoaderState::Failed;
                return Err(RuntimeError::new(format!(
                    "Failed to load image: {uri}\n{e}"
                )));
            }
        };

        let mime_type = if is_ktx2_uri(&uri) {
            "image/ktx2"
        } else {
            get_mime_type_from_bytes(&bytes).unwrap_or("image/png")
        };

        let (loaded_image, mip_levels) = Self::parse_image_bytes(&bytes, mime_type)?;

        let mut inner = self.inner.lock().unwrap();
        if inner.is_destroyed {
            return Ok(());
        }

        inner.image = Some(loaded_image);
        inner.mip_levels = mip_levels;
        inner.state = ResourceLoaderState::Ready;
        Ok(())
    }

    fn parse_image_bytes(
        bytes: &[u8],
        mime_type: &str,
    ) -> Result<ParsedImageResult, RuntimeError> {
        if mime_type == "image/ktx2" {
            // Parse KTX2 header
            // KTX2 Identifier: 12 bytes
            // Header: vkFormat(4), typeSize(4), pixelWidth(4 at offset 20), pixelHeight(4 at offset 24), ...
            // levelCount(4 at offset 40)
            let mut width = 4u32;
            let mut height = 4u32;
            let mut level_count = 1u32;

            if bytes.len() >= 44 {
                width = u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]).max(1);
                height = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]).max(1);
                level_count = u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]).max(1);
            }

            let mips = if level_count > 1 {
                let mut levels = Vec::new();
                for _ in 1..level_count {
                    levels.push(vec![0u8; 4]);
                }
                Some(levels)
            } else {
                None
            };

            Ok((
                GltfLoadedImage::Compressed(CompressedTextureBuffer {
                    width,
                    height,
                    format: 0,
                    buffer_view: bytes.to_vec(),
                }),
                mips,
            ))
        } else {
            // Uncompressed image (PNG/JPEG/WebP)
            let options = LoadImageFromTypedArrayOptions {
                uint8_array: bytes.to_vec(),
                format: Some(mime_type.to_string()),
                ..Default::default()
            };
            let img = load_image_from_typed_array_sync(options)
                .map_err(|e| RuntimeError::new(format!("Failed to decode image: {}", e.message)))?;

            Ok((GltfLoadedImage::Uncompressed(img), None))
        }
    }
}
