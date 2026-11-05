//! Reference frames matching CesiumJS `Core/ReferenceFrame.js` (Apache-2.0).

/// Constants for identifying well-known reference frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReferenceFrame {
    /// The fixed frame (Earth-Centered, Earth-Fixed / ECEF).
    #[default]
    Fixed = 0,
    /// The inertial frame (International Celestial Reference Frame / ICRF).
    Inertial = 1,
}
