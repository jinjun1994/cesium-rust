//! Scalar ranges and parameter structures matching CesiumJS `Core/NearFarScalar` and `Core/HeadingPitchRange`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::MathError;

// ============================================================================
// NearFarScalar
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NearFarScalar {
    pub near: f64,
    pub near_value: f64,
    pub far: f64,
    pub far_value: f64,
}

impl Default for NearFarScalar {
    fn default() -> Self {
        Self {
            near: 0.0,
            near_value: 0.0,
            far: 1.0,
            far_value: 0.0,
        }
    }
}

impl NearFarScalar {
    pub const PACKED_LENGTH: usize = 4;

    #[inline]
    pub const fn new(near: f64, near_value: f64, far: f64, far_value: f64) -> Self {
        Self {
            near,
            near_value,
            far,
            far_value,
        }
    }

    pub fn pack(&self, output: &mut [f64], starting_index: usize) -> Result<(), MathError> {
        let end = starting_index
            .checked_add(Self::PACKED_LENGTH)
            .ok_or(MathError)?;
        let target = output.get_mut(starting_index..end).ok_or(MathError)?;
        target[0] = self.near;
        target[1] = self.near_value;
        target[2] = self.far;
        target[3] = self.far_value;
        Ok(())
    }

    pub fn unpack(array: &[f64], starting_index: usize) -> Result<Self, MathError> {
        let end = starting_index
            .checked_add(Self::PACKED_LENGTH)
            .ok_or(MathError)?;
        let slice = array.get(starting_index..end).ok_or(MathError)?;
        Ok(Self::new(slice[0], slice[1], slice[2], slice[3]))
    }

    #[inline]
    pub fn equals(&self, other: &Self) -> bool {
        self.near == other.near
            && self.near_value == other.near_value
            && self.far == other.far
            && self.far_value == other.far_value
    }

    #[inline]
    pub fn equals_epsilon(&self, other: &Self, epsilon: f64) -> bool {
        (self.near - other.near).abs() <= epsilon
            && (self.near_value - other.near_value).abs() <= epsilon
            && (self.far - other.far).abs() <= epsilon
            && (self.far_value - other.far_value).abs() <= epsilon
    }
}

// ============================================================================
// HeadingPitchRange
// ============================================================================

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HeadingPitchRange {
    pub heading: f64,
    pub pitch: f64,
    pub range: f64,
}

impl HeadingPitchRange {
    #[inline]
    pub const fn new(heading: f64, pitch: f64, range: f64) -> Self {
        Self {
            heading,
            pitch,
            range,
        }
    }
}
