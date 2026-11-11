//! QuadtreeTileLoadState matching CesiumJS `Scene/QuadtreeTileLoadState.js` (Apache-2.0).

/// The state of a QuadtreeTile in the tile load pipeline.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum QuadtreeTileLoadState {
    /// The tile is in its initial state and hasn't been loaded yet.
    #[default]
    Start = 0,
    /// The tile is currently being loaded.
    Loading = 1,
    /// The tile has finished loading its primary content.
    Loaded = 2,
    /// The tile and all its dependencies have been fully loaded and processed.
    Done = 3,
    /// The tile failed to load.
    Failed = 4,
}

impl QuadtreeTileLoadState {
    /// Returns whether the tile needs further loading (`Start` or `Loading`).
    pub fn needs_loading(&self) -> bool {
        *self < Self::Done
    }
}
