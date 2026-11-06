//! A GeometryUpdater for walls matching CesiumJS `DataSources/WallGeometryUpdater.js` (Apache-2.0).

use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    Cartesian3, Color, ColorGeometryInstanceAttribute, DeveloperError, Entity, Event,
    GeometryInstanceAttribute, JulianDate, MaterialValue, Matrix4,
    ShowGeometryInstanceAttribute, WallGeometry, WallOutlineGeometry,
};

/// Options for configuring a wall geometry instance.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WallUpdaterOptions {
    pub positions: Option<Vec<Cartesian3>>,
    pub minimum_heights: Option<Vec<f64>>,
    pub maximum_heights: Option<Vec<f64>>,
    pub granularity: Option<f64>,
}

/// A geometry instance representing the filled portion of a wall geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct WallFillGeometryInstance {
    pub id: String,
    pub geometry: WallGeometry,
    pub model_matrix: Matrix4,
    pub attributes: HashMap<String, GeometryInstanceAttribute>,
}

/// A geometry instance representing the outline portion of a wall geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct WallOutlineGeometryInstance {
    pub id: String,
    pub geometry: WallOutlineGeometry,
    pub model_matrix: Matrix4,
    pub attributes: HashMap<String, GeometryInstanceAttribute>,
}

/// Dynamic updater for walls when positions, heights or granularity change with time.
pub struct DynamicWallGeometryUpdater {
    entity: Entity,
    options: WallUpdaterOptions,
}

impl DynamicWallGeometryUpdater {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            options: WallUpdaterOptions::default(),
        }
    }

    pub fn options(&self) -> &WallUpdaterOptions {
        &self.options
    }

    pub fn update(&mut self, time: JulianDate) {
        if let Some(wall) = &self.entity.wall {
            self.options.positions = wall
                .positions
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.minimum_heights = wall
                .minimum_heights
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.maximum_heights = wall
                .maximum_heights
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.granularity = wall
                .granularity
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
        } else {
            self.options = WallUpdaterOptions::default();
        }
    }
}

/// A GeometryUpdater for walls.
pub struct WallGeometryUpdater {
    entity: Entity,
    fill_enabled: bool,
    outline_enabled: bool,
    is_dynamic: bool,
    options: WallUpdaterOptions,
    has_valid_geometry: bool,
    pub geometry_changed: Rc<Event<()>>,
}

impl WallGeometryUpdater {
    /// Creates a new WallGeometryUpdater for an entity.
    pub fn new(entity: Entity, _scene: Option<()>) -> Result<Self, DeveloperError> {
        let geometry_changed = Rc::new(Event::new());
        let mut updater = Self {
            entity: entity.clone(),
            fill_enabled: true,
            outline_enabled: false,
            is_dynamic: false,
            options: WallUpdaterOptions::default(),
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

    /// Gets a value indicating if the geometry has an outline component.
    pub fn outline_enabled(&self) -> bool {
        self.outline_enabled
    }

    /// Gets a value indicating if the geometry is dynamic.
    pub fn is_dynamic(&self) -> bool {
        self.is_dynamic
    }

    /// Gets a value indicating if the geometry is closed. Walls are never closed.
    pub fn is_closed(&self) -> bool {
        false
    }

    /// Gets current options for wall geometry generation.
    pub fn options(&self) -> &WallUpdaterOptions {
        &self.options
    }

    /// Creates a dynamic updater.
    pub fn create_dynamic_updater(&self) -> Result<DynamicWallGeometryUpdater, DeveloperError> {
        if !self.is_dynamic {
            return Err(DeveloperError::new(
                "This instance does not represent dynamic geometry.",
            ));
        }
        Ok(DynamicWallGeometryUpdater::new(self.entity.clone()))
    }

    /// Called when an observed entity property changes.
    pub fn on_entity_property_changed(&mut self, entity: &Entity, property_name: &str) {
        if property_name != "wall" && property_name != "availability" {
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

        let wall = match &entity.wall {
            Some(w) => w,
            None => {
                self.has_valid_geometry = false;
                self.fill_enabled = false;
                self.outline_enabled = false;
                self.is_dynamic = false;
                return;
            }
        };

        if wall.positions.is_none() {
            self.has_valid_geometry = false;
            self.fill_enabled = false;
            self.outline_enabled = false;
            self.is_dynamic = false;
            return;
        }

        self.has_valid_geometry = true;

        let show = wall
            .show
            .as_ref()
            .and_then(|p| p.get_value(None))
            .unwrap_or(true);

        if !show {
            self.fill_enabled = false;
            self.outline_enabled = false;
        } else {
            self.fill_enabled = wall
                .fill
                .as_ref()
                .and_then(|p| p.get_value(None))
                .unwrap_or(true);
            self.outline_enabled = wall
                .outline
                .as_ref()
                .and_then(|p| p.get_value(None))
                .unwrap_or(false);
        }

        let mut dynamic = false;
        if let Some(pos) = &wall.positions {
            if !pos.is_constant() {
                dynamic = true;
            }
        }
        if !wall.minimum_heights.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !wall.maximum_heights.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !wall.outline_width.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !wall.granularity.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }

        self.is_dynamic = dynamic;

        self.options = WallUpdaterOptions {
            positions: wall.positions.as_ref().and_then(|p| p.get_value(None)),
            minimum_heights: wall
                .minimum_heights
                .as_ref()
                .and_then(|p| p.get_value(None)),
            maximum_heights: wall
                .maximum_heights
                .as_ref()
                .and_then(|p| p.get_value(None)),
            granularity: wall.granularity.as_ref().and_then(|p| p.get_value(None)),
        };
    }

    /// Creates the geometry instance which represents the fill of the geometry.
    pub fn create_fill_geometry_instance(
        &self,
        time: JulianDate,
    ) -> Result<WallFillGeometryInstance, DeveloperError> {
        if !self.fill_enabled {
            return Err(DeveloperError::new(
                "This instance does not represent a filled geometry.",
            ));
        }

        let wall = self
            .entity
            .wall
            .as_ref()
            .ok_or_else(|| DeveloperError::new("Entity has no wall"))?;

        let positions = self.options.positions.clone().unwrap_or_default();
        let granularity = self.options.granularity;
        let minimum_heights = self.options.minimum_heights.clone();
        let maximum_heights = self.options.maximum_heights.clone();

        let geometry = WallGeometry::new(
            positions,
            granularity,
            maximum_heights,
            minimum_heights,
            None,
            None,
        );

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

        let color = if let Some(mat_prop) = &wall.material {
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

        Ok(WallFillGeometryInstance {
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
    ) -> Result<WallOutlineGeometryInstance, DeveloperError> {
        if !self.outline_enabled {
            return Err(DeveloperError::new(
                "This instance does not represent an outlined geometry.",
            ));
        }

        let wall = self
            .entity
            .wall
            .as_ref()
            .ok_or_else(|| DeveloperError::new("Entity has no wall"))?;

        let positions = self.options.positions.clone().unwrap_or_default();
        let granularity = self.options.granularity;
        let minimum_heights = self.options.minimum_heights.clone();
        let maximum_heights = self.options.maximum_heights.clone();

        let geometry = WallOutlineGeometry::new(
            positions,
            granularity,
            maximum_heights,
            minimum_heights,
            None,
        );

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

        let color = wall
            .outline_color
            .as_ref()
            .and_then(|p| p.get_value(Some(&time)))
            .unwrap_or(Color::BLACK);

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

        Ok(WallOutlineGeometryInstance {
            id: self.entity.id.clone(),
            geometry,
            model_matrix: Matrix4::IDENTITY,
            attributes,
        })
    }
}
