//! ResourceLoader matching CesiumJS `Scene/ResourceLoader.js` (Apache-2.0).

use cesium_math::{DeveloperError, RuntimeError};
use std::sync::{Arc, Mutex};

/// Trait defining the interface of a resource loader in the cache.
pub trait ResourceLoader: Send + Sync {
    /// The unique cache key of the resource.
    fn cache_key(&self) -> Result<String, DeveloperError>;

    /// Loads the resource.
    fn load(&self) -> Result<(), DeveloperError>;

    /// Unloads the resource.
    fn unload(&mut self) {}

    /// Processes the resource until it becomes ready. Returns true when ready.
    fn process(&mut self) -> bool {
        false
    }

    /// Whether this loader has been destroyed.
    fn is_destroyed(&self) -> bool;

    /// Destroys the loaded resource.
    fn destroy(&mut self);

    /// Total geometry size in bytes for cache statistics.
    fn geometry_byte_length(&self) -> usize {
        0
    }

    /// Total texture size in bytes for cache statistics.
    fn texture_byte_length(&self) -> usize {
        0
    }
}

/// Base resource loader struct matching CesiumJS `ResourceLoader` base class behavior.
#[derive(Debug, Clone, Default)]
pub struct BaseResourceLoader {
    inner: Arc<Mutex<BaseResourceLoaderInner>>,
}

#[derive(Debug, Default)]
struct BaseResourceLoaderInner {
    cache_key: Option<String>,
    is_destroyed: bool,
}

impl BaseResourceLoader {
    /// Creates a base resource loader without a cache key (throws on cache_key() and load()).
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(BaseResourceLoaderInner {
                cache_key: None,
                is_destroyed: false,
            })),
        }
    }

    /// Creates a base resource loader with a cache key.
    pub fn with_cache_key(cache_key: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(BaseResourceLoaderInner {
                cache_key: Some(cache_key.into()),
                is_destroyed: false,
            })),
        }
    }

    /// Cache key getter, throws DeveloperError if instantiated directly without key.
    pub fn cache_key(&self) -> Result<String, DeveloperError> {
        let inner = self.inner.lock().unwrap();
        inner.cache_key.clone().ok_or_else(|| {
            DeveloperError::new(
                "This type describes an interface and is not intended to be instantiated directly.",
            )
        })
    }

    /// Load method, throws DeveloperError when called directly on base class.
    pub fn load(&self) -> Result<(), DeveloperError> {
        Err(DeveloperError::new(
            "This type describes an interface and is not intended to be instantiated directly.",
        ))
    }

    /// Constructs a RuntimeError from an error message and optional underlying error.
    pub fn get_error(
        &self,
        error_message: Option<&str>,
        error: Option<&str>,
    ) -> Result<RuntimeError, DeveloperError> {
        let msg = error_message
            .ok_or_else(|| DeveloperError::new("errorMessage is required."))?;

        let full_message = match error {
            Some(err) if !err.is_empty() => format!("{msg}\n{err}"),
            _ => msg.to_string(),
        };

        Ok(RuntimeError::new(full_message))
    }

    /// Whether this loader has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.inner.lock().unwrap().is_destroyed
    }

    /// Destroys the loader.
    pub fn destroy(&self) {
        self.inner.lock().unwrap().is_destroyed = true;
    }
}

impl ResourceLoader for BaseResourceLoader {
    fn cache_key(&self) -> Result<String, DeveloperError> {
        BaseResourceLoader::cache_key(self)
    }

    fn load(&self) -> Result<(), DeveloperError> {
        BaseResourceLoader::load(self)
    }

    fn is_destroyed(&self) -> bool {
        BaseResourceLoader::is_destroyed(self)
    }

    fn destroy(&mut self) {
        BaseResourceLoader::destroy(self);
    }
}
