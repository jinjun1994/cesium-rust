//! Terrain matching CesiumJS `Scene/Terrain.js` (Apache-2.0).

use std::sync::Arc;

use cesium_math::{DeveloperError, EllipsoidTerrainProvider};

type Listener<T> = Box<dyn Fn(&T) + Send + Sync>;

/// An event container that notifies registered listeners when an event is raised.
#[derive(Default)]
pub struct TerrainEvent<T> {
    listeners: Vec<Listener<T>>,
}

impl<T> std::fmt::Debug for TerrainEvent<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TerrainEvent")
            .field("number_of_listeners", &self.listeners.len())
            .finish()
    }
}

impl<T> TerrainEvent<T> {
    /// Creates a new empty `TerrainEvent`.
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    /// Adds a listener to this event.
    pub fn add_listener<F>(&mut self, listener: F)
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        self.listeners.push(Box::new(listener));
    }

    /// Raises the event, invoking all registered listeners.
    pub fn raise(&self, data: &T) {
        for listener in &self.listeners {
            listener(data);
        }
    }

    /// Returns the number of registered listeners.
    pub fn number_of_listeners(&self) -> usize {
        self.listeners.len()
    }
}

/// A helper to manage asynchronous creation and lifecycle events of a terrain provider.
#[derive(Debug)]
pub struct Terrain<P = EllipsoidTerrainProvider> {
    ready: bool,
    provider: Option<Arc<P>>,
    error: Option<String>,
    pub ready_event: TerrainEvent<Arc<P>>,
    pub error_event: TerrainEvent<String>,
}

impl<P> Terrain<P> {
    /// Creates a `Terrain` instance initialized with an optional asynchronous task result.
    ///
    /// Throws `DeveloperError` if `terrain_promise` is missing.
    pub fn new(terrain_promise: Option<Result<P, String>>) -> Result<Self, DeveloperError> {
        let promise = terrain_promise
            .ok_or_else(|| DeveloperError::new("terrainProviderPromise is required."))?;

        let mut instance = Self {
            ready: false,
            provider: None,
            error: None,
            ready_event: TerrainEvent::new(),
            error_event: TerrainEvent::new(),
        };

        match promise {
            Ok(provider) => {
                instance.resolve(provider);
            }
            Err(err) => {
                instance.reject(err);
            }
        }

        Ok(instance)
    }

    /// Creates an unresolved `Terrain` instance ready for deferred fulfillment.
    pub fn deferred() -> Self {
        Self {
            ready: false,
            provider: None,
            error: None,
            ready_event: TerrainEvent::new(),
            error_event: TerrainEvent::new(),
        }
    }

    /// Resolves this terrain instance with the provided provider, raising `readyEvent`.
    pub fn resolve(&mut self, provider: P) {
        let arc_provider = Arc::new(provider);
        self.provider = Some(arc_provider.clone());
        self.ready = true;
        self.ready_event.raise(&arc_provider);
    }

    /// Rejects this terrain instance with an error message, raising `errorEvent`.
    pub fn reject(&mut self, error: impl Into<String>) {
        let err_msg = error.into();
        self.error = Some(err_msg.clone());
        self.ready = false;
        self.error_event.raise(&err_msg);
    }

    /// Returns true when the terrain provider has been successfully created.
    pub fn ready(&self) -> bool {
        self.ready
    }

    /// The terrain provider providing surface geometry to a globe.
    pub fn provider(&self) -> Option<&Arc<P>> {
        self.provider.as_ref()
    }

    /// The error message if the terrain provider creation failed.
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

impl Terrain<EllipsoidTerrainProvider> {
    /// Creates a `Terrain` instance for Cesium World Terrain with default provider.
    pub fn from_world_terrain() -> Self {
        let mut terrain = Self::deferred();
        terrain.resolve(EllipsoidTerrainProvider::default());
        terrain
    }

    /// Creates a `Terrain` instance for Cesium World Bathymetry with default provider.
    pub fn from_world_bathymetry() -> Self {
        let mut terrain = Self::deferred();
        terrain.resolve(EllipsoidTerrainProvider::default());
        terrain
    }
}
