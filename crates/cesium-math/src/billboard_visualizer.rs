//! BillboardVisualizer matching CesiumJS `DataSources/BillboardVisualizer.js` (Apache-2.0).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    billboard_collection::Billboard, BoundingSphere, BoundingSphereState,
    Cartesian2, Cartesian3, Color, DeveloperError, Entity, EntityCluster,
    EntityCollection, HeightReference, HorizontalOrigin, JulianDate, ListenerId,
    SplitDirection, VerticalOrigin,
};

#[derive(Clone)]
struct EntityData {
    entity: Entity,
    _texture_value: Option<String>,
}

/// A Visualizer which maps Entity.billboard to a Billboard primitive managed by an EntityCluster.
pub struct BillboardVisualizer {
    cluster: Rc<RefCell<EntityCluster>>,
    entity_collection: Rc<RefCell<EntityCollection>>,
    items: HashMap<String, EntityData>,
    collection_subscription: Option<ListenerId>,
    destroyed: bool,
}

impl BillboardVisualizer {
    /// Creates a new BillboardVisualizer.
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
                if entity.billboard.is_some() && entity.position.is_some() {
                    items.insert(
                        entity.id.clone(),
                        EntityData {
                            entity: entity.clone(),
                            _texture_value: None,
                        },
                    );
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
                if entity.billboard.is_some() && entity.position.is_some() {
                    self.items.entry(entity.id.clone()).or_insert_with(|| EntityData {
                        entity: entity.clone(),
                        _texture_value: None,
                    });
                    if let Some(item) = self.items.get_mut(&entity.id) {
                        item.entity = entity.clone();
                    }
                } else if self.items.remove(&entity.id).is_some() {
                    self.cluster.borrow_mut().remove_billboard(&entity.id);
                }
            }

            // Remove entities that are no longer in the collection
            let removed_ids: Vec<String> = self
                .items
                .keys()
                .filter(|id| !current_ids.contains(id))
                .cloned()
                .collect();
            for id in removed_ids {
                self.items.remove(&id);
                self.cluster.borrow_mut().remove_billboard(&id);
            }
        }

        let mut cluster = self.cluster.borrow_mut();

        for item in self.items.values_mut() {
            let entity = &item.entity;
            let billboard_graphics = match &entity.billboard {
                Some(bg) => bg,
                None => {
                    cluster.remove_billboard(&entity.id);
                    continue;
                }
            };

            let show_prop = billboard_graphics
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

            let image = if show {
                billboard_graphics
                    .image
                    .as_ref()
                    .and_then(|p| p.get_value(Some(time)))
            } else {
                None
            };

            if !show || position.is_none() || image.is_none() {
                cluster.remove_billboard(&entity.id);
                continue;
            }

            let pos = position.unwrap();
            let img = image.unwrap();

            let bb: &mut Billboard = cluster.get_billboard(&entity.id);
            bb.id = Some(entity.id.clone());
            bb.show = true;
            bb.position = pos;
            bb.image = Some(img);
            bb.ready = true;

            bb.color = billboard_graphics
                .color
                .as_ref()
                .and_then(|p| p.get_value(Some(time)))
                .unwrap_or(Color::WHITE);

            bb.eye_offset = billboard_graphics
                .eye_offset
                .as_ref()
                .and_then(|p| p.get_value(Some(time)))
                .unwrap_or(Cartesian3::ZERO);

            bb.height_reference = billboard_graphics
                .height_reference
                .as_ref()
                .and_then(|p| p.get_value(Some(time)))
                .unwrap_or(HeightReference::None);

            bb.pixel_offset = billboard_graphics
                .pixel_offset
                .as_ref()
                .and_then(|p| p.get_value(Some(time)))
                .unwrap_or(Cartesian2::ZERO);

            bb.scale = billboard_graphics
                .scale
                .as_ref()
                .and_then(|p| p.get_value(Some(time)))
                .unwrap_or(1.0);

            bb.rotation = billboard_graphics
                .rotation
                .as_ref()
                .and_then(|p| p.get_value(Some(time)))
                .unwrap_or(0.0);

            bb.aligned_axis = billboard_graphics
                .aligned_axis
                .as_ref()
                .and_then(|p| p.get_value(Some(time)))
                .unwrap_or(Cartesian3::ZERO);

            bb.horizontal_origin = billboard_graphics
                .horizontal_origin
                .as_ref()
                .and_then(|p| p.get_value(Some(time)))
                .unwrap_or(HorizontalOrigin::Center);

            bb.vertical_origin = billboard_graphics
                .vertical_origin
                .as_ref()
                .and_then(|p| p.get_value(Some(time)))
                .unwrap_or(VerticalOrigin::Center);

            bb.width = billboard_graphics
                .width
                .as_ref()
                .and_then(|p| p.get_value(Some(time)));

            bb.height = billboard_graphics
                .height
                .as_ref()
                .and_then(|p| p.get_value(Some(time)));

            bb.scale_by_distance = billboard_graphics
                .scale_by_distance
                .as_ref()
                .and_then(|p| p.get_value(Some(time)));

            bb.translucency_by_distance = billboard_graphics
                .translucency_by_distance
                .as_ref()
                .and_then(|p| p.get_value(Some(time)));

            bb.pixel_offset_scale_by_distance = billboard_graphics
                .pixel_offset_scale_by_distance
                .as_ref()
                .and_then(|p| p.get_value(Some(time)));

            bb.size_in_meters = billboard_graphics
                .size_in_meters
                .as_ref()
                .and_then(|p| p.get_value(Some(time)))
                .unwrap_or(false);

            bb.distance_display_condition = billboard_graphics
                .distance_display_condition
                .as_ref()
                .and_then(|p| p.get_value(Some(time)));

            bb.disable_depth_test_distance = billboard_graphics
                .disable_depth_test_distance
                .as_ref()
                .and_then(|p| p.get_value(Some(time)));

            bb.split_direction = billboard_graphics
                .split_direction
                .as_ref()
                .and_then(|p| p.get_value(Some(time)))
                .unwrap_or(SplitDirection::None);

            bb.image_sub_region = billboard_graphics
                .image_sub_region
                .as_ref()
                .and_then(|p| p.get_value(Some(time)));
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
                cluster.remove_billboard(id);
            }
        }
        self.items.clear();
        self.destroyed = true;
    }
}

impl Drop for BillboardVisualizer {
    fn drop(&mut self) {
        self.destroy();
    }
}

impl std::fmt::Debug for BillboardVisualizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BillboardVisualizer")
            .field("destroyed", &self.destroyed)
            .finish()
    }
}

impl crate::visualizer::Visualizer for BillboardVisualizer {
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
