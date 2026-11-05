//! Tracking reference frame enumeration.

/// Constants for identifying well-known tracking reference frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrackingReferenceFrame {
    /// Auto-detect algorithm. The reference frame used to track the Entity will
    /// be automatically selected based on its trajectory: near-surface slow moving
    /// objects will be tracked in the entity's local east-north-up reference
    /// frame, while faster objects like satellites will use VVLH (Vehicle Velocity,
    /// Local Horizontal).
    #[default]
    Autodetect = 0,

    /// The entity's local East-North-Up reference frame.
    Enu = 1,

    /// The entity's inertial reference frame. If entity has no defined orientation
    /// property, it falls back to auto-detect algorithm.
    Inertial = 2,

    /// The entity's inertial reference frame with orientation fixed to its
    /// velocity orientation property, ignoring its own orientation.
    Velocity = 3,
}

impl TrackingReferenceFrame {
    /// Returns the integer value corresponding to CesiumJS enum.
    pub const fn value(self) -> i32 {
        self as i32
    }
}
