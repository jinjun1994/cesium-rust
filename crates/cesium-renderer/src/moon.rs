//! Draws the Moon in 3D matching CesiumJS `Scene/Moon.js` (Apache-2.0).

use cesium_math::{
    CelestialFrameTransforms, Ellipsoid, IauOrientationAxes, JulianDate, Matrix4,
    Simon1994PlanetaryPositions,
};

use crate::ellipsoid_primitive::{EllipsoidPrimitive, EllipsoidPrimitiveOptions};
use crate::material::Material;

/// Options for constructing a [`Moon`].
#[derive(Clone, Debug)]
pub struct MoonOptions {
    pub show: Option<bool>,
    pub texture_url: Option<String>,
    pub ellipsoid: Option<Ellipsoid>,
    pub only_sun_lighting: Option<bool>,
}

impl Default for MoonOptions {
    fn default() -> Self {
        Self {
            show: Some(true),
            texture_url: None,
            ellipsoid: Some(Ellipsoid::MOON),
            only_sun_lighting: Some(true),
        }
    }
}

/// Draws the Moon in 3D.
#[derive(Clone, Debug)]
pub struct Moon {
    pub show: bool,
    pub texture_url: String,
    pub only_sun_lighting: bool,
    ellipsoid: Ellipsoid,
    ellipsoid_primitive: EllipsoidPrimitive,
    axes: IauOrientationAxes,
    is_destroyed: bool,
}

impl Default for Moon {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Moon {
    pub const DEFAULT_TEXTURE_URL: &'static str = "Assets/Textures/moonSmall.jpg";

    pub fn new(options: Option<MoonOptions>) -> Self {
        let options = options.unwrap_or_default();
        let show = options.show.unwrap_or(true);
        let texture_url = options
            .texture_url
            .unwrap_or_else(|| Self::DEFAULT_TEXTURE_URL.to_string());
        let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::MOON);
        let only_sun_lighting = options.only_sun_lighting.unwrap_or(true);

        let material = Material::from_type(Material::IMAGE_TYPE).ok();
        let ellipsoid_primitive = EllipsoidPrimitive::new(EllipsoidPrimitiveOptions {
            radii: Some(ellipsoid.radii().into()),
            material,
            show: Some(show),
            ..Default::default()
        });

        Self {
            show,
            texture_url,
            only_sun_lighting,
            ellipsoid,
            ellipsoid_primitive,
            axes: IauOrientationAxes::default(),
            is_destroyed: false,
        }
    }

    #[inline]
    pub fn ellipsoid(&self) -> Ellipsoid {
        self.ellipsoid
    }

    #[inline]
    pub fn ellipsoid_primitive(&self) -> &EllipsoidPrimitive {
        &self.ellipsoid_primitive
    }

    #[inline]
    pub fn ellipsoid_primitive_mut(&mut self) -> &mut EllipsoidPrimitive {
        &mut self.ellipsoid_primitive
    }

    /// Evaluates and updates the moon's transformation matrix for the given date.
    pub fn compute_model_matrix(&mut self, date: &JulianDate) -> Matrix4 {
        let icrf_to_fixed =
            CelestialFrameTransforms::compute_icrf_to_central_body_fixed_matrix(date);

        let rotation = self.axes.evaluate(date);
        let rotation_transposed = rotation.transpose();
        let combined_rotation = icrf_to_fixed.multiply(&rotation_transposed);

        let moon_position =
            Simon1994PlanetaryPositions::compute_moon_position_in_earth_inertial_frame(Some(date));
        let translation = icrf_to_fixed.multiply_by_vector(moon_position);

        let model_matrix = Matrix4::from_rotation_translation(&combined_rotation, translation);
        self.ellipsoid_primitive.model_matrix = model_matrix;
        self.ellipsoid_primitive.show = self.show;
        model_matrix
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        self.ellipsoid_primitive.destroy();
        self.is_destroyed = true;
    }
}
