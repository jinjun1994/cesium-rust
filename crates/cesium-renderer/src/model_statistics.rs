//! Rendering statistics for a single model matching CesiumJS `Scene/Model/ModelStatistics.js` (Apache-2.0).

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::buffer::Buffer;
use crate::texture::Texture;

/// Trait for objects that can supply buffer identification and byte size to [`ModelStatistics`].
pub trait BufferStatisticsSource {
    fn buffer_id(&self) -> String;
    fn size_in_bytes(&self) -> usize;
}

impl BufferStatisticsSource for Buffer {
    fn buffer_id(&self) -> String {
        self.id.to_string()
    }

    fn size_in_bytes(&self) -> usize {
        self.size_in_bytes
    }
}

impl BufferStatisticsSource for (&str, usize) {
    fn buffer_id(&self) -> String {
        self.0.to_string()
    }

    fn size_in_bytes(&self) -> usize {
        self.1
    }
}

impl BufferStatisticsSource for (String, usize) {
    fn buffer_id(&self) -> String {
        self.0.clone()
    }

    fn size_in_bytes(&self) -> usize {
        self.1
    }
}

/// Helper struct for buffer statistics representation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BufferStatisticsItem {
    pub id: String,
    pub size_in_bytes: usize,
}

impl BufferStatisticsSource for BufferStatisticsItem {
    fn buffer_id(&self) -> String {
        self.id.clone()
    }

    fn size_in_bytes(&self) -> usize {
        self.size_in_bytes
    }
}

/// Trait for objects that can supply texture identification and byte size to [`ModelStatistics`].
pub trait TextureStatisticsSource {
    fn texture_id(&self) -> String;
    fn size_in_bytes(&self) -> usize;
}

impl TextureStatisticsSource for Texture {
    fn texture_id(&self) -> String {
        self.id.to_string()
    }

    fn size_in_bytes(&self) -> usize {
        self.size_in_bytes()
    }
}

impl TextureStatisticsSource for (&str, usize) {
    fn texture_id(&self) -> String {
        self.0.to_string()
    }

    fn size_in_bytes(&self) -> usize {
        self.1
    }
}

impl TextureStatisticsSource for (String, usize) {
    fn texture_id(&self) -> String {
        self.0.clone()
    }

    fn size_in_bytes(&self) -> usize {
        self.1
    }
}

/// Helper struct for texture statistics representation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureStatisticsItem {
    pub id: String,
    pub size_in_bytes: usize,
}

impl TextureStatisticsSource for TextureStatisticsItem {
    fn texture_id(&self) -> String {
        self.id.clone()
    }

    fn size_in_bytes(&self) -> usize {
        self.size_in_bytes
    }
}

/// Trait for objects that dynamically report batch texture memory size.
pub trait BatchTextureStatisticsSource: Send + Sync {
    fn batch_texture_id(&self) -> String;
    fn byte_length(&self) -> usize;
}

/// Dynamic batch texture statistics reporter matching CesiumJS `BatchTexture`.
#[derive(Clone, Debug)]
pub struct DynamicBatchTextureStatistics {
    pub id: String,
    pub batch_texture_bytes: Arc<AtomicUsize>,
    pub pick_texture_bytes: Arc<AtomicUsize>,
}

impl DynamicBatchTextureStatistics {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            batch_texture_bytes: Arc::new(AtomicUsize::new(0)),
            pick_texture_bytes: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn set_batch_texture_size(&self, size: usize) {
        self.batch_texture_bytes.store(size, Ordering::Relaxed);
    }

    pub fn set_pick_texture_size(&self, size: usize) {
        self.pick_texture_bytes.store(size, Ordering::Relaxed);
    }
}

impl BatchTextureStatisticsSource for DynamicBatchTextureStatistics {
    fn batch_texture_id(&self) -> String {
        self.id.clone()
    }

    fn byte_length(&self) -> usize {
        self.batch_texture_bytes.load(Ordering::Relaxed)
            + self.pick_texture_bytes.load(Ordering::Relaxed)
    }
}

/// Rendering statistics for a single model.
#[derive(Clone, Default)]
pub struct ModelStatistics {
    /// Total number of points across all POINTS primitives in this model.
    pub points_length: usize,

    /// Total number of triangles across all TRIANGLES, TRIANGLE_STRIP or TRIANGLE_FAN primitives.
    pub triangles_length: usize,

    /// Total size of all geometry buffers in bytes.
    pub geometry_byte_length: usize,

    /// Total size of all textures in bytes.
    pub textures_byte_length: usize,

    /// Total size of property tables in bytes.
    pub property_tables_byte_length: usize,

    /// Set of buffer IDs that have already been counted.
    pub buffer_id_set: HashSet<String>,

    /// Mapping from texture ID to byte length.
    pub texture_id_byte_lengths: HashMap<String, usize>,

    /// Map of batch textures tracked dynamically.
    pub batch_texture_id_map: HashMap<String, Arc<dyn BatchTextureStatisticsSource>>,
}

impl std::fmt::Debug for ModelStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelStatistics")
            .field("points_length", &self.points_length)
            .field("triangles_length", &self.triangles_length)
            .field("geometry_byte_length", &self.geometry_byte_length)
            .field("textures_byte_length", &self.textures_byte_length)
            .field("property_tables_byte_length", &self.property_tables_byte_length)
            .field("buffer_id_set", &self.buffer_id_set)
            .field("texture_id_byte_lengths", &self.texture_id_byte_lengths)
            .finish()
    }
}

impl ModelStatistics {
    /// Creates an empty statistics container.
    pub fn new() -> Self {
        Self::default()
    }

    /// Total size of batch textures used for picking and styling.
    pub fn batch_textures_byte_length(&self) -> usize {
        self.batch_texture_id_map
            .values()
            .map(|bt| bt.byte_length())
            .sum()
    }

    /// Reset all counters and caches.
    pub fn clear(&mut self) {
        self.points_length = 0;
        self.triangles_length = 0;
        self.geometry_byte_length = 0;
        self.textures_byte_length = 0;
        self.property_tables_byte_length = 0;
        self.buffer_id_set.clear();
        self.texture_id_byte_lengths.clear();
        self.batch_texture_id_map.clear();
    }

    /// Counts the given buffer's memory in bytes, de-duplicating by buffer ID.
    pub fn add_buffer<B: BufferStatisticsSource>(&mut self, buffer: &B, has_cpu_copy: bool) {
        let id = buffer.buffer_id();
        if !self.buffer_id_set.contains(&id) {
            let copies = if has_cpu_copy { 2 } else { 1 };
            self.geometry_byte_length += buffer.size_in_bytes() * copies;
            self.buffer_id_set.insert(id);
        }
    }

    /// Counts the given texture's memory in bytes, de-duplicating by texture ID.
    pub fn add_texture<T: TextureStatisticsSource>(&mut self, texture: &T) {
        let id = texture.texture_id();
        if !self.texture_id_byte_lengths.contains_key(&id) {
            let size = texture.size_in_bytes();
            self.textures_byte_length += size;
            self.texture_id_byte_lengths.insert(id, size);
        }
    }

    /// Returns a list of all recorded texture IDs.
    pub fn get_texture_ids(&self) -> Vec<String> {
        self.texture_id_byte_lengths.keys().cloned().collect()
    }

    /// Returns the length in bytes of the texture with the given ID, if found.
    pub fn get_texture_byte_length_by_id(&self, texture_id: &str) -> Option<usize> {
        self.texture_id_byte_lengths.get(texture_id).copied()
    }

    /// Counts the given batch texture, de-duplicating by its ID.
    pub fn add_batch_texture(&mut self, batch_texture: Arc<dyn BatchTextureStatisticsSource>) {
        let id = batch_texture.batch_texture_id();
        self.batch_texture_id_map.entry(id).or_insert(batch_texture);
    }
}
