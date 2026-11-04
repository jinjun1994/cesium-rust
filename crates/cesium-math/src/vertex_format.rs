//! VertexFormat matching CesiumJS `Core/VertexFormat`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::MathError;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VertexFormat {
    pub position: bool,
    pub normal: bool,
    pub st: bool,
    pub tangent: bool,
    pub bitangent: bool,
    pub color: bool,
}

impl VertexFormat {
    pub const PACKED_LENGTH: usize = 6;

    pub const POSITION_ONLY: Self = Self {
        position: true,
        normal: false,
        st: false,
        tangent: false,
        bitangent: false,
        color: false,
    };

    pub const POSITION_AND_NORMAL: Self = Self {
        position: true,
        normal: true,
        st: false,
        tangent: false,
        bitangent: false,
        color: false,
    };

    pub const POSITION_NORMAL_AND_ST: Self = Self {
        position: true,
        normal: true,
        st: true,
        tangent: false,
        bitangent: false,
        color: false,
    };

    pub const POSITION_AND_ST: Self = Self {
        position: true,
        normal: false,
        st: true,
        tangent: false,
        bitangent: false,
        color: false,
    };

    pub const POSITION_AND_COLOR: Self = Self {
        position: true,
        normal: false,
        st: false,
        tangent: false,
        bitangent: false,
        color: true,
    };

    pub const ALL: Self = Self {
        position: true,
        normal: true,
        st: true,
        tangent: true,
        bitangent: true,
        color: false,
    };

    pub const DEFAULT: Self = Self::POSITION_NORMAL_AND_ST;

    pub fn pack(&self, output: &mut [f64], starting_index: usize) -> Result<(), MathError> {
        let end = starting_index
            .checked_add(Self::PACKED_LENGTH)
            .ok_or(MathError)?;
        let target = output.get_mut(starting_index..end).ok_or(MathError)?;
        target[0] = if self.position { 1.0 } else { 0.0 };
        target[1] = if self.normal { 1.0 } else { 0.0 };
        target[2] = if self.st { 1.0 } else { 0.0 };
        target[3] = if self.tangent { 1.0 } else { 0.0 };
        target[4] = if self.bitangent { 1.0 } else { 0.0 };
        target[5] = if self.color { 1.0 } else { 0.0 };
        Ok(())
    }

    pub fn unpack(array: &[f64], starting_index: usize) -> Result<Self, MathError> {
        let end = starting_index
            .checked_add(Self::PACKED_LENGTH)
            .ok_or(MathError)?;
        let slice = array.get(starting_index..end).ok_or(MathError)?;
        Ok(Self {
            position: slice[0] == 1.0,
            normal: slice[1] == 1.0,
            st: slice[2] == 1.0,
            tangent: slice[3] == 1.0,
            bitangent: slice[4] == 1.0,
            color: slice[5] == 1.0,
        })
    }
}
