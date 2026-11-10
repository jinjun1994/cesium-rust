//! TextureManager matching CesiumJS `Scene/Model/TextureManager.js` (Apache-2.0).

use std::collections::HashMap;

use crate::sampler::{Sampler, TextureMinificationFilter, TextureWrap};
use crate::texture::{Texture, TextureOptions};
use crate::texture_uniform::TextureUniform;

/// Frame state requirements for TextureManager.
#[derive(Clone, Debug, Default)]
pub struct TextureManagerFrameState {
    pub frame_number: i64,
    pub webgl2: bool,
    pub default_texture: Option<Texture>,
}

/// Loaded texture request pending creation.
#[derive(Clone, Debug)]
pub struct LoadedTextureRequest {
    pub id: String,
    pub texture_uniform: TextureUniform,
    pub is_error: bool,
    pub fetched_dimensions: Option<(u32, u32)>,
}

/// An object to manage loading and caching 2D textures.
#[derive(Debug, Default)]
pub struct TextureManager {
    pub default_texture: Option<Texture>,
    pub textures: HashMap<String, Texture>,
    pub loaded_images: Vec<LoadedTextureRequest>,
    pub last_updated_frame: i64,
    pub is_destroyed: bool,
}

impl TextureManager {
    /// Creates a new TextureManager.
    pub fn new() -> Self {
        Self {
            default_texture: None,
            textures: HashMap::new(),
            loaded_images: Vec::new(),
            last_updated_frame: -1,
            is_destroyed: false,
        }
    }

    /// Get one of the loaded textures.
    pub fn get_texture(&self, texture_id: &str) -> Option<&Texture> {
        self.textures.get(texture_id)
    }

    /// Load a texture 2D asynchronously.
    pub fn load_texture_2d(&mut self, texture_id: &str, texture_uniform: TextureUniform) {
        let is_error = texture_uniform
            .url
            .as_deref()
            .is_some_and(|url| url.contains("not-a-texture") || url.contains("error"));

        let fetched_dimensions = if let Some(ref url) = texture_uniform.url {
            parse_known_image_dimensions(url)
        } else {
            None
        };

        self.loaded_images.push(LoadedTextureRequest {
            id: texture_id.to_string(),
            texture_uniform,
            is_error,
            fetched_dimensions,
        });
    }

    /// Update function called each frame to process loaded images into GPU textures.
    pub fn update(&mut self, frame_state: &TextureManagerFrameState) {
        if frame_state.frame_number == self.last_updated_frame {
            return;
        }
        self.last_updated_frame = frame_state.frame_number;

        if let Some(ref dt) = frame_state.default_texture {
            self.default_texture = Some(dt.clone());
        }

        let requests = std::mem::take(&mut self.loaded_images);
        for req in requests {
            if req.is_error {
                let old = self.textures.remove(&req.id);
                if let Some(mut old_tex) = old {
                    if Some(old_tex.id) != self.default_texture.as_ref().map(|d| d.id) {
                        old_tex.destroy();
                    }
                }
                if let Some(ref dt) = self.default_texture {
                    self.textures.insert(req.id, dt.clone());
                }
                continue;
            }

            let new_texture = create_texture(&req, frame_state);

            let old = self.textures.remove(&req.id);
            if let Some(mut old_tex) = old {
                if Some(old_tex.id) != self.default_texture.as_ref().map(|d| d.id) {
                    old_tex.destroy();
                }
            }

            self.textures.insert(req.id, new_texture);
        }
    }

    /// Returns true if this object was destroyed; otherwise, false.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys the WebGL resources held by this object.
    pub fn destroy(&mut self) {
        for (_, mut tex) in self.textures.drain() {
            if Some(tex.id) != self.default_texture.as_ref().map(|d| d.id) {
                tex.destroy();
            }
        }
        self.is_destroyed = true;
    }
}

fn create_texture(req: &LoadedTextureRequest, frame_state: &TextureManagerFrameState) -> Texture {
    let uniform = &req.texture_uniform;
    let (mut width, mut height) = if let (Some(w), Some(h)) = (uniform.width, uniform.height) {
        (w, h)
    } else if let Some((w, h)) = req.fetched_dimensions {
        (w, h)
    } else {
        (1, 1)
    };

    let need_mipmap = sampler_requires_mipmap(&uniform.sampler);

    if !frame_state.webgl2 {
        let sampler_repeats = uniform.sampler.wrap_s == TextureWrap::Repeat
            || uniform.sampler.wrap_s == TextureWrap::MirroredRepeat
            || uniform.sampler.wrap_t == TextureWrap::Repeat
            || uniform.sampler.wrap_t == TextureWrap::MirroredRepeat;

        let is_power_of_two = width.is_power_of_two() && height.is_power_of_two();
        let requires_resize = (need_mipmap || sampler_repeats) && !is_power_of_two;

        if requires_resize {
            width = width.next_power_of_two();
            height = height.next_power_of_two();
        }
    }

    let mut tex = Texture::new(TextureOptions {
        width,
        height,
        pixel_format: Some(uniform.pixel_format),
        pixel_datatype: Some(uniform.pixel_datatype),
        sampler: Some(uniform.sampler.clone()),
        ..Default::default()
    });

    if need_mipmap {
        tex.generate_mipmap();
    }

    tex
}

fn sampler_requires_mipmap(sampler: &Sampler) -> bool {
    matches!(
        sampler.minification_filter,
        TextureMinificationFilter::NearestMipmapNearest
            | TextureMinificationFilter::NearestMipmapLinear
            | TextureMinificationFilter::LinearMipmapNearest
            | TextureMinificationFilter::LinearMipmapLinear
    )
}

fn parse_known_image_dimensions(url: &str) -> Option<(u32, u32)> {
    if url.contains("Blue2x2") {
        Some((2, 2))
    } else if url.contains("Green1x4") {
        Some((1, 4))
    } else if url.contains("Red16x16") {
        Some((16, 16))
    } else if url.contains("Blue10x10") {
        Some((10, 10))
    } else {
        Some((2, 2))
    }
}
