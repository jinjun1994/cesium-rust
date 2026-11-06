//! BoundingSphereState matching CesiumJS `DataSources/BoundingSphereState.js` (Apache-2.0).

/// Represents the state of a bounding sphere calculation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundingSphereState {
    /// The bounding sphere has been calculated.
    Done = 0,
    /// The bounding sphere is pending calculation.
    Pending = 1,
    /// The bounding sphere calculation failed.
    Failed = 2,
}
