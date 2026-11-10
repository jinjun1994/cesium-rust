use std::sync::{Arc, RwLock};
use cesium_math::{DeveloperError, Ray};

use crate::imagery_layer::{ImageryLayer, ImageryLayerFeatureInfo, ImageryProvider};

/// Parameters for feature picking across imagery layers.
#[derive(Debug, Clone, Copy)]
pub struct PickImageryFeaturesOptions {
    pub surface_intersects: bool,
    pub x: u32,
    pub y: u32,
    pub level: u32,
    pub longitude: f64,
    pub latitude: f64,
}

/// An ordered collection of imagery layers for rendering raster imagery on a Globe or 3D Tileset.
#[derive(Default)]
pub struct ImageryLayerCollection {
    layers: Vec<Arc<RwLock<ImageryLayer>>>,
    is_destroyed: bool,
}

impl std::fmt::Debug for ImageryLayerCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageryLayerCollection")
            .field("layers_len", &self.layers.len())
            .field("is_destroyed", &self.is_destroyed)
            .finish()
    }
}

impl ImageryLayerCollection {
    /// Creates a new, empty `ImageryLayerCollection`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the number of layers in this collection.
    pub fn length(&self) -> usize {
        self.layers.len()
    }

    /// Returns `true` if this collection contains no layers.
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// Adds an existing `ImageryLayer` Arc to the collection.
    pub fn add(
        &mut self,
        layer: Arc<RwLock<ImageryLayer>>,
        index: Option<usize>,
    ) -> Result<usize, DeveloperError> {
        self.check_destroyed()?;

        let insert_idx = if let Some(idx) = index {
            if idx > self.layers.len() {
                return Err(DeveloperError::new(
                    "index must be less than or equal to the number of layers.",
                ));
            }
            idx
        } else {
            self.layers.len()
        };

        self.layers.insert(insert_idx, layer);
        self.update();
        Ok(insert_idx)
    }

    /// Helper to wrap an `ImageryLayer` into an Arc and add it.
    pub fn add_layer(
        &mut self,
        layer: ImageryLayer,
        index: Option<usize>,
    ) -> Result<Arc<RwLock<ImageryLayer>>, DeveloperError> {
        let arc = Arc::new(RwLock::new(layer));
        self.add(arc.clone(), index)?;
        Ok(arc)
    }

    /// Creates a new `ImageryLayer` using the given `ImageryProvider` and adds it to the collection.
    pub fn add_imagery_provider(
        &mut self,
        provider: Arc<dyn ImageryProvider>,
        index: Option<usize>,
    ) -> Result<Arc<RwLock<ImageryLayer>>, DeveloperError> {
        self.check_destroyed()?;
        let layer = ImageryLayer::new(Some(provider), None);
        self.add_layer(layer, index)
    }

    /// Removes a layer from the collection.
    pub fn remove(&mut self, layer: &Arc<RwLock<ImageryLayer>>, destroy: bool) -> bool {
        if self.is_destroyed {
            return false;
        }

        let id = layer.read().unwrap().id();
        if let Some(pos) = self.layers.iter().position(|l| l.read().unwrap().id() == id) {
            let removed = self.layers.remove(pos);
            self.update();
            if destroy {
                removed.write().unwrap().destroy();
            }
            true
        } else {
            false
        }
    }

    /// Removes all layers from the collection.
    pub fn remove_all(&mut self, destroy: bool) {
        if self.is_destroyed {
            return;
        }

        let old_layers = std::mem::take(&mut self.layers);
        if destroy {
            for layer in old_layers {
                layer.write().unwrap().destroy();
            }
        }
    }

    /// Checks to see if the collection contains a given layer.
    pub fn contains(&self, layer: &Arc<RwLock<ImageryLayer>>) -> bool {
        let id = layer.read().unwrap().id();
        self.layers.iter().any(|l| l.read().unwrap().id() == id)
    }

    /// Determines the index of a given layer in the collection.
    pub fn index_of(&self, layer: &Arc<RwLock<ImageryLayer>>) -> Option<usize> {
        let id = layer.read().unwrap().id();
        self.layers.iter().position(|l| l.read().unwrap().id() == id)
    }

    /// Gets a layer by index from the collection.
    pub fn get(&self, index: usize) -> Result<Arc<RwLock<ImageryLayer>>, DeveloperError> {
        self.check_destroyed()?;
        self.layers
            .get(index)
            .cloned()
            .ok_or_else(|| DeveloperError::new("index out of range."))
    }

    fn find_index_or_err(&self, layer: &Arc<RwLock<ImageryLayer>>) -> Result<usize, DeveloperError> {
        self.check_destroyed()?;
        self.index_of(layer)
            .ok_or_else(|| DeveloperError::new("layer is not in this collection."))
    }

    /// Raises a layer up one position in the collection.
    pub fn raise(&mut self, layer: &Arc<RwLock<ImageryLayer>>) -> Result<(), DeveloperError> {
        let index = self.find_index_or_err(layer)?;
        if index + 1 < self.layers.len() {
            self.layers.swap(index, index + 1);
            self.update();
        }
        Ok(())
    }

    /// Lowers a layer down one position in the collection.
    pub fn lower(&mut self, layer: &Arc<RwLock<ImageryLayer>>) -> Result<(), DeveloperError> {
        let index = self.find_index_or_err(layer)?;
        if index > 0 {
            self.layers.swap(index, index - 1);
            self.update();
        }
        Ok(())
    }

    /// Raises a layer to the top of the collection.
    pub fn raise_to_top(&mut self, layer: &Arc<RwLock<ImageryLayer>>) -> Result<(), DeveloperError> {
        let index = self.find_index_or_err(layer)?;
        if index + 1 < self.layers.len() {
            let item = self.layers.remove(index);
            self.layers.push(item);
            self.update();
        }
        Ok(())
    }

    /// Lowers a layer to the bottom of the collection.
    pub fn lower_to_bottom(&mut self, layer: &Arc<RwLock<ImageryLayer>>) -> Result<(), DeveloperError> {
        let index = self.find_index_or_err(layer)?;
        if index > 0 {
            let item = self.layers.remove(index);
            self.layers.insert(0, item);
            self.update();
        }
        Ok(())
    }

    /// Updates base layer flags and layer indices.
    pub fn update(&mut self) {
        let mut is_base_layer = true;
        for (i, layer_arc) in self.layers.iter().enumerate() {
            let mut layer = layer_arc.write().unwrap();
            layer.set_layer_index(i);
            if layer.show() {
                layer.set_is_base_layer(is_base_layer);
                is_base_layer = false;
            } else {
                layer.set_is_base_layer(false);
            }
            let show = layer.show();
            layer.set_previous_show(Some(show));
        }
    }

    /// Determines the imagery layers intersected by a pick ray.
    /// Returns layers ordered from top to bottom (highest index to lowest).
    pub fn pick_imagery_layers(
        &self,
        _ray: &Ray,
        surface_intersects: bool,
    ) -> Option<Vec<Arc<RwLock<ImageryLayer>>>> {
        if !surface_intersects || self.layers.is_empty() {
            return None;
        }

        let mut picked = Vec::new();
        for layer_arc in self.layers.iter().rev() {
            let layer = layer_arc.read().unwrap();
            if layer.show() && layer.is_ready() {
                drop(layer);
                picked.push(layer_arc.clone());
            }
        }

        if picked.is_empty() {
            None
        } else {
            Some(picked)
        }
    }

    /// Picks imagery layer features along a ray.
    pub fn pick_imagery_layer_features(
        &self,
        ray: &Ray,
        options: PickImageryFeaturesOptions,
    ) -> Option<Vec<ImageryLayerFeatureInfo>> {
        let layers = self.pick_imagery_layers(ray, options.surface_intersects)?;
        let mut all_features = Vec::new();

        for layer_arc in &layers {
            let layer = layer_arc.read().unwrap();
            if let Some(provider) = layer.imagery_provider() {
                if let Some(features) = provider.pick_features(
                    options.x,
                    options.y,
                    options.level,
                    options.longitude,
                    options.latitude,
                ) {
                    all_features.extend(features);
                }
            }
        }

        if all_features.is_empty() {
            None
        } else {
            Some(all_features)
        }
    }

    fn check_destroyed(&self) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }
        Ok(())
    }

    /// Returns `true` if this collection was destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys this collection and all underlying layers.
    pub fn destroy(&mut self) {
        if !self.is_destroyed {
            self.remove_all(true);
            self.is_destroyed = true;
        }
    }
}
