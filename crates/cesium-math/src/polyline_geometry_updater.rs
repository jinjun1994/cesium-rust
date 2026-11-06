//! A GeometryUpdater for polylines matching CesiumJS `DataSources/PolylineGeometryUpdater.js` (Apache-2.0).

use std::collections::HashMap;
use std::rc::Rc;
use glam::DVec3;

use crate::{
    ArcType, BoundingSphere, BoundingSphereState, Cartesian3, ClassificationType, Color,
    ColorGeometryInstanceAttribute, ColorMaterialProperty, ConstantProperty, DeveloperError,
    DistanceDisplayCondition, DynProperty, Entity, Event, GeometryInstanceAttribute, JulianDate,
    MaterialProperty, MaterialValue, Matrix4, PolylineGeometry, PolylineGeometryOptions, ShadowMode,
    ShowGeometryInstanceAttribute,
};

/// Options for configuring a polyline geometry instance.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PolylineUpdaterOptions {
    pub positions: Option<Vec<Cartesian3>>,
    pub width: Option<f64>,
    pub granularity: Option<f64>,
    pub arc_type: Option<ArcType>,
    pub clamp_to_ground: Option<bool>,
}

/// A geometry instance representing the filled portion of a polyline geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct PolylineFillGeometryInstance {
    pub id: String,
    pub geometry: PolylineGeometry,
    pub model_matrix: Matrix4,
    pub attributes: HashMap<String, GeometryInstanceAttribute>,
}

/// Dynamic updater for polylines when positions, width or other properties change with time.
pub struct DynamicPolylineGeometryUpdater {
    entity: Entity,
    options: PolylineUpdaterOptions,
}

impl DynamicPolylineGeometryUpdater {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            options: PolylineUpdaterOptions::default(),
        }
    }

    pub fn options(&self) -> &PolylineUpdaterOptions {
        &self.options
    }

    pub fn update(&mut self, time: JulianDate) {
        if let Some(poly) = &self.entity.polyline {
            self.options.positions = poly
                .positions
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.width = poly
                .width
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.granularity = poly
                .granularity
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.arc_type = poly
                .arc_type
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.clamp_to_ground = poly
                .clamp_to_ground
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
        } else {
            self.options = PolylineUpdaterOptions::default();
        }
    }

    /// Computes dynamic geometry bounding sphere.
    pub fn get_bounding_sphere(
        &self,
        result: Option<&mut BoundingSphere>,
    ) -> Result<BoundingSphereState, DeveloperError> {
        let result = result.ok_or_else(|| DeveloperError::new("result is required"))?;
        if let Some(positions) = &self.options.positions {
            if positions.len() >= 2 {
                let pts_dvec: Vec<DVec3> = positions
                    .iter()
                    .map(|p| DVec3::new(p.x, p.y, p.z))
                    .collect();
                if let Ok(bs) = BoundingSphere::from_points(&pts_dvec) {
                    *result = bs;
                    return Ok(BoundingSphereState::Done);
                }
            }
        }
        Ok(BoundingSphereState::Failed)
    }
}

/// A GeometryUpdater for polylines.
pub struct PolylineGeometryUpdater {
    entity: Entity,
    fill_enabled: bool,
    is_dynamic: bool,
    clamp_to_ground: bool,
    z_index: i32,
    arc_type: Option<ArcType>,
    fill_material_property: Option<Rc<dyn MaterialProperty>>,
    depth_fail_material_property: Option<Rc<dyn MaterialProperty>>,
    shadows_property: Option<Rc<dyn DynProperty<ShadowMode>>>,
    distance_display_condition_property: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
    classification_type_property: Option<Rc<dyn DynProperty<ClassificationType>>>,
    options: PolylineUpdaterOptions,
    has_valid_geometry: bool,
    pub geometry_changed: Rc<Event<()>>,
}

impl PolylineGeometryUpdater {
    /// Creates a new PolylineGeometryUpdater for an entity.
    pub fn new(entity: Entity, _scene: Option<()>) -> Result<Self, DeveloperError> {
        let geometry_changed = Rc::new(Event::new());
        let mut updater = Self {
            entity: entity.clone(),
            fill_enabled: false,
            is_dynamic: false,
            clamp_to_ground: false,
            z_index: 0,
            arc_type: None,
            fill_material_property: None,
            depth_fail_material_property: None,
            shadows_property: None,
            distance_display_condition_property: None,
            classification_type_property: None,
            options: PolylineUpdaterOptions::default(),
            has_valid_geometry: false,
            geometry_changed,
        };

        updater.recompute_state(&entity, true);
        Ok(updater)
    }

    /// Gets the entity associated with this geometry.
    pub fn entity(&self) -> &Entity {
        &self.entity
    }

    /// Gets a value indicating if the geometry has a fill component.
    pub fn fill_enabled(&self) -> bool {
        self.fill_enabled
    }

    /// Gets a value indicating if fill visibility varies with simulation time.
    pub fn has_constant_fill(&self) -> bool {
        if !self.fill_enabled {
            return true;
        }
        if self.entity.availability.is_some() {
            return false;
        }
        if let Some(polyline) = &self.entity.polyline {
            if let Some(show) = &polyline.show {
                return show.is_constant();
            }
        }
        true
    }

    /// Gets a value indicating if the geometry has an outline component. Polylines do not have outlines.
    pub fn outline_enabled(&self) -> bool {
        false
    }

    /// Gets a value indicating if outline visibility varies with simulation time.
    pub fn has_constant_outline(&self) -> bool {
        true
    }

    /// Gets a value indicating if the geometry is dynamic.
    pub fn is_dynamic(&self) -> bool {
        self.is_dynamic
    }

    /// Gets a value indicating if the geometry is closed. Polylines are never closed.
    pub fn is_closed(&self) -> bool {
        false
    }

    /// Gets a value indicating if polylines are clamped to ground.
    pub fn clamp_to_ground(&self) -> bool {
        self.clamp_to_ground
    }

    /// Gets the z-index.
    pub fn z_index(&self) -> i32 {
        self.z_index
    }

    /// Gets the arc type.
    pub fn arc_type(&self) -> Option<ArcType> {
        self.arc_type
    }

    /// Gets fill material property.
    pub fn fill_material_property(&self) -> Option<Rc<dyn MaterialProperty>> {
        self.fill_material_property.clone()
    }

    /// Gets depth fail material property.
    pub fn depth_fail_material_property(&self) -> Option<Rc<dyn MaterialProperty>> {
        self.depth_fail_material_property.clone()
    }

    /// Gets shadows property.
    pub fn shadows_property(&self) -> Option<Rc<dyn DynProperty<ShadowMode>>> {
        self.shadows_property.clone()
    }

    /// Gets distance display condition property.
    pub fn distance_display_condition_property(
        &self,
    ) -> Option<Rc<dyn DynProperty<DistanceDisplayCondition>>> {
        self.distance_display_condition_property.clone()
    }

    /// Gets classification type property.
    pub fn classification_type_property(
        &self,
    ) -> Option<Rc<dyn DynProperty<ClassificationType>>> {
        self.classification_type_property.clone()
    }

    /// Checks if outline is visible at given time.
    pub fn is_outline_visible(&self, _time: JulianDate) -> bool {
        false
    }

    /// Checks if line is filled (visible) at given time.
    pub fn is_filled(&self, time: JulianDate) -> bool {
        if !self.fill_enabled || !self.entity.is_showing() {
            return false;
        }
        if let Some(avail) = &self.entity.availability {
            if !avail.contains(&time) {
                return false;
            }
        }
        if let Some(poly) = &self.entity.polyline {
            if let Some(show_prop) = &poly.show {
                return show_prop.get_value(Some(&time)).unwrap_or(true);
            }
        }
        true
    }

    /// Gets current options for polyline geometry generation.
    pub fn options(&self) -> &PolylineUpdaterOptions {
        &self.options
    }

    /// Creates a dynamic updater.
    pub fn create_dynamic_updater(&self) -> Result<DynamicPolylineGeometryUpdater, DeveloperError> {
        if !self.is_dynamic {
            return Err(DeveloperError::new(
                "This instance does not represent dynamic geometry.",
            ));
        }
        Ok(DynamicPolylineGeometryUpdater::new(self.entity.clone()))
    }

    /// Called when an observed entity property changes.
    pub fn on_entity_property_changed(&mut self, entity: &Entity, property_name: &str) {
        if property_name != "polyline" && property_name != "availability" {
            return;
        }

        let old_valid = self.has_valid_geometry;
        self.recompute_state(entity, false);

        if !old_valid && !self.has_valid_geometry {
            return;
        }

        self.geometry_changed.raise_event(&());
    }

    fn recompute_state(&mut self, entity: &Entity, _initial: bool) {
        self.entity = entity.clone();

        let poly = match &entity.polyline {
            Some(p) => p,
            None => {
                self.has_valid_geometry = false;
                self.fill_enabled = false;
                self.is_dynamic = false;
                self.clamp_to_ground = false;
                self.z_index = 0;
                self.arc_type = None;
                self.fill_material_property = None;
                self.depth_fail_material_property = None;
                self.shadows_property = None;
                self.distance_display_condition_property = None;
                self.classification_type_property = None;
                return;
            }
        };

        if poly.positions.is_none() {
            self.has_valid_geometry = false;
            self.fill_enabled = false;
            self.is_dynamic = false;
            return;
        }

        self.has_valid_geometry = true;

        let show = poly
            .show
            .as_ref()
            .and_then(|p| p.get_value(None))
            .unwrap_or(true);

        self.fill_enabled = show;

        let mut dynamic = false;
        if let Some(pos) = &poly.positions {
            if !pos.is_constant() {
                dynamic = true;
            }
        }
        if !poly.width.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !poly.granularity.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !poly.arc_type.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !poly.clamp_to_ground.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !poly.z_index.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }

        self.is_dynamic = dynamic;

        self.clamp_to_ground = poly
            .clamp_to_ground
            .as_ref()
            .and_then(|p| p.get_value(None))
            .unwrap_or(false);

        self.z_index = poly
            .z_index
            .as_ref()
            .and_then(|p| p.get_value(None))
            .unwrap_or(0);

        self.arc_type = poly
            .arc_type
            .as_ref()
            .and_then(|p| p.get_value(None));

        self.fill_material_property = poly.material.clone().or_else(|| {
            Some(Rc::new(ColorMaterialProperty::from_color(Color::WHITE)) as Rc<dyn MaterialProperty>)
        });
        self.depth_fail_material_property = poly.depth_fail_material.clone();
        self.shadows_property = poly.shadows.clone().or_else(|| {
            Some(Rc::new(ConstantProperty::from_value(ShadowMode::Disabled)) as Rc<dyn DynProperty<ShadowMode>>)
        });
        self.distance_display_condition_property = poly.distance_display_condition.clone().or_else(|| {
            Some(Rc::new(ConstantProperty::from_value(DistanceDisplayCondition::default())) as Rc<dyn DynProperty<DistanceDisplayCondition>>)
        });
        self.classification_type_property = poly.classification_type.clone();

        self.options = PolylineUpdaterOptions {
            positions: poly.positions.as_ref().and_then(|p| p.get_value(None)),
            width: poly.width.as_ref().and_then(|p| p.get_value(None)),
            granularity: poly.granularity.as_ref().and_then(|p| p.get_value(None)),
            arc_type: self.arc_type,
            clamp_to_ground: Some(self.clamp_to_ground),
        };
    }

    /// Creates the geometry instance which represents the fill of the geometry.
    pub fn create_fill_geometry_instance(
        &self,
        time: JulianDate,
    ) -> Result<PolylineFillGeometryInstance, DeveloperError> {
        if !self.fill_enabled {
            return Err(DeveloperError::new(
                "This instance does not represent a filled geometry.",
            ));
        }

        let positions = self.options.positions.clone().unwrap_or_default();
        let width = self.options.width;
        let granularity = self.options.granularity;
        let arc_type = self.options.arc_type;

        let geometry = PolylineGeometry::new(PolylineGeometryOptions {
            positions,
            width,
            colors: None,
            colors_per_vertex: None,
            arc_type,
            granularity,
            vertex_format: None,
            ellipsoid: None,
        });

        let mut attributes = HashMap::new();
        let show_attr = ShowGeometryInstanceAttribute::new(self.entity.is_showing());
        attributes.insert(
            "show".to_string(),
            GeometryInstanceAttribute::new(
                show_attr.component_datatype,
                show_attr.components_per_attribute,
                false,
                vec![show_attr.value[0] as f64],
            )?,
        );

        let color = if let Some(mat_prop) = &self.fill_material_property {
            match mat_prop.get_material_value(Some(&time)) {
                Some(MaterialValue::Color(c)) => c,
                _ => Color::WHITE,
            }
        } else {
            Color::WHITE
        };
        let color_attr = ColorGeometryInstanceAttribute::from_color(color);
        attributes.insert(
            "color".to_string(),
            GeometryInstanceAttribute::new(
                color_attr.component_datatype,
                color_attr.components_per_attribute,
                color_attr.normalize,
                color_attr.value.iter().map(|&v| v as f64).collect(),
            )?,
        );

        Ok(PolylineFillGeometryInstance {
            id: self.entity.id.clone(),
            geometry,
            model_matrix: Matrix4::IDENTITY,
            attributes,
        })
    }
}
