/// Options for atmosphere lighting behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum DynamicAtmosphereLightingType {
    /// Atmosphere is not lit.
    #[default]
    None = 0,
    /// Atmosphere is lit by the sun.
    Sunlight = 1,
    /// Atmosphere is lit by the scene's light source.
    SceneLight = 2,
}

impl DynamicAtmosphereLightingType {
    /// Determines the atmosphere lighting type based on Globe lighting flags.
    pub fn from_globe_flags(
        enable_lighting: bool,
        dynamic_atmosphere_lighting: bool,
        dynamic_atmosphere_lighting_from_sun: bool,
    ) -> Self {
        if !enable_lighting || !dynamic_atmosphere_lighting {
            return Self::None;
        }

        if dynamic_atmosphere_lighting_from_sun {
            Self::Sunlight
        } else {
            Self::SceneLight
        }
    }
}
