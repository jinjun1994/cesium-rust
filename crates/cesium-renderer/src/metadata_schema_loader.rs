//! MetadataSchemaLoader matching CesiumJS `Scene/MetadataSchemaLoader.js` (Apache-2.0).

use crate::buffer_loader::ResourceLoaderState;
use crate::metadata_schema::MetadataSchema;
use cesium_math::{DeveloperError, RuntimeError};
use serde_json::Value;
use std::future::Future;
use std::sync::{Arc, Mutex};

/// Options for creating a [`MetadataSchemaLoader`].
#[derive(Debug, Clone, Default)]
pub struct MetadataSchemaLoaderOptions {
    /// In-memory schema JSON. Mutually exclusive with `resource`.
    pub schema: Option<Value>,
    /// Resource URL or path pointing to external schema JSON. Mutually exclusive with `schema`.
    pub resource: Option<String>,
    /// Cache key for the resource.
    pub cache_key: Option<String>,
}

#[derive(Debug)]
struct MetadataSchemaLoaderInner {
    resource: Option<String>,
    cache_key: Option<String>,
    schema: Option<MetadataSchema>,
    state: ResourceLoaderState,
    is_destroyed: bool,
}

/// A MetadataSchema loader.
#[derive(Debug, Clone)]
pub struct MetadataSchemaLoader {
    inner: Arc<Mutex<MetadataSchemaLoaderInner>>,
}

impl MetadataSchemaLoader {
    /// Creates a new `MetadataSchemaLoader`.
    /// Throws DeveloperError if neither or both `schema` and `resource` are defined.
    pub fn new(options: MetadataSchemaLoaderOptions) -> Result<Self, DeveloperError> {
        let has_schema = options.schema.is_some();
        let has_resource = options.resource.is_some();

        if has_schema == has_resource {
            return Err(DeveloperError::new(
                "One of options.schema and options.resource must be defined.",
            ));
        }

        let schema = if let Some(ref s) = options.schema {
            Some(MetadataSchema::from_json(Some(s))?)
        } else {
            None
        };

        Ok(Self {
            inner: Arc::new(Mutex::new(MetadataSchemaLoaderInner {
                resource: options.resource,
                cache_key: options.cache_key,
                schema,
                state: ResourceLoaderState::Unloaded,
                is_destroyed: false,
            })),
        })
    }

    /// Returns the cache key of the resource.
    pub fn cache_key(&self) -> Option<String> {
        self.inner.lock().unwrap().cache_key.clone()
    }

    /// Returns the metadata schema if loaded.
    pub fn schema(&self) -> Option<MetadataSchema> {
        self.inner.lock().unwrap().schema.clone()
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
        inner.schema = None;
        inner.is_destroyed = true;
    }

    /// Loads the schema.
    pub async fn load<F, Fut>(&self, fetch_fn: Option<F>) -> Result<(), RuntimeError>
    where
        F: FnOnce(String) -> Fut,
        Fut: Future<Output = Result<Value, String>>,
    {
        let (has_schema, resource_url, is_destroyed) = {
            let inner = self.inner.lock().unwrap();
            (
                inner.schema.is_some(),
                inner.resource.clone(),
                inner.is_destroyed,
            )
        };

        if is_destroyed {
            return Ok(());
        }

        if has_schema {
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
                "Failed to load schema: {url}\nNo fetcher provided"
            ))
        })?;

        match fetch(url.clone()).await {
            Ok(json_val) => {
                let mut inner = self.inner.lock().unwrap();
                if inner.is_destroyed {
                    return Ok(());
                }
                match MetadataSchema::from_json(Some(&json_val)) {
                    Ok(parsed_schema) => {
                        inner.schema = Some(parsed_schema);
                        inner.state = ResourceLoaderState::Ready;
                        Ok(())
                    }
                    Err(e) => {
                        inner.state = ResourceLoaderState::Failed;
                        Err(RuntimeError::new(format!(
                            "Failed to load schema: {url}\n{e}"
                        )))
                    }
                }
            }
            Err(err) => {
                let mut inner = self.inner.lock().unwrap();
                if inner.is_destroyed {
                    return Ok(());
                }
                inner.state = ResourceLoaderState::Failed;
                Err(RuntimeError::new(format!(
                    "Failed to load schema: {url}\n{err}"
                )))
            }
        }
    }
}
