//! StaticGeometryPerMaterialBatch matching CesiumJS `DataSources/StaticGeometryPerMaterialBatch.js` (Apache-2.0).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    BoundingSphere, BoundingSphereState, BoxGeometryUpdater, Cartesian3, Color,
    CorridorGeometryUpdater, CylinderGeometryUpdater, EllipseGeometryUpdater,
    EllipsoidGeometryUpdater, Entity, JulianDate, MaterialValue, Matrix4,
    PlaneGeometryUpdater, PolygonGeometryUpdater, PolylineGeometryUpdater,
    PolylineVolumeGeometryUpdater, PrimitiveAppearanceType, RectangleGeometryUpdater,
    ShadowMode, VisualizerGeometryAttributes, VisualizerGeometryPrimitive,
    WallGeometryUpdater,
};

/// Trait implemented by all geometry updater types for batching.
pub trait BatchGeometryUpdater {
    fn entity(&self) -> &Entity;
    fn has_geometry(&self) -> bool {
        let e = self.entity();
        e.ellipse.is_some()
            || e.polyline.is_some()
            || e.box_graphics.is_some()
            || e.polygon.is_some()
            || e.corridor.is_some()
            || e.cylinder.is_some()
            || e.ellipsoid.is_some()
            || e.plane.is_some()
            || e.polyline_volume.is_some()
            || e.rectangle.is_some()
            || e.wall.is_some()
    }
}

impl BatchGeometryUpdater for EllipseGeometryUpdater {
    fn entity(&self) -> &Entity {
        self.entity()
    }
    fn has_geometry(&self) -> bool {
        self.entity().ellipse.is_some()
    }
}

impl BatchGeometryUpdater for PolylineGeometryUpdater {
    fn entity(&self) -> &Entity {
        self.entity()
    }
    fn has_geometry(&self) -> bool {
        self.entity().polyline.is_some()
    }
}

impl BatchGeometryUpdater for BoxGeometryUpdater {
    fn entity(&self) -> &Entity {
        self.entity()
    }
    fn has_geometry(&self) -> bool {
        self.entity().box_graphics.is_some()
    }
}

impl BatchGeometryUpdater for PolygonGeometryUpdater {
    fn entity(&self) -> &Entity {
        self.entity()
    }
    fn has_geometry(&self) -> bool {
        self.entity().polygon.is_some()
    }
}

impl BatchGeometryUpdater for CorridorGeometryUpdater {
    fn entity(&self) -> &Entity {
        self.entity()
    }
    fn has_geometry(&self) -> bool {
        self.entity().corridor.is_some()
    }
}

impl BatchGeometryUpdater for CylinderGeometryUpdater {
    fn entity(&self) -> &Entity {
        self.entity()
    }
    fn has_geometry(&self) -> bool {
        self.entity().cylinder.is_some()
    }
}

impl BatchGeometryUpdater for EllipsoidGeometryUpdater {
    fn entity(&self) -> &Entity {
        self.entity()
    }
    fn has_geometry(&self) -> bool {
        self.entity().ellipsoid.is_some()
    }
}

impl BatchGeometryUpdater for PlaneGeometryUpdater {
    fn entity(&self) -> &Entity {
        self.entity()
    }
    fn has_geometry(&self) -> bool {
        self.entity().plane.is_some()
    }
}

impl BatchGeometryUpdater for PolylineVolumeGeometryUpdater {
    fn entity(&self) -> &Entity {
        self.entity()
    }
    fn has_geometry(&self) -> bool {
        self.entity().polyline_volume.is_some()
    }
}

impl BatchGeometryUpdater for RectangleGeometryUpdater {
    fn entity(&self) -> &Entity {
        self.entity()
    }
    fn has_geometry(&self) -> bool {
        self.entity().rectangle.is_some()
    }
}

impl BatchGeometryUpdater for WallGeometryUpdater {
    fn entity(&self) -> &Entity {
        self.entity()
    }
    fn has_geometry(&self) -> bool {
        self.entity().wall.is_some()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum MaterialBatchKey {
    Color(u32),
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
    PolylineArrow(u32),
    DefaultEmpty,
    Other(String),
}

fn color_to_u32(c: &Color) -> u32 {
    let r = (c.red.clamp(0.0, 1.0) * 255.0).round() as u32;
    let g = (c.green.clamp(0.0, 1.0) * 255.0).round() as u32;
    let b = (c.blue.clamp(0.0, 1.0) * 255.0).round() as u32;
    let a = (c.alpha.clamp(0.0, 1.0) * 255.0).round() as u32;
    (r << 24) | (g << 16) | (b << 8) | a
}

fn compute_material_key(val: Option<&MaterialValue>) -> MaterialBatchKey {
    match val {
        None => MaterialBatchKey::DefaultEmpty,
        Some(MaterialValue::Color(c)) => MaterialBatchKey::Color(color_to_u32(c)),
        Some(MaterialValue::Grid(g)) => MaterialBatchKey::Grid {
            color: color_to_u32(&g.color),
            cell_alpha_bits: (g.cell_alpha as f32).to_bits(),
            line_count_x_bits: (g.line_count.x as f32).to_bits(),
            line_count_y_bits: (g.line_count.y as f32).to_bits(),
            line_thickness_x_bits: (g.line_thickness.x as f32).to_bits(),
            line_thickness_y_bits: (g.line_thickness.y as f32).to_bits(),
        },
        Some(MaterialValue::Stripe(s)) => MaterialBatchKey::Stripe {
            even_color: color_to_u32(&s.even_color),
            odd_color: color_to_u32(&s.odd_color),
            offset_bits: (s.offset as f32).to_bits(),
            repeat_bits: (s.repeat as f32).to_bits(),
        },
        Some(MaterialValue::Checkerboard(cb)) => MaterialBatchKey::Checkerboard {
            light_color: color_to_u32(&cb.light_color),
            dark_color: color_to_u32(&cb.dark_color),
            repeat_x_bits: (cb.repeat.x as f32).to_bits(),
            repeat_y_bits: (cb.repeat.y as f32).to_bits(),
        },
        Some(MaterialValue::PolylineArrow(c)) => {
            MaterialBatchKey::PolylineArrow(color_to_u32(c))
        }
        Some(other) => MaterialBatchKey::Other(format!("{:?}", other)),
    }
}

/// Batch that renders static geometry instances grouped per distinct material.
pub struct StaticGeometryPerMaterialBatch {
    primitives: Rc<RefCell<Vec<VisualizerGeometryPrimitive>>>,
    closed: bool,
    shadows: ShadowMode,
    updaters: HashMap<String, (Entity, bool)>, // (Entity, has_geometry)
}

impl StaticGeometryPerMaterialBatch {
    /// Creates a new StaticGeometryPerMaterialBatch.
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
    pub fn add<U: BatchGeometryUpdater>(&mut self, _time: JulianDate, updater: &U) {
        let entity = updater.entity().clone();
        let has_geom = updater.has_geometry();
        self.updaters.insert(entity.id.clone(), (entity, has_geom));
    }

    /// Removes an updater from the batch.
    pub fn remove<U: BatchGeometryUpdater>(&mut self, updater: &U) {
        self.updaters.remove(&updater.entity().id);
    }

    /// Removes all primitives created by this batch.
    pub fn remove_all_primitives(&mut self) {
        self.primitives.borrow_mut().clear();
    }

    /// Updates the batch for the given time.
    pub fn update(&mut self, time: JulianDate) -> bool {
        let active_entities: Vec<Entity> = self
            .updaters
            .values()
            .filter(|(_, has_geom)| *has_geom)
            .map(|(entity, _)| entity.clone())
            .collect();

        if active_entities.is_empty() {
            self.remove_all_primitives();
            return true;
        }

        // Group entities by material key
        let mut groups: HashMap<MaterialBatchKey, Vec<Entity>> = HashMap::new();

        for entity in active_entities {
            let mat_val = if let Some(ref ellipse) = entity.ellipse {
                ellipse
                    .material
                    .as_ref()
                    .and_then(|m| m.get_material_value(Some(&time)))
            } else if let Some(ref polyline) = entity.polyline {
                polyline
                    .material
                    .as_ref()
                    .and_then(|m| m.get_material_value(Some(&time)))
            } else {
                None
            };

            let key = compute_material_key(mat_val.as_ref());
            groups.entry(key).or_default().push(entity);
        }

        let mut new_primitives = Vec::new();
        let mut group_idx = 0;

        for (_key, entities) in groups {
            group_idx += 1;
            let mut attributes_by_entity = HashMap::new();

            for entity in entities {
                let mut show_val = [0];
                let mut ddc_val = [0.0f32, f32::INFINITY];
                let mut depth_fail_color_val = None;

                if let Some(ref ellipse) = entity.ellipse {
                    // Show
                    if let Some(ref s) = ellipse.show {
                        if let Some(val) = s.get_value(Some(&time)) {
                            show_val = if val { [1] } else { [0] };
                        }
                    } else if entity.is_showing() && entity.show {
                        show_val = [1];
                    }

                    // Distance display condition
                    if let Some(ref ddc) = ellipse.distance_display_condition {
                        if let Some(val) = ddc.get_value(Some(&time)) {
                            ddc_val = [val.near as f32, val.far as f32];
                        }
                    }
                } else if let Some(ref polyline) = entity.polyline {
                    // Show
                    if let Some(ref s) = polyline.show {
                        if let Some(val) = s.get_value(Some(&time)) {
                            show_val = if val { [1] } else { [0] };
                        }
                    } else if entity.is_showing() && entity.show {
                        show_val = [1];
                    }

                    // Depth fail material / color
                    if let Some(ref dfm) = polyline.depth_fail_material {
                        if let Some(MaterialValue::Color(c)) = dfm.get_material_value(Some(&time)) {
                            depth_fail_color_val = Some([
                                (c.red * 255.0).round() as u8,
                                (c.green * 255.0).round() as u8,
                                (c.blue * 255.0).round() as u8,
                                (c.alpha * 255.0).round() as u8,
                            ]);
                        } else {
                            depth_fail_color_val = Some([255, 255, 255, 255]); // white fallback
                        }
                    }

                    // Distance display condition
                    if let Some(ref ddc) = polyline.distance_display_condition {
                        if let Some(val) = ddc.get_value(Some(&time)) {
                            ddc_val = [val.near as f32, val.far as f32];
                        }
                    }
                } else if entity.is_showing() && entity.show {
                    show_val = [1];
                }

                let center = entity
                    .position
                    .as_ref()
                    .and_then(|p| p.get_value(Some(&time)))
                    .unwrap_or(Cartesian3::ZERO);

                attributes_by_entity.insert(
                    entity.id.clone(),
                    VisualizerGeometryAttributes {
                        show: Some(show_val),
                        color: None,
                        depth_fail_color: depth_fail_color_val,
                        distance_display_condition: Some(ddc_val),
                        bounding_sphere: Some(BoundingSphere::new(center, 10.0)),
                    },
                );
            }

            let prim = VisualizerGeometryPrimitive {
                id: format!("material-batch-primitive-{}", group_idx),
                appearance: PrimitiveAppearanceType::Material {
                    closed: self.closed,
                },
                shadows: self.shadows,
                classification_type: None,
                model_matrix: Matrix4::IDENTITY,
                attributes_by_entity,
                show: true,
                is_destroyed: false,
            };

            new_primitives.push(prim);
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
