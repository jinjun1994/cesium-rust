//! HorizontalOrigin and VerticalOrigin matching CesiumJS `Scene/HorizontalOrigin.js` and `VerticalOrigin.js` (Apache-2.0).

/// The horizontal location of an origin relative to an object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HorizontalOrigin {
    #[default]
    Center = 0,
    Left = 1,
    Right = -1,
}

/// The vertical location of an origin relative to an object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerticalOrigin {
    #[default]
    Center = 0,
    Bottom = 1,
    Baseline = 2,
    Top = -1,
}
