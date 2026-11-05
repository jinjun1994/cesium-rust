//! Constants related to ISO8601 support.
//! Direct port of CesiumJS `Core/Iso8601.js`.

use std::sync::LazyLock;

use crate::{JulianDate, TimeInterval};

/// A JulianDate representing the earliest time representable by an ISO8601 date.
/// Equivalent to '0000-01-01T00:00:00Z'.
pub static ISO8601_MINIMUM_VALUE: LazyLock<JulianDate> = LazyLock::new(|| {
    JulianDate::from_iso8601("0000-01-01T00:00:00Z")
        .expect("0000-01-01T00:00:00Z should be valid ISO8601")
});

/// A JulianDate representing the latest time representable by an ISO8601 date.
/// Equivalent to '9999-12-31T24:00:00Z'.
pub static ISO8601_MAXIMUM_VALUE: LazyLock<JulianDate> = LazyLock::new(|| {
    JulianDate::from_iso8601("9999-12-31T24:00:00Z")
        .or_else(|_| JulianDate::from_iso8601("9999-12-31T23:59:59.999999Z"))
        .expect("9999-12-31 should be valid ISO8601")
});

/// Constants related to ISO8601 support.
pub struct Iso8601;

impl Iso8601 {
    /// Earliest time representable by an ISO8601 date.
    pub fn minimum_value() -> JulianDate {
        *ISO8601_MINIMUM_VALUE
    }

    /// Latest time representable by an ISO8601 date.
    pub fn maximum_value() -> JulianDate {
        *ISO8601_MAXIMUM_VALUE
    }

    /// Largest interval representable by an ISO8601 interval.
    pub fn maximum_interval() -> TimeInterval {
        TimeInterval::new(Self::minimum_value(), Self::maximum_value(), true, true)
    }
}
