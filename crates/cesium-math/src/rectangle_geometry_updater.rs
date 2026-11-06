//! A GeometryUpdater for rectangles matching CesiumJS `DataSources/RectangleGeometryUpdater.js` (Apache-2.0).

use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    Cartesian3, Color, ColorGeometryInstanceAttribute, DeveloperError, Entity, Event,
    GeoRectangle, GeometryInstanceAttribute, GeometryOffsetAttribute, HeightReference, JulianDate,
    MaterialValue, Matrix4, RectangleGeometry, RectangleGeometryOptions, RectangleOutlineGeometry,
    RectangleOutlineGeometryOptions, ShowGeometryInstanceAttribute,
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

/// Options for configuring a rectangle geometry instance.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RectangleUpdaterOptions {
    pub rectangle: Option<GeoRectangle>,
    pub height: Option<f64>,
    pub extruded_height: Option<f64>,
    pub granularity: Option<f64>,
    pub rotation: Option<f64>,
    pub st_rotation: Option<f64>,
    pub offset_attribute: Option<GeometryOffsetAttribute>,
}

/// A geometry instance representing the filled portion of a rectangle geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct RectangleFillGeometryInstance {
    pub id: String,
    pub geometry: RectangleGeometry,
    pub model_matrix: Matrix4,
    pub attributes: HashMap<String, GeometryInstanceAttribute>,
}

/// A geometry instance representing the outline portion of a rectangle geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct RectangleOutlineGeometryInstance {
    pub id: String,
    pub geometry: RectangleOutlineGeometry,
    pub model_matrix: Matrix4,
    pub attributes: HashMap<String, GeometryInstanceAttribute>,
}

/// Dynamic updater for rectangles when coordinates, heights or rotations change with time.
pub struct DynamicRectangleGeometryUpdater {
    entity: Entity,
    options: RectangleUpdaterOptions,
}

impl DynamicRectangleGeometryUpdater {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            options: RectangleUpdaterOptions::default(),
        }
    }

    pub fn options(&self) -> &RectangleUpdaterOptions {
        &self.options
    }

    pub fn update(&mut self, time: JulianDate) {
        if let Some(rect) = &self.entity.rectangle {
            self.options.rectangle = rect
                .coordinates
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.height = rect
                .height
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.extruded_height = rect
                .extruded_height
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.granularity = rect
                .granularity
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.rotation = rect
                .rotation
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.st_rotation = rect
                .st_rotation
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));

            let mut height_reference = HeightReference::None;
            if let Some(hr_prop) = &rect.height_reference {
                if let Some(hr) = hr_prop.get_value(Some(&time)) {
                    height_reference = hr;
                }
            }
            let mut extruded_height_reference = HeightReference::None;
            if let Some(ehr_prop) = &rect.extruded_height_reference {
                if let Some(ehr) = ehr_prop.get_value(Some(&time)) {
                    extruded_height_reference = ehr;
                }
            }

            self.options.offset_attribute =
                compute_geometry_offset_attribute(height_reference, extruded_height_reference);
        } else {
            self.options = RectangleUpdaterOptions::default();
        }
    }
}

/// A GeometryUpdater for rectangles.
pub struct RectangleGeometryUpdater {
    entity: Entity,
    fill_enabled: bool,
    outline_enabled: bool,
    is_dynamic: bool,
    is_closed: bool,
    options: RectangleUpdaterOptions,
    has_valid_geometry: bool,
    pub geometry_changed: Rc<Event<()>>,
}

impl RectangleGeometryUpdater {
    /// Creates a new RectangleGeometryUpdater for an entity.
    pub fn new(entity: Entity, _scene: Option<()>) -> Result<Self, DeveloperError> {
        let geometry_changed = Rc::new(Event::new());
        let mut updater = Self {
            entity: entity.clone(),
            fill_enabled: true,
            outline_enabled: false,
            is_dynamic: false,
            is_closed: false,
            options: RectangleUpdaterOptions::default(),
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

    /// Gets current options for rectangle geometry generation.
    pub fn options(&self) -> &RectangleUpdaterOptions {
        &self.options
    }

    /// Computes the center Cartesian3 position of the rectangle at a given time.
    pub fn compute_center(&self, time: JulianDate) -> Option<Cartesian3> {
        let rect_graphics = self.entity.rectangle.as_ref()?;
        let coords = rect_graphics.coordinates.as_ref()?.get_value(Some(&time))?;
        let center = coords.center();
        Cartesian3::from_radians(center.longitude, center.latitude, Some(center.height), None).ok()
    }

    /// Creates a dynamic updater.
    pub fn create_dynamic_updater(&self) -> Result<DynamicRectangleGeometryUpdater, DeveloperError> {
        if !self.is_dynamic {
            return Err(DeveloperError::new(
                "This instance does not represent dynamic geometry.",
            ));
        }
        Ok(DynamicRectangleGeometryUpdater::new(self.entity.clone()))
    }

    /// Called when an observed entity property changes.
    pub fn on_entity_property_changed(&mut self, entity: &Entity, property_name: &str) {
        if property_name != "rectangle" && property_name != "availability" {
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

        let rect = match &entity.rectangle {
            Some(r) => r,
            None => {
                self.has_valid_geometry = false;
                self.fill_enabled = false;
                self.outline_enabled = false;
                self.is_dynamic = false;
                self.is_closed = false;
                return;
            }
        };

        if rect.coordinates.is_none() {
            self.has_valid_geometry = false;
            self.fill_enabled = false;
            self.outline_enabled = false;
            self.is_dynamic = false;
            self.is_closed = false;
            return;
        }

        self.has_valid_geometry = true;

        let show = rect
            .show
            .as_ref()
            .and_then(|p| p.get_value(None))
            .unwrap_or(true);

        if !show {
            self.fill_enabled = false;
            self.outline_enabled = false;
        } else {
            self.fill_enabled = rect
                .fill
                .as_ref()
                .and_then(|p| p.get_value(None))
                .unwrap_or(true);
            self.outline_enabled = rect
                .outline
                .as_ref()
                .and_then(|p| p.get_value(None))
                .unwrap_or(false);
        }

        let mut dynamic = false;
        if let Some(coord) = &rect.coordinates {
            if !coord.is_constant() {
                dynamic = true;
            }
        }
        if !rect.height.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !rect.extruded_height.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !rect.granularity.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !rect.rotation.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !rect.st_rotation.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !rect.outline_width.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !rect.z_index.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !rect.height_reference.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !rect.extruded_height_reference.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }

        self.is_dynamic = dynamic;

        let height_val = rect.height.as_ref().and_then(|p| p.get_value(None));
        let extruded_val = rect.extruded_height.as_ref().and_then(|p| p.get_value(None));

        self.is_closed = match (height_val, extruded_val) {
            (Some(h), Some(eh)) => (h - eh).abs() > f64::EPSILON,
            (None, Some(_)) => true,
            _ => false,
        };

        let mut height_reference = HeightReference::None;
        if let Some(hr_prop) = &rect.height_reference {
            if let Some(hr) = hr_prop.get_value(None) {
                height_reference = hr;
            }
        }
        let mut extruded_height_reference = HeightReference::None;
        if let Some(ehr_prop) = &rect.extruded_height_reference {
            if let Some(ehr) = ehr_prop.get_value(None) {
                extruded_height_reference = ehr;
            }
        }

        let offset_attribute =
            compute_geometry_offset_attribute(height_reference, extruded_height_reference);

        self.options = RectangleUpdaterOptions {
            rectangle: rect.coordinates.as_ref().and_then(|p| p.get_value(None)),
            height: height_val,
            extruded_height: extruded_val,
            granularity: rect.granularity.as_ref().and_then(|p| p.get_value(None)),
            rotation: rect.rotation.as_ref().and_then(|p| p.get_value(None)),
            st_rotation: rect.st_rotation.as_ref().and_then(|p| p.get_value(None)),
            offset_attribute,
        };
    }

    /// Creates the geometry instance which represents the fill of the geometry.
    pub fn create_fill_geometry_instance(
        &self,
        time: JulianDate,
    ) -> Result<RectangleFillGeometryInstance, DeveloperError> {
        if !self.fill_enabled {
            return Err(DeveloperError::new(
                "This instance does not represent a filled geometry.",
            ));
        }

        let rect_graphics = self
            .entity
            .rectangle
            .as_ref()
            .ok_or_else(|| DeveloperError::new("Entity has no rectangle"))?;

        let rectangle = self
            .options
            .rectangle
            .ok_or_else(|| DeveloperError::new("Rectangle coordinates undefined"))?;
        let height = self.options.height;
        let extruded_height = self.options.extruded_height;
        let granularity = self.options.granularity;
        let rotation = self.options.rotation;
        let st_rotation = self.options.st_rotation;
        let offset_attribute = self.options.offset_attribute;

        let geometry = RectangleGeometry::new(RectangleGeometryOptions {
            rectangle,
            vertex_format: None,
            ellipsoid: None,
            granularity,
            height,
            rotation,
            st_rotation,
            extruded_height,
            shadow_volume: None,
            offset_attribute,
        })?;

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

        let color = if let Some(mat_prop) = &rect_graphics.material {
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

        Ok(RectangleFillGeometryInstance {
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
    ) -> Result<RectangleOutlineGeometryInstance, DeveloperError> {
        if !self.outline_enabled {
            return Err(DeveloperError::new(
                "This instance does not represent an outlined geometry.",
            ));
        }

        let rect_graphics = self
            .entity
            .rectangle
            .as_ref()
            .ok_or_else(|| DeveloperError::new("Entity has no rectangle"))?;

        let rectangle = self
            .options
            .rectangle
            .ok_or_else(|| DeveloperError::new("Rectangle coordinates undefined"))?;
        let height = self.options.height;
        let extruded_height = self.options.extruded_height;
        let granularity = self.options.granularity;
        let rotation = self.options.rotation;
        let offset_attribute = self.options.offset_attribute;

        let geometry = RectangleOutlineGeometry::new(RectangleOutlineGeometryOptions {
            rectangle,
            ellipsoid: None,
            granularity,
            height,
            rotation,
            extruded_height,
            offset_attribute,
        })?;

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

        let color = rect_graphics
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

        Ok(RectangleOutlineGeometryInstance {
            id: self.entity.id.clone(),
            geometry,
            model_matrix: Matrix4::IDENTITY,
            attributes,
        })
    }
}
