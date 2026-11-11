//! GltfTextureLoader matching CesiumJS `Scene/GltfTextureLoader.js` (Apache-2.0).

use crate::buffer_loader::ResourceLoaderState;
use crate::gltf_image_loader::{GltfImageLoader, GltfImageLoaderOptions, GltfLoadedImage};
use crate::sampler::{
    Sampler, SamplerOptions, TextureMagnificationFilter, TextureMinificationFilter, TextureWrap,
};
use crate::texture::{Texture, TextureOptions};
use cesium_math::{resize_image_to_next_power_of_two, DeveloperError, RuntimeError};
use serde_json::Value;
use std::future::Future;
use std::sync::{Arc, Mutex};

pub use crate::supported_image_formats::{SupportedImageFormats, SupportedImageFormatsOptions};

/// Options for creating a [`GltfTextureLoader`].
#[derive(Debug, Clone, Default)]
pub struct GltfTextureLoaderOptions {
    /// The glTF JSON. Required.
    pub gltf: Option<Value>,
    /// The texture info object (e.g. material.emissiveTexture, containing `index`). Required.
    pub texture_info: Option<Value>,
    /// The glTF resource URL. Required.
    pub gltf_resource: Option<String>,
    /// The base resource URL. Required.
    pub base_resource: Option<String>,
    /// Supported image formats. Required.
    pub supported_image_formats: Option<SupportedImageFormats>,
    /// Determines if texture creation is asynchronous. Default true.
    pub asynchronous: Option<bool>,
    /// Cache key.
    pub cache_key: Option<String>,
    /// Optional pre-constructed image loader.
    pub image_loader: Option<GltfImageLoader>,
}

#[derive(Debug)]
struct GltfTextureLoaderInner {
    gltf: Option<Value>,
    texture_info: Option<Value>,
    image_id: usize,
    gltf_resource: String,
    base_resource: String,
    supported_image_formats: SupportedImageFormats,
    asynchronous: bool,
    cache_key: Option<String>,
    image_loader: Option<GltfImageLoader>,
    texture: Option<Texture>,
    sampler: Sampler,
    state: ResourceLoaderState,
    is_destroyed: bool,
}

// SAFETY: Inner fields including Texture are protected by the Mutex within GltfTextureLoader.
unsafe impl Send for GltfTextureLoaderInner {}

/// Loads a texture from glTF texture info.
#[derive(Debug, Clone)]
pub struct GltfTextureLoader {
    inner: Arc<Mutex<GltfTextureLoaderInner>>,
}

fn is_power_of_two(n: u32) -> bool {
    n > 0 && (n & (n - 1)) == 0
}

impl GltfTextureLoader {
    /// Creates a new `GltfTextureLoader`.
    pub fn new(options: GltfTextureLoaderOptions) -> Result<Self, DeveloperError> {
        let gltf = options
            .gltf
            .ok_or_else(|| DeveloperError::new("options.gltf is required."))?;
        let texture_info = options
            .texture_info
            .ok_or_else(|| DeveloperError::new("options.textureInfo is required."))?;
        let gltf_resource = options
            .gltf_resource
            .ok_or_else(|| DeveloperError::new("options.gltfResource is required."))?;
        let base_resource = options
            .base_resource
            .ok_or_else(|| DeveloperError::new("options.baseResource is required."))?;
        let supported_image_formats = options
            .supported_image_formats
            .ok_or_else(|| DeveloperError::new("options.supportedImageFormats is required."))?;

        let texture_index = texture_info
            .get("index")
            .and_then(Value::as_u64)
            .ok_or_else(|| DeveloperError::new("textureInfo.index is required."))?
            as usize;

        let texture_obj = gltf
            .get("textures")
            .and_then(|t| t.get(texture_index))
            .ok_or_else(|| DeveloperError::new(format!("Texture index {texture_index} not found.")))?;

        // Determine imageId (check KHR_texture_basisu extension if basis supported)
        let image_id = if supported_image_formats.basis {
            texture_obj
                .get("extensions")
                .and_then(|ext| ext.get("KHR_texture_basisu"))
                .and_then(|b| b.get("source"))
                .and_then(Value::as_u64)
                .map(|v| v as usize)
                .or_else(|| texture_obj.get("source").and_then(Value::as_u64).map(|v| v as usize))
        } else {
            texture_obj.get("source").and_then(Value::as_u64).map(|v| v as usize)
        };

        let image_id = image_id
            .ok_or_else(|| DeveloperError::new("No image source found for texture."))?;

        let sampler = Self::parse_sampler(&gltf, texture_obj);

        let asynchronous = options.asynchronous.unwrap_or(true);

        Ok(Self {
            inner: Arc::new(Mutex::new(GltfTextureLoaderInner {
                gltf: Some(gltf),
                texture_info: Some(texture_info),
                image_id,
                gltf_resource,
                base_resource,
                supported_image_formats,
                asynchronous,
                cache_key: options.cache_key,
                image_loader: options.image_loader,
                texture: None,
                sampler,
                state: ResourceLoaderState::Unloaded,
                is_destroyed: false,
            })),
        })
    }

    fn parse_sampler(gltf: &Value, texture_obj: &Value) -> Sampler {
        let sampler_id = texture_obj.get("sampler").and_then(Value::as_u64).map(|v| v as usize);
        let sampler_obj = sampler_id
            .and_then(|id| gltf.get("samplers").and_then(|s| s.get(id)));

        if let Some(s) = sampler_obj {
            let mag_filter = s
                .get("magFilter")
                .and_then(Value::as_u64)
                .and_then(|v| TextureMagnificationFilter::from_gl_enum(v as u32));
            let min_filter = s
                .get("minFilter")
                .and_then(Value::as_u64)
                .and_then(|v| TextureMinificationFilter::from_gl_enum(v as u32));
            let wrap_s = s
                .get("wrapS")
                .and_then(Value::as_u64)
                .and_then(|v| TextureWrap::from_gl_enum(v as u32));
            let wrap_t = s
                .get("wrapT")
                .and_then(Value::as_u64)
                .and_then(|v| TextureWrap::from_gl_enum(v as u32));

            Sampler::new(SamplerOptions {
                magnification_filter: mag_filter,
                minification_filter: min_filter,
                wrap_s,
                wrap_t,
                ..Default::default()
            })
            .unwrap_or_default()
        } else {
            Sampler::default()
        }
    }

    /// The image ID being loaded.
    pub fn image_id(&self) -> usize {
        self.inner.lock().unwrap().image_id
    }

    /// Whether texture creation is asynchronous.
    pub fn asynchronous(&self) -> bool {
        self.inner.lock().unwrap().asynchronous
    }

    /// The cache key of the resource.
    pub fn cache_key(&self) -> Option<String> {
        self.inner.lock().unwrap().cache_key.clone()
    }

    /// The loaded texture.
    pub fn texture(&self) -> Option<Texture> {
        self.inner.lock().unwrap().texture.clone()
    }

    /// The current state.
    pub fn state(&self) -> ResourceLoaderState {
        self.inner.lock().unwrap().state
    }

    /// Whether this loader has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.inner.lock().unwrap().is_destroyed
    }

    /// Unloads texture and underlying image resources.
    pub fn unload(&self) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(mut tex) = inner.texture.take() {
            tex.destroy();
        }
        if let Some(loader) = inner.image_loader.take() {
            loader.unload();
        }
        inner.gltf = None;
        inner.texture_info = None;
    }

    /// Destroys the texture loader.
    pub fn destroy(&self) {
        self.unload();
        self.inner.lock().unwrap().is_destroyed = true;
    }

    /// Asynchronously loads the image and prepares the texture.
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

        let (gltf, texture_info, gltf_resource, base_resource, supported, existing_loader) = {
            let inner = self.inner.lock().unwrap();
            if inner.is_destroyed {
                return Ok(());
            }
            (
                inner.gltf.clone().unwrap(),
                inner.texture_info.clone().unwrap(),
                inner.gltf_resource.clone(),
                inner.base_resource.clone(),
                inner.supported_image_formats,
                inner.image_loader.clone(),
            )
        };

        let texture_index = texture_info["index"].as_u64().unwrap_or(0) as usize;
        let texture_obj = &gltf["textures"][texture_index];

        let image_id = if supported.basis {
            texture_obj
                .get("extensions")
                .and_then(|ext| ext.get("KHR_texture_basisu"))
                .and_then(|b| b.get("source"))
                .and_then(Value::as_u64)
                .map(|v| v as usize)
                .or_else(|| texture_obj.get("source").and_then(Value::as_u64).map(|v| v as usize))
        } else {
            texture_obj.get("source").and_then(Value::as_u64).map(|v| v as usize)
        }
        .unwrap_or(0);

        let loader = if let Some(loader) = existing_loader {
            loader
        } else {
            let new_loader = GltfImageLoader::new(GltfImageLoaderOptions {
                gltf: Some(gltf),
                image_id: Some(image_id),
                gltf_resource: Some(gltf_resource),
                base_resource: Some(base_resource),
                ..Default::default()
            })
            .map_err(|e| RuntimeError::new(e.message))?;

            self.inner.lock().unwrap().image_loader = Some(new_loader.clone());
            new_loader
        };

        if let Err(e) = loader.load(fetch_fn).await {
            self.unload();
            self.inner.lock().unwrap().state = ResourceLoaderState::Failed;
            return Err(RuntimeError::new(format!(
                "Failed to load texture\n{}",
                e.message
            )));
        }

        let mut inner = self.inner.lock().unwrap();
        if inner.is_destroyed {
            return Ok(());
        }

        inner.state = ResourceLoaderState::Processing;
        Ok(())
    }

    /// Processes the loaded image into a GPU texture.
    pub fn process(&self) -> bool {
        let mut inner = self.inner.lock().unwrap();
        if inner.is_destroyed {
            return false;
        }

        if inner.state == ResourceLoaderState::Ready {
            return true;
        }

        if inner.state != ResourceLoaderState::Processing {
            return false;
        }

        let image_loader = match &inner.image_loader {
            Some(loader) => loader.clone(),
            None => return false,
        };

        let loaded_image = match image_loader.image() {
            Some(img) => img,
            None => return false,
        };

        let mut sampler = inner.sampler.clone();

        let (final_width, final_height) = match loaded_image {
            GltfLoadedImage::Compressed(compressed) => {
                // If compressed texture without mipmap, but sampler requires mipmap, downgrade to Nearest or Linear
                let has_mips = image_loader.mip_levels().map(|m| !m.is_empty()).unwrap_or(false);
                if !has_mips && sampler.minification_filter.requires_mipmap() {
                    sampler.minification_filter = TextureMinificationFilter::Nearest;
                }
                (compressed.width, compressed.height)
            }
            GltfLoadedImage::Uncompressed(img) => {
                let requires_pot = sampler.wrap_s != TextureWrap::ClampToEdge
                    || sampler.wrap_t != TextureWrap::ClampToEdge
                    || sampler.minification_filter.requires_mipmap();

                let (w, h) = if requires_pot && (!is_power_of_two(img.width) || !is_power_of_two(img.height)) {
                    let resized = resize_image_to_next_power_of_two(img).unwrap();
                    (resized.width, resized.height)
                } else {
                    (img.width, img.height)
                };
                (w, h)
            }
        };

        let texture = Texture::new(TextureOptions {
            width: final_width,
            height: final_height,
            sampler: Some(sampler),
            ..Default::default()
        });

        inner.texture = Some(texture);
        inner.state = ResourceLoaderState::Ready;
        true
    }
}
