/// Indicates if the scene is viewed in 3D, 2D, or 2.5D Columbus view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SceneMode {
    Morphing = 0,
    ColumbusView = 1,
    Scene2D = 2,
    #[default]
    Scene3D = 3,
}

impl SceneMode {
    /// Returns the morph time for the given scene mode.
    pub fn get_morph_time(self) -> Option<f64> {
        match self {
            Self::Scene3D => Some(1.0),
            Self::Morphing => None,
            _ => Some(0.0),
        }
    }
}
