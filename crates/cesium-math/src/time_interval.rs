//! TimeInterval matching CesiumJS `Core/TimeInterval`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::julian_date::JulianDate;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimeInterval {
    pub start: JulianDate,
    pub stop: JulianDate,
    pub is_start_included: bool,
    pub is_stop_included: bool,
}

impl Default for TimeInterval {
    fn default() -> Self {
        Self {
            start: JulianDate::default(),
            stop: JulianDate::default(),
            is_start_included: true,
            is_stop_included: true,
        }
    }
}

impl TimeInterval {
    #[inline]
    pub const fn new(
        start: JulianDate,
        stop: JulianDate,
        is_start_included: bool,
        is_stop_included: bool,
    ) -> Self {
        Self {
            start,
            stop,
            is_start_included,
            is_stop_included,
        }
    }

    pub fn is_empty(&self) -> bool {
        let cmp = self.start.compare(&self.stop);
        match cmp {
            Ordering::Greater => true,
            Ordering::Equal => !self.is_start_included || !self.is_stop_included,
            Ordering::Less => false,
        }
    }

    pub fn contains(&self, date: &JulianDate) -> bool {
        if self.is_empty() {
            return false;
        }
        let start_cmp = date.compare(&self.start);
        if start_cmp == Ordering::Less {
            return false;
        }
        if start_cmp == Ordering::Equal && !self.is_start_included {
            return false;
        }
        let stop_cmp = date.compare(&self.stop);
        if stop_cmp == Ordering::Greater {
            return false;
        }
        if stop_cmp == Ordering::Equal && !self.is_stop_included {
            return false;
        }
        true
    }

    pub fn equals(&self, other: &Self) -> bool {
        self.start.equals(&other.start)
            && self.stop.equals(&other.stop)
            && self.is_start_included == other.is_start_included
            && self.is_stop_included == other.is_stop_included
    }

    pub fn equals_epsilon(&self, other: &Self, epsilon: f64) -> bool {
        self.start.equals_epsilon(&other.start, epsilon)
            && self.stop.equals_epsilon(&other.stop, epsilon)
            && self.is_start_included == other.is_start_included
            && self.is_stop_included == other.is_stop_included
    }

    pub fn intersect(&self, other: &Self) -> Option<Self> {
        if self.is_empty() || other.is_empty() {
            return None;
        }

        let (start, is_start_included) = match self.start.compare(&other.start) {
            Ordering::Less => (other.start, other.is_start_included),
            Ordering::Greater => (self.start, self.is_start_included),
            Ordering::Equal => (self.start, self.is_start_included && other.is_start_included),
        };

        let (stop, is_stop_included) = match self.stop.compare(&other.stop) {
            Ordering::Less => (self.stop, self.is_stop_included),
            Ordering::Greater => (other.stop, other.is_stop_included),
            Ordering::Equal => (self.stop, self.is_stop_included && other.is_stop_included),
        };

        let result = Self {
            start,
            stop,
            is_start_included,
            is_stop_included,
        };

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }
}
