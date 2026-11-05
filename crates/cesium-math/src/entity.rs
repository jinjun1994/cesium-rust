//! Entity instances aggregate multiple forms of visualization into a single high-level object.
//! Direct port of CesiumJS `DataSources/Entity.js`.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::{
    create_guid, BillboardGraphics, BoxGraphics, Cartesian3, Cesium3DTilesetGraphics,
    ConstantPositionProperty, ConstantProperty, CorridorGraphics, CylinderGraphics, DeveloperError,
    EllipseGraphics, EllipsoidGraphics, FixedFrameTransforms, JulianDate, LabelGraphics, Matrix3,
    Matrix4, ModelGraphics, PathGraphics, PlaneGraphics, PointGraphics, PolygonGraphics,
    PolylineGraphics, PolylineVolumeGraphics, PropertyBag, Quaternion, RectangleGraphics,
    TimeIntervalCollection, TrackingReferenceFrame, WallGraphics,
};

/// Trait for objects that can be merged with another object of the same type.
pub trait Mergeable {
    fn merge(&mut self, source: &Self);
}

macro_rules! impl_mergeable {
    ($t:ident) => {
        impl Mergeable for $t {
            fn merge(&mut self, source: &Self) {
                self.merge(source);
            }
        }
    };
}

impl_mergeable!(BillboardGraphics);
impl_mergeable!(BoxGraphics);
impl_mergeable!(CorridorGraphics);
impl_mergeable!(CylinderGraphics);
impl_mergeable!(EllipseGraphics);
impl_mergeable!(EllipsoidGraphics);
impl_mergeable!(LabelGraphics);
impl_mergeable!(ModelGraphics);
impl_mergeable!(PathGraphics);
impl_mergeable!(PlaneGraphics);
impl_mergeable!(PointGraphics);
impl_mergeable!(PolygonGraphics);
impl_mergeable!(PolylineGraphics);
impl_mergeable!(PolylineVolumeGraphics);
impl_mergeable!(RectangleGraphics);
impl_mergeable!(WallGraphics);

const RESERVED_PROPERTY_NAMES: &[&str] = &[
    "id",
    "name",
    "show",
    "trackingReferenceFrame",
    "availability",
    "description",
    "position",
    "orientation",
    "viewFrom",
    "parent",
    "children",
    "billboard",
    "box",
    "corridor",
    "cylinder",
    "ellipse",
    "ellipsoid",
    "label",
    "model",
    "tileset",
    "path",
    "plane",
    "point",
    "polygon",
    "polyline",
    "polylineVolume",
    "rectangle",
    "wall",
    "properties",
    "merge",
    "computeModelMatrix",
    "isAvailable",
    "isShowing",
    "addProperty",
    "removeProperty",
    "propertyNames",
];

/// Entity instances aggregate multiple forms of visualization into a single high-level object.
#[derive(Clone, Default)]
pub struct Entity {
    pub id: String,
    pub name: Option<String>,
    pub show: bool,
    pub tracking_reference_frame: TrackingReferenceFrame,
    pub availability: Option<TimeIntervalCollection>,
    pub description: Option<ConstantProperty<String>>,
    pub position: Option<ConstantPositionProperty>,
    pub orientation: Option<ConstantProperty<Quaternion>>,
    pub view_from: Option<ConstantProperty<Cartesian3>>,
    pub billboard: Option<BillboardGraphics>,
    pub box_graphics: Option<BoxGraphics>,
    pub corridor: Option<CorridorGraphics>,
    pub cylinder: Option<CylinderGraphics>,
    pub ellipse: Option<EllipseGraphics>,
    pub ellipsoid: Option<EllipsoidGraphics>,
    pub label: Option<LabelGraphics>,
    pub model: Option<ModelGraphics>,
    pub tileset: Option<Cesium3DTilesetGraphics>,
    pub path: Option<PathGraphics>,
    pub plane: Option<PlaneGraphics>,
    pub point: Option<PointGraphics>,
    pub polygon: Option<PolygonGraphics>,
    pub polyline: Option<PolylineGraphics>,
    pub polyline_volume: Option<PolylineVolumeGraphics>,
    pub rectangle: Option<RectangleGraphics>,
    pub wall: Option<WallGraphics>,
    pub properties: PropertyBag<serde_json::Value>,
    pub parent: Option<Rc<RefCell<Entity>>>,
}

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Entity")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("show", &self.show)
            .field("tracking_reference_frame", &self.tracking_reference_frame)
            .finish_non_exhaustive()
    }
}

impl PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Entity {
    /// Creates a new Entity with a generated unique ID and default options.
    pub fn new() -> Self {
        Self {
            id: create_guid(),
            name: None,
            show: true,
            tracking_reference_frame: TrackingReferenceFrame::Autodetect,
            availability: None,
            description: None,
            position: None,
            orientation: None,
            view_from: None,
            billboard: None,
            box_graphics: None,
            corridor: None,
            cylinder: None,
            ellipse: None,
            ellipsoid: None,
            label: None,
            model: None,
            tileset: None,
            path: None,
            plane: None,
            point: None,
            polygon: None,
            polyline: None,
            polyline_volume: None,
            rectangle: None,
            wall: None,
            properties: PropertyBag::new(),
            parent: None,
        }
    }

    /// Creates a new Entity with an explicit unique ID.
    pub fn with_id(id: impl Into<String>) -> Self {
        let mut entity = Self::new();
        entity.id = id.into();
        entity
    }

    /// Given a time, returns true if this object should have data during that time.
    pub fn is_available(&self, time: &JulianDate) -> bool {
        match &self.availability {
            Some(availability) => availability.contains(time),
            None => true,
        }
    }

    /// Gets whether this entity is currently displayed. When true, the entity is only
    /// displayed if all parent entities' show properties are also true.
    pub fn is_showing(&self) -> bool {
        if !self.show {
            return false;
        }
        match &self.parent {
            Some(parent) => parent.borrow().is_showing(),
            None => true,
        }
    }

    /// Computes the model matrix for the entity's transform at specified time.
    /// Returns None if position is None.
    pub fn compute_model_matrix(&self, time: &JulianDate) -> Option<Matrix4> {
        let position = self.position.as_ref()?.get_value(Some(time))?;
        if let Some(orientation_prop) = &self.orientation {
            if let Some(orientation) = orientation_prop.get_value(Some(time)) {
                let rot = Matrix3::from_quaternion(&orientation);
                return Some(Matrix4::from_rotation_translation(&rot, position));
            }
        }
        FixedFrameTransforms::east_north_up_to_fixed_frame(position, None).ok()
    }

    /// Computes the model matrix with a provided result buffer.
    pub fn compute_model_matrix_with_result(
        &self,
        time: &JulianDate,
        result: &mut Matrix4,
    ) -> Option<Matrix4> {
        let matrix = self.compute_model_matrix(time)?;
        *result = matrix;
        Some(matrix)
    }

    /// Assigns each unassigned property on this object to the value of the same property on the provided source object.
    pub fn merge(&mut self, source: &Entity) {
        if self.name.is_none() {
            self.name = source.name.clone();
        }
        if self.availability.is_none() {
            self.availability = source.availability.clone();
        }
        Self::merge_item(&mut self.billboard, &source.billboard);
        Self::merge_item(&mut self.box_graphics, &source.box_graphics);
        Self::merge_item(&mut self.corridor, &source.corridor);
        Self::merge_item(&mut self.cylinder, &source.cylinder);
        Self::merge_item(&mut self.ellipse, &source.ellipse);
        Self::merge_item(&mut self.ellipsoid, &source.ellipsoid);
        Self::merge_item(&mut self.label, &source.label);
        Self::merge_item(&mut self.model, &source.model);
        Self::merge_item(&mut self.tileset, &source.tileset);
        Self::merge_item(&mut self.path, &source.path);
        Self::merge_item(&mut self.plane, &source.plane);
        Self::merge_item(&mut self.point, &source.point);
        Self::merge_item(&mut self.polygon, &source.polygon);
        Self::merge_item(&mut self.polyline, &source.polyline);
        Self::merge_item(&mut self.polyline_volume, &source.polyline_volume);
        Self::merge_item(&mut self.rectangle, &source.rectangle);
        Self::merge_item(&mut self.wall, &source.wall);

        if self.description.is_none() {
            self.description = source.description.clone();
        }
        if self.position.is_none() {
            self.position = source.position.clone();
        }
        if self.orientation.is_none() {
            self.orientation = source.orientation.clone();
        }
        if self.view_from.is_none() {
            self.view_from = source.view_from.clone();
        }

        self.properties.merge(&source.properties);
    }

    fn merge_item<T: Clone + Mergeable>(target: &mut Option<T>, source: &Option<T>) {
        match (target.as_mut(), source.as_ref()) {
            (Some(t), Some(s)) => t.merge(s),
            (None, Some(s)) => *target = Some(s.clone()),
            _ => {}
        }
    }

    /// Adds a dynamic custom property to this object.
    pub fn add_property(&mut self, property_name: &str) -> Result<(), DeveloperError> {
        if property_name.is_empty() {
            return Err(DeveloperError::new("Property name cannot be empty."));
        }
        if RESERVED_PROPERTY_NAMES.contains(&property_name) {
            return Err(DeveloperError::new(format!(
                "Property name '{property_name}' is reserved."
            )));
        }
        if self.properties.has_property(property_name) {
            return Err(DeveloperError::new(format!(
                "Property '{property_name}' already exists."
            )));
        }
        self.properties
            .add_property(property_name, None);
        Ok(())
    }

    /// Removes a custom property from this object.
    pub fn remove_property(&mut self, property_name: &str) -> Result<(), DeveloperError> {
        if property_name.is_empty() {
            return Err(DeveloperError::new("Property name cannot be empty."));
        }
        if RESERVED_PROPERTY_NAMES.contains(&property_name) {
            return Err(DeveloperError::new(format!(
                "Property name '{property_name}' is reserved."
            )));
        }
        if !self.properties.has_property(property_name) {
            return Err(DeveloperError::new(format!(
                "Property '{property_name}' does not exist."
            )));
        }
        self.properties.remove_property(property_name);
        Ok(())
    }

    /// Checks if a custom property is registered.
    pub fn has_property(&self, property_name: &str) -> bool {
        self.properties.has_property(property_name)
    }

    /// Gets the list of all property names on this entity.
    pub fn property_names(&self) -> Vec<String> {
        let mut list = vec![
            "billboard".to_string(),
            "box".to_string(),
            "corridor".to_string(),
            "cylinder".to_string(),
            "description".to_string(),
            "ellipse".to_string(),
            "ellipsoid".to_string(),
            "label".to_string(),
            "model".to_string(),
            "tileset".to_string(),
            "orientation".to_string(),
            "path".to_string(),
            "plane".to_string(),
            "point".to_string(),
            "polygon".to_string(),
            "polyline".to_string(),
            "polylineVolume".to_string(),
            "position".to_string(),
            "properties".to_string(),
            "rectangle".to_string(),
            "viewFrom".to_string(),
            "wall".to_string(),
        ];
        list.extend(self.properties.property_names().iter().cloned());
        list
    }
}
