//! SkyBox matching CesiumJS `Scene/SkyBox.js` (Apache-2.0).

use cesium_math::DeveloperError;

use crate::commands::DrawCommand;
use crate::cube_map::CubeMapUrls;
use crate::cube_map_panorama::{CubeMapPanorama, CubeMapPanoramaOptions};

/// A single face source for a sky box (either image URL or raw image buffer).
#[derive(Clone, Debug, PartialEq)]
pub enum SkyBoxFaceSource {
    Url(String),
    Image(Vec<u8>),
}

impl SkyBoxFaceSource {
    pub fn is_url(&self) -> bool {
        matches!(self, Self::Url(_))
    }

    pub fn is_image(&self) -> bool {
        matches!(self, Self::Image(_))
    }

    pub fn as_url(&self) -> Option<&str> {
        match self {
            Self::Url(u) => Some(u),
            Self::Image(_) => None,
        }
    }
}

impl From<String> for SkyBoxFaceSource {
    fn from(s: String) -> Self {
        Self::Url(s)
    }
}

impl From<&str> for SkyBoxFaceSource {
    fn from(s: &str) -> Self {
        Self::Url(s.to_string())
    }
}

impl From<Vec<u8>> for SkyBoxFaceSource {
    fn from(bytes: Vec<u8>) -> Self {
        Self::Image(bytes)
    }
}

/// Six face sources for a cube map sky box.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkyBoxSources {
    pub positive_x: Option<SkyBoxFaceSource>,
    pub negative_x: Option<SkyBoxFaceSource>,
    pub positive_y: Option<SkyBoxFaceSource>,
    pub negative_y: Option<SkyBoxFaceSource>,
    pub positive_z: Option<SkyBoxFaceSource>,
    pub negative_z: Option<SkyBoxFaceSource>,
}

impl SkyBoxSources {
    /// Validates that all six faces are specified and of identical type.
    pub fn validate(&self) -> Result<(), DeveloperError> {
        let faces = [
            ("positiveX", self.positive_x.as_ref()),
            ("negativeX", self.negative_x.as_ref()),
            ("positiveY", self.positive_y.as_ref()),
            ("negativeY", self.negative_y.as_ref()),
            ("positiveZ", self.positive_z.as_ref()),
            ("negativeZ", self.negative_z.as_ref()),
        ];

        for (name, face) in faces {
            if face.is_none() {
                return Err(DeveloperError::new(format!(
                    "this.sources is required and must have positiveX, negativeX, positiveY, negativeY, positiveZ, and negativeZ properties. Missing {name}."
                )));
            }
        }

        let first = faces[0].1.unwrap();
        let is_first_url = first.is_url();

        for (name, face) in &faces[1..] {
            let f = face.unwrap();
            if f.is_url() != is_first_url {
                return Err(DeveloperError::new(format!(
                    "this.sources properties must all be the same type. {name} has mismatched type."
                )));
            }
        }

        Ok(())
    }

    /// Converts to `CubeMapUrls` if all faces are URLs.
    pub fn to_cube_map_urls(&self) -> Option<CubeMapUrls> {
        Some(CubeMapUrls {
            positive_x: self.positive_x.as_ref().and_then(|f| f.as_url().map(String::from)),
            negative_x: self.negative_x.as_ref().and_then(|f| f.as_url().map(String::from)),
            positive_y: self.positive_y.as_ref().and_then(|f| f.as_url().map(String::from)),
            negative_y: self.negative_y.as_ref().and_then(|f| f.as_url().map(String::from)),
            positive_z: self.positive_z.as_ref().and_then(|f| f.as_url().map(String::from)),
            negative_z: self.negative_z.as_ref().and_then(|f| f.as_url().map(String::from)),
        })
    }
}

/// Construction options for `SkyBox`.
#[derive(Clone, Debug, Default)]
pub struct SkyBoxOptions {
    pub sources: Option<SkyBoxSources>,
    pub show: Option<bool>,
}

/// A sky box around the scene to draw stars or environment cube maps.
#[derive(Clone, Debug)]
pub struct SkyBox {
    pub sources: Option<SkyBoxSources>,
    pub show: bool,
    panorama: CubeMapPanorama,
    is_destroyed: bool,
}

impl SkyBox {
    /// Creates a new `SkyBox`.
    pub fn new(options: SkyBoxOptions) -> Self {
        let show = options.show.unwrap_or(true);
        let cube_urls = options.sources.as_ref().and_then(|s| s.to_cube_map_urls());

        let panorama = CubeMapPanorama::new(CubeMapPanoramaOptions {
            sources: cube_urls,
            transform: None,
            show: Some(show),
            credit: None,
        });

        Self {
            sources: options.sources,
            show,
            panorama,
            is_destroyed: false,
        }
    }

    /// Updates the sky box and returns the draw command if ready and visible in 3D.
    pub fn update(
        &mut self,
        is_3d: bool,
        is_render_pass: bool,
    ) -> Result<Option<DrawCommand>, DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        let sources = self.sources.as_ref().ok_or_else(|| {
            DeveloperError::new(
                "this.sources is required and must have positiveX, negativeX, positiveY, negativeY, positiveZ, and negativeZ properties.",
            )
        })?;

        sources.validate()?;

        if !self.show || !is_3d || !is_render_pass {
            return Ok(None);
        }

        if let Some(urls) = sources.to_cube_map_urls() {
            if urls.positive_x.is_some() {
                self.panorama.show = self.show;
                self.panorama.sources = Some(urls);
                return self.panorama.update(is_3d, is_render_pass);
            }
        }

        // When sources are raw images/bitmaps, directly produce draw command
        let cmd = DrawCommand {
            model_matrix: None,
            ..Default::default()
        };
        Ok(Some(cmd))
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        self.panorama.destroy();
        self.is_destroyed = true;
    }
}
