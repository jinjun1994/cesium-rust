//! ShadowMode matching CesiumJS `Scene/ShadowMode.js` (Apache-2.0).

/// Specifies whether the object casts or receives shadows from light sources when shadows are enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ShadowMode {
    #[default]
    Disabled = 0,
    Enabled = 1,
    CastOnly = 2,
    ReceiveOnly = 3,
}

impl ShadowMode {
    pub fn cast_shadows(mode: Self) -> bool {
        matches!(mode, Self::Enabled | Self::CastOnly)
    }

    pub fn receive_shadows(mode: Self) -> bool {
        matches!(mode, Self::Enabled | Self::ReceiveOnly)
    }

    pub fn from_cast_receive(cast_shadows: bool, receive_shadows: bool) -> Self {
        match (cast_shadows, receive_shadows) {
            (true, true) => Self::Enabled,
            (true, false) => Self::CastOnly,
            (false, true) => Self::ReceiveOnly,
            (false, false) => Self::Disabled,
        }
    }
}
