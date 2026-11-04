//! Spherical coordinates matching CesiumJS `Core/Spherical`.

use crate::cartesian::Cartesian3;
use std::fmt;

/// Curvilinear 3-dimensional spherical coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spherical {
    /// Angular coordinate lying in the xy-plane measured from positive x toward positive y.
    pub clock: f64,
    /// Angular coordinate measured from positive z toward negative z (zenith angle).
    pub cone: f64,
    /// Linear coordinate measured from origin.
    pub magnitude: f64,
}

impl Default for Spherical {
    fn default() -> Self {
        Self {
            clock: 0.0,
            cone: 0.0,
            magnitude: 1.0,
        }
    }
}

impl Spherical {
    pub const fn new(clock: f64, cone: f64, magnitude: f64) -> Self {
        Self {
            clock,
            cone,
            magnitude,
        }
    }

    /// Converts Cartesian3 coordinates into Spherical coordinates.
    pub fn from_cartesian3(cartesian: Cartesian3) -> Self {
        let x = cartesian.x;
        let y = cartesian.y;
        let z = cartesian.z;
        let radial_squared = x * x + y * y;

        Self {
            clock: y.atan2(x),
            cone: radial_squared.sqrt().atan2(z),
            magnitude: (radial_squared + z * z).sqrt(),
        }
    }

    /// Computes the normalized version of this spherical with magnitude = 1.0.
    pub fn normalize(&self) -> Self {
        Self {
            clock: self.clock,
            cone: self.cone,
            magnitude: 1.0,
        }
    }

    /// Compares two spherical instances within epsilon tolerance.
    pub fn equals_epsilon(&self, other: &Self, epsilon: f64) -> bool {
        (self.clock - other.clock).abs() <= epsilon
            && (self.cone - other.cone).abs() <= epsilon
            && (self.magnitude - other.magnitude).abs() <= epsilon
    }
}

impl fmt::Display for Spherical {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.clock, self.cone, self.magnitude)
    }
}
