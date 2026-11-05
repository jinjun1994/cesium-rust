//! LabelStyle matching CesiumJS `Scene/LabelStyle.js` (Apache-2.0).

/// Describes how to draw a label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LabelStyle {
    #[default]
    Fill = 0,
    Outline = 1,
    FillAndOutline = 2,
}
