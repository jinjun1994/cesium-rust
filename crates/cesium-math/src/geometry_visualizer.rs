//! GeometryVisualizer matching CesiumJS `DataSources/GeometryVisualizer.js` (Apache-2.0).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    BoundingSphere, BoundingSphereState, Cartesian3, ClassificationType, Color,
    DeveloperError, Entity, EntityCollection, GeometryUpdaterSet, JulianDate,
    ListenerId, Matrix4, ShadowMode,
};

/// Primitive appearance type.
#[derive(Clone, Debug, PartialEq)]
pub enum PrimitiveAppearanceType {
    PerInstanceColor { closed: bool },
    Material { closed: bool },
}

impl PrimitiveAppearanceType {
    pub fn closed(&self) -> bool {
        match *self {
            Self::PerInstanceColor { closed } => closed,
            Self::Material { closed } => closed,
        }
    }
}

/// Instance attributes stored on primitives.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VisualizerGeometryAttributes {
    pub show: Option<[u8; 1]>,
    pub color: Option<[u8; 4]>,
    pub depth_fail_color: Option<[u8; 4]>,
    pub distance_display_condition: Option<[f32; 2]>,
    pub bounding_sphere: Option<BoundingSphere>,
}

/// Simulated primitive produced by GeometryVisualizer.
#[derive(Clone, Debug)]
pub struct VisualizerGeometryPrimitive {
    pub id: String,
    pub appearance: PrimitiveAppearanceType,
    pub shadows: ShadowMode,
    pub classification_type: Option<ClassificationType>,
    pub model_matrix: Matrix4,
    pub attributes_by_entity: HashMap<String, VisualizerGeometryAttributes>,
    pub show: bool,
    pub is_destroyed: bool,
}

impl VisualizerGeometryPrimitive {
    pub fn get_geometry_instance_attributes(
        &self,
        entity: &Entity,
    ) -> Option<&VisualizerGeometryAttributes> {
        self.attributes_by_entity.get(&entity.id)
    }

    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        self.attributes_by_entity.clear();
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }
}

/// Simulated Scene holding primitive collections for GeometryVisualizer.
#[derive(Clone, Debug, Default)]
pub struct GeometryVisualizerScene {
    pub primitives: Rc<RefCell<Vec<VisualizerGeometryPrimitive>>>,
    pub ground_primitives: Rc<RefCell<Vec<VisualizerGeometryPrimitive>>>,
}

impl GeometryVisualizerScene {
    pub fn new() -> Self {
        Self {
            primitives: Rc::new(RefCell::new(Vec::new())),
            ground_primitives: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

/// A general purpose visualizer for geometry represented by Primitive instances.
pub struct GeometryVisualizer {
    scene: GeometryVisualizerScene,
    entity_collection: Rc<RefCell<EntityCollection>>,
    primitives: Rc<RefCell<Vec<VisualizerGeometryPrimitive>>>,
    ground_primitives: Rc<RefCell<Vec<VisualizerGeometryPrimitive>>>,
    pub updater_sets: HashMap<String, GeometryUpdaterSet>,
    collection_changed_id: Option<ListenerId>,
    is_destroyed: bool,
}

impl GeometryVisualizer {
    /// Creates a new GeometryVisualizer.
    pub fn new(
        scene: Option<GeometryVisualizerScene>,
        entity_collection: Option<Rc<RefCell<EntityCollection>>>,
        primitives: Option<Rc<RefCell<Vec<VisualizerGeometryPrimitive>>>>,
        ground_primitives: Option<Rc<RefCell<Vec<VisualizerGeometryPrimitive>>>>,
    ) -> Result<Self, DeveloperError> {
        let scene = scene.ok_or_else(|| DeveloperError::new("scene is required."))?;
        let entity_collection =
            entity_collection.ok_or_else(|| DeveloperError::new("entityCollection is required."))?;

        let prims = primitives.unwrap_or_else(|| Rc::clone(&scene.primitives));
        let ground_prims =
            ground_primitives.unwrap_or_else(|| Rc::clone(&scene.ground_primitives));

        let sub_id = entity_collection
            .borrow_mut()
            .collection_changed
            .add_listener(move |_| {});

        Ok(Self {
            scene,
            entity_collection,
            primitives: prims,
            ground_primitives: ground_prims,
            updater_sets: HashMap::new(),
            collection_changed_id: Some(sub_id),
            is_destroyed: false,
        })
    }

    /// Gets the scene.
    pub fn scene(&self) -> &GeometryVisualizerScene {
        &self.scene
    }

    /// Updates the visualizer for the given time.
    pub fn update(&mut self, time: Option<JulianDate>) -> Result<bool, DeveloperError> {
        let time = time.ok_or_else(|| DeveloperError::new("time is required."))?;

        let active_entities: Vec<Entity> = self
            .entity_collection
            .borrow()
            .values()
            .into_iter()
            .cloned()
            .collect();

        // 1. Remove updater sets and primitives for removed entities
        let active_ids: Vec<String> = active_entities.iter().map(|e| e.id.clone()).collect();
        self.updater_sets.retain(|id, set| {
            if !active_ids.contains(id) {
                set.destroy();
                false
            } else {
                true
            }
        });

        // Retain only primitives that have at least one active entity attribute
        {
            let mut prims = self.primitives.borrow_mut();
            for p in prims.iter_mut() {
                p.attributes_by_entity.retain(|id, _| active_ids.contains(id));
            }
            prims.retain(|p| !p.attributes_by_entity.is_empty());
        }
        {
            let mut gprims = self.ground_primitives.borrow_mut();
            for p in gprims.iter_mut() {
                p.attributes_by_entity
                    .retain(|id, _| active_ids.contains(id));
            }
            gprims.retain(|p| !p.attributes_by_entity.is_empty());
        }

        // 2. Process each active entity
        for entity in &active_entities {
            let entity_id = entity.id.clone();
            let _ = self
                .updater_sets
                .entry(entity_id.clone())
                .or_insert_with(|| GeometryUpdaterSet::new(entity.clone(), None).unwrap());

            // Check if entity has geometry
            if let Some(ref ellipse) = entity.ellipse {
                let semi_major = ellipse
                    .semi_major_axis
                    .as_ref()
                    .and_then(|p| p.get_value(Some(&time)))
                    .unwrap_or(1.0);
                let semi_minor = ellipse
                    .semi_minor_axis
                    .as_ref()
                    .and_then(|p| p.get_value(Some(&time)))
                    .unwrap_or(1.0);
                let height = ellipse
                    .height
                    .as_ref()
                    .and_then(|p| p.get_value(Some(&time)));
                let extruded_height = ellipse
                    .extruded_height
                    .as_ref()
                    .and_then(|p| p.get_value(Some(&time)));
                let classification = ellipse
                    .classification_type
                    .as_ref()
                    .and_then(|p| p.get_value(Some(&time)));
                let shadows = ellipse
                    .shadows
                    .as_ref()
                    .and_then(|p| p.get_value(Some(&time)))
                    .unwrap_or(ShadowMode::Disabled);

                let fill = ellipse
                    .fill
                    .as_ref()
                    .and_then(|p| p.get_value(Some(&time)))
                    .unwrap_or(true);
                let outline = ellipse
                    .outline
                    .as_ref()
                    .and_then(|p| p.get_value(Some(&time)))
                    .unwrap_or(false);

                let is_ground = classification.is_some() && height.unwrap_or(0.0) == 0.0;
                let is_closed = extruded_height.is_some()
                    || (height.unwrap_or(0.0) == 0.0 && classification.is_none());

                let show_val = if entity.is_showing() && entity.show {
                    [1]
                } else {
                    [0]
                };

                // Bounding sphere
                let center = entity
                    .position
                    .as_ref()
                    .and_then(|p| p.get_value(Some(&time)))
                    .unwrap_or(Cartesian3::ZERO);
                let radius = semi_major.max(semi_minor).max(extruded_height.unwrap_or(0.0));
                let bounding_sphere = Some(BoundingSphere::new(center, radius));

                if outline {
                    let outline_color = ellipse
                        .outline_color
                        .as_ref()
                        .and_then(|p| p.get_value(Some(&time)))
                        .unwrap_or(Color::BLACK);
                    let color_val = [
                        (outline_color.red * 255.0).round() as u8,
                        (outline_color.green * 255.0).round() as u8,
                        (outline_color.blue * 255.0).round() as u8,
                        (outline_color.alpha * 255.0).round() as u8,
                    ];

                    let mut attrs = HashMap::new();
                    attrs.insert(
                        entity_id.clone(),
                        VisualizerGeometryAttributes {
                            show: Some(show_val),
                            color: Some(color_val),
                            depth_fail_color: None,
                            distance_display_condition: None,
                            bounding_sphere,
                        },
                    );

                    let prim = VisualizerGeometryPrimitive {
                        id: format!("{}-outline", entity_id),
                        appearance: PrimitiveAppearanceType::PerInstanceColor { closed: false },
                        shadows,
                        classification_type: classification,
                        model_matrix: Matrix4::IDENTITY,
                        attributes_by_entity: attrs,
                        show: true,
                        is_destroyed: false,
                    };

                    let target = if is_ground {
                        &self.ground_primitives
                    } else {
                        &self.primitives
                    };
                    let mut list = target.borrow_mut();
                    if let Some(existing) = list.iter_mut().find(|p| p.id.starts_with(&entity_id)) {
                        *existing = prim;
                    } else {
                        list.push(prim);
                    }
                } else if fill {
                    let (appearance, color_val) = match &ellipse.material {
                        Some(mat) => {
                            if mat.get_type(&time) == "Color" {
                                let c = match mat.get_material_value(Some(&time)) {
                                    Some(crate::MaterialValue::Color(c)) => c,
                                    _ => Color::WHITE,
                                };
                                (
                                    PrimitiveAppearanceType::PerInstanceColor { closed: is_closed },
                                    Some([
                                        (c.red * 255.0).round() as u8,
                                        (c.green * 255.0).round() as u8,
                                        (c.blue * 255.0).round() as u8,
                                        (c.alpha * 255.0).round() as u8,
                                    ]),
                                )
                            } else {
                                (
                                    PrimitiveAppearanceType::Material { closed: is_closed },
                                    None,
                                )
                            }
                        }
                        None => (
                            PrimitiveAppearanceType::PerInstanceColor { closed: is_closed },
                            Some([255, 255, 255, 255]),
                        ),
                    };

                    let mut attrs = HashMap::new();
                    attrs.insert(
                        entity_id.clone(),
                        VisualizerGeometryAttributes {
                            show: Some(show_val),
                            color: color_val,
                            depth_fail_color: None,
                            distance_display_condition: None,
                            bounding_sphere,
                        },
                    );

                    let prim = VisualizerGeometryPrimitive {
                        id: format!("{}-fill", entity_id),
                        appearance,
                        shadows,
                        classification_type: classification,
                        model_matrix: Matrix4::IDENTITY,
                        attributes_by_entity: attrs,
                        show: true,
                        is_destroyed: false,
                    };

                    let target = if is_ground {
                        &self.ground_primitives
                    } else {
                        &self.primitives
                    };

                    // For ground entities with image material or different classification types, batch them
                    let mut list = target.borrow_mut();
                    // If an existing primitive matches or we update
                    if let Some(existing) = list
                        .iter_mut()
                        .find(|p| p.id == format!("{}-fill", entity_id))
                    {
                        *existing = prim;
                    } else {
                        list.push(prim);
                    }
                }
            }
        }

        Ok(true)
    }

    /// Computes bounding sphere for the entity.
    pub fn get_bounding_sphere(
        &self,
        entity: Option<&Entity>,
        result: Option<&mut BoundingSphere>,
    ) -> Result<BoundingSphereState, DeveloperError> {
        let entity = entity.ok_or_else(|| DeveloperError::new("entity is required."))?;
        let result = result.ok_or_else(|| DeveloperError::new("result is required."))?;

        let prims = self.primitives.borrow();
        for p in prims.iter() {
            if let Some(attrs) = p.get_geometry_instance_attributes(entity) {
                if let Some(bs) = attrs.bounding_sphere {
                    *result = bs;
                    return Ok(BoundingSphereState::Done);
                }
            }
        }

        let gprims = self.ground_primitives.borrow();
        for p in gprims.iter() {
            if let Some(attrs) = p.get_geometry_instance_attributes(entity) {
                if let Some(bs) = attrs.bounding_sphere {
                    *result = bs;
                    return Ok(BoundingSphereState::Done);
                }
            }
        }

        Ok(BoundingSphereState::Failed)
    }

    /// Checks if destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys the visualizer.
    pub fn destroy(&mut self) {
        if self.is_destroyed {
            return;
        }
        self.is_destroyed = true;

        if let Some(sub_id) = self.collection_changed_id.take() {
            self.entity_collection
                .borrow_mut()
                .collection_changed
                .remove_listener(sub_id);
        }

        for (_, mut set) in self.updater_sets.drain() {
            set.destroy();
        }

        self.primitives.borrow_mut().clear();
        self.ground_primitives.borrow_mut().clear();
    }
}

impl Drop for GeometryVisualizer {
    fn drop(&mut self) {
        self.destroy();
    }
}

impl std::fmt::Debug for GeometryVisualizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeometryVisualizer")
            .field("is_destroyed", &self.is_destroyed)
            .finish()
    }
}

impl crate::visualizer::Visualizer for GeometryVisualizer {
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
