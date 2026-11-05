//! PathMode matching CesiumJS `DataSources/PathMode.js` (Apache-2.0).

/// Defines how material properties are applied along a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PathMode {
    #[default]
    Whole = 0,
    Portions = 1,
}
