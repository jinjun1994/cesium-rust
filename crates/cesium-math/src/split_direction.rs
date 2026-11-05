//! SplitDirection matching CesiumJS `Scene/SplitDirection.js` (Apache-2.0).

/// The direction to display a primitive or ImageryLayer relative to the splitPosition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SplitDirection {
    Left = -1,
    #[default]
    None = 0,
    Right = 1,
}
