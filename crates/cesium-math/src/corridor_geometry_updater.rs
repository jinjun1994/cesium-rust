//! A GeometryUpdater for corridors matching CesiumJS `DataSources/CorridorGeometryUpdater.js` (Apache-2.0).

use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    Cartesian3, Color, ColorGeometryInstanceAttribute, CornerType, DeveloperError,
    CorridorGeometry, CorridorGeometryOptions, CorridorOutlineGeometry,
    CorridorOutlineGeometryOptions, Entity, Event, GeometryInstanceAttribute,
    GeometryOffsetAttribute, HeightReference, JulianDate, MaterialValue, Matrix4,
    ShowGeometryInstanceAttribute,
};

fn compute_geometry_offset_attribute(
    height_ref: HeightReference,
    extruded_height_ref: HeightReference,
) -> Option<GeometryOffsetAttribute> {
    let has_height_ref = matches!(
        height_ref,
        HeightReference::ClampToGround
            | HeightReference::RelativeToGround
            | HeightReference::ClampToTerrain
            | HeightReference::RelativeToTerrain
    );
    let has_extruded_ref = matches!(
        extruded_height_ref,
        HeightReference::ClampToGround
            | HeightReference::RelativeToGround
            | HeightReference::ClampToTerrain
            | HeightReference::RelativeToTerrain
    );

    if has_height_ref && has_extruded_ref {
        Some(GeometryOffsetAttribute::All)
    } else if has_height_ref {
        Some(GeometryOffsetAttribute::Top)
    } else if has_extruded_ref {
        Some(GeometryOffsetAttribute::All)
    } else {
        None
    }
}

/// Options for configuring a corridor geometry instance.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CorridorUpdaterOptions {
    pub positions: Option<Vec<Cartesian3>>,
    pub width: Option<f64>,
    pub corner_type: Option<CornerType>,
    pub height: Option<f64>,
    pub extruded_height: Option<f64>,
    pub granularity: Option<f64>,
    pub offset_attribute: Option<GeometryOffsetAttribute>,
}

/// A geometry instance representing the filled portion of a corridor geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct CorridorFillGeometryInstance {
    pub id: String,
    pub geometry: CorridorGeometry,
    pub model_matrix: Matrix4,
    pub attributes: HashMap<String, GeometryInstanceAttribute>,
}

/// A geometry instance representing the outline portion of a corridor geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct CorridorOutlineGeometryInstance {
    pub id: String,
    pub geometry: CorridorOutlineGeometry,
    pub model_matrix: Matrix4,
    pub attributes: HashMap<String, GeometryInstanceAttribute>,
}

/// Dynamic updater for corridors when positions, dimensions or properties change with time.
pub struct DynamicCorridorGeometryUpdater {
    entity: Entity,
    options: CorridorUpdaterOptions,
}

impl DynamicCorridorGeometryUpdater {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            options: CorridorUpdaterOptions::default(),
        }
    }

    pub fn options(&self) -> &CorridorUpdaterOptions {
        &self.options
    }

    pub fn update(&mut self, time: JulianDate) {
        if let Some(corridor) = &self.entity.corridor {
            self.options.positions = corridor
                .positions
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.width = corridor
                .width
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.corner_type = corridor
                .corner_type
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.height = corridor
                .height
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.extruded_height = corridor
                .extruded_height
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.granularity = corridor
                .granularity
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));

            let mut height_reference = HeightReference::None;
            if let Some(hr_prop) = &corridor.height_reference {
                if let Some(hr) = hr_prop.get_value(Some(&time)) {
                    height_reference = hr;
                }
            }
            let mut extruded_height_reference = HeightReference::None;
            if let Some(ehr_prop) = &corridor.extruded_height_reference {
                if let Some(ehr) = ehr_prop.get_value(Some(&time)) {
                    extruded_height_reference = ehr;
                }
            }

            self.options.offset_attribute =
                compute_geometry_offset_attribute(height_reference, extruded_height_reference);
        } else {
            self.options = CorridorUpdaterOptions::default();
        }
    }
}

/// A GeometryUpdater for corridors.
pub struct CorridorGeometryUpdater {
    entity: Entity,
    fill_enabled: bool,
    outline_enabled: bool,
    is_dynamic: bool,
    is_closed: bool,
    options: CorridorUpdaterOptions,
    has_valid_geometry: bool,
    pub geometry_changed: Rc<Event<()>>,
}

impl CorridorGeometryUpdater {
    /// Creates a new CorridorGeometryUpdater for an entity.
    pub fn new(entity: Entity, _scene: Option<()>) -> Result<Self, DeveloperError> {
        let geometry_changed = Rc::new(Event::new());
        let mut updater = Self {
            entity: entity.clone(),
            fill_enabled: true,
            outline_enabled: false,
            is_dynamic: false,
            is_closed: false,
            options: CorridorUpdaterOptions::default(),
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
        self.is_closed
    }

    /// Gets current options for corridor geometry generation.
    pub fn options(&self) -> &CorridorUpdaterOptions {
        &self.options
    }

    /// Computes the center Cartesian3 position of the corridor at a given time.
    pub fn compute_center(&self, time: JulianDate) -> Option<Cartesian3> {
        let corridor = self.entity.corridor.as_ref()?;
        let positions = corridor.positions.as_ref()?.get_value(Some(&time))?;
        if positions.is_empty() {
            return None;
        }
        let center_idx = positions.len() / 2;
        Some(positions[center_idx])
    }

    /// Creates a dynamic updater.
    pub fn create_dynamic_updater(&self) -> Result<DynamicCorridorGeometryUpdater, DeveloperError> {
        if !self.is_dynamic {
            return Err(DeveloperError::new(
                "This instance does not represent dynamic geometry.",
            ));
        }
        Ok(DynamicCorridorGeometryUpdater::new(self.entity.clone()))
    }

    /// Called when an observed entity property changes.
    pub fn on_entity_property_changed(&mut self, entity: &Entity, property_name: &str) {
        if property_name != "corridor" && property_name != "availability" {
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

        let corridor = match &entity.corridor {
            Some(c) => c,
            None => {
                self.has_valid_geometry = false;
                self.fill_enabled = false;
                self.outline_enabled = false;
                self.is_dynamic = false;
                self.is_closed = false;
                return;
            }
        };

        let positions_valid = corridor.positions.is_some();
        let width_valid = corridor.width.is_some();

        if !positions_valid || !width_valid {
            self.has_valid_geometry = false;
            self.fill_enabled = false;
            self.outline_enabled = false;
            self.is_dynamic = false;
            self.is_closed = false;
            return;
        }

        self.has_valid_geometry = true;

        let show = corridor
            .show
            .as_ref()
            .and_then(|p| p.get_value(None))
            .unwrap_or(true);

        if !show {
            self.fill_enabled = false;
            self.outline_enabled = false;
        } else {
            self.fill_enabled = corridor
                .fill
                .as_ref()
                .and_then(|p| p.get_value(None))
                .unwrap_or(true);
            self.outline_enabled = corridor
                .outline
                .as_ref()
                .and_then(|p| p.get_value(None))
                .unwrap_or(false);
        }

        let mut dynamic = false;
        if let Some(pos) = &corridor.positions {
            if !pos.is_constant() {
                dynamic = true;
            }
        }
        if !corridor.width.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !corridor.height.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !corridor.extruded_height.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !corridor.granularity.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !corridor.outline_width.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !corridor.corner_type.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !corridor.z_index.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !corridor.height_reference.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !corridor.extruded_height_reference.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }

        self.is_dynamic = dynamic;

        let height_val = corridor.height.as_ref().and_then(|p| p.get_value(None));
        let extruded_val = corridor.extruded_height.as_ref().and_then(|p| p.get_value(None));

        self.is_closed = match (height_val, extruded_val) {
            (Some(h), Some(eh)) => (h - eh).abs() > f64::EPSILON,
            (None, Some(_)) => true,
            _ => false,
        };

        let mut height_reference = HeightReference::None;
        if let Some(hr_prop) = &corridor.height_reference {
            if let Some(hr) = hr_prop.get_value(None) {
                height_reference = hr;
            }
        }
        let mut extruded_height_reference = HeightReference::None;
        if let Some(ehr_prop) = &corridor.extruded_height_reference {
            if let Some(ehr) = ehr_prop.get_value(None) {
                extruded_height_reference = ehr;
            }
        }

        let offset_attribute =
            compute_geometry_offset_attribute(height_reference, extruded_height_reference);

        self.options = CorridorUpdaterOptions {
            positions: corridor.positions.as_ref().and_then(|p| p.get_value(None)),
            width: corridor.width.as_ref().and_then(|p| p.get_value(None)),
            corner_type: corridor.corner_type.as_ref().and_then(|p| p.get_value(None)),
            height: height_val,
            extruded_height: extruded_val,
            granularity: corridor.granularity.as_ref().and_then(|p| p.get_value(None)),
            offset_attribute,
        };
    }

    /// Creates the geometry instance which represents the fill of the geometry.
    pub fn create_fill_geometry_instance(
        &self,
        time: JulianDate,
    ) -> Result<CorridorFillGeometryInstance, DeveloperError> {
        if !self.fill_enabled {
            return Err(DeveloperError::new(
                "This instance does not represent a filled geometry.",
            ));
        }

        let corridor = self
            .entity
            .corridor
            .as_ref()
            .ok_or_else(|| DeveloperError::new("Entity has no corridor"))?;

        let positions = self.options.positions.clone().unwrap_or_default();
        let width = self.options.width.unwrap_or(0.0);
        let height = self.options.height;
        let extruded_height = self.options.extruded_height;
        let granularity = self.options.granularity;
        let corner_type = self.options.corner_type;
        let offset_attribute = self.options.offset_attribute;

        let geometry = CorridorGeometry::new(CorridorGeometryOptions {
            positions,
            width,
            corner_type,
            height,
            extruded_height,
            vertex_format: None,
            ellipsoid: None,
            granularity,
            offset_attribute,
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

        let color = if let Some(mat_prop) = &corridor.material {
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

        Ok(CorridorFillGeometryInstance {
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
    ) -> Result<CorridorOutlineGeometryInstance, DeveloperError> {
        if !self.outline_enabled {
            return Err(DeveloperError::new(
                "This instance does not represent an outlined geometry.",
            ));
        }

        let corridor = self
            .entity
            .corridor
            .as_ref()
            .ok_or_else(|| DeveloperError::new("Entity has no corridor"))?;

        let positions = self.options.positions.clone().unwrap_or_default();
        let width = self.options.width.unwrap_or(0.0);
        let height = self.options.height;
        let extruded_height = self.options.extruded_height;
        let granularity = self.options.granularity;
        let corner_type = self.options.corner_type;
        let offset_attribute = self.options.offset_attribute;

        let geometry = CorridorOutlineGeometry::new(CorridorOutlineGeometryOptions {
            positions,
            width,
            corner_type,
            height,
            extruded_height,
            ellipsoid: None,
            granularity,
            offset_attribute,
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

        let color = corridor
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

        Ok(CorridorOutlineGeometryInstance {
            id: self.entity.id.clone(),
            geometry,
            model_matrix: Matrix4::IDENTITY,
            attributes,
        })
    }
}
