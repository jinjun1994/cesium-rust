use crate::commands::DrawCommand;
use crate::material::Material;
use cesium_math::{BoundingSphere, Cartesian3, DeveloperError, Matrix4};
use glam::DVec3;

/// Options for creating an `EllipsoidPrimitive`.
#[derive(Debug, Clone, Default)]
pub struct EllipsoidPrimitiveOptions {
    /// The center of the ellipsoid in model coordinates. Defaults to Cartesian3::ZERO.
    pub center: Option<Cartesian3>,
    /// The radius along x, y, z in model coordinates.
    pub radii: Option<Cartesian3>,
    /// The 4x4 model matrix. Defaults to Matrix4::IDENTITY.
    pub model_matrix: Option<Matrix4>,
    /// Determines if this primitive will be shown. Defaults to true.
    pub show: Option<bool>,
    /// The surface appearance material. Defaults to Color material.
    pub material: Option<Material>,
    /// User-defined object returned when picked.
    pub id: Option<String>,
    /// Determines if bounding spheres are shown for debugging. Defaults to false.
    pub debug_show_bounding_volume: Option<bool>,
}

/// A renderable ellipsoid or sphere primitive.
#[derive(Debug, Clone)]
pub struct EllipsoidPrimitive {
    /// Center in model coordinates.
    pub center: Cartesian3,
    /// Radii along x, y, z.
    pub radii: Option<Cartesian3>,
    /// 4x4 transformation matrix.
    pub model_matrix: Matrix4,
    /// Determines if this primitive will be shown.
    pub show: bool,
    /// The material of the ellipsoid.
    pub material: Option<Material>,
    /// User-defined ID.
    pub id: Option<String>,
    /// Debug show bounding volume flag.
    pub debug_show_bounding_volume: bool,
    is_destroyed: bool,
}

impl Default for EllipsoidPrimitive {
    fn default() -> Self {
        Self::new(EllipsoidPrimitiveOptions::default())
    }
}

impl EllipsoidPrimitive {
    /// Creates a new `EllipsoidPrimitive`.
    pub fn new(options: EllipsoidPrimitiveOptions) -> Self {
        let material = options.material.or_else(|| Material::from_type(Material::COLOR_TYPE).ok());

        Self {
            center: options.center.unwrap_or(Cartesian3::ZERO),
            radii: options.radii,
            model_matrix: options.model_matrix.unwrap_or(Matrix4::IDENTITY),
            show: options.show.unwrap_or(true),
            material,
            id: options.id,
            debug_show_bounding_volume: options.debug_show_bounding_volume.unwrap_or(false),
            is_destroyed: false,
        }
    }

    /// Computes component-wise 1.0 / (radii * radii).
    pub fn one_over_ellipsoid_radii_squared(&self) -> Option<Cartesian3> {
        self.radii.map(|r| {
            Cartesian3::new(
                1.0 / (r.x * r.x),
                1.0 / (r.y * r.y),
                1.0 / (r.z * r.z),
            )
        })
    }

    /// Computes the bounding sphere of the ellipsoid primitive in world coordinates.
    pub fn bounding_sphere(&self) -> Option<BoundingSphere> {
        let r = self.radii?;
        let max_radius = r.x.max(r.y).max(r.z);
        let center_world = self.model_matrix.multiply_by_point(self.center);

        let dcenter = DVec3::new(center_world.x, center_world.y, center_world.z);
        Some(BoundingSphere::new(dcenter, max_radius))
    }

    /// Updates the primitive and generates draw commands.
    pub fn update(&self) -> Result<Option<DrawCommand>, DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        if !self.show || self.radii.is_none() {
            return Ok(None);
        }

        if self.material.is_none() {
            return Err(DeveloperError::new("material is required."));
        }

        let cmd = DrawCommand {
            model_matrix: Some(glam::DMat4::from(self.model_matrix)),
            ..Default::default()
        };

        Ok(Some(cmd))
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
