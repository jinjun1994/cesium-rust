//! PointVisualizer matching CesiumJS `DataSources/PointVisualizer.js` (Apache-2.0).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    BoundingSphere, BoundingSphereState, Color, DeveloperError, Entity, EntityCluster,
    EntityCollection, HeightReference, JulianDate, ListenerId, SplitDirection,
};

#[derive(Clone)]
struct EntityData {
    entity: Entity,
}

/// A Visualizer which maps Entity.point to a PointPrimitive (or Billboard if clamped to terrain).
pub struct PointVisualizer {
    cluster: Rc<RefCell<EntityCluster>>,
    entity_collection: Rc<RefCell<EntityCollection>>,
    items: HashMap<String, EntityData>,
    collection_subscription: Option<ListenerId>,
    destroyed: bool,
}

impl PointVisualizer {
    /// Creates a new PointVisualizer.
    pub fn new(
        cluster: Option<Rc<RefCell<EntityCluster>>>,
        entity_collection: Option<Rc<RefCell<EntityCollection>>>,
    ) -> Result<Self, DeveloperError> {
        let cluster = cluster.ok_or_else(|| DeveloperError::new("entityCluster is required."))?;
        let entity_collection = entity_collection
            .ok_or_else(|| DeveloperError::new("entityCollection is required."))?;

        let mut items = HashMap::new();
        {
            let col = entity_collection.borrow();
            for entity in col.values() {
                if entity.point.is_some() && entity.position.is_some() {
                    items.insert(entity.id.clone(), EntityData { entity: entity.clone() });
                }
            }
        }

        let mut visualizer = Self {
            cluster,
            entity_collection: Rc::clone(&entity_collection),
            items,
            collection_subscription: None,
            destroyed: false,
        };

        visualizer.setup_collection_listener();
        Ok(visualizer)
    }

    fn setup_collection_listener(&mut self) {
        let sub = self
            .entity_collection
            .borrow_mut()
            .collection_changed
            .add_listener(|_| {});
        self.collection_subscription = Some(sub);
    }

    /// Updates the primitives created by this visualizer to match their Entity counterpart at the given time.
    pub fn update(&mut self, time: Option<&JulianDate>) -> Result<bool, DeveloperError> {
        let time = time.ok_or_else(|| DeveloperError::new("time is required."))?;

        // Synchronize items with the entity collection
        {
            let col = self.entity_collection.borrow();
            let mut current_ids = Vec::new();
            for entity in col.values() {
                current_ids.push(entity.id.clone());
                if entity.point.is_some() && entity.position.is_some() {
                    self.items.entry(entity.id.clone()).or_insert_with(|| EntityData {
                        entity: entity.clone(),
                    });
                    if let Some(item) = self.items.get_mut(&entity.id) {
                        item.entity = entity.clone();
                    }
                } else if self.items.remove(&entity.id).is_some() {
                    let mut cl = self.cluster.borrow_mut();
                    cl.remove_point(&entity.id);
                    cl.remove_billboard(&entity.id);
                }
            }

            let removed_ids: Vec<String> = self
                .items
                .keys()
                .filter(|id| !current_ids.contains(id))
                .cloned()
                .collect();
            for id in removed_ids {
                self.items.remove(&id);
                let mut cl = self.cluster.borrow_mut();
                cl.remove_point(&id);
                cl.remove_billboard(&id);
            }
        }

        let mut cluster = self.cluster.borrow_mut();

        for item in self.items.values_mut() {
            let entity = &item.entity;
            let point_graphics = match &entity.point {
                Some(p) => p,
                None => {
                    cluster.remove_point(&entity.id);
                    cluster.remove_billboard(&entity.id);
                    continue;
                }
            };

            let show_prop = point_graphics
                .show
                .as_ref()
                .and_then(|p| p.get_value(Some(time)))
                .unwrap_or(true);

            let show = entity.is_showing() && entity.is_available(time) && show_prop;

            let position = if show {
                entity.position.as_ref().and_then(|p| p.get_value(Some(time)))
            } else {
                None
            };

            if !show || position.is_none() {
                cluster.remove_point(&entity.id);
                cluster.remove_billboard(&entity.id);
                continue;
            }

            let pos = position.unwrap();
            let height_reference = point_graphics
                .height_reference
                .as_ref()
                .and_then(|p| p.get_value(Some(time)))
                .unwrap_or(HeightReference::None);

            if height_reference == HeightReference::None {
                cluster.remove_billboard(&entity.id);
                let pt = cluster.get_point(&entity.id);
                pt.id = Some(entity.id.clone());
                pt.show = true;
                pt.position = pos;

                pt.color = point_graphics
                    .color
                    .as_ref()
                    .and_then(|p| p.get_value(Some(time)))
                    .unwrap_or(Color::WHITE);

                pt.outline_color = point_graphics
                    .outline_color
                    .as_ref()
                    .and_then(|p| p.get_value(Some(time)))
                    .unwrap_or(Color::BLACK);

                pt.outline_width = point_graphics
                    .outline_width
                    .as_ref()
                    .and_then(|p| p.get_value(Some(time)))
                    .unwrap_or(0.0);

                pt.pixel_size = point_graphics
                    .pixel_size
                    .as_ref()
                    .and_then(|p| p.get_value(Some(time)))
                    .unwrap_or(1.0);

                pt.scale_by_distance = point_graphics
                    .scale_by_distance
                    .as_ref()
                    .and_then(|p| p.get_value(Some(time)));

                pt.translucency_by_distance = point_graphics
                    .translucency_by_distance
                    .as_ref()
                    .and_then(|p| p.get_value(Some(time)));

                pt.distance_display_condition = point_graphics
                    .distance_display_condition
                    .as_ref()
                    .and_then(|p| p.get_value(Some(time)));

                pt.disable_depth_test_distance = point_graphics
                    .disable_depth_test_distance
                    .as_ref()
                    .and_then(|p| p.get_value(Some(time)))
                    .unwrap_or(0.0);

                pt.split_direction = point_graphics
                    .split_direction
                    .as_ref()
                    .and_then(|p| p.get_value(Some(time)))
                    .unwrap_or(SplitDirection::None);
            } else {
                cluster.remove_point(&entity.id);
                let bb = cluster.get_billboard(&entity.id);
                bb.id = Some(entity.id.clone());
                bb.show = true;
                bb.position = pos;
                bb.height_reference = height_reference;
                bb.ready = true;

                bb.scale_by_distance = point_graphics
                    .scale_by_distance
                    .as_ref()
                    .and_then(|p| p.get_value(Some(time)));

                bb.translucency_by_distance = point_graphics
                    .translucency_by_distance
                    .as_ref()
                    .and_then(|p| p.get_value(Some(time)));

                bb.distance_display_condition = point_graphics
                    .distance_display_condition
                    .as_ref()
                    .and_then(|p| p.get_value(Some(time)));

                bb.disable_depth_test_distance = point_graphics
                    .disable_depth_test_distance
                    .as_ref()
                    .and_then(|p| p.get_value(Some(time)));

                bb.split_direction = point_graphics
                    .split_direction
                    .as_ref()
                    .and_then(|p| p.get_value(Some(time)))
                    .unwrap_or(SplitDirection::None);
            }
        }

        Ok(true)
    }

    /// Computes a bounding sphere which encloses the visualization produced for the specified entity.
    pub fn get_bounding_sphere(
        &self,
        entity: Option<&Entity>,
        result: Option<&mut BoundingSphere>,
    ) -> Result<BoundingSphereState, DeveloperError> {
        let entity = entity.ok_or_else(|| DeveloperError::new("entity is required."))?;
        let result = result.ok_or_else(|| DeveloperError::new("result is required."))?;

        let cluster = self.cluster.borrow();

        // Check point collection
        if let Some(col) = &cluster.point_collection {
            if let Some(pt) = col.point_primitives.iter().find(|p| p.id.as_deref() == Some(&entity.id) && p.show) {
                result.center = pt.position.into();
                result.radius = 0.0;
                return Ok(BoundingSphereState::Done);
            }
        }

        // Check billboard collection (clamped points)
        if let Some(col) = &cluster.billboard_collection {
            if let Some(bb) = col.billboards.iter().find(|b| b.id.as_deref() == Some(&entity.id) && b.show) {
                result.center = bb.position.into();
                result.radius = 0.0;
                return Ok(BoundingSphereState::Done);
            }
        }

        Ok(BoundingSphereState::Failed)
    }

    /// Returns true if this object was destroyed; otherwise, false.
    pub fn is_destroyed(&self) -> bool {
        self.destroyed
    }

    /// Removes and destroys all primitives created by this instance.
    pub fn destroy(&mut self) {
        if self.destroyed {
            return;
        }
        if let Some(sub) = self.collection_subscription {
            self.entity_collection
                .borrow_mut()
                .collection_changed
                .remove_listener(sub);
            self.collection_subscription = None;
        }
        {
            let mut cluster = self.cluster.borrow_mut();
            for id in self.items.keys() {
                cluster.remove_point(id);
                cluster.remove_billboard(id);
            }
        }
        self.items.clear();
        self.destroyed = true;
    }
}

impl Drop for PointVisualizer {
    fn drop(&mut self) {
        self.destroy();
    }
}

impl std::fmt::Debug for PointVisualizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PointVisualizer")
            .field("destroyed", &self.destroyed)
            .finish()
    }
}

impl crate::visualizer::Visualizer for PointVisualizer {
    fn update(&mut self, time: &JulianDate) -> bool {
        self.update(Some(time)).unwrap_or(false)
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
