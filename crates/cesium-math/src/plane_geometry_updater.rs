//! A GeometryUpdater for planes matching CesiumJS `DataSources/PlaneGeometryUpdater.js` (Apache-2.0).

use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    Cartesian2, Cartesian3, Color, ColorGeometryInstanceAttribute, DeveloperError, Entity, Event,
    GeometryInstanceAttribute, JulianDate, Matrix3, Matrix4, Plane, PlaneGeometry,
    PlaneOutlineGeometry, ShowGeometryInstanceAttribute,
};

/// Options for configuring a plane geometry instance.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlaneGeometryOptions {
    pub plane: Option<Plane>,
    pub dimensions: Option<Cartesian2>,
}

/// A geometry instance representing the filled portion of a plane geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct PlaneFillGeometryInstance {
    pub id: String,
    pub geometry: PlaneGeometry,
    pub model_matrix: Matrix4,
    pub attributes: HashMap<String, GeometryInstanceAttribute>,
}

/// A geometry instance representing the outline portion of a plane geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct PlaneOutlineGeometryInstance {
    pub id: String,
    pub geometry: PlaneOutlineGeometry,
    pub model_matrix: Matrix4,
    pub attributes: HashMap<String, GeometryInstanceAttribute>,
}

/// Dynamic updater for planes when plane or dimensions change with time.
pub struct DynamicPlaneGeometryUpdater {
    entity: Entity,
    options: PlaneGeometryOptions,
}

impl DynamicPlaneGeometryUpdater {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            options: PlaneGeometryOptions::default(),
        }
    }

    pub fn options(&self) -> &PlaneGeometryOptions {
        &self.options
    }

    pub fn update(&mut self, time: JulianDate) {
        if let Some(plane_graphics) = &self.entity.plane {
            self.options.plane = plane_graphics
                .plane
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.dimensions = plane_graphics
                .dimensions
                .as_ref()
                .and_then(|d| d.get_value(Some(&time)));
        } else {
            self.options.plane = None;
            self.options.dimensions = None;
        }
    }
}

/// A GeometryUpdater for planes.
pub struct PlaneGeometryUpdater {
    entity: Entity,
    fill_enabled: bool,
    outline_enabled: bool,
    is_dynamic: bool,
    options: PlaneGeometryOptions,
    has_valid_geometry: bool,
    pub geometry_changed: Rc<Event<()>>,
}

impl PlaneGeometryUpdater {
    /// Creates a new PlaneGeometryUpdater for an entity.
    pub fn new(entity: Entity, _scene: Option<()>) -> Result<Self, DeveloperError> {
        let geometry_changed = Rc::new(Event::new());
        let mut updater = Self {
            entity: entity.clone(),
            fill_enabled: true,
            outline_enabled: false,
            is_dynamic: false,
            options: PlaneGeometryOptions::default(),
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
        false
    }

    /// Creates the geometry instance which represents the fill of the geometry.
    pub fn create_fill_geometry_instance(
        &self,
        time: JulianDate,
    ) -> Result<PlaneFillGeometryInstance, DeveloperError> {
        if !self.fill_enabled {
            return Err(DeveloperError::new(
                "This instance does not represent a filled geometry.",
            ));
        }

        let plane = self
            .options
            .plane
            .as_ref()
            .ok_or_else(|| DeveloperError::new("Plane is not defined."))?;
        let dimensions = self
            .options
            .dimensions
            .ok_or_else(|| DeveloperError::new("Plane dimensions are not defined."))?;

        let model_matrix = self
            .entity
            .compute_model_matrix(&time)
            .unwrap_or(Matrix4::IDENTITY);
        let primitive_matrix = Self::create_primitive_matrix(plane, dimensions, &model_matrix);

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

        Ok(PlaneFillGeometryInstance {
            id: self.entity.id.clone(),
            geometry: PlaneGeometry::default(),
            model_matrix: primitive_matrix,
            attributes,
        })
    }

    /// Creates the geometry instance which represents the outline of the geometry.
    pub fn create_outline_geometry_instance(
        &self,
        time: JulianDate,
    ) -> Result<PlaneOutlineGeometryInstance, DeveloperError> {
        if !self.outline_enabled {
            return Err(DeveloperError::new(
                "This instance does not represent an outlined geometry.",
            ));
        }

        let plane = self
            .options
            .plane
            .as_ref()
            .ok_or_else(|| DeveloperError::new("Plane is not defined."))?;
        let dimensions = self
            .options
            .dimensions
            .ok_or_else(|| DeveloperError::new("Plane dimensions are not defined."))?;

        let model_matrix = self
            .entity
            .compute_model_matrix(&time)
            .unwrap_or(Matrix4::IDENTITY);
        let primitive_matrix = Self::create_primitive_matrix(plane, dimensions, &model_matrix);

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

        Ok(PlaneOutlineGeometryInstance {
            id: self.entity.id.clone(),
            geometry: PlaneOutlineGeometry,
            model_matrix: primitive_matrix,
            attributes,
        })
    }

    /// Helper to create primitive transform matrix matching CesiumJS `createPrimitiveMatrix`.
    pub fn create_primitive_matrix(
        plane: &Plane,
        dimensions: Cartesian2,
        transform: &Matrix4,
    ) -> Matrix4 {
        let normal = Cartesian3::new(plane.normal.x, plane.normal.y, plane.normal.z);
        let distance = plane.distance;
        let translation = normal * -distance;

        let mut up = Cartesian3::UNIT_Z;
        if ((up.dot(&normal)).abs() - 1.0).abs() < 1e-8 {
            up = Cartesian3::UNIT_Y;
        }

        let mut left = up.cross(&normal);
        up = normal.cross(&left);
        left = left.normalize();
        up = up.normalize();

        let rotation = Matrix3::from_column_major_array([
            left.x, left.y, left.z, up.x, up.y, up.z, normal.x, normal.y, normal.z,
        ]);
        let scale = Cartesian3::new(dimensions.x, dimensions.y, 1.0);
        let rotation_scale = rotation.multiply_by_scale(scale);
        let local_transform = Matrix4::from_rotation_translation(&rotation_scale, translation);
        transform.multiply(&local_transform)
    }

    /// Creates a dynamic updater for this geometry.
    pub fn create_dynamic_updater(
        &self,
        primitives: Option<()>,
        ground_primitives: Option<()>,
    ) -> Result<DynamicPlaneGeometryUpdater, DeveloperError> {
        if primitives.is_none() {
            return Err(DeveloperError::new("primitives is required."));
        }
        if ground_primitives.is_none() {
            return Err(DeveloperError::new("groundPrimitives is required."));
        }
        Ok(DynamicPlaneGeometryUpdater::new(self.entity.clone()))
    }

    /// Callback invoked when an entity property changes.
    pub fn on_entity_property_changed(&mut self, entity: &Entity, property_name: &str) {
        let observed = ["availability", "position", "orientation", "plane"];
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

        let plane_graphics = match &entity.plane {
            Some(p) => p,
            None => {
                self.has_valid_geometry = false;
                self.options.plane = None;
                self.options.dimensions = None;
                return;
            }
        };

        let now = JulianDate::now();
        let mut plane = None;
        let mut plane_is_constant = true;
        if let Some(plane_prop) = &plane_graphics.plane {
            plane = plane_prop.get_value(Some(&now));
            plane_is_constant = plane_prop.is_constant();
        }

        let mut dimensions = None;
        let mut dimensions_is_constant = true;
        if let Some(dim_prop) = &plane_graphics.dimensions {
            dimensions = dim_prop.get_value(Some(&now));
            dimensions_is_constant = dim_prop.is_constant();
        }

        let position_defined = entity.position.is_some();
        let position_is_constant = entity.position.as_ref().is_none_or(|p| p.is_constant());
        let orientation_is_constant = entity
            .orientation
            .as_ref()
            .is_none_or(|o| o.is_constant());
        let outline_width_is_constant = plane_graphics
            .outline_width
            .as_ref()
            .is_none_or(|w| w.is_constant());

        self.has_valid_geometry = plane.is_some() && dimensions.is_some() && position_defined;

        self.is_dynamic = !position_is_constant
            || !orientation_is_constant
            || !plane_is_constant
            || !dimensions_is_constant
            || !outline_width_is_constant;

        if let Some(fill_prop) = &plane_graphics.fill {
            if let Some(val) = fill_prop.get_value(Some(&now)) {
                self.fill_enabled = val;
            }
        }
        if let Some(outline_prop) = &plane_graphics.outline {
            if let Some(val) = outline_prop.get_value(Some(&now)) {
                self.outline_enabled = val;
            }
        }

        self.options.plane = plane;
        self.options.dimensions = dimensions;

        if is_initial && !self.has_valid_geometry {
            // initial empty
        }
    }
}
