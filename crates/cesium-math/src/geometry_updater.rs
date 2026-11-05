//! An abstract class and base implementation for updating geometry entities.
//! Direct port of CesiumJS `DataSources/GeometryUpdater.js`.

use crate::{DeveloperError, Entity, Event};

/// Options for initializing a GeometryUpdater instance.
#[derive(Debug, Clone, Default)]
pub struct GeometryUpdaterOptions {
    pub entity: Option<Entity>,
    pub geometry_property_name: Option<String>,
    pub observed_property_names: Option<Vec<String>>,
}

/// An abstract base for updating geometry entities.
#[derive(Clone)]
pub struct GeometryUpdater {
    id: String,
    entity: Entity,
    geometry_property_name: String,
    observed_property_names: Vec<String>,
    fill_enabled: bool,
    outline_enabled: bool,
    dynamic: bool,
    is_closed: bool,
    outline_width: f64,
    pub geometry_changed: Event<()>,
}

impl GeometryUpdater {
    /// Creates a new GeometryUpdater with validated options.
    pub fn new(options: Option<GeometryUpdaterOptions>) -> Result<Self, DeveloperError> {
        let opts = options.ok_or_else(|| DeveloperError::new("options is required."))?;
        let entity = opts
            .entity
            .ok_or_else(|| DeveloperError::new("options.entity is required."))?;
        let geometry_property_name = opts
            .geometry_property_name
            .filter(|s| !s.is_empty())
            .ok_or_else(|| DeveloperError::new("options.geometryPropertyName is required."))?;
        let observed_property_names = opts
            .observed_property_names
            .ok_or_else(|| DeveloperError::new("options.observedPropertyNames is required."))?;

        let id = format!("{}-{}", geometry_property_name, entity.id);

        Ok(Self {
            id,
            entity,
            geometry_property_name,
            observed_property_names,
            fill_enabled: false,
            outline_enabled: false,
            dynamic: false,
            is_closed: false,
            outline_width: 1.0,
            geometry_changed: Event::new(),
        })
    }

    /// Gets the unique ID associated with this updater.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Gets the entity associated with this geometry.
    pub fn entity(&self) -> &Entity {
        &self.entity
    }

    /// Gets the geometry property name.
    pub fn geometry_property_name(&self) -> &str {
        &self.geometry_property_name
    }

    /// Gets the observed property names.
    pub fn observed_property_names(&self) -> &[String] {
        &self.observed_property_names
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

    /// Gets the outline width in pixels.
    pub fn outline_width(&self) -> f64 {
        self.outline_width
    }

    /// Gets a value indicating if the geometry is time-varying.
    pub fn is_dynamic(&self) -> bool {
        self.dynamic
    }

    /// Sets whether the geometry is time-varying.
    pub fn set_dynamic(&mut self, dynamic: bool) {
        self.dynamic = dynamic;
    }

    /// Gets a value indicating if the geometry is closed.
    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    /// Sets whether the geometry is closed.
    pub fn set_is_closed(&mut self, is_closed: bool) {
        self.is_closed = is_closed;
    }

    /// Validates dynamic updater creation preconditions.
    pub fn create_dynamic_updater(
        &self,
        primitives: Option<()>,
        ground_primitives: Option<()>,
    ) -> Result<(), DeveloperError> {
        if !self.dynamic {
            return Err(DeveloperError::new(
                "This geometry updater is not dynamic.",
            ));
        }
        if primitives.is_none() {
            return Err(DeveloperError::new("primitives is required."));
        }
        if ground_primitives.is_none() {
            return Err(DeveloperError::new("groundPrimitives is required."));
        }
        Ok(())
    }
}
