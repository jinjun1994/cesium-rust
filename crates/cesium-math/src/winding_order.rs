//! Winding order matching CesiumJS `Core/WindingOrder`.
//! Adapted from CesiumJS (Apache-2.0).

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WindingOrder {
    Clockwise = 0x0900,
    CounterClockwise = 0x0901,
}

impl WindingOrder {
    pub const CLOCKWISE: Self = Self::Clockwise;
    pub const COUNTER_CLOCKWISE: Self = Self::CounterClockwise;
    pub const CW: Self = Self::Clockwise;
    pub const CCW: Self = Self::CounterClockwise;

    pub const fn as_gl_enum(&self) -> u32 {
        *self as u32
    }

    pub fn validate(order: u32) -> bool {
        order == Self::Clockwise.as_gl_enum() || order == Self::CounterClockwise.as_gl_enum()
    }
}
