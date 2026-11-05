//! VelocityVectorProperty and VelocityOrientationProperty matching CesiumJS `DataSources/VelocityVectorProperty.js` and `VelocityOrientationProperty.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use crate::{
    Cartesian3, DynPositionProperty, DynProperty, Ellipsoid, Event, EventHelper,
    FixedFrameTransforms, JulianDate, Quaternion,
};

/// A Property which evaluates to a Cartesian3 vector based on the velocity of the provided PositionProperty.
#[derive(Clone)]
pub struct VelocityVectorProperty {
    position: Option<Rc<dyn DynPositionProperty>>,
    normalize: bool,
    definition_changed: Event<()>,
    event_helper: Rc<RefCell<EventHelper>>,
}

impl Default for VelocityVectorProperty {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl VelocityVectorProperty {
    pub fn new(position: Option<Rc<dyn DynPositionProperty>>, normalize: Option<bool>) -> Self {
        let mut slf = Self {
            position: None,
            normalize: normalize.unwrap_or(true),
            definition_changed: Event::new(),
            event_helper: Rc::new(RefCell::new(EventHelper::new())),
        };
        slf.set_position(position);
        slf
    }

    pub fn is_constant(&self) -> bool {
        match &self.position {
            Some(p) => p.is_constant(),
            None => true,
        }
    }

    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    pub fn normalize(&self) -> bool {
        self.normalize
    }

    pub fn set_normalize(&mut self, normalize: bool) {
        if self.normalize != normalize {
            self.normalize = normalize;
            self.definition_changed.raise_event(&());
        }
    }

    pub fn position(&self) -> Option<Rc<dyn DynPositionProperty>> {
        self.position.clone()
    }

    pub fn set_position(&mut self, position: Option<Rc<dyn DynPositionProperty>>) {
        let is_same = match (&self.position, &position) {
            (None, None) => true,
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            _ => false,
        };

        if is_same {
            return;
        }

        self.event_helper.borrow_mut().remove_all();

        if let Some(ref p) = position {
            let dc = self.definition_changed.clone();
            self.event_helper.borrow_mut().add(p.definition_changed(), {
                let ev = dc.clone();
                move |_| ev.raise_event(&())
            });
        }

        self.position = position;
        self.definition_changed.raise_event(&());
    }

    pub fn get_value(&self, time: Option<&JulianDate>) -> Option<Cartesian3> {
        self.get_value_internal(time).0
    }

    pub fn get_value_internal(
        &self,
        time: Option<&JulianDate>,
    ) -> (Option<Cartesian3>, Option<Cartesian3>) {
        let default_time;
        let t = match time {
            Some(time) => time,
            None => {
                default_time = JulianDate::now();
                &default_time
            }
        };

        let property = match &self.position {
            Some(p) => p,
            None => return (None, None),
        };

        if property.is_constant() {
            let vel = if self.normalize {
                None
            } else {
                Some(Cartesian3::ZERO)
            };
            return (vel, property.get_value(Some(t)));
        }

        let step = 1.0 / 60.0;
        let mut position1 = property.get_value(Some(t));
        let t_plus_step = JulianDate::add_seconds(t, step);
        let mut position2 = property.get_value(Some(&t_plus_step));

        if position1.is_none() {
            return (None, None);
        }

        if position2.is_none() {
            position2 = position1;
            let t_minus_step = JulianDate::add_seconds(t, -step);
            position1 = property.get_value(Some(&t_minus_step));
            if position1.is_none() {
                return (None, None);
            }
        }

        let p1 = position1.unwrap();
        let p2 = position2.unwrap();

        if p1 == p2 {
            let vel = if self.normalize {
                None
            } else {
                Some(Cartesian3::ZERO)
            };
            return (vel, Some(p1));
        }

        let diff = p2 - p1;
        let vel = if self.normalize {
            if diff.magnitude_squared() == 0.0 {
                None
            } else {
                Some(diff.normalize())
            }
        } else {
            Some(diff.divide_by_scalar(step))
        };

        (vel, Some(p1))
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        let other = match other {
            Some(o) => o,
            None => return false,
        };

        if self.normalize != other.normalize {
            return false;
        }

        match (&self.position, &other.position) {
            (None, None) => true,
            (Some(a), Some(b)) => a.equals_dyn(&**b),
            _ => false,
        }
    }
}

impl DynProperty<Cartesian3> for VelocityVectorProperty {
    fn is_constant(&self) -> bool {
        self.is_constant()
    }
    fn definition_changed(&self) -> &Event<()> {
        self.definition_changed()
    }
    fn get_value(&self, time: Option<&JulianDate>) -> Option<Cartesian3> {
        self.get_value(time)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn equals_dyn(&self, other: &dyn DynProperty<Cartesian3>) -> bool {
        if let Some(o) = other.as_any().downcast_ref::<Self>() {
            self.equals(Some(o))
        } else {
            false
        }
    }
}

/// A Property which evaluates to a Quaternion rotation based on the velocity of the provided PositionProperty.
#[derive(Clone)]
pub struct VelocityOrientationProperty {
    velocity_vector_property: VelocityVectorProperty,
    ellipsoid: Ellipsoid,
    definition_changed: Event<()>,
    _event_helper: Rc<RefCell<EventHelper>>,
}

impl Default for VelocityOrientationProperty {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl VelocityOrientationProperty {
    pub fn new(
        position: Option<Rc<dyn DynPositionProperty>>,
        ellipsoid: Option<Ellipsoid>,
    ) -> Self {
        let vv = VelocityVectorProperty::new(position, Some(true));
        let def_changed = Event::new();
        let helper = Rc::new(RefCell::new(EventHelper::new()));

        let dc = def_changed.clone();
        helper.borrow_mut().add(vv.definition_changed(), move |_| {
            dc.raise_event(&());
        });

        Self {
            velocity_vector_property: vv,
            ellipsoid: ellipsoid.unwrap_or(Ellipsoid::WGS84),
            definition_changed: def_changed,
            _event_helper: helper,
        }
    }

    pub fn is_constant(&self) -> bool {
        self.velocity_vector_property.is_constant()
    }

    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    pub fn position(&self) -> Option<Rc<dyn DynPositionProperty>> {
        self.velocity_vector_property.position()
    }

    pub fn set_position(&mut self, position: Option<Rc<dyn DynPositionProperty>>) {
        self.velocity_vector_property.set_position(position);
    }

    pub fn ellipsoid(&self) -> Ellipsoid {
        self.ellipsoid
    }

    pub fn set_ellipsoid(&mut self, ellipsoid: Ellipsoid) {
        if self.ellipsoid != ellipsoid {
            self.ellipsoid = ellipsoid;
            self.definition_changed.raise_event(&());
        }
    }

    pub fn get_value(&self, time: Option<&JulianDate>) -> Option<Quaternion> {
        let (velocity, position) = self.velocity_vector_property.get_value_internal(time);
        let vel = velocity?;
        let pos = position?;

        let rot_matrix = FixedFrameTransforms::rotation_matrix_from_position_velocity(
            pos,
            vel,
            Some(self.ellipsoid),
        )
        .ok()?;

        Some(Quaternion::from_rotation_matrix(&rot_matrix))
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        let other = match other {
            Some(o) => o,
            None => return false,
        };

        self.ellipsoid == other.ellipsoid
            && self
                .velocity_vector_property
                .equals(Some(&other.velocity_vector_property))
    }
}

impl DynProperty<Quaternion> for VelocityOrientationProperty {
    fn is_constant(&self) -> bool {
        self.is_constant()
    }
    fn definition_changed(&self) -> &Event<()> {
        self.definition_changed()
    }
    fn get_value(&self, time: Option<&JulianDate>) -> Option<Quaternion> {
        self.get_value(time)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn equals_dyn(&self, other: &dyn DynProperty<Quaternion>) -> bool {
        if let Some(o) = other.as_any().downcast_ref::<Self>() {
            self.equals(Some(o))
        } else {
            false
        }
    }
}
