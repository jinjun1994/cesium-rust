//! ShadowMap matching CesiumJS `Scene/ShadowMap.js` (Apache-2.0).

use cesium_math::{Camera, Cartesian2, Cartesian4, DeveloperError, Matrix4};

/// Bias settings applied to shadow casters and receivers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowBias {
    pub polygon_offset: bool,
    pub polygon_offset_factor: f32,
    pub polygon_offset_units: f32,
    pub normal_offset: bool,
    pub normal_offset_scale: f32,
    pub normal_shading: bool,
    pub normal_shading_smooth: f32,
    pub depth_bias: f64,
}

impl Default for ShadowBias {
    fn default() -> Self {
        Self {
            polygon_offset: true,
            polygon_offset_factor: 1.1,
            polygon_offset_units: 4.0,
            normal_offset: true,
            normal_offset_scale: 0.1,
            normal_shading: true,
            normal_shading_smooth: 0.05,
            depth_bias: 0.00002,
        }
    }
}

/// A single render pass for shadow map generation.
#[derive(Clone, Debug, PartialEq)]
pub struct ShadowPass {
    pub viewport: (u32, u32, u32, u32), // (x, y, width, height)
    pub texture_offsets: Matrix4,
}

impl Default for ShadowPass {
    fn default() -> Self {
        Self {
            viewport: (0, 0, 2048, 2048),
            texture_offsets: Matrix4::IDENTITY,
        }
    }
}

/// Construction options for `ShadowMap`.
#[derive(Clone, Debug)]
pub struct ShadowMapOptions {
    pub light_camera: Option<Camera>,
    pub enabled: Option<bool>,
    pub is_point_light: Option<bool>,
    pub point_light_radius: Option<f64>,
    pub cascades_enabled: Option<bool>,
    pub number_of_cascades: Option<u32>,
    pub maximum_distance: Option<f64>,
    pub size: Option<u32>,
    pub soft_shadows: Option<bool>,
    pub darkness: Option<f32>,
    pub normal_offset: Option<bool>,
    pub fading_enabled: Option<bool>,
    pub from_light_source: Option<bool>,
}

impl Default for ShadowMapOptions {
    fn default() -> Self {
        Self {
            light_camera: None,
            enabled: Some(true),
            is_point_light: Some(false),
            point_light_radius: Some(100.0),
            cascades_enabled: Some(true),
            number_of_cascades: Some(4),
            maximum_distance: Some(5000.0),
            size: Some(2048),
            soft_shadows: Some(false),
            darkness: Some(0.3),
            normal_offset: Some(true),
            fading_enabled: Some(true),
            from_light_source: Some(true),
        }
    }
}

/// Shadow map implementation supporting cascaded directional shadows and omnidirectional point light shadows.
#[derive(Clone, Debug)]
pub struct ShadowMap {
    pub light_camera: Camera,
    enabled: bool,
    soft_shadows: bool,
    normal_offset: bool,
    pub from_light_source: bool,
    darkness: f32,
    pub fading_enabled: bool,
    pub maximum_distance: f64,
    size: u32,
    texture_size: Cartesian2,
    is_point_light: bool,
    pub point_light_radius: f64,
    cascades_enabled: bool,
    number_of_cascades: u32,
    is_spot_light: bool,
    pub dirty: bool,
    pub debug_show: bool,
    pub debug_freeze_frame: bool,
    debug_cascade_colors: bool,
    terrain_bias: ShadowBias,
    primitive_bias: ShadowBias,
    point_bias: ShadowBias,
    passes: Vec<ShadowPass>,
    cascade_splits: [Cartesian4; 2],
    cascade_matrices: [Matrix4; 4],
    pub out_of_view: bool,
}

impl ShadowMap {
    /// Global maximum shadow distance used to prevent far off receivers from extending the shadow far plane.
    pub const MAXIMUM_DISTANCE: f64 = 20000.0;

    /// Creates a new `ShadowMap`.
    pub fn new(options: ShadowMapOptions) -> Result<Self, DeveloperError> {
        let light_camera = options.light_camera.ok_or_else(|| DeveloperError {
            message: "lightCamera is required.".to_string(),
        })?;

        if let Some(cascades) = options.number_of_cascades {
            if cascades != 1 && cascades != 4 {
                return Err(DeveloperError {
                    message: "Only one or four cascades are supported.".to_string(),
                });
            }
        }

        let is_point_light = options.is_point_light.unwrap_or(false);
        let cascades_enabled = if is_point_light {
            false
        } else {
            options.cascades_enabled.unwrap_or(true)
        };
        let number_of_cascades = if !cascades_enabled {
            0
        } else {
            options.number_of_cascades.unwrap_or(4)
        };

        let normal_offset = options.normal_offset.unwrap_or(true);
        let size = options.size.unwrap_or(2048);

        let terrain_bias = ShadowBias {
            polygon_offset: true,
            polygon_offset_factor: 1.1,
            polygon_offset_units: 4.0,
            normal_offset,
            normal_offset_scale: 0.5,
            normal_shading: true,
            normal_shading_smooth: 0.3,
            depth_bias: 0.0001,
        };

        let primitive_bias = ShadowBias {
            polygon_offset: true,
            polygon_offset_factor: 1.1,
            polygon_offset_units: 4.0,
            normal_offset,
            normal_offset_scale: 0.1,
            normal_shading: true,
            normal_shading_smooth: 0.05,
            depth_bias: 0.00002,
        };

        let point_bias = ShadowBias {
            polygon_offset: false,
            polygon_offset_factor: 1.1,
            polygon_offset_units: 4.0,
            normal_offset,
            normal_offset_scale: 0.0,
            normal_shading: true,
            normal_shading_smooth: 0.1,
            depth_bias: 0.0005,
        };

        let number_of_passes = if is_point_light {
            6
        } else if !cascades_enabled {
            1
        } else {
            number_of_cascades as usize
        };

        let mut shadow_map = Self {
            light_camera,
            enabled: options.enabled.unwrap_or(true),
            soft_shadows: options.soft_shadows.unwrap_or(false),
            normal_offset,
            from_light_source: options.from_light_source.unwrap_or(true),
            darkness: options.darkness.unwrap_or(0.3),
            fading_enabled: options.fading_enabled.unwrap_or(true),
            maximum_distance: options.maximum_distance.unwrap_or(5000.0),
            size,
            texture_size: Cartesian2::new(0.0, 0.0),
            is_point_light,
            point_light_radius: options.point_light_radius.unwrap_or(100.0),
            cascades_enabled,
            number_of_cascades,
            is_spot_light: false,
            dirty: true,
            debug_show: false,
            debug_freeze_frame: false,
            debug_cascade_colors: false,
            terrain_bias,
            primitive_bias,
            point_bias,
            passes: vec![ShadowPass::default(); number_of_passes],
            cascade_splits: [Cartesian4::ZERO, Cartesian4::ZERO],
            cascade_matrices: [Matrix4::IDENTITY; 4],
            out_of_view: false,
        };

        shadow_map.resize(size);
        Ok(shadow_map)
    }

    /// Resizes the shadow map texture and recomputes pass viewports and texture offset matrices.
    pub fn resize(&mut self, size: u32) {
        self.size = size;
        let num_passes = self.passes.len();

        if self.is_point_light {
            self.texture_size = Cartesian2::new(size as f64, size as f64);
            let vp = (0, 0, size, size);
            for pass in &mut self.passes {
                pass.viewport = vp;
                pass.texture_offsets = Matrix4::IDENTITY;
            }
        } else if num_passes == 1 {
            self.texture_size = Cartesian2::new(size as f64, size as f64);
            self.passes[0].viewport = (0, 0, size, size);
            self.passes[0].texture_offsets = Matrix4::IDENTITY;
        } else if num_passes == 4 {
            let total_size = size * 2;
            self.texture_size = Cartesian2::new(total_size as f64, total_size as f64);
            let viewports = [
                (0, 0, size, size),
                (size, 0, size, size),
                (0, size, size, size),
                (size, size, size, size),
            ];

            for (i, pass) in self.passes.iter_mut().enumerate() {
                let vp = viewports[i];
                pass.viewport = vp;

                let scale_x = vp.2 as f64 / total_size as f64;
                let scale_y = vp.3 as f64 / total_size as f64;
                let bias_x = vp.0 as f64 / total_size as f64;
                let bias_y = vp.1 as f64 / total_size as f64;

                pass.texture_offsets = Matrix4::new(
                    scale_x, 0.0, 0.0, bias_x,
                    0.0, scale_y, 0.0, bias_y,
                    0.0, 0.0, 1.0, 0.0,
                    0.0, 0.0, 0.0, 1.0,
                );
            }
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, value: bool) {
        if self.enabled != value {
            self.dirty = true;
            self.enabled = value;
        }
    }

    pub fn soft_shadows(&self) -> bool {
        self.soft_shadows
    }

    pub fn set_soft_shadows(&mut self, value: bool) {
        if self.soft_shadows != value {
            self.dirty = true;
            self.soft_shadows = value;
        }
    }

    pub fn normal_offset(&self) -> bool {
        self.normal_offset
    }

    pub fn set_normal_offset(&mut self, value: bool) {
        if self.normal_offset != value {
            self.dirty = true;
            self.normal_offset = value;
            self.terrain_bias.normal_offset = value;
            self.primitive_bias.normal_offset = value;
            self.point_bias.normal_offset = value;
        }
    }

    pub fn darkness(&self) -> f32 {
        self.darkness
    }

    pub fn set_darkness(&mut self, value: f32) {
        self.darkness = value;
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn set_size(&mut self, value: u32) {
        if self.size != value {
            self.dirty = true;
            self.resize(value);
        }
    }

    pub fn texture_size(&self) -> Cartesian2 {
        self.texture_size
    }

    pub fn is_point_light(&self) -> bool {
        self.is_point_light
    }

    pub fn is_spot_light(&self) -> bool {
        self.is_spot_light
    }

    pub fn set_is_spot_light(&mut self, value: bool) {
        self.is_spot_light = value;
    }

    pub fn cascades_enabled(&self) -> bool {
        self.cascades_enabled
    }

    pub fn number_of_cascades(&self) -> u32 {
        self.number_of_cascades
    }

    pub fn debug_cascade_colors(&self) -> bool {
        self.debug_cascade_colors
    }

    pub fn set_debug_cascade_colors(&mut self, value: bool) {
        if self.debug_cascade_colors != value {
            self.dirty = true;
            self.debug_cascade_colors = value;
        }
    }

    pub fn terrain_bias(&self) -> &ShadowBias {
        &self.terrain_bias
    }

    pub fn primitive_bias(&self) -> &ShadowBias {
        &self.primitive_bias
    }

    pub fn point_bias(&self) -> &ShadowBias {
        &self.point_bias
    }

    pub fn passes(&self) -> &[ShadowPass] {
        &self.passes
    }

    pub fn passes_mut(&mut self) -> &mut [ShadowPass] {
        &mut self.passes
    }

    pub fn cascade_splits(&self) -> &[Cartesian4; 2] {
        &self.cascade_splits
    }

    pub fn cascade_matrices(&self) -> &[Matrix4; 4] {
        &self.cascade_matrices
    }

    /// Computes practical split scheme near/far cascade bounds.
    pub fn compute_cascade_splits(near: f64, far: f64, lambda: f64) -> [f64; 4] {
        let mut splits = [0.0; 4];
        let num = 4.0;
        for (i, split) in splits.iter_mut().enumerate() {
            let p = (i as f64 + 1.0) / num;
            let log_split = near * (far / near).powf(p);
            let uniform_split = near + (far - near) * p;
            *split = lambda * log_split + (1.0 - lambda) * uniform_split;
        }
        splits
    }
}
