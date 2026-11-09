use cesium_math::{DeveloperError, RuntimeError};
use std::future::Future;
use std::sync::{Arc, Mutex};

/// State of a resource loader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceLoaderState {
    Unloaded,
    Loading,
    Ready,
    Failed,
}

/// Options for creating a [`BufferLoader`].
#[derive(Debug, Clone, Default)]
pub struct BufferLoaderOptions {
    /// In-memory buffer data. Mutually exclusive with `resource`.
    pub typed_array: Option<Vec<u8>>,
    /// Resource URL or path pointing to external buffer. Mutually exclusive with `typed_array`.
    pub resource: Option<String>,
    /// Cache key for the resource.
    pub cache_key: Option<String>,
}

#[derive(Debug)]
struct BufferLoaderInner {
    typed_array: Option<Vec<u8>>,
    resource: Option<String>,
    cache_key: Option<String>,
    state: ResourceLoaderState,
    is_destroyed: bool,
}

/// Loads an embedded or external buffer.
#[derive(Debug, Clone)]
pub struct BufferLoader {
    inner: Arc<Mutex<BufferLoaderInner>>,
}

impl BufferLoader {
    /// Creates a new `BufferLoader`.
    /// Throws DeveloperError if neither or both `typed_array` and `resource` are defined.
    pub fn new(options: BufferLoaderOptions) -> Result<Self, DeveloperError> {
        let has_typed_array = options.typed_array.is_some();
        let has_resource = options.resource.is_some();

        if has_typed_array == has_resource {
            return Err(DeveloperError::new(
                "One of options.typedArray and options.resource must be defined.",
            ));
        }

        Ok(Self {
            inner: Arc::new(Mutex::new(BufferLoaderInner {
                typed_array: options.typed_array,
                resource: options.resource,
                cache_key: options.cache_key,
                state: ResourceLoaderState::Unloaded,
                is_destroyed: false,
            })),
        })
    }

    /// Returns the cache key of the resource.
    pub fn cache_key(&self) -> Option<String> {
        self.inner.lock().unwrap().cache_key.clone()
    }

    /// Returns a copy or reference to the loaded buffer contents.
    pub fn typed_array(&self) -> Option<Vec<u8>> {
        self.inner.lock().unwrap().typed_array.clone()
    }

    /// Returns the current state of the loader.
    pub fn state(&self) -> ResourceLoaderState {
        self.inner.lock().unwrap().state
    }

    /// Whether this loader has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.inner.lock().unwrap().is_destroyed
    }

    /// Unloads and destroys the loader.
    pub fn destroy(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.typed_array = None;
        inner.is_destroyed = true;
    }

    /// Loads the buffer with an external fetcher function if resource is external.
    pub async fn load<F, Fut>(&self, fetch_fn: Option<F>) -> Result<(), RuntimeError>
    where
        F: FnOnce(String) -> Fut,
        Fut: Future<Output = Result<Vec<u8>, String>>,
    {
        let (has_typed_array, resource_url, is_destroyed) = {
            let inner = self.inner.lock().unwrap();
            (
                inner.typed_array.is_some(),
                inner.resource.clone(),
                inner.is_destroyed,
            )
        };

        if is_destroyed {
            return Ok(());
        }

        if has_typed_array {
            let mut inner = self.inner.lock().unwrap();
            inner.state = ResourceLoaderState::Ready;
            return Ok(());
        }

        let url = resource_url.unwrap();
        {
            let mut inner = self.inner.lock().unwrap();
            inner.state = ResourceLoaderState::Loading;
        }

        let fetch = fetch_fn.ok_or_else(|| {
            RuntimeError::new(format!(
                "Failed to load external buffer: {url}\nNo fetcher provided"
            ))
        })?;

        match fetch(url.clone()).await {
            Ok(bytes) => {
                let mut inner = self.inner.lock().unwrap();
                if inner.is_destroyed {
                    return Ok(());
                }
                inner.typed_array = Some(bytes);
                inner.state = ResourceLoaderState::Ready;
                Ok(())
            }
            Err(err) => {
                let mut inner = self.inner.lock().unwrap();
                if inner.is_destroyed {
                    return Ok(());
                }
                inner.state = ResourceLoaderState::Failed;
                Err(RuntimeError::new(format!(
                    "Failed to load external buffer: {url}\n{err}"
                )))
            }
        }
    }
}
