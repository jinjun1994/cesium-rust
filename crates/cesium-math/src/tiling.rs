//! Geographic and Web Mercator tiling adapted from CesiumJS (Apache-2.0).
use crate::{
    Cartographic, Ellipsoid, GeoRectangle, GeographicProjection, MapProjection, MathError,
    WebMercatorProjection,
};
use glam::{DVec2, DVec3};
use std::f64::consts::{PI, TAU};

/// Bounds in projection-native units (degrees or metres, depending on scheme).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeRectangle {
    pub west: f64,
    pub south: f64,
    pub east: f64,
    pub north: f64,
}

/// Operations common to geographic and Web Mercator tiling schemes.
pub trait TilingScheme {
    type Projection: MapProjection;
    fn ellipsoid(&self) -> Ellipsoid;
    fn rectangle(&self) -> GeoRectangle;
    fn projection(&self) -> Self::Projection;
    fn tile_counts(&self, level: u32) -> Option<(u32, u32)>;
    fn rectangle_to_native(&self, rectangle: GeoRectangle) -> Option<NativeRectangle>;
    fn tile_native_rectangle(&self, x: u32, y: u32, level: u32) -> Option<NativeRectangle>;
    fn tile_rectangle(&self, x: u32, y: u32, level: u32) -> Option<GeoRectangle>;
    fn position_to_tile(&self, position: Cartographic, level: u32) -> Option<(u32, u32)>;
}

fn counts(x: u32, y: u32, level: u32) -> Option<(u32, u32)> {
    let factor = 1_u32.checked_shl(level)?;
    Some((x.checked_mul(factor)?, y.checked_mul(factor)?))
}
fn normalized_longitude(lon: f64) -> f64 {
    if (-PI..=PI).contains(&lon) {
        lon
    } else {
        (lon + PI).rem_euclid(TAU) - PI
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GeographicTilingScheme {
    ellipsoid: Ellipsoid,
    rectangle: GeoRectangle,
    root_x: u32,
    root_y: u32,
}
impl Default for GeographicTilingScheme {
    fn default() -> Self {
        Self {
            ellipsoid: Ellipsoid::WGS84,
            rectangle: GeoRectangle::MAXIMUM,
            root_x: 2,
            root_y: 1,
        }
    }
}
impl GeographicTilingScheme {
    pub fn new(
        ellipsoid: Ellipsoid,
        rectangle: GeoRectangle,
        root_x: u32,
        root_y: u32,
    ) -> Result<Self, MathError> {
        if root_x == 0
            || root_y == 0
            || ellipsoid.maximum_radius() <= 0.0
            || rectangle.width() <= 0.0
            || rectangle.height() <= 0.0
            || GeoRectangle::radians(
                rectangle.west,
                rectangle.south,
                rectangle.east,
                rectangle.north,
            )
            .is_err()
        {
            return Err(MathError);
        }
        Ok(Self {
            ellipsoid,
            rectangle,
            root_x,
            root_y,
        })
    }
    pub fn ellipsoid(self) -> Ellipsoid {
        self.ellipsoid
    }
    pub fn rectangle(self) -> GeoRectangle {
        self.rectangle
    }
    pub fn projection(self) -> GeographicProjection {
        GeographicProjection::new(self.ellipsoid).expect("tiling scheme has a valid ellipsoid")
    }
    pub fn tile_counts(self, level: u32) -> Option<(u32, u32)> {
        counts(self.root_x, self.root_y, level)
    }
    pub fn rectangle_to_native(self, r: GeoRectangle) -> NativeRectangle {
        NativeRectangle {
            west: r.west.to_degrees(),
            south: r.south.to_degrees(),
            east: r.east.to_degrees(),
            north: r.north.to_degrees(),
        }
    }
    pub fn tile_rectangle(self, x: u32, y: u32, level: u32) -> Option<GeoRectangle> {
        let (nx, ny) = self.tile_counts(level)?;
        if x >= nx || y >= ny {
            return None;
        }
        let width = self.rectangle.width() / nx as f64;
        let height = self.rectangle.height() / ny as f64;
        let w = self.rectangle.west + x as f64 * width;
        let e = self.rectangle.west + (x as f64 + 1.0) * width;
        Some(GeoRectangle {
            west: normalized_longitude(w),
            south: self.rectangle.north - (y as f64 + 1.0) * height,
            east: normalized_longitude(e),
            north: self.rectangle.north - y as f64 * height,
        })
    }
    pub fn tile_native_rectangle(self, x: u32, y: u32, level: u32) -> Option<NativeRectangle> {
        let r = self.tile_rectangle(x, y, level)?;
        Some(NativeRectangle {
            west: r.west.to_degrees(),
            south: r.south.to_degrees(),
            east: r.east.to_degrees(),
            north: r.north.to_degrees(),
        })
    }
    pub fn position_to_tile(self, p: Cartographic, level: u32) -> Option<(u32, u32)> {
        if !self.rectangle.contains(p) {
            return None;
        }
        let (nx, ny) = self.tile_counts(level)?;
        let lon = if self.rectangle.east < self.rectangle.west && p.longitude < self.rectangle.west
        {
            p.longitude + TAU
        } else {
            p.longitude
        };
        let x = (((lon - self.rectangle.west) / self.rectangle.width()) * nx as f64)
            .floor()
            .max(0.0) as u32;
        let y = (((self.rectangle.north - p.latitude) / self.rectangle.height()) * ny as f64)
            .floor()
            .max(0.0) as u32;
        Some((x.min(nx - 1), y.min(ny - 1)))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WebMercatorTilingScheme {
    projection: WebMercatorProjection,
    rectangle: GeoRectangle,
    southwest: DVec2,
    northeast: DVec2,
    root_x: u32,
    root_y: u32,
}
impl Default for WebMercatorTilingScheme {
    fn default() -> Self {
        Self::new(Ellipsoid::WGS84, None, 1, 1).expect("valid defaults")
    }
}
impl WebMercatorTilingScheme {
    /// Custom bounds are in projected metres, southwest then northeast.
    pub fn new(
        ellipsoid: Ellipsoid,
        bounds: Option<(DVec2, DVec2)>,
        root_x: u32,
        root_y: u32,
    ) -> Result<Self, MathError> {
        if root_x == 0 || root_y == 0 {
            return Err(MathError);
        }
        let projection = WebMercatorProjection::new(ellipsoid)?;
        let extent = ellipsoid.maximum_radius() * PI;
        let (southwest, northeast) =
            bounds.unwrap_or((DVec2::splat(-extent), DVec2::splat(extent)));
        if !southwest.is_finite()
            || !northeast.is_finite()
            || southwest.x >= northeast.x
            || southwest.y >= northeast.y
        {
            return Err(MathError);
        }
        let sw = projection
            .unproject(southwest.extend(0.0))
            .ok_or(MathError)?;
        let ne = projection
            .unproject(northeast.extend(0.0))
            .ok_or(MathError)?;
        let rectangle =
            GeoRectangle::radians(sw.longitude, sw.latitude, ne.longitude, ne.latitude)?;
        Ok(Self {
            projection,
            rectangle,
            southwest,
            northeast,
            root_x,
            root_y,
        })
    }
    pub fn ellipsoid(self) -> Ellipsoid {
        self.projection.ellipsoid()
    }
    pub fn rectangle(self) -> GeoRectangle {
        self.rectangle
    }
    pub fn projection(self) -> WebMercatorProjection {
        self.projection
    }
    pub fn tile_counts(self, level: u32) -> Option<(u32, u32)> {
        counts(self.root_x, self.root_y, level)
    }
    pub fn rectangle_to_native(self, r: GeoRectangle) -> Option<NativeRectangle> {
        let sw = self.projection.project(r.southwest())?;
        let ne = self.projection.project(r.northeast())?;
        Some(NativeRectangle {
            west: sw.x,
            south: sw.y,
            east: ne.x,
            north: ne.y,
        })
    }
    pub fn tile_native_rectangle(self, x: u32, y: u32, level: u32) -> Option<NativeRectangle> {
        let (nx, ny) = self.tile_counts(level)?;
        if x >= nx || y >= ny {
            return None;
        }
        let size = (self.northeast - self.southwest) / DVec2::new(nx as f64, ny as f64);
        Some(NativeRectangle {
            west: self.southwest.x + x as f64 * size.x,
            east: self.southwest.x + (x as f64 + 1.0) * size.x,
            north: self.northeast.y - y as f64 * size.y,
            south: self.northeast.y - (y as f64 + 1.0) * size.y,
        })
    }
    pub fn tile_rectangle(self, x: u32, y: u32, level: u32) -> Option<GeoRectangle> {
        let r = self.tile_native_rectangle(x, y, level)?;
        let sw = self
            .projection
            .unproject(DVec3::new(r.west, r.south, 0.0))?;
        let ne = self
            .projection
            .unproject(DVec3::new(r.east, r.north, 0.0))?;
        Some(GeoRectangle {
            west: sw.longitude,
            south: sw.latitude,
            east: ne.longitude,
            north: ne.latitude,
        })
    }
    pub fn position_to_tile(self, p: Cartographic, level: u32) -> Option<(u32, u32)> {
        if !self.rectangle.contains(p) {
            return None;
        }
        let (nx, ny) = self.tile_counts(level)?;
        let q = self.projection.project(p)?;
        let x = (((q.x - self.southwest.x) / (self.northeast.x - self.southwest.x)) * nx as f64)
            .floor()
            .max(0.0) as u32;
        let y = (((self.northeast.y - q.y) / (self.northeast.y - self.southwest.y)) * ny as f64)
            .floor()
            .max(0.0) as u32;
        Some((x.min(nx - 1), y.min(ny - 1)))
    }
}

impl TilingScheme for GeographicTilingScheme {
    type Projection = GeographicProjection;
    fn ellipsoid(&self) -> Ellipsoid {
        (*self).ellipsoid()
    }
    fn rectangle(&self) -> GeoRectangle {
        (*self).rectangle()
    }
    fn projection(&self) -> Self::Projection {
        (*self).projection()
    }
    fn tile_counts(&self, level: u32) -> Option<(u32, u32)> {
        (*self).tile_counts(level)
    }
    fn rectangle_to_native(&self, r: GeoRectangle) -> Option<NativeRectangle> {
        Some((*self).rectangle_to_native(r))
    }
    fn tile_native_rectangle(&self, x: u32, y: u32, level: u32) -> Option<NativeRectangle> {
        (*self).tile_native_rectangle(x, y, level)
    }
    fn tile_rectangle(&self, x: u32, y: u32, level: u32) -> Option<GeoRectangle> {
        (*self).tile_rectangle(x, y, level)
    }
    fn position_to_tile(&self, p: Cartographic, level: u32) -> Option<(u32, u32)> {
        (*self).position_to_tile(p, level)
    }
}

impl TilingScheme for WebMercatorTilingScheme {
    type Projection = WebMercatorProjection;
    fn ellipsoid(&self) -> Ellipsoid {
        (*self).ellipsoid()
    }
    fn rectangle(&self) -> GeoRectangle {
        (*self).rectangle()
    }
    fn projection(&self) -> Self::Projection {
        (*self).projection()
    }
    fn tile_counts(&self, level: u32) -> Option<(u32, u32)> {
        (*self).tile_counts(level)
    }
    fn rectangle_to_native(&self, r: GeoRectangle) -> Option<NativeRectangle> {
        (*self).rectangle_to_native(r)
    }
    fn tile_native_rectangle(&self, x: u32, y: u32, level: u32) -> Option<NativeRectangle> {
        (*self).tile_native_rectangle(x, y, level)
    }
    fn tile_rectangle(&self, x: u32, y: u32, level: u32) -> Option<GeoRectangle> {
        (*self).tile_rectangle(x, y, level)
    }
    fn position_to_tile(&self, p: Cartographic, level: u32) -> Option<(u32, u32)> {
        (*self).position_to_tile(p, level)
    }
}
