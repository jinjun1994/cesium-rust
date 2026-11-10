//! Panorama matching CesiumJS `Scene/Panorama.js` (Apache-2.0).

use cesium_math::{Credit, DeveloperError, Matrix4};

use crate::cube_map_panorama::CubeMapPanorama;
use crate::equirectangular_panorama::EquirectangularPanorama;

/// Trait defining the common interface for all Panorama types.
pub trait PanoramaTrait: Send + Sync {
    /// Determines if the panorama will be shown.
    fn show(&self) -> bool;
    /// Sets whether the panorama will be shown.
    fn set_show(&mut self, show: bool);
    /// Gets the transform of the panorama.
    fn transform(&self) -> Option<&Matrix4>;
    /// Gets the credits of the panorama.
    fn credit(&self) -> Option<&Credit>;
    /// Returns true if destroyed.
    fn is_destroyed(&self) -> bool;
    /// Destroys resources.
    fn destroy(&mut self);
}

/// Abstract panorama type matching CesiumJS `Scene/Panorama.js`.
/// This type describes an interface and cannot be instantiated directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Panorama;

impl Panorama {
    pub const INSTANTIATION_ERROR_MESSAGE: &'static str =
        "This type describes an interface and is not intended to be instantiated directly.";

    /// Constructing `Panorama` directly returns a `DeveloperError` instantiation error.
    pub fn new() -> Result<Self, DeveloperError> {
        Err(DeveloperError::new(Self::INSTANTIATION_ERROR_MESSAGE))
    }

    /// Accessing `show` on uninstantiated prototype throws `DeveloperError`.
    pub fn show(&self) -> Result<bool, DeveloperError> {
        Err(DeveloperError::new(Self::INSTANTIATION_ERROR_MESSAGE))
    }

    /// Setting `show` on uninstantiated prototype throws `DeveloperError`.
    pub fn set_show(&mut self, _show: bool) -> Result<(), DeveloperError> {
        Err(DeveloperError::new(Self::INSTANTIATION_ERROR_MESSAGE))
    }

    /// Accessing `transform` on uninstantiated prototype throws `DeveloperError`.
    pub fn transform(&self) -> Result<&Matrix4, DeveloperError> {
        Err(DeveloperError::new(Self::INSTANTIATION_ERROR_MESSAGE))
    }

    /// Accessing `credit` on uninstantiated prototype throws `DeveloperError`.
    pub fn credit(&self) -> Result<&Credit, DeveloperError> {
        Err(DeveloperError::new(Self::INSTANTIATION_ERROR_MESSAGE))
    }
}

impl PanoramaTrait for CubeMapPanorama {
    fn show(&self) -> bool {
        self.show
    }

    fn set_show(&mut self, show: bool) {
        self.show = show;
    }

    fn transform(&self) -> Option<&Matrix4> {
        self.transform.as_ref()
    }

    fn credit(&self) -> Option<&Credit> {
        None
    }

    fn is_destroyed(&self) -> bool {
        self.is_destroyed()
    }

    fn destroy(&mut self) {
        self.destroy();
    }
}

impl PanoramaTrait for EquirectangularPanorama {
    fn show(&self) -> bool {
        self.show()
    }

    fn set_show(&mut self, show: bool) {
        self.set_show(show);
    }

    fn transform(&self) -> Option<&Matrix4> {
        Some(self.transform())
    }

    fn credit(&self) -> Option<&Credit> {
        self.credit()
    }

    fn is_destroyed(&self) -> bool {
        self.is_destroyed()
    }

    fn destroy(&mut self) {
        self.destroy();
    }
}
