//! ReferenceProperty matching CesiumJS `DataSources/ReferenceProperty.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use crate::{
    Cartesian3, Color, ConstantPositionProperty, DeveloperError, DynProperty, EntityCollection,
    Event, JulianDate, ListenerId, MaterialProperty, MaterialValue, ReferenceFrame,
};

/// Internal representation of a resolved target property.
#[derive(Clone)]
pub enum ResolvedPropertyTarget {
    F64(Rc<dyn DynProperty<f64>>),
    Bool(Rc<dyn DynProperty<bool>>),
    String(Rc<dyn DynProperty<String>>),
    Cartesian3(Rc<dyn DynProperty<Cartesian3>>),
    Color(Rc<dyn DynProperty<Color>>),
    Position(ConstantPositionProperty),
    Material(Rc<dyn MaterialProperty>),
}

impl std::fmt::Debug for ResolvedPropertyTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::F64(_) => write!(f, "ResolvedPropertyTarget::F64"),
            Self::Bool(_) => write!(f, "ResolvedPropertyTarget::Bool"),
            Self::String(_) => write!(f, "ResolvedPropertyTarget::String"),
            Self::Cartesian3(_) => write!(f, "ResolvedPropertyTarget::Cartesian3"),
            Self::Color(_) => write!(f, "ResolvedPropertyTarget::Color"),
            Self::Position(p) => write!(f, "ResolvedPropertyTarget::Position({p:?})"),
            Self::Material(_) => write!(f, "ResolvedPropertyTarget::Material"),
        }
    }
}

impl ResolvedPropertyTarget {
    pub fn is_constant(&self) -> bool {
        match self {
            Self::F64(p) => p.is_constant(),
            Self::Bool(p) => p.is_constant(),
            Self::String(p) => p.is_constant(),
            Self::Cartesian3(p) => p.is_constant(),
            Self::Color(p) => p.is_constant(),
            Self::Position(p) => p.is_constant(),
            Self::Material(p) => p.is_constant(),
        }
    }
}

struct ReferencePropertyState {
    target_attached: bool,
    _collection_listener_id: Option<ListenerId>,
    target_listener_id: Option<ListenerId>,
}

/// A Property which transparently links to another property on a provided object.
#[derive(Clone)]
pub struct ReferenceProperty {
    target_collection: Rc<RefCell<EntityCollection>>,
    target_id: String,
    target_property_names: Vec<String>,
    definition_changed: Event<()>,
    state: Rc<RefCell<ReferencePropertyState>>,
}

impl std::fmt::Debug for ReferenceProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReferenceProperty")
            .field("target_id", &self.target_id)
            .field("target_property_names", &self.target_property_names)
            .finish()
    }
}

impl ReferenceProperty {
    /// Creates a new ReferenceProperty.
    pub fn new(
        target_collection: Rc<RefCell<EntityCollection>>,
        target_id: impl Into<String>,
        target_property_names: Vec<String>,
    ) -> Result<Self, DeveloperError> {
        let target_id = target_id.into();
        if target_id.is_empty() {
            return Err(DeveloperError::new("targetId is required."));
        }
        if target_property_names.is_empty() {
            return Err(DeveloperError::new("targetPropertyNames is required."));
        }
        for name in &target_property_names {
            if name.is_empty() {
                return Err(DeveloperError::new("reference contains invalid properties."));
            }
        }

        let definition_changed = Event::new();
        let state = Rc::new(RefCell::new(ReferencePropertyState {
            target_attached: false,
            _collection_listener_id: None,
            target_listener_id: None,
        }));

        let property = Self {
            target_collection,
            target_id,
            target_property_names,
            definition_changed,
            state,
        };

        property.setup_listeners();
        Ok(property)
    }

    /// Creates a new instance given the entity collection and a string indicating the target entity id and property.
    /// The format is "objectId#foo.bar", where # separates id from property path and . separates sub-properties.
    pub fn from_string(
        target_collection: Rc<RefCell<EntityCollection>>,
        reference_string: &str,
    ) -> Result<Self, DeveloperError> {
        if reference_string.is_empty() {
            return Err(DeveloperError::new("referenceString is required."));
        }

        let mut identifier = None;
        let mut values = Vec::new();
        let mut in_identifier = true;
        let mut is_escaped = false;
        let mut token = String::new();

        for c in reference_string.chars() {
            if is_escaped {
                token.push(c);
                is_escaped = false;
            } else if c == '\\' {
                is_escaped = true;
            } else if in_identifier && c == '#' {
                identifier = Some(token);
                in_identifier = false;
                token = String::new();
            } else if !in_identifier && c == '.' {
                values.push(token);
                token = String::new();
            } else {
                token.push(c);
            }
        }
        values.push(token);

        let target_id = match identifier {
            Some(id) if !id.is_empty() => id,
            _ => return Err(DeveloperError::new("targetId is required.")),
        };

        Self::new(target_collection, target_id, values)
    }

    fn setup_listeners(&self) {
        let dc = self.definition_changed.clone();
        let state = Rc::clone(&self.state);
        let target_id = self.target_id.clone();
        let property_names = self.target_property_names.clone();

        // Check if target entity currently exists in collection
        {
            let col = self.target_collection.borrow();
            if let Some(target_entity) = col.get_by_id(&target_id) {
                if resolve_from_entity(target_entity, &property_names).is_some() {
                    let mut st = state.borrow_mut();
                    st.target_attached = true;
                    let dc_inner = dc.clone();
                    let first_prop = property_names[0].clone();
                    let lid = target_entity.definition_changed.add_listener(move |changed_name: &String| {
                        if changed_name.is_empty() || *changed_name == first_prop {
                            dc_inner.raise_event(&());
                        }
                    });
                    st.target_listener_id = Some(lid);
                }
            }
        }

        let state_col = Rc::clone(&state);
        let target_id_col = target_id.clone();
        let property_names_col = property_names.clone();
        let dc_col = dc.clone();

        let col_lid = self
            .target_collection
            .borrow_mut()
            .collection_changed
            .add_listener(move |(added, removed, _)| {
                let mut st = state_col.borrow_mut();
                if st.target_attached {
                    // Check if target entity was removed
                    let was_removed = removed.iter().any(|e| e.id == target_id_col);
                    if was_removed {
                        st.target_attached = false;
                        st.target_listener_id = None;
                        // Note: Removing entity from collection does NOT raise definition_changed,
                        // matching CesiumJS ReferencePropertySpec behavior.
                    }
                } else {
                    // Check if target entity was added and can be resolved
                    if let Some(entity) = added.iter().find(|e| e.id == target_id_col) {
                        if resolve_from_entity(entity, &property_names_col).is_some() {
                            st.target_attached = true;
                            let dc_inner = dc_col.clone();
                            let first_prop = property_names_col[0].clone();
                            let lid = entity.definition_changed.add_listener(move |changed_name: &String| {
                                if changed_name.is_empty() || *changed_name == first_prop {
                                    dc_inner.raise_event(&());
                                }
                            });
                            st.target_listener_id = Some(lid);
                            dc_col.raise_event(&());
                        }
                    }
                }
            });

        self.state.borrow_mut()._collection_listener_id = Some(col_lid);
    }

    /// Gets the entity collection which will be used to resolve the reference.
    pub fn target_collection(&self) -> &Rc<RefCell<EntityCollection>> {
        &self.target_collection
    }

    /// Gets the id of the entity which is being referenced.
    pub fn target_id(&self) -> &str {
        &self.target_id
    }

    /// Gets the names of the property on the target entity which we will use.
    pub fn target_property_names(&self) -> &[String] {
        &self.target_property_names
    }

    /// Gets the event that is raised whenever the definition of this property changes.
    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    /// Resolves the current target property from the collection.
    pub fn resolve(&self) -> Option<ResolvedPropertyTarget> {
        let col = self.target_collection.borrow();
        let entity = col.get_by_id(&self.target_id)?;
        resolve_from_entity(entity, &self.target_property_names)
    }

    /// Gets whether the underlying property has been resolved.
    pub fn has_resolved_property(&self) -> bool {
        self.resolve().is_some()
    }

    /// Gets the resolved f64 scale property, if resolved.
    pub fn resolved_scale_property(&self) -> Option<Rc<dyn DynProperty<f64>>> {
        match self.resolve()? {
            ResolvedPropertyTarget::F64(p) => Some(p),
            _ => None,
        }
    }

    /// Gets a value indicating if this property is constant.
    pub fn is_constant(&self) -> bool {
        match self.resolve() {
            Some(target) => target.is_constant(),
            None => true, // Property.isConstant(undefined) is true in CesiumJS
        }
    }

    /// Gets the reference frame that the position is defined in.
    pub fn reference_frame(&self) -> Option<ReferenceFrame> {
        match self.resolve()? {
            ResolvedPropertyTarget::Position(p) => Some(p.reference_frame()),
            _ => None,
        }
    }

    /// Gets the Material type at the provided time.
    pub fn get_type(&self, time: Option<&JulianDate>) -> Option<String> {
        let target = self.resolve()?;
        match target {
            ResolvedPropertyTarget::Material(mat) => {
                let default_time;
                let t = match time {
                    Some(t) => t,
                    None => {
                        default_time = JulianDate::now();
                        &default_time
                    }
                };
                Some(mat.get_type(t).to_string())
            }
            _ => None,
        }
    }

    /// Gets the value of the property at the provided time for a given type.
    pub fn get_value<V: Clone + 'static>(&self, time: Option<&JulianDate>) -> Option<V> {
        let target = self.resolve()?;
        match target {
            ResolvedPropertyTarget::F64(p) => {
                let val = p.get_value(time)?;
                (&val as &dyn std::any::Any).downcast_ref::<V>().cloned()
            }
            ResolvedPropertyTarget::Bool(p) => {
                let val = p.get_value(time)?;
                (&val as &dyn std::any::Any).downcast_ref::<V>().cloned()
            }
            ResolvedPropertyTarget::String(p) => {
                let val = p.get_value(time)?;
                (&val as &dyn std::any::Any).downcast_ref::<V>().cloned()
            }
            ResolvedPropertyTarget::Cartesian3(p) => {
                let val = p.get_value(time)?;
                (&val as &dyn std::any::Any).downcast_ref::<V>().cloned()
            }
            ResolvedPropertyTarget::Color(p) => {
                let val = p.get_value(time)?;
                (&val as &dyn std::any::Any).downcast_ref::<V>().cloned()
            }
            ResolvedPropertyTarget::Position(p) => {
                let val = p.get_value(time)?;
                (&val as &dyn std::any::Any).downcast_ref::<V>().cloned()
            }
            ResolvedPropertyTarget::Material(mat) => {
                if let Some(mv) = mat.get_material_value(time) {
                    if let Some(v) = (&mv as &dyn std::any::Any).downcast_ref::<V>() {
                        return Some(v.clone());
                    }
                    if let MaterialValue::Color(c) = mv {
                        if let Some(v) = (&c as &dyn std::any::Any).downcast_ref::<V>() {
                            return Some(v.clone());
                        }
                    }
                }
                None
            }
        }
    }

    /// Gets the value of the property at the provided time and in the provided reference frame.
    pub fn get_value_in_reference_frame(
        &self,
        time: Option<&JulianDate>,
        reference_frame: ReferenceFrame,
    ) -> Option<Cartesian3> {
        match self.resolve()? {
            ResolvedPropertyTarget::Position(p) => {
                p.get_value_in_reference_frame(time, reference_frame)
            }
            _ => None,
        }
    }

    /// Compares this property to the provided property and returns true if they are equal.
    pub fn equals(&self, other: &Self) -> bool {
        self.target_collection.borrow().id() == other.target_collection.borrow().id()
            && self.target_id == other.target_id
            && self.target_property_names == other.target_property_names
    }
}

impl PartialEq for ReferenceProperty {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

fn resolve_from_entity(
    entity: &crate::Entity,
    property_names: &[String],
) -> Option<ResolvedPropertyTarget> {
    if property_names.is_empty() {
        return None;
    }

    let first = property_names[0].as_str();

    if property_names.len() == 1 {
        match first {
            "position" => {
                entity.position.as_ref().map(|p| ResolvedPropertyTarget::Position(p.clone()))
            }
            name => entity
                .custom_materials
                .get(name)
                .map(|mat| ResolvedPropertyTarget::Material(Rc::clone(mat))),
        }
    } else if property_names.len() == 2 {
        let second = property_names[1].as_str();
        match first {
            "billboard" => {
                let bb = entity.billboard.as_ref()?;
                match second {
                    "scale" => bb.scale.as_ref().map(|p| ResolvedPropertyTarget::F64(Rc::clone(p))),
                    "rotation" => bb.rotation.as_ref().map(|p| ResolvedPropertyTarget::F64(Rc::clone(p))),
                    "width" => bb.width.as_ref().map(|p| ResolvedPropertyTarget::F64(Rc::clone(p))),
                    "height" => bb.height.as_ref().map(|p| ResolvedPropertyTarget::F64(Rc::clone(p))),
                    "show" => bb.show.as_ref().map(|p| ResolvedPropertyTarget::Bool(Rc::clone(p))),
                    "color" => bb.color.as_ref().map(|p| ResolvedPropertyTarget::Color(Rc::clone(p))),
                    "aligned_axis" => bb.aligned_axis.as_ref().map(|p| ResolvedPropertyTarget::Cartesian3(Rc::clone(p))),
                    _ => None,
                }
            }
            "box" | "box_graphics" => {
                let b = entity.box_graphics.as_ref()?;
                match second {
                    "material" => b.material.as_ref().map(|p| ResolvedPropertyTarget::Material(Rc::clone(p))),
                    "dimensions" => b.dimensions.as_ref().map(|p| ResolvedPropertyTarget::Cartesian3(Rc::clone(p))),
                    _ => None,
                }
            }
            "model" => {
                let m = entity.model.as_ref()?;
                match second {
                    "scale" => m.scale.as_ref().map(|p| ResolvedPropertyTarget::F64(Rc::clone(p))),
                    _ => None,
                }
            }
            "corridor" => {
                let c = entity.corridor.as_ref()?;
                match second {
                    "material" => c.material.as_ref().map(|p| ResolvedPropertyTarget::Material(Rc::clone(p))),
                    _ => None,
                }
            }
            "polyline" => {
                let p = entity.polyline.as_ref()?;
                match second {
                    "material" => p.material.as_ref().map(|m| ResolvedPropertyTarget::Material(Rc::clone(m))),
                    _ => None,
                }
            }
            "polygon" => {
                let p = entity.polygon.as_ref()?;
                match second {
                    "material" => p.material.as_ref().map(|m| ResolvedPropertyTarget::Material(Rc::clone(m))),
                    _ => None,
                }
            }
            _ => None,
        }
    } else {
        None
    }
}
