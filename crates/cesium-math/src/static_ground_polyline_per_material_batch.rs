//! StaticGroundPolylinePerMaterialBatch matching CesiumJS `DataSources/StaticGroundPolylinePerMaterialBatch.js` (Apache-2.0).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::static_geometry_per_material_batch::BatchGeometryUpdater;
use crate::{
    BoundingSphere, BoundingSphereState, ClassificationType, Color, Entity, JulianDate,
    MaterialValue, Matrix4, PrimitiveAppearanceType, ShadowMode, VisualizerGeometryAttributes,
    VisualizerGeometryPrimitive,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum GroundPolylineMaterialKey {
    ColorAppearance,
    PolylineOutline {
        color: u32,
        outline_color: u32,
        outline_width_bits: u32,
    },
    PolylineArrow(u32),
    PolylineDash {
        color: u32,
        gap_color: u32,
        dash_length_bits: u32,
        dash_pattern: u32,
    },
    PolylineGlow {
        color: u32,
        glow_power_bits: u32,
        taper_power_bits: u32,
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

fn compute_polyline_material_key(entity: &Entity, time: &JulianDate) -> GroundPolylineMaterialKey {
    let mat_val = entity
        .polyline
        .as_ref()
        .and_then(|p| p.material.as_ref())
        .and_then(|m| m.get_material_value(Some(time)));

    match mat_val {
        None | Some(MaterialValue::Color(_)) => GroundPolylineMaterialKey::ColorAppearance,
        Some(MaterialValue::PolylineOutline(o)) => GroundPolylineMaterialKey::PolylineOutline {
            color: color_to_u32(&o.color),
            outline_color: color_to_u32(&o.outline_color),
            outline_width_bits: (o.outline_width as f32).to_bits(),
        },
        Some(MaterialValue::PolylineArrow(c)) => {
            GroundPolylineMaterialKey::PolylineArrow(color_to_u32(&c))
        }
        Some(MaterialValue::PolylineDash(d)) => GroundPolylineMaterialKey::PolylineDash {
            color: color_to_u32(&d.color),
            gap_color: color_to_u32(&d.gap_color),
            dash_length_bits: (d.dash_length as f32).to_bits(),
            dash_pattern: d.dash_pattern as u32,
        },
        Some(MaterialValue::PolylineGlow(g)) => GroundPolylineMaterialKey::PolylineGlow {
            color: color_to_u32(&g.color),
            glow_power_bits: (g.glow_power as f32).to_bits(),
            taper_power_bits: (g.taper_power as f32).to_bits(),
        },
        Some(other) => GroundPolylineMaterialKey::Other(format!("{:?}", other)),
    }
}

fn polyline_bounding_sphere(entity: &Entity, time: &JulianDate) -> BoundingSphere {
    if let Some(ref poly) = entity.polyline {
        if let Some(ref pos_prop) = poly.positions {
            if let Some(positions) = pos_prop.get_value(Some(time)) {
                if !positions.is_empty() {
                    let pts: Vec<glam::DVec3> = positions.into_iter().map(Into::into).collect();
                    return BoundingSphere::from_points(&pts).unwrap_or_default();
                }
            }
        }
    }
    BoundingSphere::default()
}

/// Batch that renders static ground polylines grouped by material.
pub struct StaticGroundPolylinePerMaterialBatch {
    primitives: Rc<RefCell<Vec<VisualizerGeometryPrimitive>>>,
    classification_type: ClassificationType,
    _async_updates: bool,
    updaters: HashMap<String, Entity>,
}

impl StaticGroundPolylinePerMaterialBatch {
    /// Creates a new StaticGroundPolylinePerMaterialBatch.
    pub fn new(
        primitives: Rc<RefCell<Vec<VisualizerGeometryPrimitive>>>,
        classification_type: ClassificationType,
        async_updates: bool,
    ) -> Self {
        Self {
            primitives,
            classification_type,
            _async_updates: async_updates,
            updaters: HashMap::new(),
        }
    }

    /// Adds a polyline updater to the batch.
    pub fn add<U: BatchGeometryUpdater>(&mut self, _time: JulianDate, updater: &U) {
        let entity = updater.entity().clone();
        self.updaters.insert(entity.id.clone(), entity);
    }

    /// Removes a polyline updater from the batch.
    pub fn remove<U: BatchGeometryUpdater>(&mut self, updater: &U) {
        self.updaters.remove(&updater.entity().id);
    }

    /// Removes all primitives created by this batch.
    pub fn remove_all_primitives(&mut self) {
        self.primitives.borrow_mut().clear();
    }

    /// Updates the batch for the given time.
    pub fn update(&mut self, time: JulianDate) -> bool {
        if self.updaters.is_empty() {
            self.remove_all_primitives();
            return true;
        }

        let mut groups: HashMap<GroundPolylineMaterialKey, Vec<Entity>> = HashMap::new();
        for entity in self.updaters.values() {
            let key = compute_polyline_material_key(entity, &time);
            groups.entry(key).or_default().push(entity.clone());
        }

        let mut new_primitives = Vec::with_capacity(groups.len());
        let mut idx = 0;

        for (_key, entities) in groups {
            idx += 1;
            let mut attributes_by_entity = HashMap::new();

            for entity in entities {
                let mut color_val = Some([255, 255, 255, 255]);
                let mut show_val = [0];
                let mut ddc_val = [0.0f32, f32::INFINITY];

                if let Some(ref poly) = entity.polyline {
                    if let Some(ref mat) = poly.material {
                        if let Some(MaterialValue::Color(c)) = mat.get_material_value(Some(&time)) {
                            color_val = Some([
                                (c.red * 255.0).round() as u8,
                                (c.green * 255.0).round() as u8,
                                (c.blue * 255.0).round() as u8,
                                (c.alpha * 255.0).round() as u8,
                            ]);
                        }
                    }

                    if let Some(ref s) = poly.show {
                        if let Some(val) = s.get_value(Some(&time)) {
                            show_val = if val { [1] } else { [0] };
                        }
                    } else if entity.is_showing() && entity.show {
                        show_val = [1];
                    }

                    if let Some(ref ddc) = poly.distance_display_condition {
                        if let Some(val) = ddc.get_value(Some(&time)) {
                            ddc_val = [val.near as f32, val.far as f32];
                        }
                    }
                } else if entity.is_showing() && entity.show {
                    show_val = [1];
                }

                let bs = polyline_bounding_sphere(&entity, &time);

                attributes_by_entity.insert(
                    entity.id.clone(),
                    VisualizerGeometryAttributes {
                        show: Some(show_val),
                        color: color_val,
                        depth_fail_color: None,
                        distance_display_condition: Some(ddc_val),
                        bounding_sphere: Some(bs),
                    },
                );
            }

            new_primitives.push(VisualizerGeometryPrimitive {
                id: format!("ground-polyline-batch-{}", idx),
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
        if !self.updaters.contains_key(&updater.entity().id) {
            return BoundingSphereState::Failed;
        }
        let prims = self.primitives.borrow();
        for p in prims.iter() {
            if let Some(attrs) = p.get_geometry_instance_attributes(updater.entity()) {
                if let Some(bs) = attrs.bounding_sphere {
                    *result = bs;
                    return BoundingSphereState::Done;
                }
            }
        }
        BoundingSphereState::Pending
    }
}
