//! SkyAtmosphere matching CesiumJS `Scene/SkyAtmosphere.js` (Apache-2.0).

use cesium_math::{Cartesian3, Ellipsoid, Matrix4, SceneMode};
use glam::DMat4;

use crate::commands::{DrawCommand, Pass};
use crate::dynamic_atmosphere_lighting_type::DynamicAtmosphereLightingType;

/// Frame passes state for sky atmosphere rendering.
#[derive(Clone, Copy, Debug)]
pub struct SkyAtmosphereFramePasses {
    pub render: bool,
}

impl Default for SkyAtmosphereFramePasses {
    fn default() -> Self {
        Self { render: true }
    }
}

/// Minimal frame state required by `SkyAtmosphere::update`.
#[derive(Clone, Debug, Default)]
pub struct SkyAtmosphereFrameState {
    pub mode: SceneMode,
    pub passes: SkyAtmosphereFramePasses,
    pub inverse_view_rotation: Option<Matrix4>,
    pub globe_translucent: bool,
}

/// An atmosphere drawn around the limb of the provided ellipsoid.
#[derive(Clone, Debug)]
pub struct SkyAtmosphere {
    pub show: bool,
    pub per_fragment_atmosphere: bool,
    ellipsoid: Ellipsoid,
    pub atmosphere_light_intensity: f32,
    pub atmosphere_rayleigh_coefficient: Cartesian3,
    pub atmosphere_mie_coefficient: Cartesian3,
    pub atmosphere_rayleigh_scale_height: f64,
    pub atmosphere_mie_scale_height: f64,
    pub atmosphere_mie_anisotropy: f32,
    pub hue_shift: f32,
    pub saturation_shift: f32,
    pub brightness_shift: f32,
    pub radii_and_dynamic_atmosphere_color: Cartesian3,
    pub scale_matrix: Matrix4,
    pub model_matrix: Matrix4,
    destroyed: bool,
}

impl Default for SkyAtmosphere {
    fn default() -> Self {
        Self::new(None)
    }
}

impl SkyAtmosphere {
    pub const OUTER_ELLIPSOID_SCALE: f64 = 1.025;

    /// Creates a new `SkyAtmosphere`.
    pub fn new(ellipsoid: Option<Ellipsoid>) -> Self {
        let el = ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let radii = el.radii();
        let scale_vector = Cartesian3::new(
            radii.x * Self::OUTER_ELLIPSOID_SCALE,
            radii.y * Self::OUTER_ELLIPSOID_SCALE,
            radii.z * Self::OUTER_ELLIPSOID_SCALE,
        );
        let scale_matrix = Matrix4::from_scale(scale_vector);

        let max_r = el.maximum_radius();
        let radii_and_dynamic = Cartesian3::new(
            max_r * Self::OUTER_ELLIPSOID_SCALE,
            max_r,
            0.0,
        );

        Self {
            show: true,
            per_fragment_atmosphere: false,
            ellipsoid: el,
            atmosphere_light_intensity: 50.0,
            atmosphere_rayleigh_coefficient: Cartesian3::new(5.5e-6, 13.0e-6, 28.4e-6),
            atmosphere_mie_coefficient: Cartesian3::new(21e-6, 21e-6, 21e-6),
            atmosphere_rayleigh_scale_height: 10000.0,
            atmosphere_mie_scale_height: 3200.0,
            atmosphere_mie_anisotropy: 0.9,
            hue_shift: 0.0,
            saturation_shift: 0.0,
            brightness_shift: 0.0,
            radii_and_dynamic_atmosphere_color: radii_and_dynamic,
            scale_matrix,
            model_matrix: Matrix4::IDENTITY,
            destroyed: false,
        }
    }

    pub fn ellipsoid(&self) -> Ellipsoid {
        self.ellipsoid
    }

    /// Set the dynamic lighting enum value for the shader.
    pub fn set_dynamic_lighting(&mut self, lighting: DynamicAtmosphereLightingType) {
        let flag = match lighting {
            DynamicAtmosphereLightingType::None => 0.0,
            DynamicAtmosphereLightingType::SceneLight => 1.0,
            DynamicAtmosphereLightingType::Sunlight => 2.0,
        };
        self.radii_and_dynamic_atmosphere_color.z = flag;
    }

    pub fn has_color_correction(&self) -> bool {
        self.hue_shift.abs() > 1e-7
            || self.saturation_shift.abs() > 1e-7
            || self.brightness_shift.abs() > 1e-7
    }

    /// Updates atmosphere state and returns the draw command to execute during rendering.
    pub fn update(
        &mut self,
        frame_state: &SkyAtmosphereFrameState,
        globe_show: Option<bool>,
    ) -> Option<DrawCommand> {
        if !self.show {
            return None;
        }

        if frame_state.mode != SceneMode::Scene3D && frame_state.mode != SceneMode::Morphing {
            return None;
        }

        if !frame_state.passes.render {
            return None;
        }

        let is_globe_shown = globe_show.unwrap_or(true);
        let _per_fragment = self.per_fragment_atmosphere
            || frame_state.globe_translucent
            || !is_globe_shown;

        if let Some(inv_view) = frame_state.inverse_view_rotation {
            self.model_matrix = self.scale_matrix.multiply(&inv_view);
        }

        let cmd = DrawCommand {
            pass: Some(Pass::Environment),
            model_matrix: Some(DMat4::from_cols_array(&self.model_matrix.values)),
            ..Default::default()
        };
        Some(cmd)
    }

    pub fn is_destroyed(&self) -> bool {
        self.destroyed
    }

    pub fn destroy(&mut self) {
        self.destroyed = true;
    }
}
