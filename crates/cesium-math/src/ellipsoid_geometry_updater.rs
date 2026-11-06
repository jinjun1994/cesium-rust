//! A GeometryUpdater for ellipsoids matching CesiumJS `DataSources/EllipsoidGeometryUpdater.js` (Apache-2.0).

use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    Cartesian3, Color, ColorGeometryInstanceAttribute, DeveloperError, EllipsoidGeometry,
    EllipsoidOutlineGeometry, Entity, Event, GeometryInstanceAttribute, GeometryOffsetAttribute,
    HeightReference, JulianDate, Matrix4, ShowGeometryInstanceAttribute,
};

/// Options for configuring an ellipsoid geometry instance.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EllipsoidGeometryOptions {
    pub radii: Option<Cartesian3>,
    pub inner_radii: Option<Cartesian3>,
    pub minimum_clock: Option<f64>,
    pub maximum_clock: Option<f64>,
    pub minimum_cone: Option<f64>,
    pub maximum_cone: Option<f64>,
    pub stack_partitions: Option<i32>,
    pub slice_partitions: Option<i32>,
    pub subdivisions: Option<i32>,
    pub offset_attribute: Option<GeometryOffsetAttribute>,
}

/// A geometry instance representing the filled portion of an ellipsoid geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct EllipsoidFillGeometryInstance {
    pub id: String,
    pub geometry: EllipsoidGeometry,
    pub model_matrix: Matrix4,
    pub attributes: HashMap<String, GeometryInstanceAttribute>,
}

/// A geometry instance representing the outline portion of an ellipsoid geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct EllipsoidOutlineGeometryInstance {
    pub id: String,
    pub geometry: EllipsoidOutlineGeometry,
    pub model_matrix: Matrix4,
    pub attributes: HashMap<String, GeometryInstanceAttribute>,
}

/// Dynamic updater for ellipsoids when radii, partitions, angles or position change with time.
pub struct DynamicEllipsoidGeometryUpdater {
    entity: Entity,
    options: EllipsoidGeometryOptions,
}

impl DynamicEllipsoidGeometryUpdater {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            options: EllipsoidGeometryOptions::default(),
        }
    }

    pub fn options(&self) -> &EllipsoidGeometryOptions {
        &self.options
    }

    pub fn update(&mut self, time: JulianDate) {
        let mut height_reference = HeightReference::None;
        if let Some(ell) = &self.entity.ellipsoid {
            if let Some(hr_prop) = &ell.height_reference {
                if let Some(hr) = hr_prop.get_value(Some(&time)) {
                    height_reference = hr;
                }
            }
            self.options.radii = ell.radii.as_ref().and_then(|p| p.get_value(Some(&time)));
            self.options.inner_radii = ell
                .inner_radii
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.minimum_clock = ell
                .minimum_clock
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.maximum_clock = ell
                .maximum_clock
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.minimum_cone = ell
                .minimum_cone
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.maximum_cone = ell
                .maximum_cone
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.stack_partitions = ell
                .stack_partitions
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.slice_partitions = ell
                .slice_partitions
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.subdivisions = ell
                .subdivisions
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
        } else {
            self.options = EllipsoidGeometryOptions::default();
        }

        self.options.offset_attribute = match height_reference {
            HeightReference::ClampToGround | HeightReference::RelativeToGround => {
                Some(GeometryOffsetAttribute::All)
            }
            _ => None,
        };
    }
}

/// A GeometryUpdater for ellipsoids.
pub struct EllipsoidGeometryUpdater {
    entity: Entity,
    fill_enabled: bool,
    outline_enabled: bool,
    is_dynamic: bool,
    options: EllipsoidGeometryOptions,
    has_valid_geometry: bool,
    pub geometry_changed: Rc<Event<()>>,
}

impl EllipsoidGeometryUpdater {
    /// Creates a new EllipsoidGeometryUpdater for an entity.
    pub fn new(entity: Entity, _scene: Option<()>) -> Result<Self, DeveloperError> {
        let geometry_changed = Rc::new(Event::new());
        let mut updater = Self {
            entity: entity.clone(),
            fill_enabled: true,
            outline_enabled: false,
            is_dynamic: false,
            options: EllipsoidGeometryOptions::default(),
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

    /// Sets whether fill is enabled.
    pub fn set_fill_enabled(&mut self, enabled: bool) {
        if self.fill_enabled != enabled {
            self.fill_enabled = enabled;
            self.geometry_changed.raise_event(&());
        }
    }

    /// Gets a value indicating if fill visibility varies with simulation time.
    pub fn has_constant_fill(&self) -> bool {
        !self.fill_enabled || self.entity.availability.is_none()
    }

    /// Gets a value indicating if the geometry has an outline component.
    pub fn outline_enabled(&self) -> bool {
        self.outline_enabled
    }

    /// Sets whether outline is enabled.
    pub fn set_outline_enabled(&mut self, enabled: bool) {
        if self.outline_enabled != enabled {
            self.outline_enabled = enabled;
            self.geometry_changed.raise_event(&());
        }
    }

    /// Gets a value indicating if outline visibility varies with simulation time.
    pub fn has_constant_outline(&self) -> bool {
        !self.outline_enabled || self.entity.availability.is_none()
    }

    /// Gets a value indicating if the geometry is time-varying.
    pub fn is_dynamic(&self) -> bool {
        self.is_dynamic
    }

    /// Gets a value indicating if the geometry is closed.
    pub fn is_closed(&self) -> bool {
        true
    }

    /// Computes the center position of the ellipsoid at the specified time.
    pub fn compute_center(&self, time: JulianDate) -> Option<Cartesian3> {
        self.entity
            .position
            .as_ref()
            .and_then(|p| p.get_value(Some(&time)))
    }

    /// Creates the geometry instance which represents the fill of the geometry.
    pub fn create_fill_geometry_instance(
        &self,
        time: JulianDate,
    ) -> Result<EllipsoidFillGeometryInstance, DeveloperError> {
        if !self.fill_enabled {
            return Err(DeveloperError::new(
                "This instance does not represent a filled geometry.",
            ));
        }

        let radii = self
            .options
            .radii
            .ok_or_else(|| DeveloperError::new("radii is not defined."))?;

        let geometry = EllipsoidGeometry::new(
            Some(radii),
            self.options.inner_radii,
            self.options.minimum_clock,
            self.options.maximum_clock,
            self.options.minimum_cone,
            self.options.maximum_cone,
            self.options.stack_partitions,
            self.options.slice_partitions,
            None,
        )
        .with_offset_attribute(self.options.offset_attribute);

        let center = self.compute_center(time).unwrap_or(Cartesian3::ZERO);
        let model_matrix = Matrix4::from_translation(center);

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

        let color_attr = ColorGeometryInstanceAttribute::from_color(Color::WHITE);
        attributes.insert(
            "color".to_string(),
            GeometryInstanceAttribute::new(
                color_attr.component_datatype,
                color_attr.components_per_attribute,
                color_attr.normalize,
                color_attr.value.iter().map(|&v| v as f64).collect(),
            )?,
        );

        Ok(EllipsoidFillGeometryInstance {
            id: self.entity.id.clone(),
            geometry,
            model_matrix,
            attributes,
        })
    }

    /// Creates the geometry instance which represents the outline of the geometry.
    pub fn create_outline_geometry_instance(
        &self,
        time: JulianDate,
    ) -> Result<EllipsoidOutlineGeometryInstance, DeveloperError> {
        if !self.outline_enabled {
            return Err(DeveloperError::new(
                "This instance does not represent an outlined geometry.",
            ));
        }

        let radii = self
            .options
            .radii
            .ok_or_else(|| DeveloperError::new("radii is not defined."))?;

        let geometry = EllipsoidOutlineGeometry::new(
            Some(radii),
            self.options.inner_radii,
            self.options.minimum_clock,
            self.options.maximum_clock,
            self.options.minimum_cone,
            self.options.maximum_cone,
            self.options.stack_partitions,
            self.options.slice_partitions,
            self.options.subdivisions,
        )
        .with_offset_attribute(self.options.offset_attribute);

        let center = self.compute_center(time).unwrap_or(Cartesian3::ZERO);
        let model_matrix = Matrix4::from_translation(center);

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

        let color_attr = ColorGeometryInstanceAttribute::from_color(Color::BLACK);
        attributes.insert(
            "color".to_string(),
            GeometryInstanceAttribute::new(
                color_attr.component_datatype,
                color_attr.components_per_attribute,
                color_attr.normalize,
                color_attr.value.iter().map(|&v| v as f64).collect(),
            )?,
        );

        Ok(EllipsoidOutlineGeometryInstance {
            id: self.entity.id.clone(),
            geometry,
            model_matrix,
            attributes,
        })
    }

    /// Creates a dynamic updater for this geometry.
    pub fn create_dynamic_updater(
        &self,
        primitives: Option<()>,
        ground_primitives: Option<()>,
    ) -> Result<DynamicEllipsoidGeometryUpdater, DeveloperError> {
        if primitives.is_none() {
            return Err(DeveloperError::new("primitives is required."));
        }
        if ground_primitives.is_none() {
            return Err(DeveloperError::new("groundPrimitives is required."));
        }
        Ok(DynamicEllipsoidGeometryUpdater::new(self.entity.clone()))
    }

    /// Callback invoked when an entity property changes.
    pub fn on_entity_property_changed(&mut self, entity: &Entity, property_name: &str) {
        let observed = ["availability", "position", "orientation", "ellipsoid"];
        if !observed.contains(&property_name) {
            return;
        }

        let old_had_valid_geometry = self.has_valid_geometry;
        self.recompute_state(entity, false);

        if old_had_valid_geometry || self.has_valid_geometry {
            self.geometry_changed.raise_event(&());
        }
    }

    fn recompute_state(&mut self, entity: &Entity, is_initial: bool) {
        self.entity = entity.clone();

        let ell = match &entity.ellipsoid {
            Some(e) => e,
            None => {
                self.has_valid_geometry = false;
                self.fill_enabled = false;
                self.outline_enabled = false;
                self.is_dynamic = false;
                self.options = EllipsoidGeometryOptions::default();
                return;
            }
        };

        let now = JulianDate::now();
        let mut radii = None;
        let mut radii_is_constant = true;
        if let Some(prop) = &ell.radii {
            radii = prop.get_value(Some(&now));
            radii_is_constant = prop.is_constant();
        }

        let mut inner_radii = None;
        let mut inner_radii_is_constant = true;
        if let Some(prop) = &ell.inner_radii {
            inner_radii = prop.get_value(Some(&now));
            inner_radii_is_constant = prop.is_constant();
        }

        let mut min_clock = None;
        let mut min_clock_is_constant = true;
        if let Some(prop) = &ell.minimum_clock {
            min_clock = prop.get_value(Some(&now));
            min_clock_is_constant = prop.is_constant();
        }

        let mut max_clock = None;
        let mut max_clock_is_constant = true;
        if let Some(prop) = &ell.maximum_clock {
            max_clock = prop.get_value(Some(&now));
            max_clock_is_constant = prop.is_constant();
        }

        let mut min_cone = None;
        let mut min_cone_is_constant = true;
        if let Some(prop) = &ell.minimum_cone {
            min_cone = prop.get_value(Some(&now));
            min_cone_is_constant = prop.is_constant();
        }

        let mut max_cone = None;
        let mut max_cone_is_constant = true;
        if let Some(prop) = &ell.maximum_cone {
            max_cone = prop.get_value(Some(&now));
            max_cone_is_constant = prop.is_constant();
        }

        let mut stack_partitions = None;
        let mut stack_partitions_is_constant = true;
        if let Some(prop) = &ell.stack_partitions {
            stack_partitions = prop.get_value(Some(&now));
            stack_partitions_is_constant = prop.is_constant();
        }

        let mut slice_partitions = None;
        let mut slice_partitions_is_constant = true;
        if let Some(prop) = &ell.slice_partitions {
            slice_partitions = prop.get_value(Some(&now));
            slice_partitions_is_constant = prop.is_constant();
        }

        let mut subdivisions = None;
        let mut subdivisions_is_constant = true;
        if let Some(prop) = &ell.subdivisions {
            subdivisions = prop.get_value(Some(&now));
            subdivisions_is_constant = prop.is_constant();
        }

        let position_defined = entity.position.is_some();
        let position_is_constant = entity.position.as_ref().is_none_or(|p| p.is_constant());
        let orientation_is_constant = entity
            .orientation
            .as_ref()
            .is_none_or(|o| o.is_constant());
        let outline_width_is_constant = ell
            .outline_width
            .as_ref()
            .is_none_or(|w| w.is_constant());

        let has_geometry = radii.is_some() && position_defined;
        self.has_valid_geometry = has_geometry;

        if !has_geometry {
            self.fill_enabled = false;
            self.outline_enabled = false;
            self.is_dynamic = false;
            self.options = EllipsoidGeometryOptions::default();
            return;
        }

        self.is_dynamic = !position_is_constant
            || !orientation_is_constant
            || !radii_is_constant
            || !inner_radii_is_constant
            || !min_clock_is_constant
            || !max_clock_is_constant
            || !min_cone_is_constant
            || !max_cone_is_constant
            || !stack_partitions_is_constant
            || !slice_partitions_is_constant
            || !subdivisions_is_constant
            || !outline_width_is_constant;

        self.fill_enabled = true;
        if let Some(fill_prop) = &ell.fill {
            if let Some(val) = fill_prop.get_value(Some(&now)) {
                self.fill_enabled = val;
            }
        }

        self.outline_enabled = false;
        if let Some(outline_prop) = &ell.outline {
            if let Some(val) = outline_prop.get_value(Some(&now)) {
                self.outline_enabled = val;
            }
        }

        let mut height_reference = HeightReference::None;
        if let Some(hr_prop) = &ell.height_reference {
            if let Some(val) = hr_prop.get_value(Some(&now)) {
                height_reference = val;
            }
        }

        self.options.radii = radii;
        self.options.inner_radii = inner_radii;
        self.options.minimum_clock = min_clock;
        self.options.maximum_clock = max_clock;
        self.options.minimum_cone = min_cone;
        self.options.maximum_cone = max_cone;
        self.options.stack_partitions = stack_partitions;
        self.options.slice_partitions = slice_partitions;
        self.options.subdivisions = subdivisions;
        self.options.offset_attribute = match height_reference {
            HeightReference::ClampToGround | HeightReference::RelativeToGround => {
                Some(GeometryOffsetAttribute::All)
            }
            _ => None,
        };

        if is_initial && !self.has_valid_geometry {
            // initial empty
        }
    }
}
