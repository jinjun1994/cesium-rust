//! A GeometryUpdater for cylinders matching CesiumJS `DataSources/CylinderGeometryUpdater.js` (Apache-2.0).

use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    Cartesian3, Color, ColorGeometryInstanceAttribute, CylinderGeometry, CylinderOutlineGeometry,
    DeveloperError, Entity, Event, GeometryInstanceAttribute, GeometryOffsetAttribute,
    HeightReference, JulianDate, Matrix4, ShowGeometryInstanceAttribute,
};

/// Options for configuring a cylinder geometry instance.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CylinderGeometryOptions {
    pub length: Option<f64>,
    pub top_radius: Option<f64>,
    pub bottom_radius: Option<f64>,
    pub number_of_vertical_lines: Option<u32>,
    pub slices: Option<u32>,
    pub offset_attribute: Option<GeometryOffsetAttribute>,
}

/// A geometry instance representing the filled portion of a cylinder geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct CylinderFillGeometryInstance {
    pub id: String,
    pub geometry: CylinderGeometry,
    pub model_matrix: Matrix4,
    pub attributes: HashMap<String, GeometryInstanceAttribute>,
}

/// A geometry instance representing the outline portion of a cylinder geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct CylinderOutlineGeometryInstance {
    pub id: String,
    pub geometry: CylinderOutlineGeometry,
    pub model_matrix: Matrix4,
    pub attributes: HashMap<String, GeometryInstanceAttribute>,
}

/// Dynamic updater for cylinders when dimensions or position change with time.
pub struct DynamicCylinderGeometryUpdater {
    entity: Entity,
    options: CylinderGeometryOptions,
}

impl DynamicCylinderGeometryUpdater {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            options: CylinderGeometryOptions::default(),
        }
    }

    pub fn options(&self) -> &CylinderGeometryOptions {
        &self.options
    }

    pub fn update(&mut self, time: JulianDate) {
        let mut height_reference = HeightReference::None;
        if let Some(cyl) = &self.entity.cylinder {
            if let Some(hr_prop) = &cyl.height_reference {
                if let Some(hr) = hr_prop.get_value(Some(&time)) {
                    height_reference = hr;
                }
            }
            self.options.length = cyl.length.as_ref().and_then(|p| p.get_value(Some(&time)));
            self.options.top_radius = cyl
                .top_radius
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.bottom_radius = cyl
                .bottom_radius
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.number_of_vertical_lines = cyl
                .number_of_vertical_lines
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)))
                .map(|v| v.max(0) as u32);
            self.options.slices = cyl
                .slices
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)))
                .map(|v| v.max(0) as u32);
        } else {
            self.options.length = None;
            self.options.top_radius = None;
            self.options.bottom_radius = None;
            self.options.number_of_vertical_lines = None;
            self.options.slices = None;
        }

        self.options.offset_attribute = match height_reference {
            HeightReference::ClampToGround | HeightReference::RelativeToGround => {
                Some(GeometryOffsetAttribute::All)
            }
            _ => None,
        };
    }
}

/// A GeometryUpdater for cylinders.
pub struct CylinderGeometryUpdater {
    entity: Entity,
    fill_enabled: bool,
    outline_enabled: bool,
    is_dynamic: bool,
    options: CylinderGeometryOptions,
    has_valid_geometry: bool,
    pub geometry_changed: Rc<Event<()>>,
}

impl CylinderGeometryUpdater {
    /// Creates a new CylinderGeometryUpdater for an entity.
    pub fn new(entity: Entity, _scene: Option<()>) -> Result<Self, DeveloperError> {
        let geometry_changed = Rc::new(Event::new());
        let mut updater = Self {
            entity: entity.clone(),
            fill_enabled: true,
            outline_enabled: false,
            is_dynamic: false,
            options: CylinderGeometryOptions::default(),
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

    /// Computes the center position of the cylinder at the specified time.
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
    ) -> Result<CylinderFillGeometryInstance, DeveloperError> {
        if !self.fill_enabled {
            return Err(DeveloperError::new(
                "This instance does not represent a filled geometry.",
            ));
        }

        let length = self
            .options
            .length
            .ok_or_else(|| DeveloperError::new("Cylinder length is not defined."))?;
        let top_radius = self
            .options
            .top_radius
            .ok_or_else(|| DeveloperError::new("Cylinder topRadius is not defined."))?;
        let bottom_radius = self
            .options
            .bottom_radius
            .ok_or_else(|| DeveloperError::new("Cylinder bottomRadius is not defined."))?;

        let geometry = CylinderGeometry::new(
            length,
            top_radius,
            bottom_radius,
            self.options.slices,
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

        Ok(CylinderFillGeometryInstance {
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
    ) -> Result<CylinderOutlineGeometryInstance, DeveloperError> {
        if !self.outline_enabled {
            return Err(DeveloperError::new(
                "This instance does not represent an outlined geometry.",
            ));
        }

        let length = self
            .options
            .length
            .ok_or_else(|| DeveloperError::new("Cylinder length is not defined."))?;
        let top_radius = self
            .options
            .top_radius
            .ok_or_else(|| DeveloperError::new("Cylinder topRadius is not defined."))?;
        let bottom_radius = self
            .options
            .bottom_radius
            .ok_or_else(|| DeveloperError::new("Cylinder bottomRadius is not defined."))?;

        let geometry = CylinderOutlineGeometry::new(
            length,
            top_radius,
            bottom_radius,
            self.options.slices,
            self.options.number_of_vertical_lines,
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

        Ok(CylinderOutlineGeometryInstance {
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
    ) -> Result<DynamicCylinderGeometryUpdater, DeveloperError> {
        if primitives.is_none() {
            return Err(DeveloperError::new("primitives is required."));
        }
        if ground_primitives.is_none() {
            return Err(DeveloperError::new("groundPrimitives is required."));
        }
        Ok(DynamicCylinderGeometryUpdater::new(self.entity.clone()))
    }

    /// Callback invoked when an entity property changes.
    pub fn on_entity_property_changed(&mut self, entity: &Entity, property_name: &str) {
        let observed = ["availability", "position", "orientation", "cylinder"];
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

        let cylinder = match &entity.cylinder {
            Some(c) => c,
            None => {
                self.has_valid_geometry = false;
                self.fill_enabled = false;
                self.outline_enabled = false;
                self.is_dynamic = false;
                self.options = CylinderGeometryOptions::default();
                return;
            }
        };

        let now = JulianDate::now();
        let mut length = None;
        let mut length_is_constant = true;
        if let Some(l_prop) = &cylinder.length {
            length = l_prop.get_value(Some(&now));
            length_is_constant = l_prop.is_constant();
        }

        let mut top_radius = None;
        let mut top_radius_is_constant = true;
        if let Some(tr_prop) = &cylinder.top_radius {
            top_radius = tr_prop.get_value(Some(&now));
            top_radius_is_constant = tr_prop.is_constant();
        }

        let mut bottom_radius = None;
        let mut bottom_radius_is_constant = true;
        if let Some(br_prop) = &cylinder.bottom_radius {
            bottom_radius = br_prop.get_value(Some(&now));
            bottom_radius_is_constant = br_prop.is_constant();
        }

        let mut num_lines = None;
        let mut num_lines_is_constant = true;
        if let Some(nl_prop) = &cylinder.number_of_vertical_lines {
            num_lines = nl_prop.get_value(Some(&now)).map(|v| v.max(0) as u32);
            num_lines_is_constant = nl_prop.is_constant();
        }

        let mut slices = None;
        if let Some(s_prop) = &cylinder.slices {
            slices = s_prop.get_value(Some(&now)).map(|v| v.max(0) as u32);
        }

        let position_defined = entity.position.is_some();
        let position_is_constant = entity.position.as_ref().is_none_or(|p| p.is_constant());
        let orientation_is_constant = entity
            .orientation
            .as_ref()
            .is_none_or(|o| o.is_constant());
        let outline_width_is_constant = cylinder
            .outline_width
            .as_ref()
            .is_none_or(|w| w.is_constant());

        let has_geometry = length.is_some()
            && top_radius.is_some()
            && bottom_radius.is_some()
            && position_defined;

        self.has_valid_geometry = has_geometry;

        if !has_geometry {
            self.fill_enabled = false;
            self.outline_enabled = false;
            self.is_dynamic = false;
            self.options = CylinderGeometryOptions::default();
            return;
        }

        self.is_dynamic = !position_is_constant
            || !orientation_is_constant
            || !length_is_constant
            || !top_radius_is_constant
            || !bottom_radius_is_constant
            || !num_lines_is_constant
            || !outline_width_is_constant;

        self.fill_enabled = true;
        if let Some(fill_prop) = &cylinder.fill {
            if let Some(val) = fill_prop.get_value(Some(&now)) {
                self.fill_enabled = val;
            }
        }

        self.outline_enabled = false;
        if let Some(outline_prop) = &cylinder.outline {
            if let Some(val) = outline_prop.get_value(Some(&now)) {
                self.outline_enabled = val;
            }
        }

        let mut height_reference = HeightReference::None;
        if let Some(hr_prop) = &cylinder.height_reference {
            if let Some(val) = hr_prop.get_value(Some(&now)) {
                height_reference = val;
            }
        }

        self.options.length = length;
        self.options.top_radius = top_radius;
        self.options.bottom_radius = bottom_radius;
        self.options.number_of_vertical_lines = num_lines;
        self.options.slices = slices;
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
