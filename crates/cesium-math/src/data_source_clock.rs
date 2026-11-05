//! Represents desired clock settings for a particular DataSource.
//! Direct port of CesiumJS `DataSources/DataSourceClock.js`.

use crate::{Clock, ClockOptions, ClockRange, ClockStep, DeveloperError, Event, JulianDate};

/// Represents desired clock settings for a particular DataSource.
#[derive(Debug, Clone, Default)]
pub struct DataSourceClock {
    pub start_time: Option<JulianDate>,
    pub stop_time: Option<JulianDate>,
    pub current_time: Option<JulianDate>,
    pub clock_range: Option<ClockRange>,
    pub clock_step: Option<ClockStep>,
    pub multiplier: Option<f64>,
    definition_changed: Event<()>,
}

impl PartialEq for DataSourceClock {
    fn eq(&self, other: &Self) -> bool {
        self.start_time == other.start_time
            && self.stop_time == other.stop_time
            && self.current_time == other.current_time
            && self.clock_range == other.clock_range
            && self.clock_step == other.clock_step
            && self.multiplier == other.multiplier
    }
}

impl DataSourceClock {
    /// Creates a new DataSourceClock with all properties set to None.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the event that is raised whenever a property is assigned.
    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    /// Duplicates into a provided result reference.
    pub fn clone_to(&self, result: &mut Self) {
        result.start_time = self.start_time;
        result.stop_time = self.stop_time;
        result.current_time = self.current_time;
        result.clock_range = self.clock_range;
        result.clock_step = self.clock_step;
        result.multiplier = self.multiplier;
    }

    /// Assigns each unassigned property on this object to the value of the same property on the provided source object.
    pub fn merge(&mut self, source: &Self) {
        if self.start_time.is_none() {
            self.start_time = source.start_time;
        }
        if self.stop_time.is_none() {
            self.stop_time = source.stop_time;
        }
        if self.current_time.is_none() {
            self.current_time = source.current_time;
        }
        if self.clock_range.is_none() {
            self.clock_range = source.clock_range;
        }
        if self.clock_step.is_none() {
            self.clock_step = source.clock_step;
        }
        if self.multiplier.is_none() {
            self.multiplier = source.multiplier;
        }
    }

    /// Merges with error checking.
    pub fn try_merge(&mut self, source: Option<&Self>) -> Result<(), DeveloperError> {
        let source = source.ok_or_else(|| DeveloperError::new("source is required."))?;
        self.merge(source);
        Ok(())
    }

    /// Gets the value of this clock instance as a Clock object.
    pub fn get_value(&self) -> Clock {
        let options = ClockOptions {
            start_time: self.start_time,
            stop_time: self.stop_time,
            current_time: self.current_time,
            clock_step: self.clock_step,
            multiplier: self.multiplier,
            clock_range: self.clock_range,
            can_animate: None,
            should_animate: None,
        };
        Clock::new(options).unwrap_or_else(|_| Clock::default())
    }

    /// Gets the value of this clock instance using a provided Clock result buffer.
    pub fn get_value_with_result(&self, result: &mut Clock) {
        *result = self.get_value();
    }
}
