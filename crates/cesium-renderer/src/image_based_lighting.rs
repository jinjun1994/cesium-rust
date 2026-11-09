use cesium_math::{Cartesian2, Cartesian3, DeveloperError};

/// Options for constructing an [`ImageBasedLighting`] instance.
#[derive(Debug, Clone, Default)]
pub struct ImageBasedLightingOptions {
    /// Scales diffuse and specular image-based lighting from the earth, sky, atmosphere and star skybox.
    pub image_based_lighting_factor: Option<Cartesian2>,
    /// The third order spherical harmonic coefficients used for diffuse irradiance (must have 9 values).
    pub spherical_harmonic_coefficients: Option<Vec<Cartesian3>>,
    /// A URL to a KTX2 / environment cube map file.
    pub specular_environment_maps: Option<String>,
}

/// Properties and runtime state for managing image-based lighting on tilesets and models.
#[derive(Debug, Clone)]
pub struct ImageBasedLighting {
    image_based_lighting_factor: Cartesian2,
    spherical_harmonic_coefficients: Option<Vec<Cartesian3>>,
    specular_environment_maps: Option<String>,

    previous_image_based_lighting_factor: Cartesian2,
    previous_spherical_harmonic_coefficients: Option<Vec<Cartesian3>>,

    specular_environment_cube_map_dirty: bool,
    specular_environment_map_loaded: bool,
    previous_specular_environment_map_loaded: bool,

    use_default_specular_maps: bool,
    use_default_spherical_harmonics: bool,
    should_regenerate_shaders: bool,

    previous_frame_number: Option<u64>,
    is_destroyed: bool,
}

impl Default for ImageBasedLighting {
    fn default() -> Self {
        Self::new(None).expect("Default ImageBasedLighting options must be valid")
    }
}

impl ImageBasedLighting {
    fn validate_factor(name: &str, factor: &Cartesian2) -> Result<(), DeveloperError> {
        if factor.x < 0.0 || factor.x > 1.0 {
            return Err(DeveloperError::new(format!(
                "{name}.x must be between 0.0 and 1.0, got {}",
                factor.x
            )));
        }
        if factor.y < 0.0 || factor.y > 1.0 {
            return Err(DeveloperError::new(format!(
                "{name}.y must be between 0.0 and 1.0, got {}",
                factor.y
            )));
        }
        Ok(())
    }

    fn validate_coefficients(
        coeffs: &Option<Vec<Cartesian3>>,
    ) -> Result<(), DeveloperError> {
        if let Some(list) = coeffs {
            if list.len() != 9 {
                return Err(DeveloperError::new(
                    "sphericalHarmonicCoefficients must be an array of 9 Cartesian3 values.",
                ));
            }
        }
        Ok(())
    }

    /// Creates a new `ImageBasedLighting` instance with optional configuration.
    pub fn new(options: Option<ImageBasedLightingOptions>) -> Result<Self, DeveloperError> {
        let opts = options.unwrap_or_default();
        let factor = opts.image_based_lighting_factor.unwrap_or(Cartesian2::new(1.0, 1.0));
        Self::validate_factor("options.imageBasedLightingFactor", &factor)?;
        Self::validate_coefficients(&opts.spherical_harmonic_coefficients)?;

        let previous_factor = factor;
        let previous_coeffs = opts.spherical_harmonic_coefficients.clone();

        Ok(Self {
            image_based_lighting_factor: factor,
            spherical_harmonic_coefficients: opts.spherical_harmonic_coefficients,
            specular_environment_maps: opts.specular_environment_maps,

            previous_image_based_lighting_factor: previous_factor,
            previous_spherical_harmonic_coefficients: previous_coeffs,

            specular_environment_cube_map_dirty: true,
            specular_environment_map_loaded: false,
            previous_specular_environment_map_loaded: false,

            use_default_specular_maps: false,
            use_default_spherical_harmonics: false,
            should_regenerate_shaders: false,

            previous_frame_number: None,
            is_destroyed: false,
        })
    }

    /// Image-based lighting factor for scaling diffuse and specular lighting contributions.
    pub fn image_based_lighting_factor(&self) -> Cartesian2 {
        self.image_based_lighting_factor
    }

    /// Sets the image-based lighting factor.
    pub fn set_image_based_lighting_factor(&mut self, factor: Cartesian2) -> Result<(), DeveloperError> {
        self.check_destroyed()?;
        Self::validate_factor("imageBasedLightingFactor", &factor)?;
        self.previous_image_based_lighting_factor = self.image_based_lighting_factor;
        self.image_based_lighting_factor = factor;
        Ok(())
    }

    /// Gets the previous image-based lighting factor before the last update.
    pub fn previous_image_based_lighting_factor(&self) -> Cartesian2 {
        self.previous_image_based_lighting_factor
    }

    /// Spherical harmonic coefficients used for diffuse lighting.
    pub fn spherical_harmonic_coefficients(&self) -> Option<&[Cartesian3]> {
        self.spherical_harmonic_coefficients.as_deref()
    }

    /// Sets the spherical harmonic coefficients (must have length 9 if Some).
    pub fn set_spherical_harmonic_coefficients(
        &mut self,
        coeffs: Option<Vec<Cartesian3>>,
    ) -> Result<(), DeveloperError> {
        self.check_destroyed()?;
        Self::validate_coefficients(&coeffs)?;
        self.previous_spherical_harmonic_coefficients = self.spherical_harmonic_coefficients.clone();
        self.spherical_harmonic_coefficients = coeffs;
        Ok(())
    }

    /// Gets the previous spherical harmonic coefficients.
    pub fn previous_spherical_harmonic_coefficients(&self) -> Option<&[Cartesian3]> {
        self.previous_spherical_harmonic_coefficients.as_deref()
    }

    /// URL to environment cube map.
    pub fn specular_environment_maps(&self) -> Option<&str> {
        self.specular_environment_maps.as_deref()
    }

    /// Sets the URL to specular environment map.
    pub fn set_specular_environment_maps(&mut self, maps: Option<String>) -> Result<(), DeveloperError> {
        self.check_destroyed()?;
        if self.specular_environment_maps != maps {
            self.specular_environment_cube_map_dirty = true;
            self.specular_environment_map_loaded = false;
        }
        self.specular_environment_maps = maps;
        Ok(())
    }

    /// True if either x or y factor is strictly greater than 0.0.
    pub fn enabled(&self) -> bool {
        self.image_based_lighting_factor.x > 0.0 || self.image_based_lighting_factor.y > 0.0
    }

    /// Whether or not models using this lighting should regenerate their shaders.
    pub fn should_regenerate_shaders(&self) -> bool {
        self.should_regenerate_shaders
    }

    /// Whether or not default specular maps are active.
    pub fn use_default_specular_maps(&self) -> bool {
        self.use_default_specular_maps
    }

    /// Whether or not default spherical harmonics coefficients are active.
    pub fn use_default_spherical_harmonics(&self) -> bool {
        self.use_default_spherical_harmonics
    }

    /// Whether specular environment maps are in use.
    pub fn use_specular_environment_maps(&self) -> bool {
        self.specular_environment_map_loaded || self.use_default_specular_maps
    }

    /// Updates image based lighting state per frame.
    pub fn update(
        &mut self,
        frame_number: u64,
        frame_specular_maps: Option<&str>,
        frame_sh_coeffs: Option<&[Cartesian3]>,
        cube_map_ready: bool,
    ) -> Result<(), DeveloperError> {
        self.check_destroyed()?;

        if self.previous_frame_number == Some(frame_number) {
            return Ok(());
        }
        self.previous_frame_number = Some(frame_number);
        self.should_regenerate_shaders = false;

        let ibl_factor = self.image_based_lighting_factor;
        let prev_ibl_factor = self.previous_image_based_lighting_factor;

        if ibl_factor != prev_ibl_factor {
            let x_toggled = (ibl_factor.x > 0.0 && prev_ibl_factor.x == 0.0)
                || (ibl_factor.x == 0.0 && prev_ibl_factor.x > 0.0);
            let y_toggled = (ibl_factor.y > 0.0 && prev_ibl_factor.y == 0.0)
                || (ibl_factor.y == 0.0 && prev_ibl_factor.y > 0.0);
            if x_toggled || y_toggled {
                self.should_regenerate_shaders = true;
            }
            self.previous_image_based_lighting_factor = ibl_factor;
        }

        let sh_presence_changed = self.previous_spherical_harmonic_coefficients.is_some()
            != self.spherical_harmonic_coefficients.is_some();
        if sh_presence_changed {
            self.should_regenerate_shaders = true;
            self.previous_spherical_harmonic_coefficients = self.spherical_harmonic_coefficients.clone();
        }

        if self.specular_environment_maps.is_some() {
            if self.specular_environment_cube_map_dirty {
                self.should_regenerate_shaders = true;
                self.specular_environment_cube_map_dirty = false;
            }
            if cube_map_ready {
                self.specular_environment_map_loaded = true;
            }
        } else {
            self.specular_environment_map_loaded = false;
        }

        if self.previous_specular_environment_map_loaded != self.specular_environment_map_loaded {
            self.should_regenerate_shaders = true;
            self.previous_specular_environment_map_loaded = self.specular_environment_map_loaded;
        }

        let recompile_with_default_cube_map = self.specular_environment_maps.is_none()
            && frame_specular_maps.is_some()
            && !self.use_default_specular_maps;
        let recompile_without_default_cube_map =
            frame_specular_maps.is_none() && self.use_default_specular_maps;

        let recompile_with_default_sh_coeffs = self.spherical_harmonic_coefficients.is_none()
            && frame_sh_coeffs.is_some()
            && !self.use_default_spherical_harmonics;
        let recompile_without_default_sh_coeffs =
            frame_sh_coeffs.is_none() && self.use_default_spherical_harmonics;

        if recompile_with_default_cube_map
            || recompile_without_default_cube_map
            || recompile_with_default_sh_coeffs
            || recompile_without_default_sh_coeffs
        {
            self.should_regenerate_shaders = true;
        }

        self.use_default_specular_maps =
            self.specular_environment_maps.is_none() && frame_specular_maps.is_some();
        self.use_default_spherical_harmonics =
            self.spherical_harmonic_coefficients.is_none() && frame_sh_coeffs.is_some();

        Ok(())
    }

    fn check_destroyed(&self) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }
        Ok(())
    }

    /// Whether this object has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys GPU and internal resources.
    pub fn destroy(&mut self) -> Result<(), DeveloperError> {
        self.check_destroyed()?;
        self.is_destroyed = true;
        self.specular_environment_maps = None;
        self.spherical_harmonic_coefficients = None;
        Ok(())
    }
}
