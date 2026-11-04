//! Geographic projection adapted from CesiumJS GeographicProjection (Apache-2.0).
use crate::{Cartographic, Ellipsoid, MathError};
use glam::DVec3;

/// Common map-projection operations used by tiling schemes.
pub trait MapProjection {
    fn ellipsoid(&self) -> Ellipsoid;
    fn project(&self, cartographic: Cartographic) -> Option<DVec3>;
    fn unproject(&self, position: DVec3) -> Option<Cartographic>;
}

/// Equirectangular projection with the ellipsoid's maximum radius as scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeographicProjection {
    ellipsoid: Ellipsoid,
}

impl Default for GeographicProjection {
    fn default() -> Self {
        Self {
            ellipsoid: Ellipsoid::WGS84,
        }
    }
}

impl GeographicProjection {
    pub fn new(ellipsoid: Ellipsoid) -> Result<Self, MathError> {
        if ellipsoid.maximum_radius() <= 0.0 {
            return Err(MathError);
        }
        Ok(Self { ellipsoid })
    }
    pub fn ellipsoid(self) -> Ellipsoid {
        self.ellipsoid
    }
    pub fn project(self, cartographic: Cartographic) -> Option<DVec3> {
        let scale = self.ellipsoid.maximum_radius();
        let result = DVec3::new(
            cartographic.longitude * scale,
            cartographic.latitude * scale,
            cartographic.height,
        );
        result.is_finite().then_some(result)
    }
    pub fn unproject(self, position: DVec3) -> Option<Cartographic> {
        if !position.is_finite() {
            return None;
        }
        // Cesium's projection permits positions outside geographic ranges.
        let scale = self.ellipsoid.maximum_radius();
        Some(Cartographic {
            longitude: position.x / scale,
            latitude: position.y / scale,
            height: position.z,
        })
    }
}

/// Spherical Mercator projection, with latitude limited to the square world.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WebMercatorProjection {
    ellipsoid: Ellipsoid,
}

impl Default for WebMercatorProjection {
    fn default() -> Self {
        Self {
            ellipsoid: Ellipsoid::WGS84,
        }
    }
}
impl WebMercatorProjection {
    /// atan(sinh(π)), approximately 85.05112878°.
    pub const MAXIMUM_LATITUDE: f64 = 1.484_422_229_745_332_2;
    pub fn new(ellipsoid: Ellipsoid) -> Result<Self, MathError> {
        if ellipsoid.maximum_radius() <= 0.0 {
            return Err(MathError);
        }
        Ok(Self { ellipsoid })
    }
    pub fn ellipsoid(self) -> Ellipsoid {
        self.ellipsoid
    }
    pub fn mercator_angle_to_geodetic_latitude(angle: f64) -> f64 {
        std::f64::consts::FRAC_PI_2 - 2.0 * (-angle).exp().atan()
    }
    pub fn geodetic_latitude_to_mercator_angle(latitude: f64) -> f64 {
        let sin = latitude
            .clamp(-Self::MAXIMUM_LATITUDE, Self::MAXIMUM_LATITUDE)
            .sin();
        0.5 * ((1.0 + sin) / (1.0 - sin)).ln()
    }
    pub fn project(self, c: Cartographic) -> Option<DVec3> {
        if !c.longitude.is_finite() || !c.latitude.is_finite() || !c.height.is_finite() {
            return None;
        }
        let scale = self.ellipsoid.maximum_radius();
        let point = DVec3::new(
            c.longitude * scale,
            Self::geodetic_latitude_to_mercator_angle(c.latitude) * scale,
            c.height,
        );
        point.is_finite().then_some(point)
    }
    pub fn unproject(self, p: DVec3) -> Option<Cartographic> {
        if !p.is_finite() {
            return None;
        }
        let inverse = 1.0 / self.ellipsoid.maximum_radius();
        Some(Cartographic {
            longitude: p.x * inverse,
            latitude: Self::mercator_angle_to_geodetic_latitude(p.y * inverse),
            height: p.z,
        })
    }
}

impl MapProjection for GeographicProjection {
    fn ellipsoid(&self) -> Ellipsoid {
        (*self).ellipsoid()
    }
    fn project(&self, c: Cartographic) -> Option<DVec3> {
        (*self).project(c)
    }
    fn unproject(&self, p: DVec3) -> Option<Cartographic> {
        (*self).unproject(p)
    }
}

impl MapProjection for WebMercatorProjection {
    fn ellipsoid(&self) -> Ellipsoid {
        (*self).ellipsoid()
    }
    fn project(&self, c: Cartographic) -> Option<DVec3> {
        (*self).project(c)
    }
    fn unproject(&self, p: DVec3) -> Option<Cartographic> {
        (*self).unproject(p)
    }
}
