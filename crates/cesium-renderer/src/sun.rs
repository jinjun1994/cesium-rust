//! Sun billboard matching CesiumJS `Scene/Sun.js` (Apache-2.0).

use cesium_math::{BoundingSphere, Cartesian3, Matrix4, SceneMode};

use crate::commands::DrawCommand;
use crate::pass_state::PassState;

/// Passes state for sun rendering queries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SunFramePasses {
    pub render: bool,
    pub pick: bool,
}

impl Default for SunFramePasses {
    fn default() -> Self {
        Self {
            render: true,
            pick: false,
        }
    }
}

/// Frame state required for updating and sizing the Sun primitive.
#[derive(Clone, Debug)]
pub struct SunFrameState {
    pub mode: SceneMode,
    pub passes: SunFramePasses,
    pub sun_position_wc: Cartesian3,
    pub sun_position_columbus_view: Cartesian3,
    pub camera_position: Cartesian3,
    pub projection_matrix: Matrix4,
}

impl Default for SunFrameState {
    fn default() -> Self {
        Self {
            mode: SceneMode::Scene3D,
            passes: SunFramePasses::default(),
            sun_position_wc: Cartesian3::new(1.496e11, 0.0, 0.0),
            sun_position_columbus_view: Cartesian3::new(1.496e11, 0.0, 0.0),
            camera_position: Cartesian3::ZERO,
            projection_matrix: Matrix4::IDENTITY,
        }
    }
}

/// Container for commands generated during sun updating.
#[derive(Clone, Debug, Default)]
pub struct SunCommands {
    pub draw_command: Option<DrawCommand>,
    pub compute_command_needed: bool,
}

/// Draws a sun billboard in 3D and Columbus View.
#[derive(Clone, Debug)]
pub struct Sun {
    /// Determines if the sun will be shown.
    pub show: bool,
    /// Controls the brightness/extent of the sun's lens flare. Zero shows just the sun disc.
    pub glow_factor: f64,
    /// Glow length in texture space.
    pub glow_length_ts: f64,
    /// Radius of the sun disc in texture space.
    pub radius_ts: f64,
    /// Computed billboard screen size.
    pub size: f64,
    /// 3D bounding sphere.
    pub bounding_volume: BoundingSphere,
    /// Columbus View bounding sphere.
    pub bounding_volume_2d: BoundingSphere,
    /// Whether this object has been destroyed.
    is_destroyed: bool,
    /// Internal cached commands.
    commands: SunCommands,
}

impl Default for Sun {
    fn default() -> Self {
        Self::new()
    }
}

impl Sun {
    /// Solar radius in meters (`6.955e8`).
    pub const SOLAR_RADIUS: f64 = 6.955e8;

    /// Constructs a new `Sun` with default glow factor and visible state.
    pub fn new() -> Self {
        let mut sun = Self {
            show: true,
            glow_factor: 1.0,
            glow_length_ts: 5.0,
            radius_ts: 0.5 / 11.0,
            size: 1.0,
            bounding_volume: BoundingSphere::default(),
            bounding_volume_2d: BoundingSphere::default(),
            is_destroyed: false,
            commands: SunCommands::default(),
        };
        sun.update_texture_metrics();
        sun
    }

    /// Gets the current glow factor.
    pub fn glow_factor(&self) -> f64 {
        self.glow_factor
    }

    /// Sets the glow factor, clamping negative values to zero.
    pub fn set_glow_factor(&mut self, factor: f64) {
        self.glow_factor = factor.max(0.0);
        self.update_texture_metrics();
    }

    fn update_texture_metrics(&mut self) {
        self.glow_length_ts = self.glow_factor * 5.0;
        self.radius_ts = (1.0 / (1.0 + 2.0 * self.glow_length_ts)) * 0.5;
    }

    /// Updates the Sun primitive and returns draw commands if visible in the current mode and pass.
    pub fn update(
        &mut self,
        frame_state: &SunFrameState,
        _pass_state: Option<&PassState>,
        _use_hdr: bool,
    ) -> Option<&SunCommands> {
        if self.is_destroyed || !self.show {
            return None;
        }

        if frame_state.mode == SceneMode::Scene2D || frame_state.mode == SceneMode::Morphing {
            return None;
        }

        if !frame_state.passes.render {
            return None;
        }

        self.update_texture_metrics();

        let radius = Self::SOLAR_RADIUS + Self::SOLAR_RADIUS * self.glow_length_ts;
        self.bounding_volume = BoundingSphere::new(frame_state.sun_position_wc, radius);

        let cv = frame_state.sun_position_columbus_view;
        let cv_center = Cartesian3::new(cv.z, cv.x, cv.y);
        self.bounding_volume_2d = BoundingSphere::new(cv_center, radius);

        // Approximate screen billboard size
        self.size = 2.0 * 32.0 * (1.0 + 2.0 * self.glow_length_ts);

        let mut draw_cmd = DrawCommand::default();
        if frame_state.mode == SceneMode::Scene3D {
            draw_cmd.bounding_volume = Some(self.bounding_volume);
        } else if frame_state.mode == SceneMode::ColumbusView {
            draw_cmd.bounding_volume = Some(self.bounding_volume_2d);
        }

        self.commands.draw_command = Some(draw_cmd);
        self.commands.compute_command_needed = true;

        Some(&self.commands)
    }

    /// Returns true if this object was destroyed; otherwise, false.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys resources held by this object.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        self.commands = SunCommands::default();
    }
}
