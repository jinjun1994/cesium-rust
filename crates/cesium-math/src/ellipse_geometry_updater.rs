//! A GeometryUpdater for ellipses matching CesiumJS `DataSources/EllipseGeometryUpdater.js` (Apache-2.0).

use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    Cartesian3, Color, ColorGeometryInstanceAttribute, DeveloperError, EllipseGeometry,
    EllipseOutlineGeometry, Entity, Event, GeometryInstanceAttribute, GeometryOffsetAttribute,
    JulianDate, Matrix4, ShowGeometryInstanceAttribute,
};

/// Options for configuring an ellipse geometry instance.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EllipseGeometryOptions {
    pub semi_major_axis: Option<f64>,
    pub semi_minor_axis: Option<f64>,
    pub height: Option<f64>,
    pub extruded_height: Option<f64>,
    pub rotation: Option<f64>,
    pub st_rotation: Option<f64>,
    pub granularity: Option<f64>,
    pub number_of_vertical_lines: Option<usize>,
    pub offset_attribute: Option<GeometryOffsetAttribute>,
}

/// A geometry instance representing the filled portion of an ellipse geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct EllipseFillGeometryInstance {
    pub id: String,
    pub geometry: EllipseGeometry,
    pub model_matrix: Matrix4,
    pub attributes: HashMap<String, GeometryInstanceAttribute>,
}

/// A geometry instance representing the outline portion of an ellipse geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct EllipseOutlineGeometryInstance {
    pub id: String,
    pub geometry: EllipseOutlineGeometry,
    pub model_matrix: Matrix4,
    pub attributes: HashMap<String, GeometryInstanceAttribute>,
}

/// Dynamic updater for ellipses when axes, heights or position change with time.
pub struct DynamicEllipseGeometryUpdater {
    entity: Entity,
    options: EllipseGeometryOptions,
}

impl DynamicEllipseGeometryUpdater {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            options: EllipseGeometryOptions::default(),
        }
    }

    pub fn options(&self) -> &EllipseGeometryOptions {
        &self.options
    }

    pub fn update(&mut self, time: JulianDate) {
        if let Some(ellipse) = &self.entity.ellipse {
            self.options.semi_major_axis = ellipse
                .semi_major_axis
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.semi_minor_axis = ellipse
                .semi_minor_axis
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.height = ellipse
                .height
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.extruded_height = ellipse
                .extruded_height
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.rotation = ellipse
                .rotation
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.st_rotation = ellipse
                .st_rotation
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.granularity = ellipse
                .granularity
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.number_of_vertical_lines = ellipse
                .number_of_vertical_lines
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)))
                .map(|v| v.max(0) as usize);
        } else {
            self.options = EllipseGeometryOptions::default();
        }
    }
}

/// A GeometryUpdater for ellipses.
pub struct EllipseGeometryUpdater {
    entity: Entity,
    fill_enabled: bool,
    outline_enabled: bool,
    is_dynamic: bool,
    is_closed: bool,
    options: EllipseGeometryOptions,
    has_valid_geometry: bool,
    pub geometry_changed: Rc<Event<()>>,
}

impl EllipseGeometryUpdater {
    /// Creates a new EllipseGeometryUpdater for an entity.
    pub fn new(entity: Entity, _scene: Option<()>) -> Result<Self, DeveloperError> {
        let geometry_changed = Rc::new(Event::new());
        let mut updater = Self {
            entity: entity.clone(),
            fill_enabled: true,
            outline_enabled: false,
            is_dynamic: false,
            is_closed: false,
            options: EllipseGeometryOptions::default(),
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
        self.is_closed
    }

    /// Computes the center position of the ellipse at the specified time.
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
    ) -> Result<EllipseFillGeometryInstance, DeveloperError> {
        if !self.fill_enabled {
            return Err(DeveloperError::new(
                "This instance does not represent a filled geometry.",
            ));
        }

        let center = self
            .compute_center(time)
            .ok_or_else(|| DeveloperError::new("Center position is not defined."))?;
        let semi_major_axis = self
            .options
            .semi_major_axis
            .ok_or_else(|| DeveloperError::new("semiMajorAxis is not defined."))?;
        let semi_minor_axis = self
            .options
            .semi_minor_axis
            .ok_or_else(|| DeveloperError::new("semiMinorAxis is not defined."))?;

        let geometry = EllipseGeometry::new(
            center,
            semi_major_axis,
            semi_minor_axis,
            None,
            self.options.height,
            self.options.extruded_height,
            self.options.granularity,
            self.options.rotation,
            self.options.st_rotation,
            None,
        )
        .with_offset_attribute(self.options.offset_attribute);

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

        Ok(EllipseFillGeometryInstance {
            id: self.entity.id.clone(),
            geometry,
            model_matrix: Matrix4::IDENTITY,
            attributes,
        })
    }

    /// Creates the geometry instance which represents the outline of the geometry.
    pub fn create_outline_geometry_instance(
        &self,
        time: JulianDate,
    ) -> Result<EllipseOutlineGeometryInstance, DeveloperError> {
        if !self.outline_enabled {
            return Err(DeveloperError::new(
                "This instance does not represent an outlined geometry.",
            ));
        }

        let center = self
            .compute_center(time)
            .ok_or_else(|| DeveloperError::new("Center position is not defined."))?;
        let semi_major_axis = self
            .options
            .semi_major_axis
            .ok_or_else(|| DeveloperError::new("semiMajorAxis is not defined."))?;
        let semi_minor_axis = self
            .options
            .semi_minor_axis
            .ok_or_else(|| DeveloperError::new("semiMinorAxis is not defined."))?;

        let geometry = EllipseOutlineGeometry::new(
            center,
            semi_major_axis,
            semi_minor_axis,
            None,
            self.options.height,
            self.options.extruded_height,
            self.options.granularity,
            self.options.rotation,
            self.options.number_of_vertical_lines,
        )
        .with_offset_attribute(self.options.offset_attribute);

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

        Ok(EllipseOutlineGeometryInstance {
            id: self.entity.id.clone(),
            geometry,
            model_matrix: Matrix4::IDENTITY,
            attributes,
        })
    }

    /// Creates a dynamic updater for this geometry.
    pub fn create_dynamic_updater(
        &self,
        primitives: Option<()>,
        ground_primitives: Option<()>,
    ) -> Result<DynamicEllipseGeometryUpdater, DeveloperError> {
        if primitives.is_none() {
            return Err(DeveloperError::new("primitives is required."));
        }
        if ground_primitives.is_none() {
            return Err(DeveloperError::new("groundPrimitives is required."));
        }
        Ok(DynamicEllipseGeometryUpdater::new(self.entity.clone()))
    }

    /// Callback invoked when an entity property changes.
    pub fn on_entity_property_changed(&mut self, entity: &Entity, property_name: &str) {
        let observed = ["availability", "position", "orientation", "ellipse", "isShowing", "show"];
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

        let ellipse = match &entity.ellipse {
            Some(e) => e,
            None => {
                self.has_valid_geometry = false;
                self.fill_enabled = false;
                self.outline_enabled = false;
                self.is_dynamic = false;
                self.is_closed = false;
                self.options = EllipseGeometryOptions::default();
                return;
            }
        };

        let now = JulianDate::now();
        let mut semi_major_axis = None;
        let mut semi_major_is_constant = true;
        if let Some(prop) = &ellipse.semi_major_axis {
            semi_major_axis = prop.get_value(Some(&now));
            semi_major_is_constant = prop.is_constant();
        }

        let mut semi_minor_axis = None;
        let mut semi_minor_is_constant = true;
        if let Some(prop) = &ellipse.semi_minor_axis {
            semi_minor_axis = prop.get_value(Some(&now));
            semi_minor_is_constant = prop.is_constant();
        }

        let mut height = None;
        let mut height_is_constant = true;
        if let Some(prop) = &ellipse.height {
            height = prop.get_value(Some(&now));
            height_is_constant = prop.is_constant();
        }

        let mut extruded_height = None;
        let mut extruded_height_is_constant = true;
        if let Some(prop) = &ellipse.extruded_height {
            extruded_height = prop.get_value(Some(&now));
            extruded_height_is_constant = prop.is_constant();
        }

        let mut rotation = None;
        let mut rotation_is_constant = true;
        if let Some(prop) = &ellipse.rotation {
            rotation = prop.get_value(Some(&now));
            rotation_is_constant = prop.is_constant();
        }

        let mut st_rotation = None;
        let mut st_rotation_is_constant = true;
        if let Some(prop) = &ellipse.st_rotation {
            st_rotation = prop.get_value(Some(&now));
            st_rotation_is_constant = prop.is_constant();
        }

        let mut granularity = None;
        let mut granularity_is_constant = true;
        if let Some(prop) = &ellipse.granularity {
            granularity = prop.get_value(Some(&now));
            granularity_is_constant = prop.is_constant();
        }

        let mut num_lines = None;
        let mut num_lines_is_constant = true;
        if let Some(prop) = &ellipse.number_of_vertical_lines {
            num_lines = prop.get_value(Some(&now)).map(|v| v.max(0) as usize);
            num_lines_is_constant = prop.is_constant();
        }

        let position_defined = entity.position.is_some();
        let position_is_constant = entity.position.as_ref().is_none_or(|p| p.is_constant());
        let orientation_is_constant = entity
            .orientation
            .as_ref()
            .is_none_or(|o| o.is_constant());
        let outline_width_is_constant = ellipse
            .outline_width
            .as_ref()
            .is_none_or(|w| w.is_constant());

        let has_geometry = semi_major_axis.is_some() && semi_minor_axis.is_some() && position_defined;

        self.has_valid_geometry = has_geometry;

        if !has_geometry {
            self.fill_enabled = false;
            self.outline_enabled = false;
            self.is_dynamic = false;
            self.is_closed = false;
            self.options = EllipseGeometryOptions::default();
            return;
        }

        self.is_dynamic = !position_is_constant
            || !orientation_is_constant
            || !semi_major_is_constant
            || !semi_minor_is_constant
            || !height_is_constant
            || !extruded_height_is_constant
            || !rotation_is_constant
            || !st_rotation_is_constant
            || !granularity_is_constant
            || !num_lines_is_constant
            || !outline_width_is_constant;

        self.is_closed = extruded_height.is_some() && extruded_height != height;

        self.fill_enabled = true;
        if let Some(fill_prop) = &ellipse.fill {
            if let Some(val) = fill_prop.get_value(Some(&now)) {
                self.fill_enabled = val;
            }
        }

        self.outline_enabled = false;
        if let Some(outline_prop) = &ellipse.outline {
            if let Some(val) = outline_prop.get_value(Some(&now)) {
                self.outline_enabled = val;
            }
        }

        self.options.semi_major_axis = semi_major_axis;
        self.options.semi_minor_axis = semi_minor_axis;
        self.options.height = height;
        self.options.extruded_height = extruded_height;
        self.options.rotation = rotation;
        self.options.st_rotation = st_rotation;
        self.options.granularity = granularity;
        self.options.number_of_vertical_lines = num_lines;
        self.options.offset_attribute = None;

        if is_initial && !self.has_valid_geometry {
            // initial empty
        }
    }
}
