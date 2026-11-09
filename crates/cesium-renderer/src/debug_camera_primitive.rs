use cesium_math::{Camera, Color, DeveloperError};

/// Options for creating a `DebugCameraPrimitive`.
#[derive(Debug, Clone, Default)]
pub struct DebugCameraPrimitiveOptions {
    /// The camera to visualize.
    pub camera: Option<Camera>,
    /// Distances to the near and far planes of the camera frustums.
    pub frustum_splits: Option<Vec<f64>>,
    /// The color of the debug outline. Defaults to Color::CYAN.
    pub color: Option<Color>,
    /// Whether the primitive updates when the underlying camera changes. Defaults to true.
    pub update_on_change: Option<bool>,
    /// Determines if this primitive will be shown. Defaults to true.
    pub show: Option<bool>,
    /// User-defined ID.
    pub id: Option<String>,
}

/// Draws the outline of the camera's view frustum.
#[derive(Debug, Clone)]
pub struct DebugCameraPrimitive {
    /// The camera being debugged.
    pub camera: Camera,
    /// Distances to the near and far planes.
    pub frustum_splits: Option<Vec<f64>>,
    /// The color of the debug outline.
    pub color: Color,
    /// Whether the primitive updates when camera changes.
    pub update_on_change: bool,
    /// Determines if this primitive will be shown.
    pub show: bool,
    /// User-defined ID.
    pub id: Option<String>,
    /// Internal generation counter used to track updates.
    pub generation: u64,
    is_destroyed: bool,
}

impl DebugCameraPrimitive {
    /// Creates a new `DebugCameraPrimitive`.
    pub fn new(options: DebugCameraPrimitiveOptions) -> Result<Self, DeveloperError> {
        let camera = options
            .camera
            .ok_or_else(|| DeveloperError::new("options.camera is required."))?;

        Ok(Self {
            camera,
            frustum_splits: options.frustum_splits,
            color: options.color.unwrap_or(Color::CYAN),
            update_on_change: options.update_on_change.unwrap_or(true),
            show: options.show.unwrap_or(true),
            id: options.id,
            generation: 0,
            is_destroyed: false,
        })
    }

    /// Updates the primitive state.
    pub fn update(&mut self) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        if !self.show {
            return Ok(());
        }

        if self.update_on_change {
            self.generation = self.generation.wrapping_add(1);
        }

        Ok(())
    }

    /// Returns true if destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys resources.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }
}
