//! ResourceCache matching CesiumJS `Scene/ResourceCache.js` (Apache-2.0).

use crate::buffer_loader::{BufferLoader, BufferLoaderOptions};
use crate::resource_cache_key::{
    IndexBufferCacheKeyOptions, ResourceCacheKey, SchemaCacheKeyOptions, TextureCacheKeyOptions,
    VertexBufferCacheKeyOptions,
};
use crate::resource_cache_statistics::ResourceCacheStatistics;
use crate::resource_loader::{BaseResourceLoader, ResourceLoader};
use cesium_math::DeveloperError;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// A reference-counted cache entry.
pub struct CacheEntry {
    pub reference_count: usize,
    pub resource_loader: Arc<Mutex<dyn ResourceLoader>>,
}

/// Cache for resources shared across 3D Tiles and glTF.
#[derive(Default)]
pub struct ResourceCache {
    cache_entries: HashMap<String, CacheEntry>,
    statistics: ResourceCacheStatistics,
}

static GLOBAL_CACHE: OnceLock<Mutex<ResourceCache>> = OnceLock::new();

fn global_cache() -> &'static Mutex<ResourceCache> {
    GLOBAL_CACHE.get_or_init(|| Mutex::new(ResourceCache::new()))
}

impl ResourceCache {
    /// Constructs a new empty `ResourceCache`.
    pub fn new() -> Self {
        Self {
            cache_entries: HashMap::new(),
            statistics: ResourceCacheStatistics::new(),
        }
    }

    /// Clears global cache for tests.
    pub fn clear_for_specs() {
        let mut cache = global_cache().lock().unwrap();
        cache.clear();
    }

    /// Clears all entries and statistics in this cache.
    pub fn clear(&mut self) {
        for (_, entry) in self.cache_entries.drain() {
            entry.resource_loader.lock().unwrap().destroy();
        }
        self.statistics.clear();
    }

    /// Access reference to cache entries.
    pub fn cache_entries(&self) -> &HashMap<String, CacheEntry> {
        &self.cache_entries
    }

    /// Access reference to cache statistics.
    pub fn statistics(&self) -> &ResourceCacheStatistics {
        &self.statistics
    }

    /// Mutable reference to cache statistics.
    pub fn statistics_mut(&mut self) -> &mut ResourceCacheStatistics {
        &mut self.statistics
    }

    /// Gets a resource from the cache by cache key. Increments reference count if found.
    pub fn get(&mut self, cache_key: &str) -> Option<Arc<Mutex<dyn ResourceLoader>>> {
        if cache_key.is_empty() {
            return None;
        }

        if let Some(entry) = self.cache_entries.get_mut(cache_key) {
            entry.reference_count += 1;
            Some(Arc::clone(&entry.resource_loader))
        } else {
            None
        }
    }

    /// Adds a loader to the cache with an initial reference count of 1.
    pub fn add(
        &mut self,
        loader: Arc<Mutex<dyn ResourceLoader>>,
    ) -> Result<Arc<Mutex<dyn ResourceLoader>>, DeveloperError> {
        let cache_key = loader.lock().unwrap().cache_key()?;
        if cache_key.is_empty() {
            return Err(DeveloperError::new(
                "options.resourceLoader.cacheKey must be defined.",
            ));
        }

        if self.cache_entries.contains_key(&cache_key) {
            return Err(DeveloperError::new(format!(
                "Resource with this cacheKey is already in the cache: {cache_key}"
            )));
        }

        let entry = CacheEntry {
            reference_count: 1,
            resource_loader: Arc::clone(&loader),
        };
        self.cache_entries.insert(cache_key, entry);

        Ok(loader)
    }

    /// Unloads a resource from the cache. When reference count reaches 0, the resource is destroyed and removed.
    pub fn unload(&mut self, loader: &Arc<Mutex<dyn ResourceLoader>>) -> Result<(), DeveloperError> {
        let cache_key = loader.lock().unwrap().cache_key()?;
        self.unload_by_key(&cache_key)
    }

    /// Unloads a resource by cache key.
    pub fn unload_by_key(&mut self, cache_key: &str) -> Result<(), DeveloperError> {
        if cache_key.is_empty() {
            return Err(DeveloperError::new("cacheKey cannot be empty"));
        }

        let should_remove = {
            let entry = self.cache_entries.get_mut(cache_key).ok_or_else(|| {
                DeveloperError::new(format!("Resource is not in the cache: {cache_key}"))
            })?;

            if entry.reference_count == 0 {
                return Err(DeveloperError::new(
                    "Cannot unload resource that has no references.",
                ));
            }

            entry.reference_count -= 1;
            entry.reference_count == 0
        };

        if should_remove {
            if let Some(entry) = self.cache_entries.remove(cache_key) {
                let _ = self.statistics.remove_by_key(cache_key);
                entry.resource_loader.lock().unwrap().destroy();
            }
        }

        Ok(())
    }

    /// Gets or creates a schema loader.
    pub fn get_schema_loader(
        &mut self,
        schema: Option<&Value>,
        resource: Option<&str>,
    ) -> Result<Arc<Mutex<dyn ResourceLoader>>, DeveloperError> {
        let cache_key = ResourceCacheKey::get_schema_cache_key(SchemaCacheKeyOptions {
            schema,
            resource,
        })?;

        if let Some(existing) = self.get(&cache_key) {
            return Ok(existing);
        }

        let loader: Arc<Mutex<dyn ResourceLoader>> =
            Arc::new(Mutex::new(BaseResourceLoader::with_cache_key(cache_key)));
        self.add(loader)
    }

    /// Gets or creates an embedded buffer loader.
    pub fn get_embedded_buffer_loader(
        &mut self,
        parent_resource: Option<&str>,
        buffer_id: Option<usize>,
        typed_array: Option<Vec<u8>>,
    ) -> Result<Arc<Mutex<dyn ResourceLoader>>, DeveloperError> {
        let parent = parent_resource
            .ok_or_else(|| DeveloperError::new("options.parentResource is required."))?;
        let id = buffer_id.ok_or_else(|| DeveloperError::new("options.bufferId is required."))?;

        let cache_key = ResourceCacheKey::get_embedded_buffer_cache_key(parent, id);
        if let Some(existing) = self.get(&cache_key) {
            return Ok(existing);
        }

        let bytes = typed_array
            .ok_or_else(|| DeveloperError::new("options.typedArray is required."))?;

        let loader = BufferLoader::new(BufferLoaderOptions {
            typed_array: Some(bytes),
            resource: None,
            cache_key: Some(cache_key),
        })?;

        self.add(Arc::new(Mutex::new(loader)))
    }

    /// Gets or creates an external buffer loader.
    pub fn get_external_buffer_loader(
        &mut self,
        resource: Option<&str>,
    ) -> Result<Arc<Mutex<dyn ResourceLoader>>, DeveloperError> {
        let res = resource.ok_or_else(|| DeveloperError::new("options.resource is required."))?;

        let cache_key = ResourceCacheKey::get_external_buffer_cache_key(res);
        if let Some(existing) = self.get(&cache_key) {
            return Ok(existing);
        }

        let loader = BufferLoader::new(BufferLoaderOptions {
            typed_array: None,
            resource: Some(res.to_string()),
            cache_key: Some(cache_key),
        })?;

        self.add(Arc::new(Mutex::new(loader)))
    }

    /// Gets or creates a glTF JSON loader.
    pub fn get_gltf_json_loader(
        &mut self,
        gltf_resource: Option<&str>,
        base_resource: Option<&str>,
    ) -> Result<Arc<Mutex<dyn ResourceLoader>>, DeveloperError> {
        let gltf_res = gltf_resource
            .ok_or_else(|| DeveloperError::new("options.gltfResource is required."))?;
        let _base_res = base_resource
            .ok_or_else(|| DeveloperError::new("options.baseResource is required."))?;

        let cache_key = ResourceCacheKey::get_gltf_cache_key(gltf_res);
        if let Some(existing) = self.get(&cache_key) {
            return Ok(existing);
        }

        let loader: Arc<Mutex<dyn ResourceLoader>> =
            Arc::new(Mutex::new(BaseResourceLoader::with_cache_key(cache_key)));
        self.add(loader)
    }

    /// Gets or creates a buffer view loader.
    pub fn get_buffer_view_loader(
        &mut self,
        gltf: Option<&Value>,
        buffer_view_id: Option<usize>,
        gltf_resource: Option<&str>,
        base_resource: Option<&str>,
    ) -> Result<Arc<Mutex<dyn ResourceLoader>>, DeveloperError> {
        let g = gltf.ok_or_else(|| DeveloperError::new("options.gltf is required."))?;
        let bvid = buffer_view_id
            .ok_or_else(|| DeveloperError::new("options.bufferViewId is required."))?;
        let gltf_res = gltf_resource
            .ok_or_else(|| DeveloperError::new("options.gltfResource is required."))?;
        let base_res = base_resource
            .ok_or_else(|| DeveloperError::new("options.baseResource is required."))?;

        let cache_key =
            ResourceCacheKey::get_buffer_view_cache_key(g, bvid, gltf_res, base_res)?;
        if let Some(existing) = self.get(&cache_key) {
            return Ok(existing);
        }

        let loader: Arc<Mutex<dyn ResourceLoader>> =
            Arc::new(Mutex::new(BaseResourceLoader::with_cache_key(cache_key)));
        self.add(loader)
    }

    /// Gets or creates a Draco loader.
    pub fn get_draco_loader(
        &mut self,
        gltf: Option<&Value>,
        primitive: Option<&Value>,
        draco: Option<&Value>,
        gltf_resource: Option<&str>,
        base_resource: Option<&str>,
    ) -> Result<Arc<Mutex<dyn ResourceLoader>>, DeveloperError> {
        let g = gltf.ok_or_else(|| DeveloperError::new("options.gltf is required."))?;
        let _p = primitive.ok_or_else(|| DeveloperError::new("options.primitive is required."))?;
        let d = draco.ok_or_else(|| DeveloperError::new("options.draco is required."))?;
        let gltf_res = gltf_resource
            .ok_or_else(|| DeveloperError::new("options.gltfResource is required."))?;
        let base_res = base_resource
            .ok_or_else(|| DeveloperError::new("options.baseResource is required."))?;

        let cache_key = ResourceCacheKey::get_draco_cache_key(g, d, gltf_res, base_res)?;
        if let Some(existing) = self.get(&cache_key) {
            return Ok(existing);
        }

        let loader: Arc<Mutex<dyn ResourceLoader>> =
            Arc::new(Mutex::new(BaseResourceLoader::with_cache_key(cache_key)));
        self.add(loader)
    }

    /// Gets or creates an SPZ loader.
    pub fn get_spz_loader(
        &mut self,
        gltf: Option<&Value>,
        primitive: Option<&Value>,
        spz: Option<&Value>,
        gltf_resource: Option<&str>,
        base_resource: Option<&str>,
    ) -> Result<Arc<Mutex<dyn ResourceLoader>>, DeveloperError> {
        let g = gltf.ok_or_else(|| DeveloperError::new("options.gltf is required."))?;
        let _p = primitive.ok_or_else(|| DeveloperError::new("options.primitive is required."))?;
        let s = spz.ok_or_else(|| DeveloperError::new("options.spz is required."))?;
        let gltf_res = gltf_resource
            .ok_or_else(|| DeveloperError::new("options.gltfResource is required."))?;
        let base_res = base_resource
            .ok_or_else(|| DeveloperError::new("options.baseResource is required."))?;

        let cache_key = ResourceCacheKey::get_spz_cache_key(g, s, gltf_res, base_res)?;
        if let Some(existing) = self.get(&cache_key) {
            return Ok(existing);
        }

        let loader: Arc<Mutex<dyn ResourceLoader>> =
            Arc::new(Mutex::new(BaseResourceLoader::with_cache_key(cache_key)));
        self.add(loader)
    }

    /// Gets or creates a vertex buffer loader.
    pub fn get_vertex_buffer_loader(
        &mut self,
        options: VertexBufferCacheKeyOptions,
    ) -> Result<Arc<Mutex<dyn ResourceLoader>>, DeveloperError> {
        let cache_key = ResourceCacheKey::get_vertex_buffer_cache_key(options)?;
        if let Some(existing) = self.get(&cache_key) {
            return Ok(existing);
        }

        let loader: Arc<Mutex<dyn ResourceLoader>> =
            Arc::new(Mutex::new(BaseResourceLoader::with_cache_key(cache_key)));
        self.add(loader)
    }

    /// Gets or creates an index buffer loader.
    pub fn get_index_buffer_loader(
        &mut self,
        options: IndexBufferCacheKeyOptions,
    ) -> Result<Arc<Mutex<dyn ResourceLoader>>, DeveloperError> {
        let cache_key = ResourceCacheKey::get_index_buffer_cache_key(options)?;
        if let Some(existing) = self.get(&cache_key) {
            return Ok(existing);
        }

        let loader: Arc<Mutex<dyn ResourceLoader>> =
            Arc::new(Mutex::new(BaseResourceLoader::with_cache_key(cache_key)));
        self.add(loader)
    }

    /// Gets or creates an image loader.
    pub fn get_image_loader(
        &mut self,
        gltf: Option<&Value>,
        image_id: Option<usize>,
        gltf_resource: Option<&str>,
        base_resource: Option<&str>,
    ) -> Result<Arc<Mutex<dyn ResourceLoader>>, DeveloperError> {
        let g = gltf.ok_or_else(|| DeveloperError::new("options.gltf is required."))?;
        let img_id = image_id.ok_or_else(|| DeveloperError::new("options.imageId is required."))?;
        let gltf_res = gltf_resource
            .ok_or_else(|| DeveloperError::new("options.gltfResource is required."))?;
        let base_res = base_resource
            .ok_or_else(|| DeveloperError::new("options.baseResource is required."))?;

        let cache_key = ResourceCacheKey::get_image_cache_key(g, img_id, gltf_res, base_res)?;
        if let Some(existing) = self.get(&cache_key) {
            return Ok(existing);
        }

        let loader: Arc<Mutex<dyn ResourceLoader>> =
            Arc::new(Mutex::new(BaseResourceLoader::with_cache_key(cache_key)));
        self.add(loader)
    }

    /// Gets or creates a texture loader.
    pub fn get_texture_loader(
        &mut self,
        options: TextureCacheKeyOptions,
    ) -> Result<Arc<Mutex<dyn ResourceLoader>>, DeveloperError> {
        let cache_key = ResourceCacheKey::get_texture_cache_key(options)?;
        if let Some(existing) = self.get(&cache_key) {
            return Ok(existing);
        }

        let loader: Arc<Mutex<dyn ResourceLoader>> =
            Arc::new(Mutex::new(BaseResourceLoader::with_cache_key(cache_key)));
        self.add(loader)
    }
}
