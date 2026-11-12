//! LabelVisualizer matching CesiumJS `DataSources/LabelVisualizer.js` (Apache-2.0).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    BoundingSphere, BoundingSphereState, Cartesian2, Cartesian3, Color, DeveloperError, Entity,
    EntityCluster, EntityCollection, HeightReference, HorizontalOrigin, JulianDate, LabelStyle,
    ListenerId, VerticalOrigin,
};

const DEFAULT_SCALE: f64 = 1.0;
const DEFAULT_FONT: &str = "30px sans-serif";
const DEFAULT_STYLE: LabelStyle = LabelStyle::Fill;
const DEFAULT_FILL_COLOR: Color = Color::WHITE;
const DEFAULT_OUTLINE_COLOR: Color = Color::BLACK;
const DEFAULT_OUTLINE_WIDTH: f64 = 1.0;
const DEFAULT_SHOW_BACKGROUND: bool = false;
const DEFAULT_BACKGROUND_COLOR: Color = Color::new(0.165, 0.165, 0.165, 0.8);
const DEFAULT_BACKGROUND_PADDING: Cartesian2 = Cartesian2::new(7.0, 5.0);
const DEFAULT_PIXEL_OFFSET: Cartesian2 = Cartesian2::ZERO;
const DEFAULT_EYE_OFFSET: Cartesian3 = Cartesian3::ZERO;
const DEFAULT_HEIGHT_REFERENCE: HeightReference = HeightReference::None;
const DEFAULT_HORIZONTAL_ORIGIN: HorizontalOrigin = HorizontalOrigin::Center;
const DEFAULT_VERTICAL_ORIGIN: VerticalOrigin = VerticalOrigin::Center;

#[derive(Clone)]
struct EntityData {
    entity: Entity,
}

/// A Visualizer which maps Entity.label to a Label primitive.
pub struct LabelVisualizer {
    cluster: Rc<RefCell<EntityCluster>>,
    entity_collection: Rc<RefCell<EntityCollection>>,
    items: HashMap<String, EntityData>,
    collection_subscription: Option<ListenerId>,
    destroyed: bool,
}

impl LabelVisualizer {
    /// Creates a new LabelVisualizer.
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
                if entity.label.is_some() && entity.position.is_some() {
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
        let opt_time = Some(time);

        // Synchronize items with the entity collection
        {
            let col = self.entity_collection.borrow();
            let mut current_ids = Vec::new();
            for entity in col.values() {
                current_ids.push(entity.id.clone());
                if entity.label.is_some() && entity.position.is_some() {
                    self.items
                        .entry(entity.id.clone())
                        .and_modify(|d| d.entity = entity.clone())
                        .or_insert_with(|| EntityData {
                            entity: entity.clone(),
                        });
                }
            }

            // Remove items that are no longer in the entity collection
            let removed_ids: Vec<String> = self
                .items
                .keys()
                .filter(|id| !current_ids.contains(id))
                .cloned()
                .collect();
            for id in removed_ids {
                self.items.remove(&id);
                self.cluster.borrow_mut().remove_label(&id);
            }
        }

        for item in self.items.values_mut() {
            let entity = &item.entity;
            let label_graphics = match &entity.label {
                Some(lg) => lg,
                None => {
                    self.cluster.borrow_mut().remove_label(&entity.id);
                    continue;
                }
            };

            let is_showing = entity.is_showing();
            let is_available = entity.is_available(time);
            let show_prop = label_graphics
                .show
                .as_ref()
                .and_then(|p| p.get_value(opt_time))
                .unwrap_or(true);

            let show = is_showing && is_available && show_prop;

            let position = if show {
                entity.position.as_ref().and_then(|p| p.get_value(opt_time))
            } else {
                None
            };

            let text = if show {
                label_graphics.text.as_ref().and_then(|t| t.get_value(opt_time))
            } else {
                None
            };

            if let (Some(pos), Some(txt)) = (position, text) {
                let mut cluster = self.cluster.borrow_mut();
                let label = cluster.get_label(&entity.id);

                label.position = pos;
                label.text = txt;
                label.show = true;

                label.font = label_graphics
                    .font
                    .as_ref()
                    .and_then(|p| p.get_value(opt_time))
                    .unwrap_or_else(|| DEFAULT_FONT.to_string());

                label.style = label_graphics
                    .style
                    .as_ref()
                    .and_then(|p| p.get_value(opt_time))
                    .unwrap_or(DEFAULT_STYLE);

                label.fill_color = label_graphics
                    .fill_color
                    .as_ref()
                    .and_then(|p| p.get_value(opt_time))
                    .unwrap_or(DEFAULT_FILL_COLOR);

                label.outline_color = label_graphics
                    .outline_color
                    .as_ref()
                    .and_then(|p| p.get_value(opt_time))
                    .unwrap_or(DEFAULT_OUTLINE_COLOR);

                label.outline_width = label_graphics
                    .outline_width
                    .as_ref()
                    .and_then(|p| p.get_value(opt_time))
                    .unwrap_or(DEFAULT_OUTLINE_WIDTH);

                label.show_background = label_graphics
                    .show_background
                    .as_ref()
                    .and_then(|p| p.get_value(opt_time))
                    .unwrap_or(DEFAULT_SHOW_BACKGROUND);

                label.background_color = label_graphics
                    .background_color
                    .as_ref()
                    .and_then(|p| p.get_value(opt_time))
                    .unwrap_or(DEFAULT_BACKGROUND_COLOR);

                label.background_padding = label_graphics
                    .background_padding
                    .as_ref()
                    .and_then(|p| p.get_value(opt_time))
                    .unwrap_or(DEFAULT_BACKGROUND_PADDING);

                label.pixel_offset = label_graphics
                    .pixel_offset
                    .as_ref()
                    .and_then(|p| p.get_value(opt_time))
                    .unwrap_or(DEFAULT_PIXEL_OFFSET);

                label.eye_offset = label_graphics
                    .eye_offset
                    .as_ref()
                    .and_then(|p| p.get_value(opt_time))
                    .unwrap_or(DEFAULT_EYE_OFFSET);

                let height_reference = label_graphics
                    .height_reference
                    .as_ref()
                    .and_then(|p| p.get_value(opt_time))
                    .unwrap_or(DEFAULT_HEIGHT_REFERENCE);
                label.height_reference = height_reference;

                label.horizontal_origin = label_graphics
                    .horizontal_origin
                    .as_ref()
                    .and_then(|p| p.get_value(opt_time))
                    .unwrap_or(DEFAULT_HORIZONTAL_ORIGIN);

                label.vertical_origin = label_graphics
                    .vertical_origin
                    .as_ref()
                    .and_then(|p| p.get_value(opt_time))
                    .unwrap_or(DEFAULT_VERTICAL_ORIGIN);

                label.scale = label_graphics
                    .scale
                    .as_ref()
                    .and_then(|p| p.get_value(opt_time))
                    .unwrap_or(DEFAULT_SCALE);

                label.translucency_by_distance = label_graphics
                    .translucency_by_distance
                    .as_ref()
                    .and_then(|p| p.get_value(opt_time));

                label.pixel_offset_scale_by_distance = label_graphics
                    .pixel_offset_scale_by_distance
                    .as_ref()
                    .and_then(|p| p.get_value(opt_time));

                label.scale_by_distance = label_graphics
                    .scale_by_distance
                    .as_ref()
                    .and_then(|p| p.get_value(opt_time));

                label.distance_display_condition = label_graphics
                    .distance_display_condition
                    .as_ref()
                    .and_then(|p| p.get_value(opt_time));

                label.disable_depth_test_distance = label_graphics
                    .disable_depth_test_distance
                    .as_ref()
                    .and_then(|p| p.get_value(opt_time));

                label.update_clamping();
            } else {
                self.cluster.borrow_mut().remove_label(&entity.id);
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
        if let Some(col) = &cluster.label_collection {
            if let Some(l) = col.labels.iter().find(|l| l.id.as_deref() == Some(&entity.id) && l.show) {
                let center_pos = l.clamped_position.unwrap_or(l.position);
                result.center = center_pos.into();
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
        if let Some(sub) = self.collection_subscription.take() {
            self.entity_collection
                .borrow_mut()
                .collection_changed
                .remove_listener(sub);
        }
        let col = self.entity_collection.borrow();
        let mut cluster = self.cluster.borrow_mut();
        for entity in col.values() {
            cluster.remove_label(&entity.id);
        }
        self.items.clear();
        self.destroyed = true;
    }
}

impl Drop for LabelVisualizer {
    fn drop(&mut self) {
        if !self.destroyed {
            self.destroy();
        }
    }
}

impl std::fmt::Debug for LabelVisualizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LabelVisualizer")
            .field("destroyed", &self.destroyed)
            .finish()
    }
}

impl crate::visualizer::Visualizer for LabelVisualizer {
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
