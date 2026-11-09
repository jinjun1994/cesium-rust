//! Equirectangular panorama matching CesiumJS `Scene/EquirectangularPanorama.js` (Apache-2.0).

use crate::commands::DrawCommand;
use cesium_math::{Cartesian2, Credit, DeveloperError, Matrix4};

pub const DEFAULT_RADIUS: f64 = 100000.0;

/// Uniforms for the equirectangular panorama material.
#[derive(Debug, Clone, PartialEq)]
pub struct EquirectangularUniforms {
    pub image: String,
    pub repeat: Cartesian2,
}

/// Material for equirectangular panorama.
#[derive(Debug, Clone, PartialEq)]
pub struct EquirectangularMaterial {
    pub uniforms: EquirectangularUniforms,
}

/// Appearance for equirectangular panorama.
#[derive(Debug, Clone, PartialEq)]
pub struct EquirectangularAppearance {
    pub material: EquirectangularMaterial,
}

/// Internal primitive representation for equirectangular panorama.
#[derive(Debug, Clone, PartialEq)]
pub struct EquirectangularPrimitive {
    pub appearance: EquirectangularAppearance,
    pub show: bool,
    is_destroyed: bool,
}

impl EquirectangularPrimitive {
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }
}

/// Initialization options for the `EquirectangularPanorama`.
#[derive(Debug, Clone, Default)]
pub struct EquirectangularPanoramaOptions {
    /// A URL to an image resource, or a preloaded image object. Required.
    pub image: Option<String>,
    /// A 4x4 transformation matrix that defines the panorama's position and orientation.
    pub transform: Option<Matrix4>,
    /// The radius of the panorama in meters. Defaults to 100000.0.
    pub radius: Option<f64>,
    /// The number of times to repeat the texture horizontally. Defaults to 1.0.
    pub repeat_horizontal: Option<f64>,
    /// The number of times to repeat the texture vertically. Defaults to 1.0.
    pub repeat_vertical: Option<f64>,
    /// A credit for the panorama, which is displayed on the canvas.
    pub credit: Option<Credit>,
    /// Determines if the equirectangular panorama will be shown. Defaults to true.
    pub show: Option<bool>,
}

/// A Panorama that displays imagery in equirectangular format in a scene.
#[derive(Debug, Clone, PartialEq)]
pub struct EquirectangularPanorama {
    image: String,
    transform: Matrix4,
    radius: f64,
    repeat_horizontal: f64,
    repeat_vertical: f64,
    credit: Option<Credit>,
    primitive: EquirectangularPrimitive,
    is_destroyed: bool,
}

impl EquirectangularPanorama {
    /// Creates a new `EquirectangularPanorama`.
    pub fn new(options: EquirectangularPanoramaOptions) -> Result<Self, DeveloperError> {
        let image = options
            .image
            .ok_or_else(|| DeveloperError::new("options.image is required."))?;

        let transform = options.transform.unwrap_or(Matrix4::IDENTITY);
        let radius = options.radius.unwrap_or(DEFAULT_RADIUS);
        let repeat_horizontal = options.repeat_horizontal.unwrap_or(1.0);
        let repeat_vertical = options.repeat_vertical.unwrap_or(1.0);
        let show = options.show.unwrap_or(true);

        let repeat = Cartesian2::new(-repeat_horizontal, repeat_vertical);

        let primitive = EquirectangularPrimitive {
            appearance: EquirectangularAppearance {
                material: EquirectangularMaterial {
                    uniforms: EquirectangularUniforms {
                        image: image.clone(),
                        repeat,
                    },
                },
            },
            show,
            is_destroyed: false,
        };

        Ok(Self {
            image,
            transform,
            radius,
            repeat_horizontal,
            repeat_vertical,
            credit: options.credit,
            primitive,
            is_destroyed: false,
        })
    }

    /// Gets the source image of the panorama.
    pub fn image(&self) -> &str {
        &self.image
    }

    /// Gets the transform of the panorama.
    pub fn transform(&self) -> &Matrix4 {
        &self.transform
    }

    /// Gets the radius of the panorama.
    pub fn radius(&self) -> f64 {
        self.radius
    }

    /// Gets horizontal repeat count.
    pub fn repeat_horizontal(&self) -> f64 {
        self.repeat_horizontal
    }

    /// Gets vertical repeat count.
    pub fn repeat_vertical(&self) -> f64 {
        self.repeat_vertical
    }

    /// Gets the credits of the panorama.
    pub fn credit(&self) -> Option<&Credit> {
        self.credit.as_ref()
    }

    /// Determines if the equirectangular panorama will be shown.
    pub fn show(&self) -> bool {
        self.primitive.show
    }

    /// Sets whether the equirectangular panorama will be shown.
    pub fn set_show(&mut self, show: bool) {
        self.primitive.show = show;
    }

    /// Gets the underlying primitive.
    pub fn primitive(&self) -> &EquirectangularPrimitive {
        &self.primitive
    }

    /// Gets the underlying primitive mutably.
    pub fn primitive_mut(&mut self) -> &mut EquirectangularPrimitive {
        &mut self.primitive
    }

    /// Updates the panorama and returns the draw command if ready and visible in 3D.
    pub fn update(
        &mut self,
        is_3d: bool,
        _is_render_pass: bool,
    ) -> Result<Option<DrawCommand>, DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        if !self.show() || !is_3d {
            return Ok(None);
        }

        let cmd = DrawCommand {
            model_matrix: Some(glam::DMat4::from(self.transform)),
            ..Default::default()
        };

        Ok(Some(cmd))
    }

    /// Returns true if this object was destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys resources.
    pub fn destroy(&mut self) {
        self.primitive.destroy();
        self.is_destroyed = true;
    }
}
