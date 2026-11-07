//! Texture and TextureCache matching CesiumJS `Renderer/Texture*.js` (Apache-2.0).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::framebuffer::{PixelDatatype, PixelFormat};
use crate::sampler::Sampler;

static NEXT_TEXTURE_ID: AtomicU64 = AtomicU64::new(1);

impl PixelFormat {
    pub fn components_length(&self) -> usize {
        match self {
            Self::DepthComponent | Self::Alpha | Self::Luminance => 1,
            Self::DepthStencil | Self::LuminanceAlpha => 2,
            Self::Rgb => 3,
            Self::Rgba => 4,
        }
    }
}

/// A 2D GPU texture with lifecycle and caching support.
#[derive(Clone, Debug)]
pub struct Texture {
    pub id: u64,
    pub width: u32,
    pub height: u32,
    pub pixel_format: PixelFormat,
    pub pixel_datatype: PixelDatatype,
    pub sampler: Sampler,
    pub flip_y: bool,
    pub(crate) cached_entry: Option<Weak<RefCell<CachedTextureEntry>>>,
    pub(crate) is_destroyed: bool,
}

impl PartialEq for Texture {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.width == other.width
            && self.height == other.height
            && self.pixel_format == other.pixel_format
            && self.pixel_datatype == other.pixel_datatype
            && self.flip_y == other.flip_y
            && self.is_destroyed == other.is_destroyed
    }
}

#[derive(Clone, Debug)]
pub struct TextureOptions {
    pub width: u32,
    pub height: u32,
    pub pixel_format: Option<PixelFormat>,
    pub pixel_datatype: Option<PixelDatatype>,
    pub sampler: Option<Sampler>,
    pub flip_y: Option<bool>,
}

impl Default for TextureOptions {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            pixel_format: Some(PixelFormat::Rgba),
            pixel_datatype: Some(PixelDatatype::UnsignedByte),
            sampler: None,
            flip_y: Some(false),
        }
    }
}

impl Texture {
    pub fn new(options: TextureOptions) -> Self {
        let id = NEXT_TEXTURE_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            id,
            width: options.width,
            height: options.height,
            pixel_format: options.pixel_format.unwrap_or(PixelFormat::Rgba),
            pixel_datatype: options.pixel_datatype.unwrap_or(PixelDatatype::UnsignedByte),
            sampler: options.sampler.unwrap_or_default(),
            flip_y: options.flip_y.unwrap_or(false),
            cached_entry: None,
            is_destroyed: false,
        }
    }

    pub fn size_in_bytes(&self) -> usize {
        (self.width as usize) * (self.height as usize) * self.pixel_format.components_length()
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        if let Some(ref weak) = self.cached_entry {
            if let Some(entry) = weak.upgrade() {
                let cache_weak = entry.borrow().cache.clone();
                if let Some(cache_rc) = cache_weak.and_then(|c| c.upgrade()) {
                    cache_rc.borrow_mut().release_entry(entry.clone());
                    return;
                }
            }
        }
        self.is_destroyed = true;
    }

    pub(crate) fn final_destroy(&mut self) {
        self.is_destroyed = true;
    }
}

pub struct CachedTextureEntry {
    pub keyword: String,
    pub texture: Rc<RefCell<Texture>>,
    pub count: usize,
    pub(crate) cache: Option<Weak<RefCell<TextureCacheInner>>>,
}

pub struct TextureCacheInner {
    pub textures: HashMap<String, Rc<RefCell<CachedTextureEntry>>>,
    pub textures_to_release: HashMap<String, Rc<RefCell<CachedTextureEntry>>>,
    pub number_of_textures: usize,
    pub is_destroyed: bool,
}

impl TextureCacheInner {
    pub fn release_entry(&mut self, entry: Rc<RefCell<CachedTextureEntry>>) {
        let mut e = entry.borrow_mut();
        if e.count > 0 {
            e.count -= 1;
        }
        if e.count == 0 {
            self.textures_to_release
                .insert(e.keyword.clone(), entry.clone());
        }
    }
}

/// A cache for sharing and recycling GPU textures.
#[derive(Clone)]
pub struct TextureCache {
    inner: Rc<RefCell<TextureCacheInner>>,
}

impl Default for TextureCache {
    fn default() -> Self {
        Self::new()
    }
}

impl TextureCache {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(TextureCacheInner {
                textures: HashMap::new(),
                textures_to_release: HashMap::new(),
                number_of_textures: 0,
                is_destroyed: false,
            })),
        }
    }

    pub fn number_of_textures(&self) -> usize {
        self.inner.borrow().number_of_textures
    }

    pub fn is_destroyed(&self) -> bool {
        self.inner.borrow().is_destroyed
    }

    pub fn add_texture(
        &self,
        keyword: impl Into<String>,
        texture: Rc<RefCell<Texture>>,
    ) {
        let key = keyword.into();
        let mut inner = self.inner.borrow_mut();

        let entry = Rc::new(RefCell::new(CachedTextureEntry {
            keyword: key.clone(),
            texture: texture.clone(),
            count: 1,
            cache: Some(Rc::downgrade(&self.inner)),
        }));

        texture.borrow_mut().cached_entry = Some(Rc::downgrade(&entry));
        inner.textures.insert(key, entry);
        inner.number_of_textures += 1;
    }

    pub fn get_texture(&self, keyword: &str) -> Option<Rc<RefCell<Texture>>> {
        let mut inner = self.inner.borrow_mut();
        if let Some(cached) = inner.textures.get(keyword).cloned() {
            inner.textures_to_release.remove(keyword);
            cached.borrow_mut().count += 1;
            return Some(cached.borrow().texture.clone());
        }
        None
    }

    pub fn destroy_released_textures(&self) {
        let mut inner = self.inner.borrow_mut();
        let to_release: Vec<_> = inner.textures_to_release.values().cloned().collect();
        inner.textures_to_release.clear();

        for entry in to_release {
            let key = entry.borrow().keyword.clone();
            inner.textures.remove(&key);
            entry.borrow().texture.borrow_mut().final_destroy();
            if inner.number_of_textures > 0 {
                inner.number_of_textures -= 1;
            }
        }
    }

    pub fn destroy(&self) {
        let mut inner = self.inner.borrow_mut();
        for entry in inner.textures.values() {
            entry.borrow().texture.borrow_mut().final_destroy();
        }
        inner.textures.clear();
        inner.textures_to_release.clear();
        inner.number_of_textures = 0;
        inner.is_destroyed = true;
    }
}
