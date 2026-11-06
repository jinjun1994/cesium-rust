//! A GeometryUpdater for boxes matching CesiumJS `DataSources/BoxGeometryUpdater.js` (Apache-2.0).

use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    BoxGeometry, BoxOutlineGeometry, Cartesian3, Color, ColorGeometryInstanceAttribute,
    DeveloperError, Entity, Event, GeometryInstanceAttribute, GeometryOffsetAttribute,
    HeightReference, JulianDate, Matrix4, ShowGeometryInstanceAttribute,
};

/// Options for configuring a box geometry instance.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BoxGeometryOptions {
    pub dimensions: Option<Cartesian3>,
    pub offset_attribute: Option<GeometryOffsetAttribute>,
}

/// A geometry instance representing the filled portion of a box geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct BoxFillGeometryInstance {
    pub id: String,
    pub geometry: BoxGeometry,
    pub model_matrix: Matrix4,
    pub attributes: HashMap<String, GeometryInstanceAttribute>,
}

/// A geometry instance representing the outline portion of a box geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct BoxOutlineGeometryInstance {
    pub id: String,
    pub geometry: BoxOutlineGeometry,
    pub model_matrix: Matrix4,
    pub attributes: HashMap<String, GeometryInstanceAttribute>,
}

/// Dynamic updater for boxes when dimensions or position change with time.
pub struct DynamicBoxGeometryUpdater {
    entity: Entity,
    options: BoxGeometryOptions,
}

impl DynamicBoxGeometryUpdater {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            options: BoxGeometryOptions::default(),
        }
    }

    pub fn options(&self) -> &BoxGeometryOptions {
        &self.options
    }

    pub fn update(&mut self, time: JulianDate) {
        let mut height_reference = HeightReference::None;
        if let Some(box_graphics) = &self.entity.box_graphics {
            if let Some(hr_prop) = &box_graphics.height_reference {
                if let Some(hr) = hr_prop.get_value(Some(&time)) {
                    height_reference = hr;
                }
            }
            if let Some(dims_prop) = &box_graphics.dimensions {
                self.options.dimensions = dims_prop.get_value(Some(&time));
            } else {
                self.options.dimensions = None;
            }
        } else {
            self.options.dimensions = None;
        }

        self.options.offset_attribute = match height_reference {
            HeightReference::ClampToGround | HeightReference::RelativeToGround => {
                Some(GeometryOffsetAttribute::All)
            }
            _ => None,
        };
    }
}

/// A GeometryUpdater for boxes.
pub struct BoxGeometryUpdater {
    entity: Entity,
    fill_enabled: bool,
    outline_enabled: bool,
    is_dynamic: bool,
    is_closed: bool,
    options: BoxGeometryOptions,
    has_valid_geometry: bool,
    pub geometry_changed: Rc<Event<()>>,
}

impl BoxGeometryUpdater {
    /// Creates a new BoxGeometryUpdater for an entity.
    pub fn new(entity: Entity, _scene: Option<()>) -> Result<Self, DeveloperError> {
        let geometry_changed = Rc::new(Event::new());
        let mut updater = Self {
            entity: entity.clone(),
            fill_enabled: true,
            outline_enabled: false,
            is_dynamic: false,
            is_closed: true,
            options: BoxGeometryOptions::default(),
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

    /// Computes the center position of the box at the specified time.
    pub fn compute_center(&self, time: JulianDate) -> Option<Cartesian3> {
        self.entity.position.as_ref().and_then(|p| p.get_value(Some(&time)))
    }

    /// Creates the geometry instance which represents the fill of the geometry.
    pub fn create_fill_geometry_instance(
        &self,
        time: JulianDate,
    ) -> Result<BoxFillGeometryInstance, DeveloperError> {
        if !self.fill_enabled {
            return Err(DeveloperError::new(
                "This instance does not represent a filled geometry.",
            ));
        }

        let dimensions = self
            .options
            .dimensions
            .ok_or_else(|| DeveloperError::new("Box dimensions are not defined."))?;

        let geometry = BoxGeometry::from_dimensions(dimensions, None)
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

        Ok(BoxFillGeometryInstance {
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
    ) -> Result<BoxOutlineGeometryInstance, DeveloperError> {
        if !self.outline_enabled {
            return Err(DeveloperError::new(
                "This instance does not represent an outlined geometry.",
            ));
        }

        let dimensions = self
            .options
            .dimensions
            .ok_or_else(|| DeveloperError::new("Box dimensions are not defined."))?;

        let geometry = BoxOutlineGeometry::from_dimensions(dimensions)
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

        Ok(BoxOutlineGeometryInstance {
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
    ) -> Result<DynamicBoxGeometryUpdater, DeveloperError> {
        if primitives.is_none() {
            return Err(DeveloperError::new("primitives is required."));
        }
        if ground_primitives.is_none() {
            return Err(DeveloperError::new("groundPrimitives is required."));
        }
        Ok(DynamicBoxGeometryUpdater::new(self.entity.clone()))
    }

    /// Callback invoked when an entity property changes.
    pub fn on_entity_property_changed(&mut self, entity: &Entity, property_name: &str) {
        let observed = ["availability", "position", "orientation", "box"];
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

        let box_graphics = match &entity.box_graphics {
            Some(b) => b,
            None => {
                self.has_valid_geometry = false;
                self.options.dimensions = None;
                return;
            }
        };

        let now = JulianDate::now();
        let mut dimensions = None;
        let mut dimensions_is_constant = true;
        if let Some(dim_prop) = &box_graphics.dimensions {
            dimensions = dim_prop.get_value(Some(&now));
            dimensions_is_constant = dim_prop.is_constant();
        }

        let position_defined = entity.position.is_some();
        let position_is_constant = entity.position.as_ref().is_none_or(|p| p.is_constant());
        let orientation_is_constant = entity
            .orientation
            .as_ref()
            .is_none_or(|o| o.is_constant());
        let outline_width_is_constant = box_graphics
            .outline_width
            .as_ref()
            .is_none_or(|w| w.is_constant());

        self.has_valid_geometry = dimensions.is_some() && position_defined;

        self.is_dynamic = !position_is_constant
            || !orientation_is_constant
            || !dimensions_is_constant
            || !outline_width_is_constant;

        if let Some(fill_prop) = &box_graphics.fill {
            if let Some(val) = fill_prop.get_value(Some(&now)) {
                self.fill_enabled = val;
            }
        }
        if let Some(outline_prop) = &box_graphics.outline {
            if let Some(val) = outline_prop.get_value(Some(&now)) {
                self.outline_enabled = val;
            }
        }

        let mut height_reference = HeightReference::None;
        if let Some(hr_prop) = &box_graphics.height_reference {
            if let Some(val) = hr_prop.get_value(Some(&now)) {
                height_reference = val;
            }
        }

        self.options.dimensions = dimensions;
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
