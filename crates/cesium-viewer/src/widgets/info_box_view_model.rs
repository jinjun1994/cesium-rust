//! InfoBoxViewModel matching CesiumJS `Widgets/InfoBox/InfoBoxViewModel.js` (Apache-2.0).

use cesium_math::Event;

pub const CAMERA_ENABLED_PATH: &str =
    "M 13.84375 7.03125 C 11.412798 7.03125 9.46875 8.975298 9.46875 11.40625 L 9.46875 11.59375 L 2.53125 7.21875 L 2.53125 24.0625 L 9.46875 19.6875 C 9.4853444 22.104033 11.423165 24.0625 13.84375 24.0625 L 25.875 24.0625 C 28.305952 24.0625 30.28125 22.087202 30.28125 19.65625 L 30.28125 11.40625 C 30.28125 8.975298 28.305952 7.03125 25.875 7.03125 L 13.84375 7.03125 z";

pub const CAMERA_DISABLED_PATH: &str =
    "M 27.34375 1.65625 L 5.28125 27.9375 L 8.09375 30.3125 L 30.15625 4.03125 L 27.34375 1.65625 z M 13.84375 7.03125 C 11.412798 7.03125 9.46875 8.975298 9.46875 11.40625 L 9.46875 11.59375 L 2.53125 7.21875 L 2.53125 24.0625 L 9.46875 19.6875 C 9.4724893 20.232036 9.5676108 20.7379 9.75 21.21875 L 21.65625 7.03125 L 13.84375 7.03125 z M 28.21875 7.71875 L 14.53125 24.0625 L 25.875 24.0625 C 28.305952 24.0625 30.28125 22.087202 30.28125 19.65625 L 30.28125 11.40625 C 30.28125 9.8371439 29.456025 8.4902779 28.21875 7.71875 z";

/// View model for [`InfoBox`].
#[derive(Clone, Debug)]
pub struct InfoBoxViewModel {
    pub max_height: f64,
    pub enable_camera: bool,
    pub is_camera_tracking: bool,
    pub show_info: bool,
    pub title_text: String,
    pub description: String,
    pub camera_clicked: Event<()>,
    pub close_clicked: Event<()>,
}

impl Default for InfoBoxViewModel {
    fn default() -> Self {
        Self::new()
    }
}

impl InfoBoxViewModel {
    /// Creates a new `InfoBoxViewModel`.
    pub fn new() -> Self {
        Self {
            max_height: 500.0,
            enable_camera: false,
            is_camera_tracking: false,
            show_info: false,
            title_text: String::new(),
            description: String::new(),
            camera_clicked: Event::new(),
            close_clicked: Event::new(),
        }
    }

    /// Gets whether the info box has no description body.
    pub fn is_bodyless(&self) -> bool {
        self.description.is_empty()
    }

    /// Alias for JS matching `_bodyless`.
    pub fn _bodyless(&self) -> bool {
        self.is_bodyless()
    }

    /// Gets the current SVG path for the camera icon.
    pub fn camera_icon_path(&self) -> &'static str {
        if !self.enable_camera || self.is_camera_tracking {
            CAMERA_DISABLED_PATH
        } else {
            CAMERA_ENABLED_PATH
        }
    }

    /// Gets the maximum height of sections within the info box, minus an offset, in CSS-ready form.
    pub fn max_height_offset(&self, offset: f64) -> String {
        format!("{}px", self.max_height - offset)
    }
}
