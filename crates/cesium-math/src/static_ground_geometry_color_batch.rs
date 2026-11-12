//! StaticGroundGeometryColorBatch matching CesiumJS `DataSources/StaticGroundGeometryColorBatch.js` (Apache-2.0).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    BoundingSphere, BoundingSphereState, Cartesian3, ClassificationType, Color, Entity,
    JulianDate, MaterialValue, Matrix4, PrimitiveAppearanceType, ShadowMode,
    VisualizerGeometryAttributes, VisualizerGeometryPrimitive,
};
use crate::static_geometry_per_material_batch::BatchGeometryUpdater;

/// Internal sub-batch holding non-overlapping ground geometry entities sharing a color key.
#[derive(Clone, Debug)]
pub struct GroundColorSubBatch {
    pub color_key: [u8; 4],
    pub entities: Vec<(Entity, BoundingSphere)>,
}

fn bounding_spheres_overlap(a: &BoundingSphere, b: &BoundingSphere) -> bool {
    let diff = a.center - b.center;
    let dist_sq = diff.length_squared();
    let sum_r = a.radius + b.radius;
    dist_sq <= sum_r * sum_r
}

fn entity_bounding_sphere(entity: &Entity, time: &JulianDate) -> BoundingSphere {
    let center = entity
        .position
        .as_ref()
        .and_then(|p| p.get_value(Some(time)))
        .unwrap_or(Cartesian3::ZERO);

    let radius = if let Some(ref ellipse) = entity.ellipse {
        ellipse
            .semi_major_axis
            .as_ref()
            .and_then(|a| a.get_value(Some(time)))
            .unwrap_or(10.0)
    } else {
        10.0
    };

    BoundingSphere::new(center, radius)
}

fn entity_color(entity: &Entity, time: &JulianDate) -> [u8; 4] {
    if let Some(ref ellipse) = entity.ellipse {
        if let Some(ref mat) = ellipse.material {
            if let Some(MaterialValue::Color(c)) = mat.get_material_value(Some(time)) {
                return [
                    (c.red * 255.0).round() as u8,
                    (c.green * 255.0).round() as u8,
                    (c.blue * 255.0).round() as u8,
                    (c.alpha * 255.0).round() as u8,
                ];
            }
        }
        if let Some(ref oc) = ellipse.outline_color {
            if let Some(c) = oc.get_value(Some(time)) {
                return [
                    (c.red * 255.0).round() as u8,
                    (c.green * 255.0).round() as u8,
                    (c.blue * 255.0).round() as u8,
                    (c.alpha * 255.0).round() as u8,
                ];
            }
        }
    } else if let Some(ref poly) = entity.polygon {
        if let Some(ref mat) = poly.material {
            if let Some(MaterialValue::Color(c)) = mat.get_material_value(Some(time)) {
                return [
                    (c.red * 255.0).round() as u8,
                    (c.green * 255.0).round() as u8,
                    (c.blue * 255.0).round() as u8,
                    (c.alpha * 255.0).round() as u8,
                ];
            }
        }
    }
    let c = Color::WHITE;
    [
        (c.red * 255.0).round() as u8,
        (c.green * 255.0).round() as u8,
        (c.blue * 255.0).round() as u8,
        (c.alpha * 255.0).round() as u8,
    ]
}

/// Batch that renders static ground geometry with per-instance color attributes,
/// separating spatially overlapping geometries into distinct sub-batches.
pub struct StaticGroundGeometryColorBatch {
    primitives: Rc<RefCell<Vec<VisualizerGeometryPrimitive>>>,
    classification_type: ClassificationType,
    pub batches: Vec<GroundColorSubBatch>,
}

impl StaticGroundGeometryColorBatch {
    /// Creates a new StaticGroundGeometryColorBatch.
    pub fn new(
        primitives: Rc<RefCell<Vec<VisualizerGeometryPrimitive>>>,
        classification_type: ClassificationType,
    ) -> Self {
        Self {
            primitives,
            classification_type,
            batches: Vec::new(),
        }
    }

    /// Adds an updater to the batch, separating overlapping geometries into distinct sub-batches.
    pub fn add<U: BatchGeometryUpdater>(&mut self, time: JulianDate, updater: &U) {
        let entity = updater.entity().clone();
        // Remove existing entry for this entity if re-adding
        self.remove_by_id(&entity.id);

        let bs = entity_bounding_sphere(&entity, &time);
        let color_key = entity_color(&entity, &time);

        for sub in &mut self.batches {
            if sub.color_key == color_key {
                let overlaps = sub
                    .entities
                    .iter()
                    .any(|(_, other_bs)| bounding_spheres_overlap(&bs, other_bs));
                if !overlaps {
                    sub.entities.push((entity, bs));
                    return;
                }
            }
        }

        self.batches.push(GroundColorSubBatch {
            color_key,
            entities: vec![(entity, bs)],
        });
    }

    fn remove_by_id(&mut self, entity_id: &str) {
        for sub in &mut self.batches {
            sub.entities.retain(|(e, _)| e.id != entity_id);
        }
        self.batches.retain(|sub| !sub.entities.is_empty());
    }

    /// Removes an updater from the batch.
    pub fn remove<U: BatchGeometryUpdater>(&mut self, updater: &U) {
        self.remove_by_id(&updater.entity().id);
    }

    /// Removes all primitives created by this batch.
    pub fn remove_all_primitives(&mut self) {
        self.primitives.borrow_mut().clear();
    }

    /// Updates the batch for the given time.
    pub fn update(&mut self, time: JulianDate) -> bool {
        if self.batches.is_empty() {
            self.remove_all_primitives();
            return true;
        }

        let mut new_primitives = Vec::with_capacity(self.batches.len());

        for (idx, sub) in self.batches.iter().enumerate() {
            let mut attributes_by_entity = HashMap::new();

            for (entity, _) in &sub.entities {
                let color_val = entity_color(entity, &time);
                let mut show_val = [0];
                let mut ddc_val = [0.0f32, f32::INFINITY];

                if let Some(ref ellipse) = entity.ellipse {
                    if let Some(ref s) = ellipse.show {
                        if let Some(val) = s.get_value(Some(&time)) {
                            show_val = if val { [1] } else { [0] };
                        }
                    } else if entity.is_showing() && entity.show {
                        show_val = [1];
                    }

                    if let Some(ref ddc) = ellipse.distance_display_condition {
                        if let Some(val) = ddc.get_value(Some(&time)) {
                            ddc_val = [val.near as f32, val.far as f32];
                        }
                    }
                } else if entity.is_showing() && entity.show {
                    show_val = [1];
                }

                let bs = entity_bounding_sphere(entity, &time);

                attributes_by_entity.insert(
                    entity.id.clone(),
                    VisualizerGeometryAttributes {
                        show: Some(show_val),
                        color: Some(color_val),
                        depth_fail_color: None,
                        distance_display_condition: Some(ddc_val),
                        bounding_sphere: Some(bs),
                    },
                );
            }

            new_primitives.push(VisualizerGeometryPrimitive {
                id: format!("ground-color-batch-primitive-{}", idx),
                appearance: PrimitiveAppearanceType::PerInstanceColor { closed: false },
                shadows: ShadowMode::Disabled,
                classification_type: Some(self.classification_type),
                model_matrix: Matrix4::IDENTITY,
                attributes_by_entity,
                show: true,
                is_destroyed: false,
            });
        }

        *self.primitives.borrow_mut() = new_primitives;
        true
    }

    /// Computes bounding sphere for updater.
    pub fn get_bounding_sphere<U: BatchGeometryUpdater>(
        &self,
        updater: &U,
        result: &mut BoundingSphere,
    ) -> BoundingSphereState {
        let prims = self.primitives.borrow();
        for p in prims.iter() {
            if let Some(attrs) = p.get_geometry_instance_attributes(updater.entity()) {
                if let Some(bs) = attrs.bounding_sphere {
                    *result = bs;
                    return BoundingSphereState::Done;
                }
            }
        }
        BoundingSphereState::Failed
    }
}
