//! Atmospheric horizon fog matching CesiumJS `Scene/Fog.js`.
use cesium_math::{Cartesian3, Cartographic, DeveloperError, SceneMode, EPSILON4};

/// Per-frame fog output written by [`Fog::update`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FogFrameOutput {
    pub enabled: bool,
    pub renderable: bool,
    pub density: f64,
    pub visual_density_scalar: f64,
    pub sse: f64,
    pub minimum_brightness: f64,
}

/// Camera inputs needed by [`Fog::update`].
#[derive(Clone, Copy, Debug)]
pub struct FogCameraState {
    pub position_wc: Cartesian3,
    pub direction_wc: Cartesian3,
    pub position_cartographic: Option<Cartographic>,
}

impl Default for FogCameraState {
    fn default() -> Self {
        Self {
            position_wc: Cartesian3::new(0.0, -1.0e7, 0.0),
            direction_wc: Cartesian3::new(0.0, 1.0, 0.0),
            position_cartographic: Some(Cartographic::new(0.0, 0.0, 10_000.0)),
        }
    }
}

/// Frame state passed to [`Fog::update`].
#[derive(Clone, Copy, Debug)]
pub struct FogFrameState {
    pub mode: SceneMode,
    pub camera: FogCameraState,
    pub fog: FogFrameOutput,
}

impl Default for FogFrameState {
    fn default() -> Self {
        Self {
            mode: SceneMode::Scene3D,
            camera: FogCameraState::default(),
            fog: FogFrameOutput::default(),
        }
    }
}

/// Blends the atmosphere to geometry far from the camera for horizon views.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fog {
    pub enabled: bool,
    pub renderable: bool,
    pub density: f64,
    pub height_scalar: f64,
    height_falloff: f64,
    pub max_height: f64,
    pub visual_density_scalar: f64,
    pub screen_space_error_factor: f64,
    pub minimum_brightness: f64,
}

impl Default for Fog {
    fn default() -> Self {
        Self {
            enabled: true,
            renderable: true,
            density: 0.0006,
            height_scalar: 0.001,
            height_falloff: 0.59,
            max_height: 800_000.0,
            visual_density_scalar: 0.15,
            screen_space_error_factor: 2.0,
            minimum_brightness: 0.03,
        }
    }
}

impl Fog {
    /// Creates a new [`Fog`] with CesiumJS default parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the height falloff exponent.
    pub fn height_falloff(&self) -> f64 {
        self.height_falloff
    }

    /// Sets the height falloff exponent; returns `DeveloperError` if negative.
    pub fn set_height_falloff(&mut self, value: f64) -> Result<(), DeveloperError> {
        if value < 0.0 {
            return Err(DeveloperError::new("value must be positive."));
        }
        self.height_falloff = value;
        Ok(())
    }

    /// Updates `frame_state.fog` from the current fog configuration and camera state.
    pub fn update(&self, frame_state: &mut FogFrameState) {
        frame_state.fog.enabled = self.enabled;
        if !self.enabled {
            return;
        }

        frame_state.fog.renderable = self.renderable;

        let pos_carto = frame_state.camera.position_cartographic;
        if pos_carto.is_none()
            || pos_carto.is_some_and(|c| c.height > self.max_height)
            || frame_state.mode != SceneMode::Scene3D
        {
            frame_state.fog.enabled = false;
            frame_state.fog.density = 0.0;
            return;
        }

        let height = pos_carto.unwrap().height;
        let mut density = self.density
            * self.height_scalar
            * (height / self.max_height)
                .max(EPSILON4)
                .powf(-self.height_falloff.max(0.0));

        let position_normal = frame_state.camera.position_wc.normalize();
        let dot = frame_state
            .camera
            .direction_wc
            .dot(&position_normal)
            .abs();
        density *= 1.0 - dot;

        frame_state.fog.density = density;
        frame_state.fog.visual_density_scalar = self.visual_density_scalar;
        frame_state.fog.sse = self.screen_space_error_factor;
        frame_state.fog.minimum_brightness = self.minimum_brightness;
    }
}
