//! Cesium3DTilesetVisualizer matching CesiumJS `DataSources/Cesium3DTilesetVisualizer.js` (Apache-2.0).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    BoundingSphere, BoundingSphereState, DeveloperError, Entity, EntityCollection,
    JulianDate, ListenerId, Matrix4,
};

/// Simulated 3D Tileset primitive created by the visualizer.
#[derive(Clone, Debug)]
pub struct Cesium3DTilesetPrimitive {
    pub url: String,
    pub show: bool,
    pub model_matrix: Option<Matrix4>,
    pub maximum_screen_space_error: f64,
    pub bounding_sphere: BoundingSphere,
    pub ready: bool,
}

impl Cesium3DTilesetPrimitive {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            show: true,
            model_matrix: None,
            maximum_screen_space_error: 16.0,
            bounding_sphere: BoundingSphere::default(),
            ready: true,
        }
    }
}

/// Simulated scene for Cesium3DTilesetVisualizer.
#[derive(Clone, Debug, Default)]
pub struct TilesetVisualizerScene {
    pub primitives: Rc<RefCell<Vec<Cesium3DTilesetPrimitive>>>,
}

impl TilesetVisualizerScene {
    pub fn new() -> Self {
        Self {
            primitives: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn primitives(&self) -> Rc<RefCell<Vec<Cesium3DTilesetPrimitive>>> {
        Rc::clone(&self.primitives)
    }
}

struct TilesetData {
    url: String,
    primitive_idx: usize,
}

/// A Visualizer which maps Entity.tileset to a Cesium3DTileset.
pub struct Cesium3DTilesetVisualizer {
    scene: Rc<RefCell<TilesetVisualizerScene>>,
    entity_collection: Rc<RefCell<EntityCollection>>,
    tileset_hash: HashMap<String, TilesetData>,
    collection_subscription: Option<ListenerId>,
    destroyed: bool,
}

impl Cesium3DTilesetVisualizer {
    /// Creates a new Cesium3DTilesetVisualizer.
    pub fn new(
        scene: Option<Rc<RefCell<TilesetVisualizerScene>>>,
        entity_collection: Option<Rc<RefCell<EntityCollection>>>,
    ) -> Result<Self, DeveloperError> {
        let scene = scene.ok_or_else(|| DeveloperError::new("scene is required."))?;
        let entity_collection = entity_collection
            .ok_or_else(|| DeveloperError::new("entityCollection is required."))?;

        let mut visualizer = Self {
            scene,
            entity_collection: Rc::clone(&entity_collection),
            tileset_hash: HashMap::new(),
            collection_subscription: None,
            destroyed: false,
        };

        visualizer.setup_collection_listener();
        Ok(visualizer)
    }

    fn setup_collection_listener(&mut self) {
        let sub = self.entity_collection.borrow_mut().collection_changed.add_listener(|_| {});
        self.collection_subscription = Some(sub);
    }

    /// Updates tileset primitives to match entity definitions at given time.
    pub fn update(&mut self, time: Option<JulianDate>) -> Result<bool, DeveloperError> {
        let time = time.ok_or_else(|| DeveloperError::new("time is required."))?;

        let binding = self.entity_collection.borrow();
        let entities = binding.values();
        let mut primitives = self.scene.borrow().primitives.borrow().clone();

        for entity in entities {
            let tileset_graphics = match &entity.tileset {
                Some(tg) => tg,
                None => continue,
            };

            let show = entity.is_showing()
                && entity.is_available(&time)
                && tileset_graphics
                    .show
                    .as_ref()
                    .and_then(|p| p.get_value(Some(&time)))
                    .unwrap_or(true);

            let uri = tileset_graphics
                .uri
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));

            let model_matrix = if show {
                entity.compute_model_matrix(&time)
            } else {
                None
            };

            let max_sse = tileset_graphics
                .maximum_screen_space_error
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)))
                .unwrap_or(16.0);

            if let Some(uri_str) = uri {
                if let Some(data) = self.tileset_hash.get_mut(&entity.id) {
                    if data.url != uri_str {
                        if data.primitive_idx < primitives.len() {
                            primitives.remove(data.primitive_idx);
                        }
                        let mut prim = Cesium3DTilesetPrimitive::new(&uri_str);
                        prim.show = show;
                        prim.model_matrix = model_matrix;
                        prim.maximum_screen_space_error = max_sse;
                        primitives.push(prim);
                        data.url = uri_str;
                        data.primitive_idx = primitives.len() - 1;
                    } else if data.primitive_idx < primitives.len() {
                        let prim = &mut primitives[data.primitive_idx];
                        prim.show = show;
                        prim.model_matrix = model_matrix;
                        prim.maximum_screen_space_error = max_sse;
                    }
                } else {
                    let mut prim = Cesium3DTilesetPrimitive::new(&uri_str);
                    prim.show = show;
                    prim.model_matrix = model_matrix;
                    prim.maximum_screen_space_error = max_sse;
                    primitives.push(prim);
                    self.tileset_hash.insert(
                        entity.id.clone(),
                        TilesetData {
                            url: uri_str,
                            primitive_idx: primitives.len() - 1,
                        },
                    );
                }
            } else if let Some(data) = self.tileset_hash.remove(&entity.id) {
                if data.primitive_idx < primitives.len() {
                    primitives.remove(data.primitive_idx);
                }
            }
        }

        *self.scene.borrow().primitives.borrow_mut() = primitives;
        Ok(true)
    }

    /// Computes bounding sphere enclosing the visualization.
    pub fn get_bounding_sphere(
        &self,
        entity: Option<&Entity>,
        result: Option<&mut BoundingSphere>,
    ) -> Result<BoundingSphereState, DeveloperError> {
        let entity = entity.ok_or_else(|| DeveloperError::new("entity is required."))?;
        let result = result.ok_or_else(|| DeveloperError::new("result is required."))?;

        if let Some(data) = self.tileset_hash.get(&entity.id) {
            let scene = self.scene.borrow();
            let primitives = scene.primitives.borrow();
            if data.primitive_idx < primitives.len() {
                let prim = &primitives[data.primitive_idx];
                if !prim.ready {
                    return Ok(BoundingSphereState::Pending);
                }
                *result = prim.bounding_sphere;
                return Ok(BoundingSphereState::Done);
            }
        }

        Ok(BoundingSphereState::Failed)
    }

    /// Returns true if this object was destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.destroyed
    }

    /// Destroys this visualizer.
    pub fn destroy(&mut self) {
        if let Some(sub) = self.collection_subscription.take() {
            self.entity_collection.borrow_mut().collection_changed.remove_listener(sub);
        }
        self.tileset_hash.clear();
        self.scene.borrow().primitives.borrow_mut().clear();
        self.destroyed = true;
    }
}

impl Drop for Cesium3DTilesetVisualizer {
    fn drop(&mut self) {
        if !self.destroyed {
            self.destroy();
        }
    }
}

impl std::fmt::Debug for Cesium3DTilesetVisualizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cesium3DTilesetVisualizer")
            .field("destroyed", &self.destroyed)
            .finish()
    }
}

impl crate::visualizer::Visualizer for Cesium3DTilesetVisualizer {
    fn update(&mut self, time: &JulianDate) -> bool {
        self.update(Some(*time)).unwrap_or(false)
    }

    fn get_bounding_sphere(
        &self,
        entity: &Entity,
        result: &mut BoundingSphere,
    ) -> BoundingSphereState {
        self.get_bounding_sphere(Some(entity), Some(result))
            .unwrap_or(BoundingSphereState::Failed)
    }

    fn is_destroyed(&self) -> bool {
        self.is_destroyed()
    }

    fn destroy(&mut self) {
        self.destroy();
    }
}
