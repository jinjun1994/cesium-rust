//! StaticGroundGeometryPerMaterialBatch matching CesiumJS `DataSources/StaticGroundGeometryPerMaterialBatch.js` (Apache-2.0).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::static_geometry_per_material_batch::BatchGeometryUpdater;
use crate::{
    BoundingSphere, BoundingSphereState, Cartesian3, ClassificationType, Color, Entity,
    JulianDate, MaterialValue, Matrix4, PrimitiveAppearanceType, ShadowMode,
    VisualizerGeometryAttributes, VisualizerGeometryPrimitive,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GroundMaterialKey {
    /// All pure Color materials share a single per-instance color appearance key when non-overlapping.
    ColorAppearance,
    Grid {
        color: u32,
        cell_alpha_bits: u32,
        line_count_x_bits: u32,
        line_count_y_bits: u32,
        line_thickness_x_bits: u32,
        line_thickness_y_bits: u32,
    },
    Stripe {
        even_color: u32,
        odd_color: u32,
        offset_bits: u32,
        repeat_bits: u32,
    },
    Checkerboard {
        light_color: u32,
        dark_color: u32,
        repeat_x_bits: u32,
        repeat_y_bits: u32,
    },
    Other(String),
}

fn color_to_u32(c: &Color) -> u32 {
    let r = (c.red.clamp(0.0, 1.0) * 255.0).round() as u32;
    let g = (c.green.clamp(0.0, 1.0) * 255.0).round() as u32;
    let b = (c.blue.clamp(0.0, 1.0) * 255.0).round() as u32;
    let a = (c.alpha.clamp(0.0, 1.0) * 255.0).round() as u32;
    (r << 24) | (g << 16) | (b << 8) | a
}

fn compute_ground_material_key(entity: &Entity, time: &JulianDate) -> GroundMaterialKey {
    let mat_val = if let Some(ref ellipse) = entity.ellipse {
        ellipse
            .material
            .as_ref()
            .and_then(|m| m.get_material_value(Some(time)))
    } else if let Some(ref poly) = entity.polygon {
        poly.material
            .as_ref()
            .and_then(|m| m.get_material_value(Some(time)))
    } else {
        None
    };

    match mat_val {
        None | Some(MaterialValue::Color(_)) => GroundMaterialKey::ColorAppearance,
        Some(MaterialValue::Grid(g)) => GroundMaterialKey::Grid {
            color: color_to_u32(&g.color),
            cell_alpha_bits: (g.cell_alpha as f32).to_bits(),
            line_count_x_bits: (g.line_count.x as f32).to_bits(),
            line_count_y_bits: (g.line_count.y as f32).to_bits(),
            line_thickness_x_bits: (g.line_thickness.x as f32).to_bits(),
            line_thickness_y_bits: (g.line_thickness.y as f32).to_bits(),
        },
        Some(MaterialValue::Stripe(s)) => GroundMaterialKey::Stripe {
            even_color: color_to_u32(&s.even_color),
            odd_color: color_to_u32(&s.odd_color),
            offset_bits: (s.offset as f32).to_bits(),
            repeat_bits: (s.repeat as f32).to_bits(),
        },
        Some(MaterialValue::Checkerboard(cb)) => GroundMaterialKey::Checkerboard {
            light_color: color_to_u32(&cb.light_color),
            dark_color: color_to_u32(&cb.dark_color),
            repeat_x_bits: (cb.repeat.x as f32).to_bits(),
            repeat_y_bits: (cb.repeat.y as f32).to_bits(),
        },
        Some(other) => GroundMaterialKey::Other(format!("{:?}", other)),
    }
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

/// Sub-batch for StaticGroundGeometryPerMaterialBatch.
#[derive(Clone, Debug)]
pub struct GroundMaterialSubBatch {
    pub material_id: usize,
    pub material_key: GroundMaterialKey,
    pub entities: Vec<(Entity, BoundingSphere)>,
}

/// Batch that renders static ground geometry grouped by material key and spatial non-overlap.
pub struct StaticGroundGeometryPerMaterialBatch {
    primitives: Rc<RefCell<Vec<VisualizerGeometryPrimitive>>>,
    classification_type: ClassificationType,
    pub batches: Vec<GroundMaterialSubBatch>,
    next_material_id: usize,
}

impl StaticGroundGeometryPerMaterialBatch {
    /// Creates a new StaticGroundGeometryPerMaterialBatch.
    pub fn new(
        primitives: Rc<RefCell<Vec<VisualizerGeometryPrimitive>>>,
        classification_type: ClassificationType,
        _appearance_type: Option<()>,
    ) -> Self {
        Self {
            primitives,
            classification_type,
            batches: Vec::new(),
            next_material_id: 1,
        }
    }

    /// Adds an updater to the batch.
    pub fn add<U: BatchGeometryUpdater>(&mut self, time: JulianDate, updater: &U) {
        let entity = updater.entity().clone();
        self.remove_by_id(&entity.id);

        let bs = entity_bounding_sphere(&entity, &time);
        let key = compute_ground_material_key(&entity, &time);

        for sub in &mut self.batches {
            if sub.material_key == key {
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

        let mat_id = self.next_material_id;
        self.next_material_id += 1;
        self.batches.push(GroundMaterialSubBatch {
            material_id: mat_id,
            material_key: key,
            entities: vec![(entity, bs)],
        });
    }

    fn remove_by_id(&mut self, entity_id: &str) {
        for sub in &mut self.batches {
            sub.entities.retain(|(e, _)| e.id != entity_id);
        }
        // Note: empty sub-batches are intentionally kept until update() so re-adds reuse the same material_id
    }

    /// Removes an updater from the batch (defers removing empty sub-batches until update).
    pub fn remove<U: BatchGeometryUpdater>(&mut self, updater: &U) {
        self.remove_by_id(&updater.entity().id);
    }

    /// Removes all primitives created by this batch.
    pub fn remove_all_primitives(&mut self) {
        self.primitives.borrow_mut().clear();
    }

    /// Updates the batch for the given time.
    pub fn update(&mut self, time: JulianDate) -> bool {
        // Re-evaluate material keys in case a shared material property was mutated on an existing entity
        let mut all_entities: Vec<(usize, Entity)> = Vec::new();
        for sub in &self.batches {
            for (entity, _) in &sub.entities {
                all_entities.push((sub.material_id, entity.clone()));
            }
        }

        // Remove empty sub-batches
        self.batches.retain(|sub| !sub.entities.is_empty());

        if all_entities.is_empty() {
            self.remove_all_primitives();
            return true;
        }

        // Rebuild batches honoring updated material keys while preserving existing material_ids where keys still match
        let mut rebuilt_batches: Vec<GroundMaterialSubBatch> = Vec::new();
        for (old_mat_id, entity) in all_entities {
            let bs = entity_bounding_sphere(&entity, &time);
            let key = compute_ground_material_key(&entity, &time);

            let mut placed = false;
            for sub in &mut rebuilt_batches {
                if sub.material_key == key {
                    let overlaps = sub
                        .entities
                        .iter()
                        .any(|(_, other_bs)| bounding_spheres_overlap(&bs, other_bs));
                    if !overlaps {
                        sub.entities.push((entity.clone(), bs));
                        placed = true;
                        break;
                    }
                }
            }

            if !placed {
                // Keep old_mat_id if no rebuilt batch already uses it
                let mat_id = if rebuilt_batches.iter().any(|b| b.material_id == old_mat_id) {
                    let id = self.next_material_id;
                    self.next_material_id += 1;
                    id
                } else {
                    old_mat_id
                };
                rebuilt_batches.push(GroundMaterialSubBatch {
                    material_id: mat_id,
                    material_key: key,
                    entities: vec![(entity, bs)],
                });
            }
        }

        self.batches = rebuilt_batches;

        let mut new_primitives = Vec::with_capacity(self.batches.len());
        for sub in &self.batches {
            let mut attributes_by_entity = HashMap::new();

            for (entity, _) in &sub.entities {
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
                        color: None,
                        depth_fail_color: None,
                        distance_display_condition: Some(ddc_val),
                        bounding_sphere: Some(bs),
                    },
                );
            }

            new_primitives.push(VisualizerGeometryPrimitive {
                id: format!("ground-material-batch-{}", sub.material_id),
                appearance: PrimitiveAppearanceType::Material { closed: false },
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
