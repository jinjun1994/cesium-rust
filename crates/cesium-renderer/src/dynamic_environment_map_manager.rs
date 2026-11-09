use cesium_math::{Cartesian3, Color};

/// Initialization options for `DynamicEnvironmentMapManager`.
#[derive(Debug, Clone)]
pub struct DynamicEnvironmentMapManagerOptions {
    /// If true, the environment map and related properties will continue to update.
    pub enabled: Option<bool>,
    /// The maximum desired number of mipmap levels to generate. Defaults to 7.
    pub mipmap_levels: Option<i32>,
    /// The maximum amount of elapsed seconds before a new environment map is created.
    pub maximum_seconds_difference: Option<f64>,
    /// The maximum difference in position before a new environment map is created, in meters.
    pub maximum_position_epsilon: Option<f64>,
    /// The intensity of the scattered light emitted from the atmosphere.
    pub atmosphere_scattering_intensity: Option<f64>,
    /// The gamma correction to apply.
    pub gamma: Option<f64>,
    /// The brightness of light emitted.
    pub brightness: Option<f64>,
    /// The saturation of the light emitted.
    pub saturation: Option<f64>,
    /// Solid color used to represent the ground.
    pub ground_color: Option<Color>,
    /// The percentage of light reflected from the ground.
    pub ground_albedo: Option<f64>,
}

impl Default for DynamicEnvironmentMapManagerOptions {
    fn default() -> Self {
        Self {
            enabled: Some(true),
            mipmap_levels: Some(7),
            maximum_seconds_difference: Some(3600.0),
            maximum_position_epsilon: Some(1000.0),
            atmosphere_scattering_intensity: Some(2.0),
            gamma: Some(1.0),
            brightness: Some(1.0),
            saturation: Some(1.0),
            ground_color: None,
            ground_albedo: Some(0.31),
        }
    }
}

/// Generates an environment map at the given position based on scene lighting conditions.
#[derive(Debug, Clone)]
pub struct DynamicEnvironmentMapManager {
    /// If true, the environment map will continue to update.
    pub enabled: bool,
    /// Whether the environment map should update.
    pub should_update: bool,
    /// Maximum elapsed seconds before recreation.
    pub maximum_seconds_difference: f64,
    /// Maximum difference in position before recreation, in meters.
    pub maximum_position_epsilon: f64,
    /// The intensity of the scattered light emitted from the atmosphere.
    pub atmosphere_scattering_intensity: f64,
    /// Gamma correction.
    pub gamma: f64,
    /// Brightness.
    pub brightness: f64,
    /// Saturation.
    pub saturation: f64,
    /// Ground color.
    pub ground_color: Color,
    /// Ground albedo.
    pub ground_albedo: f64,
    /// Current position in world coordinates.
    pub position: Option<Cartesian3>,
    /// Spherical harmonic coefficients (9 order 3 coefficients).
    pub spherical_harmonic_coefficients: [Cartesian3; 9],
    /// Mipmap levels.
    pub mipmap_levels: i32,
    is_destroyed: bool,
}

impl DynamicEnvironmentMapManager {
    /// Average hue of ground color on earth (#717145).
    pub const AVERAGE_EARTH_GROUND_COLOR: Color = Color::new(
        113.0 / 255.0,
        113.0 / 255.0,
        69.0 / 255.0,
        1.0,
    );

    /// Default 3rd order spherical harmonic coefficients.
    pub const DEFAULT_SPHERICAL_HARMONIC_COEFFICIENTS: [Cartesian3; 9] = [
        Cartesian3::new(0.35449, 0.35449, 0.35449),
        Cartesian3::ZERO,
        Cartesian3::ZERO,
        Cartesian3::ZERO,
        Cartesian3::ZERO,
        Cartesian3::ZERO,
        Cartesian3::ZERO,
        Cartesian3::ZERO,
        Cartesian3::ZERO,
    ];

    /// Creates a new `DynamicEnvironmentMapManager`.
    pub fn new(options: Option<DynamicEnvironmentMapManagerOptions>) -> Self {
        let opts = options.unwrap_or_default();
        let mipmap_levels = opts.mipmap_levels.unwrap_or(7).max(0);

        Self {
            enabled: opts.enabled.unwrap_or(true),
            should_update: true,
            maximum_seconds_difference: opts.maximum_seconds_difference.unwrap_or(3600.0),
            maximum_position_epsilon: opts.maximum_position_epsilon.unwrap_or(1000.0),
            atmosphere_scattering_intensity: opts.atmosphere_scattering_intensity.unwrap_or(2.0),
            gamma: opts.gamma.unwrap_or(1.0),
            brightness: opts.brightness.unwrap_or(1.0),
            saturation: opts.saturation.unwrap_or(1.0),
            ground_color: opts.ground_color.unwrap_or(Self::AVERAGE_EARTH_GROUND_COLOR),
            ground_albedo: opts.ground_albedo.unwrap_or(0.31),
            position: None,
            spherical_harmonic_coefficients: Self::DEFAULT_SPHERICAL_HARMONIC_COEFFICIENTS,
            mipmap_levels,
            is_destroyed: false,
        }
    }

    /// Resets the spherical harmonics and internal state.
    pub fn reset(&mut self) {
        self.spherical_harmonic_coefficients = Self::DEFAULT_SPHERICAL_HARMONIC_COEFFICIENTS;
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
