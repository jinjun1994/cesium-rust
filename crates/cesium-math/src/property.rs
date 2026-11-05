//! Property interfaces and implementations matching CesiumJS `DataSources/Property.js` (Apache-2.0).

use crate::{
    Cartesian3, CelestialFrameTransforms, Event, JulianDate, ReferenceFrame,
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
#[derive(Clone)]
pub struct ConstantProperty<T: Clone + PartialEq> {
    value: Option<T>,
    definition_changed: Event<()>,
}

impl<T: Clone + PartialEq> ConstantProperty<T> {
    pub fn new(value: Option<T>) -> Self {
        Self {
            value,
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
        self.value.clone()
    }

    pub fn set_value(&mut self, value: Option<T>) {
        if self.value != value {
            self.value = value;
            self.definition_changed.raise_event(&());
        }
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        match other {
            Some(o) => self.value == o.value,
            None => false,
        }
    }
}

impl<T: Clone + PartialEq + std::fmt::Display> std::fmt::Display for ConstantProperty<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
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
#[derive(Clone)]
pub struct ConstantPositionProperty {
    value: Option<Cartesian3>,
    reference_frame: ReferenceFrame,
    definition_changed: Event<()>,
}

impl ConstantPositionProperty {
    pub fn new(value: Option<Cartesian3>, reference_frame: Option<ReferenceFrame>) -> Self {
        Self {
            value,
            reference_frame: reference_frame.unwrap_or(ReferenceFrame::Fixed),
            definition_changed: Event::new(),
        }
    }

    pub fn reference_frame(&self) -> ReferenceFrame {
        self.reference_frame
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
            self.value,
            self.reference_frame,
            target_frame,
        )
    }

    pub fn set_value(
        &mut self,
        value: Option<Cartesian3>,
        reference_frame: Option<ReferenceFrame>,
    ) {
        let new_frame = reference_frame.unwrap_or(self.reference_frame);
        let changed = self.value != value || self.reference_frame != new_frame;
        if changed {
            self.value = value;
            self.reference_frame = new_frame;
            self.definition_changed.raise_event(&());
        }
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        match other {
            Some(o) => self.reference_frame == o.reference_frame && self.value == o.value,
            None => false,
        }
    }
}
