use crate::{Cartesian3, Matrix4, Quaternion};

/// An affine transformation defined by a translation, rotation, and scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TranslationRotationScale {
    pub translation: Cartesian3,
    pub rotation: Quaternion,
    pub scale: Cartesian3,
}

impl Default for TranslationRotationScale {
    fn default() -> Self {
        Self {
            translation: Cartesian3::ZERO,
            rotation: Quaternion::IDENTITY,
            scale: Cartesian3::new(1.0, 1.0, 1.0),
        }
    }
}

impl TranslationRotationScale {
    pub fn new(translation: Cartesian3, rotation: Quaternion, scale: Cartesian3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    #[inline]
    pub fn equals(&self, other: &Self) -> bool {
        self == other
    }

    /// Computes the 4x4 affine transformation matrix for this translation, rotation, and scale.
    pub fn to_matrix(&self) -> Matrix4 {
        Matrix4::from_translation_quaternion_rotation_scale(
            self.translation,
            self.rotation,
            self.scale,
        )
    }
}
