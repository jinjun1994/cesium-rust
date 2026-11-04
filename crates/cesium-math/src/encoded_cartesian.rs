//! EncodedCartesian3 matching CesiumJS `Core/EncodedCartesian3`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::Cartesian3;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EncodedCartesian3 {
    pub high: Cartesian3,
    pub low: Cartesian3,
}

impl EncodedCartesian3 {
    pub const ZERO: Self = Self {
        high: Cartesian3::ZERO,
        low: Cartesian3::ZERO,
    };

    #[inline]
    pub const fn new(high: Cartesian3, low: Cartesian3) -> Self {
        Self { high, low }
    }

    #[inline]
    pub fn encode(value: f64) -> (f64, f64) {
        if value >= 0.0 {
            let double_high = (value / 65536.0).floor() * 65536.0;
            (double_high, value - double_high)
        } else {
            let double_high = (-value / 65536.0).floor() * 65536.0;
            (-double_high, value + double_high)
        }
    }

    pub fn from_cartesian(cartesian: Cartesian3) -> Self {
        let (hx, lx) = Self::encode(cartesian.x);
        let (hy, ly) = Self::encode(cartesian.y);
        let (hz, lz) = Self::encode(cartesian.z);

        Self {
            high: Cartesian3::new(hx, hy, hz),
            low: Cartesian3::new(lx, ly, lz),
        }
    }

    pub fn write_elements(cartesian: Cartesian3, array: &mut [f32], index: usize) {
        let encoded = Self::from_cartesian(cartesian);
        array[index] = encoded.high.x as f32;
        array[index + 1] = encoded.high.y as f32;
        array[index + 2] = encoded.high.z as f32;
        array[index + 3] = encoded.low.x as f32;
        array[index + 4] = encoded.low.y as f32;
        array[index + 5] = encoded.low.z as f32;
    }
}
