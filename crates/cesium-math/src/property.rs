//! Property interfaces and implementations matching CesiumJS `DataSources/Property.js` (Apache-2.0).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::{
    Cartesian3, CelestialFrameTransforms, Event, EventHelper, JulianDate, ReferenceFrame,
    TimeIntervalCollection,
};

/// The interface for all properties, which represent a value that can optionally vary over time.
pub trait Property<T: Clone> {
    /// Gets a value indicating if this property is constant.
    fn is_constant(&self) -> bool;

    /// Gets the event that is raised whenever the definition of this property changes.
    fn definition_changed(&self) -> &Event<()>;

    /// Gets the value of the property at the provided time.
    fn get_value(&self, time: Option<&JulianDate>) -> Option<T>;

    /// Compares this property to the provided property and returns true if they are equal.
    fn equals(&self, other: Option<&Self>) -> bool;
}

/// A Property whose value does not change with respect to simulation time.
#[derive(Debug)]
pub struct ConstantProperty<T: Clone + PartialEq> {
    value: RefCell<Option<T>>,
    definition_changed: Event<()>,
}

impl<T: Clone + PartialEq> Clone for ConstantProperty<T> {
    fn clone(&self) -> Self {
        Self {
            value: RefCell::new(self.value.borrow().clone()),
            definition_changed: self.definition_changed.clone(),
        }
    }
}

impl<T: Clone + PartialEq> ConstantProperty<T> {
    pub fn new(value: Option<T>) -> Self {
        Self {
            value: RefCell::new(value),
            definition_changed: Event::new(),
        }
    }

    pub fn from_value(value: T) -> Self {
        Self::new(Some(value))
    }

    pub fn is_constant(&self) -> bool {
        true
    }

    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    pub fn get_value(&self, _time: Option<&JulianDate>) -> Option<T> {
        self.value.borrow().clone()
    }

    pub fn set_value(&self, value: Option<T>) {
        let mut val = self.value.borrow_mut();
        if *val != value {
            *val = value;
            drop(val);
            self.definition_changed.raise_event(&());
        }
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        match other {
            Some(o) => *self.value.borrow() == *o.value.borrow(),
            None => false,
        }
    }
}

impl<T: Clone + PartialEq> PartialEq for ConstantProperty<T> {
    fn eq(&self, other: &Self) -> bool {
        *self.value.borrow() == *other.value.borrow()
    }
}

impl<T: Clone + PartialEq + std::fmt::Display> std::fmt::Display for ConstantProperty<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &*self.value.borrow() {
            Some(v) => write!(f, "{v}"),
            None => write!(f, "undefined"),
        }
    }
}

/// A Property that represents a position that can optionally vary over time.
pub struct PositionProperty;

impl PositionProperty {
    /// Converts a position from one reference frame to another at the given time.
    pub fn convert_to_reference_frame(
        time: Option<&JulianDate>,
        value: Option<Cartesian3>,
        input_frame: ReferenceFrame,
        output_frame: ReferenceFrame,
    ) -> Option<Cartesian3> {
        let val = value?;
        if input_frame == output_frame {
            return Some(val);
        }
        let t = time?;
        let icrf_to_fixed =
            CelestialFrameTransforms::compute_icrf_to_central_body_fixed_matrix(t);
        let transformed = match input_frame {
            ReferenceFrame::Inertial => icrf_to_fixed.multiply_by_vector(val),
            ReferenceFrame::Fixed => icrf_to_fixed.transpose().multiply_by_vector(val),
        };
        Some(transformed)
    }
}

/// A PositionProperty whose value does not change with respect to simulation time.
#[derive(Debug)]
pub struct ConstantPositionProperty {
    value: RefCell<Option<Cartesian3>>,
    reference_frame: Cell<ReferenceFrame>,
    definition_changed: Event<()>,
}

impl Clone for ConstantPositionProperty {
    fn clone(&self) -> Self {
        Self {
            value: RefCell::new(*self.value.borrow()),
            reference_frame: Cell::new(self.reference_frame.get()),
            definition_changed: self.definition_changed.clone(),
        }
    }
}

impl ConstantPositionProperty {
    pub fn new(value: Option<Cartesian3>, reference_frame: Option<ReferenceFrame>) -> Self {
        Self {
            value: RefCell::new(value),
            reference_frame: Cell::new(reference_frame.unwrap_or(ReferenceFrame::Fixed)),
            definition_changed: Event::new(),
        }
    }

    pub fn reference_frame(&self) -> ReferenceFrame {
        self.reference_frame.get()
    }

    pub fn is_constant(&self) -> bool {
        true
    }

    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    pub fn get_value(&self, time: Option<&JulianDate>) -> Option<Cartesian3> {
        self.get_value_in_reference_frame(time, ReferenceFrame::Fixed)
    }

    pub fn get_value_in_reference_frame(
        &self,
        time: Option<&JulianDate>,
        target_frame: ReferenceFrame,
    ) -> Option<Cartesian3> {
        PositionProperty::convert_to_reference_frame(
            time,
            *self.value.borrow(),
            self.reference_frame.get(),
            target_frame,
        )
    }

    pub fn set_value(
        &self,
        value: Option<Cartesian3>,
        reference_frame: Option<ReferenceFrame>,
    ) {
        let new_frame = reference_frame.unwrap_or(self.reference_frame.get());
        let mut val = self.value.borrow_mut();
        let changed = *val != value || self.reference_frame.get() != new_frame;
        if changed {
            *val = value;
            self.reference_frame.set(new_frame);
            drop(val);
            self.definition_changed.raise_event(&());
        }
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        match other {
            Some(o) => {
                self.reference_frame.get() == o.reference_frame.get()
                    && *self.value.borrow() == *o.value.borrow()
            }
            None => false,
        }
    }
}

impl PartialEq for ConstantPositionProperty {
    fn eq(&self, other: &Self) -> bool {
        self.reference_frame.get() == other.reference_frame.get()
            && *self.value.borrow() == *other.value.borrow()
    }
}

/// Interval data wrapper for CompositeProperty holding a dynamic property.
#[derive(Clone)]
pub struct PropertyInterval<T: Clone + 'static> {
    pub property: Rc<dyn DynProperty<T>>,
}

impl<T: Clone + 'static> PropertyInterval<T> {
    pub fn new(property: Rc<dyn DynProperty<T>>) -> Self {
        Self { property }
    }
}

impl<T: Clone + 'static> PartialEq for PropertyInterval<T> {
    fn eq(&self, other: &Self) -> bool {
        if Rc::ptr_eq(&self.property, &other.property) {
            return true;
        }
        self.property.equals_dyn(&*other.property)
    }
}

impl<T: Clone + 'static> From<Rc<dyn DynProperty<T>>> for PropertyInterval<T> {
    fn from(property: Rc<dyn DynProperty<T>>) -> Self {
        Self { property }
    }
}

impl<T: Clone + PartialEq + 'static> From<ConstantProperty<T>> for PropertyInterval<T> {
    fn from(prop: ConstantProperty<T>) -> Self {
        Self {
            property: Rc::new(prop),
        }
    }
}

impl<T: Clone + PartialEq + 'static> From<Rc<ConstantProperty<T>>> for PropertyInterval<T> {
    fn from(prop: Rc<ConstantProperty<T>>) -> Self {
        Self { property: prop }
    }
}

/// A Property which is defined by a TimeIntervalCollection, where the data property of each
/// TimeInterval is another Property instance which is evaluated at the provided time.
pub struct CompositeProperty<T: Clone + 'static> {
    pub intervals: TimeIntervalCollection<PropertyInterval<T>>,
    definition_changed: Event<()>,
    event_helper: Rc<RefCell<EventHelper>>,
}

impl<T: Clone + 'static> Default for CompositeProperty<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + 'static> Clone for CompositeProperty<T> {
    fn clone(&self) -> Self {
        let mut slf = Self::new();
        slf.intervals = self.intervals.clone();
        slf.re_subscribe(&slf.intervals);
        slf
    }
}

impl<T: Clone + 'static> CompositeProperty<T> {
    pub fn new() -> Self {
        let intervals: TimeIntervalCollection<PropertyInterval<T>> = TimeIntervalCollection::new();
        let definition_changed = Event::new();
        let event_helper = Rc::new(RefCell::new(EventHelper::new()));

        let dc = definition_changed.clone();
        let eh = Rc::clone(&event_helper);
        intervals.changed_event().add_listener(move |curr_intervals| {
            eh.borrow_mut().remove_all();
            for interval in curr_intervals {
                if let Some(prop_data) = &interval.data {
                    let dc_inner = dc.clone();
                    eh.borrow_mut().add(prop_data.property.definition_changed(), move |_| {
                        dc_inner.raise_event(&());
                    });
                }
            }
            dc.raise_event(&());
        });

        Self {
            intervals,
            definition_changed,
            event_helper,
        }
    }

    fn re_subscribe(&self, intervals: &TimeIntervalCollection<PropertyInterval<T>>) {
        let mut eh = self.event_helper.borrow_mut();
        eh.remove_all();
        let dc = self.definition_changed.clone();
        for i in 0..intervals.len() {
            if let Some(interval) = intervals.get(i) {
                if let Some(prop_data) = &interval.data {
                    let dc_inner = dc.clone();
                    eh.add(prop_data.property.definition_changed(), move |_| {
                        dc_inner.raise_event(&());
                    });
                }
            }
        }
    }

    pub fn is_constant(&self) -> bool {
        self.intervals.is_empty()
    }

    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    pub fn get_value(&self, time: Option<&JulianDate>) -> Option<T> {
        let default_time;
        let t = match time {
            Some(t) => t,
            None => {
                default_time = JulianDate::now();
                &default_time
            }
        };
        let prop = self.intervals.find_data_for_interval_containing_date(t)?;
        prop.property.get_value(Some(t))
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        match other {
            Some(o) => self.intervals == o.intervals,
            None => false,
        }
    }

    pub fn raise_definition_changed(&self) {
        self.definition_changed.raise_event(&());
    }
}

/// Interval data wrapper for CompositePositionProperty holding a dynamic position property.
#[derive(Clone)]
pub struct PositionPropertyInterval {
    pub property: Rc<dyn DynPositionProperty>,
}

impl PositionPropertyInterval {
    pub fn new(property: Rc<dyn DynPositionProperty>) -> Self {
        Self { property }
    }
}

impl PartialEq for PositionPropertyInterval {
    fn eq(&self, other: &Self) -> bool {
        if Rc::ptr_eq(&self.property, &other.property) {
            return true;
        }
        self.property.equals_dyn(&*other.property)
    }
}

impl From<Rc<dyn DynPositionProperty>> for PositionPropertyInterval {
    fn from(property: Rc<dyn DynPositionProperty>) -> Self {
        Self { property }
    }
}

impl From<ConstantPositionProperty> for PositionPropertyInterval {
    fn from(prop: ConstantPositionProperty) -> Self {
        Self {
            property: Rc::new(prop),
        }
    }
}

impl From<Rc<ConstantPositionProperty>> for PositionPropertyInterval {
    fn from(prop: Rc<ConstantPositionProperty>) -> Self {
        Self { property: prop }
    }
}

/// A PositionProperty which is comprised of multiple other properties over different time intervals.
pub struct CompositePositionProperty {
    pub intervals: TimeIntervalCollection<PositionPropertyInterval>,
    pub reference_frame: ReferenceFrame,
    definition_changed: Event<()>,
    event_helper: Rc<RefCell<EventHelper>>,
}

impl Default for CompositePositionProperty {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Clone for CompositePositionProperty {
    fn clone(&self) -> Self {
        let mut slf = Self::new(Some(self.reference_frame));
        slf.intervals = self.intervals.clone();
        slf.re_subscribe(&slf.intervals);
        slf
    }
}

impl CompositePositionProperty {
    pub fn new(reference_frame: Option<ReferenceFrame>) -> Self {
        let intervals: TimeIntervalCollection<PositionPropertyInterval> = TimeIntervalCollection::new();
        let definition_changed = Event::new();
        let event_helper = Rc::new(RefCell::new(EventHelper::new()));

        let dc = definition_changed.clone();
        let eh = Rc::clone(&event_helper);
        intervals.changed_event().add_listener(move |curr_intervals| {
            eh.borrow_mut().remove_all();
            for interval in curr_intervals {
                if let Some(prop_data) = &interval.data {
                    let dc_inner = dc.clone();
                    eh.borrow_mut().add(prop_data.property.definition_changed(), move |_| {
                        dc_inner.raise_event(&());
                    });
                }
            }
            dc.raise_event(&());
        });

        Self {
            intervals,
            reference_frame: reference_frame.unwrap_or(ReferenceFrame::Fixed),
            definition_changed,
            event_helper,
        }
    }

    fn re_subscribe(&self, intervals: &TimeIntervalCollection<PositionPropertyInterval>) {
        let mut eh = self.event_helper.borrow_mut();
        eh.remove_all();
        let dc = self.definition_changed.clone();
        for i in 0..intervals.len() {
            if let Some(interval) = intervals.get(i) {
                if let Some(prop_data) = &interval.data {
                    let dc_inner = dc.clone();
                    eh.add(prop_data.property.definition_changed(), move |_| {
                        dc_inner.raise_event(&());
                    });
                }
            }
        }
    }

    pub fn reference_frame(&self) -> ReferenceFrame {
        self.reference_frame
    }

    pub fn is_constant(&self) -> bool {
        self.intervals.is_empty()
    }

    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    pub fn get_value(&self, time: Option<&JulianDate>) -> Option<Cartesian3> {
        let default_time;
        let t = match time {
            Some(t) => t,
            None => {
                default_time = JulianDate::now();
                &default_time
            }
        };
        self.get_value_in_reference_frame(Some(t), ReferenceFrame::Fixed)
    }

    pub fn get_value_in_reference_frame(
        &self,
        time: Option<&JulianDate>,
        target_frame: ReferenceFrame,
    ) -> Option<Cartesian3> {
        let default_time;
        let t = match time {
            Some(t) => t,
            None => {
                default_time = JulianDate::now();
                &default_time
            }
        };
        let prop = self.intervals.find_data_for_interval_containing_date(t)?;
        prop.property.get_value_in_reference_frame(Some(t), target_frame)
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        match other {
            Some(o) => {
                self.reference_frame == o.reference_frame && self.intervals == o.intervals
            }
            None => false,
        }
    }

    pub fn raise_definition_changed(&self) {
        self.definition_changed.raise_event(&());
    }
}

pub type PropertyCallback<T> = std::rc::Rc<dyn Fn(&JulianDate) -> Option<T>>;

/// A Property whose value is produced by a callback function.
#[derive(Clone)]
pub struct CallbackProperty<T: Clone> {
    callback: PropertyCallback<T>,
    is_constant: bool,
    definition_changed: Event<()>,
}

impl<T: Clone> CallbackProperty<T> {
    pub fn new(callback: PropertyCallback<T>, is_constant: bool) -> Self {
        Self {
            callback,
            is_constant,
            definition_changed: Event::new(),
        }
    }

    pub fn is_constant(&self) -> bool {
        self.is_constant
    }

    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    pub fn get_value(&self, time: Option<&JulianDate>) -> Option<T> {
        let default_time;
        let t = match time {
            Some(t) => t,
            None => {
                default_time = JulianDate::now();
                &default_time
            }
        };
        (self.callback)(t)
    }

    pub fn set_callback(&mut self, callback: PropertyCallback<T>, is_constant: bool) {
        self.callback = callback;
        self.is_constant = is_constant;
        self.definition_changed.raise_event(&());
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        match other {
            Some(o) => {
                self.is_constant == o.is_constant
                    && std::rc::Rc::ptr_eq(&self.callback, &o.callback)
            }
            None => false,
        }
    }
}

/// A PositionProperty whose value is produced by a callback function.
#[derive(Clone)]
pub struct CallbackPositionProperty {
    callback: PropertyCallback<Cartesian3>,
    is_constant: bool,
    reference_frame: ReferenceFrame,
    definition_changed: Event<()>,
}

impl CallbackPositionProperty {
    pub fn new(
        callback: PropertyCallback<Cartesian3>,
        is_constant: bool,
        reference_frame: Option<ReferenceFrame>,
    ) -> Self {
        Self {
            callback,
            is_constant,
            reference_frame: reference_frame.unwrap_or(ReferenceFrame::Fixed),
            definition_changed: Event::new(),
        }
    }

    pub fn reference_frame(&self) -> ReferenceFrame {
        self.reference_frame
    }

    pub fn is_constant(&self) -> bool {
        self.is_constant
    }

    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    pub fn get_value(&self, time: Option<&JulianDate>) -> Option<Cartesian3> {
        let default_time;
        let t = match time {
            Some(t) => t,
            None => {
                default_time = JulianDate::now();
                &default_time
            }
        };
        self.get_value_in_reference_frame(Some(t), ReferenceFrame::Fixed)
    }

    pub fn get_value_in_reference_frame(
        &self,
        time: Option<&JulianDate>,
        target_frame: ReferenceFrame,
    ) -> Option<Cartesian3> {
        let t = time?;
        let raw = (self.callback)(t)?;
        PositionProperty::convert_to_reference_frame(
            Some(t),
            Some(raw),
            self.reference_frame,
            target_frame,
        )
    }

    pub fn set_callback(&mut self, callback: PropertyCallback<Cartesian3>, is_constant: bool) {
        self.callback = callback;
        self.is_constant = is_constant;
        self.definition_changed.raise_event(&());
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        match other {
            Some(o) => {
                self.is_constant == o.is_constant
                    && self.reference_frame == o.reference_frame
                    && std::rc::Rc::ptr_eq(&self.callback, &o.callback)
            }
            None => false,
        }
    }
}

/// A Property which is comprised of multiple values over different time intervals.
pub struct TimeIntervalCollectionProperty<T: Clone + PartialEq + 'static> {
    pub intervals: TimeIntervalCollection<T>,
    definition_changed: Event<()>,
}

impl<T: Clone + PartialEq + 'static> Default for TimeIntervalCollectionProperty<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + PartialEq + 'static> Clone for TimeIntervalCollectionProperty<T> {
    fn clone(&self) -> Self {
        let intervals = self.intervals.clone();
        let definition_changed = Event::new();
        let dc = definition_changed.clone();
        intervals.changed_event().add_listener(move |_| {
            dc.raise_event(&());
        });
        Self {
            intervals,
            definition_changed,
        }
    }
}

impl<T: Clone + PartialEq + 'static> TimeIntervalCollectionProperty<T> {
    pub fn new() -> Self {
        let intervals: TimeIntervalCollection<T> = TimeIntervalCollection::new();
        let definition_changed = Event::new();
        let dc = definition_changed.clone();
        intervals.changed_event().add_listener(move |_| {
            dc.raise_event(&());
        });
        Self {
            intervals,
            definition_changed,
        }
    }

    pub fn is_constant(&self) -> bool {
        self.intervals.is_empty()
    }

    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    pub fn get_value(&self, time: Option<&JulianDate>) -> Option<T> {
        let default_time;
        let t = match time {
            Some(t) => t,
            None => {
                default_time = JulianDate::now();
                &default_time
            }
        };
        self.intervals.find_data_for_interval_containing_date(t).cloned()
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        match other {
            Some(o) => self.intervals == o.intervals,
            None => false,
        }
    }

    pub fn raise_definition_changed(&self) {
        self.definition_changed.raise_event(&());
    }
}

/// A PositionProperty which is comprised of multiple values over different time intervals.
pub struct TimeIntervalCollectionPositionProperty {
    pub intervals: TimeIntervalCollection<Cartesian3>,
    pub reference_frame: ReferenceFrame,
    definition_changed: Event<()>,
}

impl Default for TimeIntervalCollectionPositionProperty {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Clone for TimeIntervalCollectionPositionProperty {
    fn clone(&self) -> Self {
        let intervals = self.intervals.clone();
        let definition_changed = Event::new();
        let dc = definition_changed.clone();
        intervals.changed_event().add_listener(move |_| {
            dc.raise_event(&());
        });
        Self {
            intervals,
            reference_frame: self.reference_frame,
            definition_changed,
        }
    }
}

impl TimeIntervalCollectionPositionProperty {
    pub fn new(reference_frame: Option<ReferenceFrame>) -> Self {
        let intervals: TimeIntervalCollection<Cartesian3> = TimeIntervalCollection::new();
        let definition_changed = Event::new();
        let dc = definition_changed.clone();
        intervals.changed_event().add_listener(move |_| {
            dc.raise_event(&());
        });
        Self {
            intervals,
            reference_frame: reference_frame.unwrap_or(ReferenceFrame::Fixed),
            definition_changed,
        }
    }

    pub fn reference_frame(&self) -> ReferenceFrame {
        self.reference_frame
    }

    pub fn is_constant(&self) -> bool {
        self.intervals.is_empty()
    }

    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    pub fn get_value(&self, time: Option<&JulianDate>) -> Option<Cartesian3> {
        let default_time;
        let t = match time {
            Some(t) => t,
            None => {
                default_time = JulianDate::now();
                &default_time
            }
        };
        self.get_value_in_reference_frame(Some(t), ReferenceFrame::Fixed)
    }

    pub fn get_value_in_reference_frame(
        &self,
        time: Option<&JulianDate>,
        target_frame: ReferenceFrame,
    ) -> Option<Cartesian3> {
        let t = time?;
        let raw = *self.intervals.find_data_for_interval_containing_date(t)?;
        PositionProperty::convert_to_reference_frame(
            Some(t),
            Some(raw),
            self.reference_frame,
            target_frame,
        )
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        match other {
            Some(o) => self.reference_frame == o.reference_frame && self.intervals == o.intervals,
            None => false,
        }
    }

    pub fn raise_definition_changed(&self) {
        self.definition_changed.raise_event(&());
    }
}

/// Dynamic trait object interface for generic Property evaluation and comparison.
pub trait DynProperty<T: Clone>: std::any::Any {
    fn is_constant(&self) -> bool;
    fn definition_changed(&self) -> &Event<()>;
    fn get_value(&self, time: Option<&JulianDate>) -> Option<T>;
    fn as_any(&self) -> &dyn std::any::Any;
    fn equals_dyn(&self, other: &dyn DynProperty<T>) -> bool;
}

impl<T: Clone + PartialEq + 'static> DynProperty<T> for ConstantProperty<T> {
    fn is_constant(&self) -> bool {
        self.is_constant()
    }
    fn definition_changed(&self) -> &Event<()> {
        self.definition_changed()
    }
    fn get_value(&self, time: Option<&JulianDate>) -> Option<T> {
        self.get_value(time)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn equals_dyn(&self, other: &dyn DynProperty<T>) -> bool {
        if let Some(o) = other.as_any().downcast_ref::<Self>() {
            self.equals(Some(o))
        } else {
            false
        }
    }
}

impl<T: Clone + 'static> DynProperty<T> for CallbackProperty<T> {
    fn is_constant(&self) -> bool {
        self.is_constant()
    }
    fn definition_changed(&self) -> &Event<()> {
        self.definition_changed()
    }
    fn get_value(&self, time: Option<&JulianDate>) -> Option<T> {
        self.get_value(time)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn equals_dyn(&self, other: &dyn DynProperty<T>) -> bool {
        if let Some(o) = other.as_any().downcast_ref::<Self>() {
            self.equals(Some(o))
        } else {
            false
        }
    }
}

impl<T: Clone + PartialEq + 'static> DynProperty<T> for CompositeProperty<T> {
    fn is_constant(&self) -> bool {
        self.is_constant()
    }
    fn definition_changed(&self) -> &Event<()> {
        self.definition_changed()
    }
    fn get_value(&self, time: Option<&JulianDate>) -> Option<T> {
        self.get_value(time)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn equals_dyn(&self, other: &dyn DynProperty<T>) -> bool {
        if let Some(o) = other.as_any().downcast_ref::<Self>() {
            self.equals(Some(o))
        } else {
            false
        }
    }
}

impl<T: Clone + PartialEq + 'static> DynProperty<T> for TimeIntervalCollectionProperty<T> {
    fn is_constant(&self) -> bool {
        self.is_constant()
    }
    fn definition_changed(&self) -> &Event<()> {
        self.definition_changed()
    }
    fn get_value(&self, time: Option<&JulianDate>) -> Option<T> {
        self.get_value(time)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn equals_dyn(&self, other: &dyn DynProperty<T>) -> bool {
        if let Some(o) = other.as_any().downcast_ref::<Self>() {
            self.equals(Some(o))
        } else {
            false
        }
    }
}

/// Dynamic trait object interface for PositionProperty instances.
pub trait DynPositionProperty: std::any::Any {
    fn is_constant(&self) -> bool;
    fn definition_changed(&self) -> &Event<()>;
    fn get_value(&self, time: Option<&JulianDate>) -> Option<Cartesian3>;
    fn get_value_in_reference_frame(
        &self,
        time: Option<&JulianDate>,
        target_frame: ReferenceFrame,
    ) -> Option<Cartesian3>;
    fn as_any(&self) -> &dyn std::any::Any;
    fn equals_dyn(&self, other: &dyn DynPositionProperty) -> bool;
}

impl DynPositionProperty for ConstantPositionProperty {
    fn is_constant(&self) -> bool {
        self.is_constant()
    }
    fn definition_changed(&self) -> &Event<()> {
        self.definition_changed()
    }
    fn get_value(&self, time: Option<&JulianDate>) -> Option<Cartesian3> {
        self.get_value(time)
    }
    fn get_value_in_reference_frame(
        &self,
        time: Option<&JulianDate>,
        target_frame: ReferenceFrame,
    ) -> Option<Cartesian3> {
        self.get_value_in_reference_frame(time, target_frame)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn equals_dyn(&self, other: &dyn DynPositionProperty) -> bool {
        if let Some(o) = other.as_any().downcast_ref::<Self>() {
            self.equals(Some(o))
        } else {
            false
        }
    }
}

impl DynPositionProperty for CallbackPositionProperty {
    fn is_constant(&self) -> bool {
        self.is_constant()
    }
    fn definition_changed(&self) -> &Event<()> {
        self.definition_changed()
    }
    fn get_value(&self, time: Option<&JulianDate>) -> Option<Cartesian3> {
        self.get_value(time)
    }
    fn get_value_in_reference_frame(
        &self,
        time: Option<&JulianDate>,
        target_frame: ReferenceFrame,
    ) -> Option<Cartesian3> {
        self.get_value_in_reference_frame(time, target_frame)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn equals_dyn(&self, other: &dyn DynPositionProperty) -> bool {
        if let Some(o) = other.as_any().downcast_ref::<Self>() {
            self.equals(Some(o))
        } else {
            false
        }
    }
}

impl DynPositionProperty for CompositePositionProperty {
    fn is_constant(&self) -> bool {
        self.is_constant()
    }
    fn definition_changed(&self) -> &Event<()> {
        self.definition_changed()
    }
    fn get_value(&self, time: Option<&JulianDate>) -> Option<Cartesian3> {
        self.get_value(time)
    }
    fn get_value_in_reference_frame(
        &self,
        time: Option<&JulianDate>,
        target_frame: ReferenceFrame,
    ) -> Option<Cartesian3> {
        self.get_value_in_reference_frame(time, target_frame)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn equals_dyn(&self, other: &dyn DynPositionProperty) -> bool {
        if let Some(o) = other.as_any().downcast_ref::<Self>() {
            self.equals(Some(o))
        } else {
            false
        }
    }
}

impl DynPositionProperty for TimeIntervalCollectionPositionProperty {
    fn is_constant(&self) -> bool {
        self.is_constant()
    }
    fn definition_changed(&self) -> &Event<()> {
        self.definition_changed()
    }
    fn get_value(&self, time: Option<&JulianDate>) -> Option<Cartesian3> {
        self.get_value(time)
    }
    fn get_value_in_reference_frame(
        &self,
        time: Option<&JulianDate>,
        target_frame: ReferenceFrame,
    ) -> Option<Cartesian3> {
        self.get_value_in_reference_frame(time, target_frame)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn equals_dyn(&self, other: &dyn DynPositionProperty) -> bool {
        if let Some(o) = other.as_any().downcast_ref::<Self>() {
            self.equals(Some(o))
        } else {
            false
        }
    }
}

/// A Property whose value is an array whose items are the computed value of other property instances.
#[derive(Clone)]
pub struct PropertyArray<T: Clone + 'static> {
    value: Option<Vec<Rc<dyn DynProperty<T>>>>,
    definition_changed: Event<()>,
    event_helper: Rc<RefCell<EventHelper>>,
}

impl<T: Clone + PartialEq + 'static> Default for PropertyArray<T> {
    fn default() -> Self {
        Self::new(None)
    }
}

impl<T: Clone + PartialEq + 'static> PropertyArray<T> {
    pub fn new(value: Option<Vec<Rc<dyn DynProperty<T>>>>) -> Self {
        let mut slf = Self {
            value: None,
            definition_changed: Event::new(),
            event_helper: Rc::new(RefCell::new(EventHelper::new())),
        };
        slf.set_value(value);
        slf
    }

    pub fn from_rc_properties(props: Vec<Rc<dyn DynProperty<T>>>) -> Self {
        Self::new(Some(props))
    }

    pub fn is_constant(&self) -> bool {
        match &self.value {
            None => true,
            Some(items) => items.iter().all(|item| item.is_constant()),
        }
    }

    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    pub fn get_value(&self, time: Option<&JulianDate>) -> Option<Vec<T>> {
        let default_time;
        let t = match time {
            Some(time) => time,
            None => {
                default_time = JulianDate::now();
                &default_time
            }
        };

        let items = self.value.as_ref()?;
        let mut result = Vec::with_capacity(items.len());
        for item in items {
            if let Some(v) = item.get_value(Some(t)) {
                result.push(v);
            }
        }
        Some(result)
    }

    pub fn set_value(&mut self, value: Option<Vec<Rc<dyn DynProperty<T>>>>) {
        self.event_helper.borrow_mut().remove_all();

        if let Some(items) = &value {
            let dc = self.definition_changed.clone();
            for item in items {
                self.event_helper.borrow_mut().add(item.definition_changed(), {
                    let ev = dc.clone();
                    move |_| ev.raise_event(&())
                });
            }
            self.value = Some(items.clone());
        } else {
            self.value = None;
        }

        self.definition_changed.raise_event(&());
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        let other = match other {
            Some(o) => o,
            None => return false,
        };
        match (&self.value, &other.value) {
            (None, None) => true,
            (Some(a), Some(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                a.iter().zip(b.iter()).all(|(x, y)| x.equals_dyn(&**y))
            }
            _ => false,
        }
    }
}

impl<T: Clone + PartialEq + 'static> DynProperty<Vec<T>> for PropertyArray<T> {
    fn is_constant(&self) -> bool {
        self.is_constant()
    }
    fn definition_changed(&self) -> &Event<()> {
        self.definition_changed()
    }
    fn get_value(&self, time: Option<&JulianDate>) -> Option<Vec<T>> {
        self.get_value(time)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn equals_dyn(&self, other: &dyn DynProperty<Vec<T>>) -> bool {
        if let Some(o) = other.as_any().downcast_ref::<Self>() {
            self.equals(Some(o))
        } else {
            false
        }
    }
}

/// A PositionProperty whose value is an array whose items are the computed value of other PositionProperty instances.
#[derive(Clone)]
pub struct PositionPropertyArray {
    value: Option<Vec<Rc<dyn DynPositionProperty>>>,
    reference_frame: ReferenceFrame,
    definition_changed: Event<()>,
    event_helper: Rc<RefCell<EventHelper>>,
}

impl Default for PositionPropertyArray {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl PositionPropertyArray {
    pub fn new(
        value: Option<Vec<Rc<dyn DynPositionProperty>>>,
        reference_frame: Option<ReferenceFrame>,
    ) -> Self {
        let mut slf = Self {
            value: None,
            reference_frame: reference_frame.unwrap_or(ReferenceFrame::Fixed),
            definition_changed: Event::new(),
            event_helper: Rc::new(RefCell::new(EventHelper::new())),
        };
        slf.set_value(value);
        slf
    }

    pub fn from_rc_properties(
        props: Vec<Rc<dyn DynPositionProperty>>,
        reference_frame: Option<ReferenceFrame>,
    ) -> Self {
        Self::new(Some(props), reference_frame)
    }

    pub fn reference_frame(&self) -> ReferenceFrame {
        self.reference_frame
    }

    pub fn is_constant(&self) -> bool {
        match &self.value {
            None => true,
            Some(items) => items.iter().all(|item| item.is_constant()),
        }
    }

    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    pub fn get_value(&self, time: Option<&JulianDate>) -> Option<Vec<Cartesian3>> {
        let default_time;
        let t = match time {
            Some(time) => time,
            None => {
                default_time = JulianDate::now();
                &default_time
            }
        };
        self.get_value_in_reference_frame(Some(t), ReferenceFrame::Fixed)
    }

    pub fn get_value_in_reference_frame(
        &self,
        time: Option<&JulianDate>,
        target_frame: ReferenceFrame,
    ) -> Option<Vec<Cartesian3>> {
        let t = time?;
        let items = self.value.as_ref()?;
        let mut result = Vec::with_capacity(items.len());
        for item in items {
            if let Some(v) = item.get_value_in_reference_frame(Some(t), target_frame) {
                result.push(v);
            }
        }
        Some(result)
    }

    pub fn set_value(&mut self, value: Option<Vec<Rc<dyn DynPositionProperty>>>) {
        self.event_helper.borrow_mut().remove_all();

        if let Some(items) = &value {
            let dc = self.definition_changed.clone();
            for item in items {
                self.event_helper.borrow_mut().add(item.definition_changed(), {
                    let ev = dc.clone();
                    move |_| ev.raise_event(&())
                });
            }
            self.value = Some(items.clone());
        } else {
            self.value = None;
        }

        self.definition_changed.raise_event(&());
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        let other = match other {
            Some(o) => o,
            None => return false,
        };
        if self.reference_frame != other.reference_frame {
            return false;
        }
        match (&self.value, &other.value) {
            (None, None) => true,
            (Some(a), Some(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                a.iter().zip(b.iter()).all(|(x, y)| x.equals_dyn(&**y))
            }
            _ => false,
        }
    }
}



