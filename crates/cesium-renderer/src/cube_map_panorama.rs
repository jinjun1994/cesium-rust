use crate::commands::DrawCommand;
use crate::cube_map::CubeMapUrls;
use cesium_math::{DeveloperError, Matrix4};

/// Initialization options for the `CubeMapPanorama`.
#[derive(Debug, Clone, Default)]
pub struct CubeMapPanoramaOptions {
    /// The source URLs for each of the six cube map faces.
    pub sources: Option<CubeMapUrls>,
    /// A 4x4 or 3x3 transformation matrix defining the panorama orientation.
    pub transform: Option<Matrix4>,
    /// Determines if this primitive will be shown. Defaults to true.
    pub show: Option<bool>,
    /// A credit string for the panorama.
    pub credit: Option<String>,
}

/// A Panorama that displays imagery in cube map format in a scene.
#[derive(Debug, Clone)]
pub struct CubeMapPanorama {
    /// The sources used to create the cube map faces.
    pub sources: Option<CubeMapUrls>,
    /// The transform defining orientation.
    pub transform: Option<Matrix4>,
    /// Determines if this primitive will be shown.
    pub show: bool,
    /// Credit string.
    pub credit: Option<String>,
    is_destroyed: bool,
}

impl CubeMapPanorama {
    /// Creates a new `CubeMapPanorama`.
    pub fn new(options: CubeMapPanoramaOptions) -> Self {
        Self {
            sources: options.sources,
            transform: options.transform,
            show: options.show.unwrap_or(true),
            credit: options.credit,
            is_destroyed: false,
        }
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

        if !self.show || !is_3d {
            return Ok(None);
        }

        let sources = self
            .sources
            .as_ref()
            .ok_or_else(|| DeveloperError::new("sources must be defined."))?;

        if sources.positive_x.is_none()
            || sources.negative_x.is_none()
            || sources.positive_y.is_none()
            || sources.negative_y.is_none()
            || sources.positive_z.is_none()
            || sources.negative_z.is_none()
        {
            return Err(DeveloperError::new(
                "this.sources must have positiveX, negativeX, positiveY, negativeY, positiveZ, and negativeZ properties.",
            ));
        }

        let cmd = DrawCommand {
            model_matrix: self.transform.map(glam::DMat4::from),
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
        self.is_destroyed = true;
    }
}
