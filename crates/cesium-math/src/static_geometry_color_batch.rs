//! StaticGeometryColorBatch matching CesiumJS `DataSources/StaticGeometryColorBatch.js` (Apache-2.0).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    BoundingSphere, BoundingSphereState, Cartesian3,
    EllipseGeometryUpdater, Entity, JulianDate, Matrix4, PrimitiveAppearanceType,
    ShadowMode, VisualizerGeometryAttributes, VisualizerGeometryPrimitive,
};

/// Batch that renders static geometry with per-instance color attributes.
pub struct StaticGeometryColorBatch {
    primitives: Rc<RefCell<Vec<VisualizerGeometryPrimitive>>>,
    closed: bool,
    shadows: ShadowMode,
    updaters: HashMap<String, Entity>,
}

impl StaticGeometryColorBatch {
    /// Creates a new StaticGeometryColorBatch.
    pub fn new(
        primitives: Rc<RefCell<Vec<VisualizerGeometryPrimitive>>>,
        _appearance_type: Option<()>,
        _depth_fail_appearance_type: Option<()>,
        closed: bool,
        shadows: ShadowMode,
    ) -> Self {
        Self {
            primitives,
            closed,
            shadows,
            updaters: HashMap::new(),
        }
    }

    /// Adds an updater to the batch.
    pub fn add(&mut self, _time: JulianDate, updater: &EllipseGeometryUpdater) {
        self.updaters
            .insert(updater.entity().id.clone(), updater.entity().clone());
    }

    /// Removes an updater from the batch.
    pub fn remove(&mut self, updater: &EllipseGeometryUpdater) {
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

        let mut attributes_by_entity = HashMap::new();

        for (id, entity) in &self.updaters {
            let mut color_val = [255, 255, 255, 255]; // default white if out of range
            let mut show_val = [0];
            let mut ddc_val = [0.0f32, f32::INFINITY];

            if let Some(ref ellipse) = entity.ellipse {
                // Color attribute
                if let Some(ref mat) = ellipse.material {
                    if let Some(crate::MaterialValue::Color(c)) = mat.get_material_value(Some(&time)) {
                        color_val = [
                            (c.red * 255.0).round() as u8,
                            (c.green * 255.0).round() as u8,
                            (c.blue * 255.0).round() as u8,
                            (c.alpha * 255.0).round() as u8,
                        ];
                    }
                }

                // Show attribute
                if let Some(ref s) = ellipse.show {
                    if let Some(val) = s.get_value(Some(&time)) {
                        show_val = if val { [1] } else { [0] };
                    }
                } else if entity.is_showing() && entity.show {
                    show_val = [1];
                }

                // Distance display condition attribute
                if let Some(ref ddc) = ellipse.distance_display_condition {
                    if let Some(val) = ddc.get_value(Some(&time)) {
                        ddc_val = [val.near as f32, val.far as f32];
                    }
                }
            }

            let center = entity
                .position
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)))
                .unwrap_or(Cartesian3::ZERO);

            attributes_by_entity.insert(
                id.clone(),
                VisualizerGeometryAttributes {
                    show: Some(show_val),
                    color: Some(color_val),
                    depth_fail_color: None,
                    distance_display_condition: Some(ddc_val),
                    bounding_sphere: Some(BoundingSphere::new(center, 10.0)),
                },
            );
        }

        let prim = VisualizerGeometryPrimitive {
            id: "batch-primitive".to_string(),
            appearance: PrimitiveAppearanceType::PerInstanceColor {
                closed: self.closed,
            },
            shadows: self.shadows,
            classification_type: None,
            model_matrix: Matrix4::IDENTITY,
            attributes_by_entity,
            show: true,
            is_destroyed: false,
        };

        let mut prims = self.primitives.borrow_mut();
        if prims.is_empty() {
            prims.push(prim);
        } else {
            prims[0] = prim;
        }

        true
    }

    /// Computes bounding sphere for updater.
    pub fn get_bounding_sphere(
        &self,
        updater: &EllipseGeometryUpdater,
        result: &mut BoundingSphere,
    ) -> BoundingSphereState {
        let prims = self.primitives.borrow();
        if let Some(p) = prims.first() {
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
