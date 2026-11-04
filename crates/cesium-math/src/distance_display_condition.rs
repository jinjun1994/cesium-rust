//! DistanceDisplayCondition matching CesiumJS `Core/DistanceDisplayCondition`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::MathError;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DistanceDisplayCondition {
    pub near: f64,
    pub far: f64,
}

impl Default for DistanceDisplayCondition {
    fn default() -> Self {
        Self {
            near: 0.0,
            far: f64::MAX,
        }
    }
}

impl DistanceDisplayCondition {
    pub const PACKED_LENGTH: usize = 2;

    pub fn new(near: f64, far: f64) -> Self {
        Self { near, far }
    }

    pub fn equals(&self, other: &Self) -> bool {
        self.near == other.near && self.far == other.far
    }

    pub fn pack(&self, array: &mut [f64], starting_index: usize) -> Result<(), MathError> {
        if array.len() < starting_index + Self::PACKED_LENGTH {
            return Err(MathError);
        }
        array[starting_index] = self.near;
        array[starting_index + 1] = self.far;
        Ok(())
    }

    pub fn unpack(array: &[f64], starting_index: usize) -> Result<Self, MathError> {
        if array.len() < starting_index + Self::PACKED_LENGTH {
            return Err(MathError);
        }
        Ok(Self {
            near: array[starting_index],
            far: array[starting_index + 1],
        })
    }
}
