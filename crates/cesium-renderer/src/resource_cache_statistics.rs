//! ResourceCacheStatistics matching CesiumJS `Scene/ResourceCacheStatistics.js` (Apache-2.0).

use cesium_math::DeveloperError;
use std::collections::HashMap;

/// Statistics for the GPU and CPU memory used by the models loaded through `ResourceCache`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResourceCacheStatistics {
    /// The size of vertex buffers and index buffers loaded in the cache in bytes.
    pub geometry_byte_length: usize,
    /// The size of all textures loaded in the cache in bytes.
    pub textures_byte_length: usize,
    geometry_sizes: HashMap<String, usize>,
    texture_sizes: HashMap<String, usize>,
}

impl ResourceCacheStatistics {
    /// Constructs a new `ResourceCacheStatistics`.
    pub fn new() -> Self {
        Self {
            geometry_byte_length: 0,
            textures_byte_length: 0,
            geometry_sizes: HashMap::new(),
            texture_sizes: HashMap::new(),
        }
    }

    /// Resets all memory counts and tracked entries.
    pub fn clear(&mut self) {
        self.geometry_byte_length = 0;
        self.textures_byte_length = 0;
        self.geometry_sizes.clear();
        self.texture_sizes.clear();
    }

    /// Read-only map of tracked geometry sizes.
    pub fn geometry_sizes(&self) -> &HashMap<String, usize> {
        &self.geometry_sizes
    }

    /// Read-only map of tracked texture sizes.
    pub fn texture_sizes(&self) -> &HashMap<String, usize> {
        &self.texture_sizes
    }

    /// Track geometry memory for a given cache key and byte length.
    /// If the resource is already tracked, it will not be double-counted.
    pub fn add_geometry(&mut self, cache_key: &str, byte_length: usize) -> Result<(), DeveloperError> {
        if cache_key.is_empty() {
            return Err(DeveloperError::new("cacheKey cannot be empty"));
        }
        if self.geometry_sizes.contains_key(cache_key) {
            return Ok(());
        }
        self.geometry_sizes.insert(cache_key.to_string(), byte_length);
        self.geometry_byte_length += byte_length;
        Ok(())
    }

    /// Track texture memory for a given cache key and byte length.
    /// If the resource is already tracked, it will not be double-counted.
    pub fn add_texture(&mut self, cache_key: &str, byte_length: usize) -> Result<(), DeveloperError> {
        if cache_key.is_empty() {
            return Err(DeveloperError::new("cacheKey cannot be empty"));
        }
        if self.texture_sizes.contains_key(cache_key) {
            return Ok(());
        }
        self.texture_sizes.insert(cache_key.to_string(), byte_length);
        self.textures_byte_length += byte_length;
        Ok(())
    }

    /// Removes a loader's resources by cache key. Gracefully handles un-tracked resources.
    pub fn remove_by_key(&mut self, cache_key: &str) -> Result<(), DeveloperError> {
        if cache_key.is_empty() {
            return Err(DeveloperError::new("cacheKey cannot be empty"));
        }

        if let Some(geom_size) = self.geometry_sizes.remove(cache_key) {
            self.geometry_byte_length = self.geometry_byte_length.saturating_sub(geom_size);
        }

        if let Some(tex_size) = self.texture_sizes.remove(cache_key) {
            self.textures_byte_length = self.textures_byte_length.saturating_sub(tex_size);
        }

        Ok(())
    }
}
