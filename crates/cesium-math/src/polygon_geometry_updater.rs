//! A GeometryUpdater for polygons matching CesiumJS `DataSources/PolygonGeometryUpdater.js` (Apache-2.0).

use std::collections::HashMap;
use std::rc::Rc;
use glam::{DVec2, DVec3};

use crate::{
    ArcType, Cartesian3, Color, ColorGeometryInstanceAttribute, DeveloperError,
    Ellipsoid, EllipsoidTangentPlane, Entity, Event, GeometryInstanceAttribute,
    GeometryOffsetAttribute, HeightReference, JulianDate, MaterialValue, Matrix4,
    PolygonGeometry, PolygonGeometryOptions, PolygonHierarchy, PolygonOutlineGeometry,
    PolygonOutlineGeometryOptions, ShowGeometryInstanceAttribute,
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

/// Options for configuring a polygon geometry instance.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PolygonUpdaterOptions {
    pub polygon_hierarchy: Option<PolygonHierarchy>,
    pub height: Option<f64>,
    pub extruded_height: Option<f64>,
    pub granularity: Option<f64>,
    pub st_rotation: Option<f64>,
    pub per_position_height: Option<bool>,
    pub close_top: Option<bool>,
    pub close_bottom: Option<bool>,
    pub arc_type: Option<ArcType>,
    pub texture_coordinates: Option<PolygonHierarchy>,
    pub offset_attribute: Option<GeometryOffsetAttribute>,
}

/// A geometry instance representing the filled portion of a polygon geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct PolygonFillGeometryInstance {
    pub id: String,
    pub geometry: PolygonGeometry,
    pub model_matrix: Matrix4,
    pub attributes: HashMap<String, GeometryInstanceAttribute>,
}

/// A geometry instance representing the outline portion of a polygon geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct PolygonOutlineGeometryInstance {
    pub id: String,
    pub geometry: PolygonOutlineGeometry,
    pub model_matrix: Matrix4,
    pub attributes: HashMap<String, GeometryInstanceAttribute>,
}

/// Dynamic updater for polygons when hierarchy, heights or properties change with time.
pub struct DynamicPolygonGeometryUpdater {
    entity: Entity,
    options: PolygonUpdaterOptions,
}

impl DynamicPolygonGeometryUpdater {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            options: PolygonUpdaterOptions::default(),
        }
    }

    pub fn options(&self) -> &PolygonUpdaterOptions {
        &self.options
    }

    pub fn update(&mut self, time: JulianDate) {
        if let Some(poly) = &self.entity.polygon {
            self.options.polygon_hierarchy = poly
                .hierarchy
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.height = poly
                .height
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.extruded_height = poly
                .extruded_height
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.granularity = poly
                .granularity
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.st_rotation = poly
                .st_rotation
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.per_position_height = poly
                .per_position_height
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.close_top = poly
                .close_top
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.close_bottom = poly
                .close_bottom
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.arc_type = poly
                .arc_type
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));
            self.options.texture_coordinates = poly
                .texture_coordinates
                .as_ref()
                .and_then(|p| p.get_value(Some(&time)));

            let mut height_reference = HeightReference::None;
            if let Some(hr_prop) = &poly.height_reference {
                if let Some(hr) = hr_prop.get_value(Some(&time)) {
                    height_reference = hr;
                }
            }
            let mut extruded_height_reference = HeightReference::None;
            if let Some(ehr_prop) = &poly.extruded_height_reference {
                if let Some(ehr) = ehr_prop.get_value(Some(&time)) {
                    extruded_height_reference = ehr;
                }
            }

            self.options.offset_attribute =
                compute_geometry_offset_attribute(height_reference, extruded_height_reference);
        } else {
            self.options = PolygonUpdaterOptions::default();
        }
    }
}

/// A GeometryUpdater for polygons.
pub struct PolygonGeometryUpdater {
    entity: Entity,
    fill_enabled: bool,
    outline_enabled: bool,
    is_dynamic: bool,
    is_closed: bool,
    on_terrain: bool,
    options: PolygonUpdaterOptions,
    has_valid_geometry: bool,
    pub geometry_changed: Rc<Event<()>>,
}

impl PolygonGeometryUpdater {
    /// Creates a new PolygonGeometryUpdater for an entity.
    pub fn new(entity: Entity, _scene: Option<()>) -> Result<Self, DeveloperError> {
        let geometry_changed = Rc::new(Event::new());
        let mut updater = Self {
            entity: entity.clone(),
            fill_enabled: true,
            outline_enabled: false,
            is_dynamic: false,
            is_closed: false,
            on_terrain: false,
            options: PolygonUpdaterOptions::default(),
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

    /// Gets a value indicating if this geometry can be clamped to terrain.
    pub fn on_terrain(&self) -> bool {
        self.on_terrain
    }

    /// Gets current options for polygon geometry generation.
    pub fn options(&self) -> &PolygonUpdaterOptions {
        &self.options
    }

    /// Computes the center Cartesian3 position of the polygon at a given time.
    pub fn compute_center(&self, time: JulianDate) -> Option<Cartesian3> {
        let poly = self.entity.polygon.as_ref()?;
        let hierarchy = poly.hierarchy.as_ref()?.get_value(Some(&time))?;
        let positions = &hierarchy.positions;
        if positions.is_empty() {
            return None;
        }

        let ellipsoid = Ellipsoid::WGS84;
        let points_dvec: Vec<DVec3> = positions.iter().map(|p| DVec3::new(p.x, p.y, p.z)).collect();
        let tangent_plane = EllipsoidTangentPlane::from_points(&points_dvec, ellipsoid).ok()?;
        let positions_2d = tangent_plane.project_points_onto_plane(&points_dvec);

        let length = positions_2d.len();
        if length == 0 {
            return None;
        }

        let mut area = 0.0;
        let mut j = length - 1;
        let mut centroid_2d = DVec2::ZERO;

        for i in 0..length {
            let p1 = positions_2d[i];
            let p2 = positions_2d[j];
            let f = p1.x * p2.y - p2.x * p1.y;

            centroid_2d += (p1 + p2) * f;
            area += f;
            j = i;
        }

        if area.abs() > 1e-12 {
            let a = 1.0 / (area * 3.0);
            centroid_2d *= a;
        }

        let center_3d = tangent_plane.project_point_onto_ellipsoid(centroid_2d)?;
        Some(Cartesian3::new(center_3d.x, center_3d.y, center_3d.z))
    }

    /// Creates a dynamic updater.
    pub fn create_dynamic_updater(&self) -> Result<DynamicPolygonGeometryUpdater, DeveloperError> {
        if !self.is_dynamic {
            return Err(DeveloperError::new(
                "This instance does not represent dynamic geometry.",
            ));
        }
        Ok(DynamicPolygonGeometryUpdater::new(self.entity.clone()))
    }

    /// Called when an observed entity property changes.
    pub fn on_entity_property_changed(&mut self, entity: &Entity, property_name: &str) {
        if property_name != "polygon" && property_name != "availability" {
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

        let poly = match &entity.polygon {
            Some(p) => p,
            None => {
                self.has_valid_geometry = false;
                self.fill_enabled = false;
                self.outline_enabled = false;
                self.is_dynamic = false;
                self.is_closed = false;
                self.on_terrain = false;
                return;
            }
        };

        if poly.hierarchy.is_none() {
            self.has_valid_geometry = false;
            self.fill_enabled = false;
            self.outline_enabled = false;
            self.is_dynamic = false;
            self.is_closed = false;
            self.on_terrain = false;
            return;
        }

        self.has_valid_geometry = true;

        let show = poly
            .show
            .as_ref()
            .and_then(|p| p.get_value(None))
            .unwrap_or(true);

        if !show {
            self.fill_enabled = false;
            self.outline_enabled = false;
        } else {
            self.fill_enabled = poly
                .fill
                .as_ref()
                .and_then(|p| p.get_value(None))
                .unwrap_or(true);
            self.outline_enabled = poly
                .outline
                .as_ref()
                .and_then(|p| p.get_value(None))
                .unwrap_or(false);
        }

        let mut dynamic = false;
        if let Some(hier) = &poly.hierarchy {
            if !hier.is_constant() {
                dynamic = true;
            }
        }
        if !poly.height.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !poly.extruded_height.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !poly.granularity.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !poly.st_rotation.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !poly.outline_width.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !poly.per_position_height.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !poly.close_top.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !poly.close_bottom.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !poly.arc_type.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !poly.z_index.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !poly.height_reference.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }
        if !poly.extruded_height_reference.as_ref().is_none_or(|p| p.is_constant()) {
            dynamic = true;
        }

        self.is_dynamic = dynamic;

        let height_val = poly.height.as_ref().and_then(|p| p.get_value(None));
        let extruded_val = poly.extruded_height.as_ref().and_then(|p| p.get_value(None));
        let per_pos_height = poly.per_position_height.as_ref().and_then(|p| p.get_value(None)).unwrap_or(false);
        let close_top = poly.close_top.as_ref().and_then(|p| p.get_value(None)).unwrap_or(true);
        let close_bottom = poly.close_bottom.as_ref().and_then(|p| p.get_value(None)).unwrap_or(true);

        let is_extruded = if per_pos_height {
            extruded_val.is_some()
        } else {
            match (height_val, extruded_val) {
                (Some(h), Some(eh)) => (h - eh).abs() > f64::EPSILON,
                (None, Some(_)) => true,
                _ => false,
            }
        };

        self.is_closed = is_extruded && close_top && close_bottom;

        // Ground geometry / onTerrain check
        let is_filled = self.fill_enabled;
        let no_height = height_val.is_none() && extruded_val.is_none();
        self.on_terrain = is_filled && no_height && !per_pos_height;

        let mut height_reference = HeightReference::None;
        if let Some(hr_prop) = &poly.height_reference {
            if let Some(hr) = hr_prop.get_value(None) {
                height_reference = hr;
            }
        }
        let mut extruded_height_reference = HeightReference::None;
        if let Some(ehr_prop) = &poly.extruded_height_reference {
            if let Some(ehr) = ehr_prop.get_value(None) {
                extruded_height_reference = ehr;
            }
        }

        let offset_attribute =
            compute_geometry_offset_attribute(height_reference, extruded_height_reference);

        self.options = PolygonUpdaterOptions {
            polygon_hierarchy: poly.hierarchy.as_ref().and_then(|p| p.get_value(None)),
            height: height_val,
            extruded_height: extruded_val,
            granularity: poly.granularity.as_ref().and_then(|p| p.get_value(None)),
            st_rotation: poly.st_rotation.as_ref().and_then(|p| p.get_value(None)),
            per_position_height: Some(per_pos_height),
            close_top: Some(close_top),
            close_bottom: Some(close_bottom),
            arc_type: poly.arc_type.as_ref().and_then(|p| p.get_value(None)),
            texture_coordinates: poly.texture_coordinates.as_ref().and_then(|p| p.get_value(None)),
            offset_attribute,
        };
    }

    /// Creates the geometry instance which represents the fill of the geometry.
    pub fn create_fill_geometry_instance(
        &self,
        time: JulianDate,
    ) -> Result<PolygonFillGeometryInstance, DeveloperError> {
        if !self.fill_enabled {
            return Err(DeveloperError::new(
                "This instance does not represent a filled geometry.",
            ));
        }

        let poly_graphics = self
            .entity
            .polygon
            .as_ref()
            .ok_or_else(|| DeveloperError::new("Entity has no polygon"))?;

        let hierarchy = self
            .options
            .polygon_hierarchy
            .clone()
            .ok_or_else(|| DeveloperError::new("Polygon hierarchy undefined"))?;

        let geometry = PolygonGeometry::new(PolygonGeometryOptions {
            polygon_hierarchy: hierarchy,
            height: self.options.height,
            extruded_height: self.options.extruded_height,
            vertex_format: None,
            ellipsoid: None,
            granularity: self.options.granularity,
            per_position_height: self.options.per_position_height,
            close_top: self.options.close_top,
            close_bottom: self.options.close_bottom,
            arc_type: self.options.arc_type,
            st_rotation: self.options.st_rotation,
            offset_attribute: self.options.offset_attribute,
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

        let color = if let Some(mat_prop) = &poly_graphics.material {
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

        Ok(PolygonFillGeometryInstance {
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
    ) -> Result<PolygonOutlineGeometryInstance, DeveloperError> {
        if !self.outline_enabled {
            return Err(DeveloperError::new(
                "This instance does not represent an outlined geometry.",
            ));
        }

        let poly_graphics = self
            .entity
            .polygon
            .as_ref()
            .ok_or_else(|| DeveloperError::new("Entity has no polygon"))?;

        let hierarchy = self
            .options
            .polygon_hierarchy
            .clone()
            .ok_or_else(|| DeveloperError::new("Polygon hierarchy undefined"))?;

        let geometry = PolygonOutlineGeometry::new(PolygonOutlineGeometryOptions {
            polygon_hierarchy: hierarchy,
            height: self.options.height,
            extruded_height: self.options.extruded_height,
            ellipsoid: None,
            granularity: self.options.granularity,
            per_position_height: self.options.per_position_height,
            arc_type: self.options.arc_type,
            offset_attribute: self.options.offset_attribute,
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

        let color = poly_graphics
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

        Ok(PolygonOutlineGeometryInstance {
            id: self.entity.id.clone(),
            geometry,
            model_matrix: Matrix4::IDENTITY,
            attributes,
        })
    }
}
