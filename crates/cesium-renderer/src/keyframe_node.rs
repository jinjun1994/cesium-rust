use std::cmp::Ordering;

/// Lifecycle state for streaming a keyframe's texture/geometry payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum KeyframeLoadState {
    #[default]
    Unloaded = 0,
    Receiving = 1,
    Processing = 2,
    Loaded = 3,
    Failed = 4,
    Unavailable = 5,
}

/// A node representing a temporal keyframe within a spatial node hierarchy (e.g. megatexture).
#[derive(Debug, Clone, PartialEq)]
pub struct KeyframeNode<T = serde_json::Value> {
    pub spatial_node: Option<T>,
    pub keyframe: i32,
    pub state: KeyframeLoadState,
    pub content: Option<serde_json::Value>,
    pub megatexture_index: i32,
    pub priority: f64,
    pub high_priority_frame_number: i64,
}

impl<T> KeyframeNode<T> {
    /// Creates a new KeyframeNode associated with a spatial node reference.
    pub fn new(spatial_node: T, keyframe: i32) -> Self {
        Self {
            spatial_node: Some(spatial_node),
            keyframe,
            state: KeyframeLoadState::Unloaded,
            content: None,
            megatexture_index: -1,
            priority: -f64::MAX,
            high_priority_frame_number: -1,
        }
    }

    /// Unloads and frees resources attached to this keyframe.
    pub fn unload(&mut self) {
        self.content = None;
        self.spatial_node = None;
        self.state = KeyframeLoadState::Unloaded;
        self.megatexture_index = -1;
        self.priority = -f64::MAX;
        self.high_priority_frame_number = -1;
    }

    /// Compares two nodes based on priority (a.priority - b.priority).
    pub fn priority_comparator(a: &Self, b: &Self) -> f64 {
        a.priority - b.priority
    }

    /// Priority ordering comparator for sorting.
    pub fn priority_ordering(a: &Self, b: &Self) -> Ordering {
        a.priority.partial_cmp(&b.priority).unwrap_or(Ordering::Equal)
    }

    /// Compares two nodes by keyframe index (a.keyframe - b.keyframe).
    pub fn search_comparator(a: &Self, b: &Self) -> i32 {
        a.keyframe - b.keyframe
    }
}
