//! ModelVisualizer matching CesiumJS `DataSources/ModelVisualizer.js` (Apache-2.0).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    BoundingSphere, BoundingSphereState, DeveloperError, Entity, EntityCollection,
    JulianDate, ListenerId, Matrix4,
};

/// Simulated Model primitive created by the visualizer.
#[derive(Clone, Debug)]
pub struct ModelPrimitive {
    pub url: String,
    pub show: bool,
    pub model_matrix: Matrix4,
    pub scale: f64,
    pub minimum_pixel_size: f64,
    pub maximum_scale: Option<f64>,
    pub bounding_sphere: BoundingSphere,
    pub ready: bool,
}

impl ModelPrimitive {
    pub fn new(url: impl Into<String>, model_matrix: Matrix4) -> Self {
        Self {
            url: url.into(),
            show: true,
            model_matrix,
            scale: 1.0,
            minimum_pixel_size: 0.0,
            maximum_scale: None,
            bounding_sphere: BoundingSphere::default(),
            ready: true,
        }
    }
}

/// Simulated scene for ModelVisualizer.
#[derive(Clone, Debug, Default)]
pub struct ModelVisualizerScene {
    pub primitives: Rc<RefCell<Vec<ModelPrimitive>>>,
}

impl ModelVisualizerScene {
    pub fn new() -> Self {
        Self {
            primitives: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn primitives(&self) -> Rc<RefCell<Vec<ModelPrimitive>>> {
        Rc::clone(&self.primitives)
    }
}

struct ModelData {
    url: String,
    primitive_idx: usize,
}

/// A Visualizer which maps Entity.model to a Model primitive.
pub struct ModelVisualizer {
    scene: Rc<RefCell<ModelVisualizerScene>>,
    entity_collection: Rc<RefCell<EntityCollection>>,
    model_hash: HashMap<String, ModelData>,
    collection_subscription: Option<ListenerId>,
    destroyed: bool,
}

impl ModelVisualizer {
    /// Creates a new ModelVisualizer.
    pub fn new(
        scene: Option<Rc<RefCell<ModelVisualizerScene>>>,
        entity_collection: Option<Rc<RefCell<EntityCollection>>>,
    ) -> Result<Self, DeveloperError> {
        let scene = scene.ok_or_else(|| DeveloperError::new("scene is required."))?;
        let entity_collection = entity_collection
            .ok_or_else(|| DeveloperError::new("entityCollection is required."))?;

        let mut visualizer = Self {
            scene,
            entity_collection: Rc::clone(&entity_collection),
            model_hash: HashMap::new(),
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

    /// Updates models to match entity definitions at given time.
    pub fn update(&mut self, time: Option<JulianDate>) -> Result<bool, DeveloperError> {
        let time = time.ok_or_else(|| DeveloperError::new("time is required."))?;

        let binding = self.entity_collection.borrow();
        let entities = binding.values();
        let mut primitives = self.scene.borrow().primitives.borrow().clone();

        for entity in entities {
            let model_graphics = match &entity.model {
                Some(mg) => mg,
                None => continue,
            };

            let show = entity.is_showing()
                && entity.is_available(&time)
                && model_graphics
                    .show
                    .as_ref()
                    .and_then(|p| p.get_value(Some(&time)))
                    .unwrap_or(true);

            let uri = model_graphics
                .uri
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));

            let model_matrix = entity.compute_model_matrix(&time);

            // In CesiumJS, if position is undefined, no model is created
            if model_matrix.is_none() || uri.is_none() || !show {
                if let Some(data) = self.model_hash.remove(&entity.id) {
                    if data.primitive_idx < primitives.len() {
                        primitives.remove(data.primitive_idx);
                    }
                }
                continue;
            }

            let uri_str = uri.unwrap();
            let matrix = model_matrix.unwrap();

            let scale = model_graphics
                .scale
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)))
                .unwrap_or(1.0);

            let min_pixel_size = model_graphics
                .minimum_pixel_size
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)))
                .unwrap_or(0.0);

            let max_scale = model_graphics
                .maximum_scale
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));

            if let Some(data) = self.model_hash.get_mut(&entity.id) {
                if data.url != uri_str {
                    if data.primitive_idx < primitives.len() {
                        primitives.remove(data.primitive_idx);
                    }
                    let mut prim = ModelPrimitive::new(&uri_str, matrix);
                    prim.show = show;
                    prim.scale = scale;
                    prim.minimum_pixel_size = min_pixel_size;
                    prim.maximum_scale = max_scale;
                    primitives.push(prim);
                    data.url = uri_str;
                    data.primitive_idx = primitives.len() - 1;
                } else if data.primitive_idx < primitives.len() {
                    let prim = &mut primitives[data.primitive_idx];
                    prim.show = show;
                    prim.model_matrix = matrix;
                    prim.scale = scale;
                    prim.minimum_pixel_size = min_pixel_size;
                    prim.maximum_scale = max_scale;
                }
            } else {
                let mut prim = ModelPrimitive::new(&uri_str, matrix);
                prim.show = show;
                prim.scale = scale;
                prim.minimum_pixel_size = min_pixel_size;
                prim.maximum_scale = max_scale;
                primitives.push(prim);
                self.model_hash.insert(
                    entity.id.clone(),
                    ModelData {
                        url: uri_str,
                        primitive_idx: primitives.len() - 1,
                    },
                );
            }
        }

        *self.scene.borrow().primitives.borrow_mut() = primitives;
        Ok(true)
    }

    /// Computes bounding sphere enclosing the model visualization.
    pub fn get_bounding_sphere(
        &self,
        entity: Option<&Entity>,
        result: Option<&mut BoundingSphere>,
    ) -> Result<BoundingSphereState, DeveloperError> {
        let entity = entity.ok_or_else(|| DeveloperError::new("entity is required."))?;
        let result = result.ok_or_else(|| DeveloperError::new("result is required."))?;

        if let Some(data) = self.model_hash.get(&entity.id) {
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

    /// Returns true if destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.destroyed
    }

    /// Destroys this visualizer.
    pub fn destroy(&mut self) {
        if let Some(sub) = self.collection_subscription.take() {
            self.entity_collection.borrow_mut().collection_changed.remove_listener(sub);
        }
        self.model_hash.clear();
        self.scene.borrow().primitives.borrow_mut().clear();
        self.destroyed = true;
    }
}

impl Drop for ModelVisualizer {
    fn drop(&mut self) {
        if !self.destroyed {
            self.destroy();
        }
    }
}
