//! A GeometryUpdater for polyline volumes matching CesiumJS `DataSources/PolylineVolumeGeometryUpdater.js` (Apache-2.0).

use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    Cartesian2, Cartesian3, Color, ColorGeometryInstanceAttribute, CornerType, DeveloperError,
    Entity, Event, GeometryInstanceAttribute, JulianDate, MaterialValue, Matrix4,
    PolylineVolumeGeometry, PolylineVolumeGeometryOptions, PolylineVolumeOutlineGeometry,
    PolylineVolumeOutlineGeometryOptions, ShowGeometryInstanceAttribute,
};

/// Options for configuring a polyline volume geometry instance.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PolylineVolumeUpdaterOptions {
    pub polyline_positions: Option<Vec<Cartesian3>>,
    pub shape_positions: Option<Vec<Cartesian2>>,
    pub granularity: Option<f64>,
    pub corner_type: Option<CornerType>,
}

/// A geometry instance representing the filled portion of a polyline volume geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct PolylineVolumeFillGeometryInstance {
    pub id: String,
    pub geometry: PolylineVolumeGeometry,
    pub model_matrix: Matrix4,
    pub attributes: HashMap<String, GeometryInstanceAttribute>,
}

/// A geometry instance representing the outline portion of a polyline volume geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct PolylineVolumeOutlineGeometryInstance {
    pub id: String,
    pub geometry: PolylineVolumeOutlineGeometry,
    pub model_matrix: Matrix4,
    pub attributes: HashMap<String, GeometryInstanceAttribute>,
}

/// Dynamic updater for polyline volumes when positions, shape or properties change with time.
pub struct DynamicPolylineVolumeGeometryUpdater {
    entity: Entity,
    options: PolylineVolumeUpdaterOptions,
}

impl DynamicPolylineVolumeGeometryUpdater {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            options: PolylineVolumeUpdaterOptions::default(),
        }
    }

    pub fn options(&self) -> &PolylineVolumeUpdaterOptions {
        &self.options
    }

    pub fn update(&mut self, time: JulianDate) {
        if let Some(vol) = &self.entity.polyline_volume {
            self.options.polyline_positions = vol
                .positions
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.shape_positions = vol
                .shape
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.granularity = vol
                .granularity
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.corner_type = vol
                .corner_type
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
        } else {
            self.options = PolylineVolumeUpdaterOptions::default();
        }
    }
}

/// A GeometryUpdater for polyline volumes.
pub struct PolylineVolumeGeometryUpdater {
    entity: Entity,
    fill_enabled: bool,
    outline_enabled: bool,
    is_dynamic: bool,
    options: PolylineVolumeUpdaterOptions,
    has_valid_geometry: bool,
    pub geometry_changed: Rc<Event<()>>,
}

impl PolylineVolumeGeometryUpdater {
    /// Creates a new PolylineVolumeGeometryUpdater for an entity.
    pub fn new(entity: Entity, _scene: Option<()>) -> Result<Self, DeveloperError> {
        let geometry_changed = Rc::new(Event::new());
        let mut updater = Self {
            entity: entity.clone(),
            fill_enabled: true,
            outline_enabled: false,
            is_dynamic: false,
            options: PolylineVolumeUpdaterOptions::default(),
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

    /// Gets a value indicating if the geometry is closed.
    pub fn is_closed(&self) -> bool {
        false
    }

    /// Gets current options for polyline volume geometry generation.
    pub fn options(&self) -> &PolylineVolumeUpdaterOptions {
        &self.options
    }

    /// Creates a dynamic updater.
    pub fn create_dynamic_updater(&self) -> Result<DynamicPolylineVolumeGeometryUpdater, DeveloperError> {
        if !self.is_dynamic {
            return Err(DeveloperError::new(
                "This instance does not represent dynamic geometry.",
            ));
        }
        Ok(DynamicPolylineVolumeGeometryUpdater::new(self.entity.clone()))
    }

    /// Called when an observed entity property changes.
    pub fn on_entity_property_changed(&mut self, entity: &Entity, property_name: &str) {
        if property_name != "polylineVolume" && property_name != "availability" {
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

        let vol = match &entity.polyline_volume {
            Some(v) => v,
            None => {
                self.has_valid_geometry = false;
                self.fill_enabled = false;
                self.outline_enabled = false;
                self.is_dynamic = false;
                return;
            }
        };

        if vol.positions.is_none() || vol.shape.is_none() {
            self.has_valid_geometry = false;
            self.fill_enabled = false;
            self.outline_enabled = false;
            self.is_dynamic = false;
            return;
        }

        self.has_valid_geometry = true;

        let show = vol
            .show
            .as_ref()
            .and_then(|p| p.get_value(None))
            .unwrap_or(true);

        if !show {
            self.fill_enabled = false;
            self.outline_enabled = false;
        } else {
            self.fill_enabled = vol
                .fill
                .as_ref()
                .and_then(|p| p.get_value(None))
                .unwrap_or(true);
            self.outline_enabled = vol
                .outline
                .as_ref()
                .and_then(|p| p.get_value(None))
                .unwrap_or(false);
        }

        let mut dynamic = false;
        if let Some(pos) = &vol.positions {
            if !pos.is_constant() {
                dynamic = true;
            }
        }
        if let Some(shape) = &vol.shape {
            if !shape.is_constant() {
                dynamic = true;
            }
        }
        if !vol.granularity.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !vol.corner_type.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !vol.outline_width.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }

        self.is_dynamic = dynamic;

        self.options = PolylineVolumeUpdaterOptions {
            polyline_positions: vol.positions.as_ref().and_then(|p| p.get_value(None)),
            shape_positions: vol.shape.as_ref().and_then(|p| p.get_value(None)),
            granularity: vol.granularity.as_ref().and_then(|p| p.get_value(None)),
            corner_type: vol.corner_type.as_ref().and_then(|p| p.get_value(None)),
        };
    }

    /// Creates the geometry instance which represents the fill of the geometry.
    pub fn create_fill_geometry_instance(
        &self,
        time: JulianDate,
    ) -> Result<PolylineVolumeFillGeometryInstance, DeveloperError> {
        if !self.fill_enabled {
            return Err(DeveloperError::new(
                "This instance does not represent a filled geometry.",
            ));
        }

        let polyline_positions = self.options.polyline_positions.clone().unwrap_or_default();
        let shape_positions = self.options.shape_positions.clone().unwrap_or_default();
        let granularity = self.options.granularity;
        let corner_type = self.options.corner_type;

        let geometry = PolylineVolumeGeometry::new(PolylineVolumeGeometryOptions {
            polyline_positions,
            shape_positions,
            ellipsoid: None,
            corner_type,
            vertex_format: None,
            granularity,
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

        let color = if let Some(vol) = &self.entity.polyline_volume {
            if let Some(mat_prop) = &vol.material {
                match mat_prop.get_material_value(Some(&time)) {
                    Some(MaterialValue::Color(c)) => c,
                    _ => Color::WHITE,
                }
            } else {
                Color::WHITE
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

        Ok(PolylineVolumeFillGeometryInstance {
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
    ) -> Result<PolylineVolumeOutlineGeometryInstance, DeveloperError> {
        if !self.outline_enabled {
            return Err(DeveloperError::new(
                "This instance does not represent an outlined geometry.",
            ));
        }

        let polyline_positions = self.options.polyline_positions.clone().unwrap_or_default();
        let shape_positions = self.options.shape_positions.clone().unwrap_or_default();
        let granularity = self.options.granularity;
        let corner_type = self.options.corner_type;

        let geometry = PolylineVolumeOutlineGeometry::new(PolylineVolumeOutlineGeometryOptions {
            polyline_positions,
            shape_positions,
            ellipsoid: None,
            corner_type,
            granularity,
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

        let color = if let Some(vol) = &self.entity.polyline_volume {
            vol.outline_color
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)))
                .unwrap_or(Color::BLACK)
        } else {
            Color::BLACK
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

        Ok(PolylineVolumeOutlineGeometryInstance {
            id: self.entity.id.clone(),
            geometry,
            model_matrix: Matrix4::IDENTITY,
            attributes,
        })
    }
}
