//! Geographic rectangles adapted from CesiumJS Rectangle (Apache-2.0).
use crate::{Cartographic, Ellipsoid, MathError, Sphere};
use glam::DVec3;
use std::f64::consts::{FRAC_PI_2, PI, TAU};

/// Longitude and latitude bounds in radians. West > east crosses the antimeridian.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GeoRectangle {
    pub west: f64,
    pub south: f64,
    pub east: f64,
    pub north: f64,
}

fn wrap(angle: f64) -> f64 {
    if (-PI..=PI).contains(&angle) {
        return angle;
    }
    let shifted = (angle + PI).rem_euclid(TAU);
    if shifted.abs() < 1e-14 && (angle + PI).abs() > 1e-14 {
        PI
    } else {
        shifted - PI
    }
}

impl GeoRectangle {
    pub const MAXIMUM: Self = Self {
        west: -PI,
        south: -FRAC_PI_2,
        east: PI,
        north: FRAC_PI_2,
    };
    pub fn radians(west: f64, south: f64, east: f64, north: f64) -> Result<Self, MathError> {
        if ![west, south, east, north].iter().all(|x| x.is_finite())
            || !(-PI..=PI).contains(&west)
            || !(-PI..=PI).contains(&east)
            || !(-FRAC_PI_2..=FRAC_PI_2).contains(&south)
            || !(-FRAC_PI_2..=FRAC_PI_2).contains(&north)
        {
            return Err(MathError);
        }
        Ok(Self {
            west,
            south,
            east,
            north,
        })
    }
    pub fn degrees(west: f64, south: f64, east: f64, north: f64) -> Result<Self, MathError> {
        Self::radians(
            west.to_radians(),
            south.to_radians(),
            east.to_radians(),
            north.to_radians(),
        )
    }
    pub fn width(self) -> f64 {
        self.east - self.west + if self.east < self.west { TAU } else { 0.0 }
    }
    pub fn height(self) -> f64 {
        self.north - self.south
    }
    pub fn center(self) -> Cartographic {
        Cartographic {
            longitude: wrap(self.west + self.width() * 0.5),
            latitude: (self.south + self.north) * 0.5,
            height: 0.0,
        }
    }
    pub fn southwest(self) -> Cartographic {
        Cartographic {
            longitude: self.west,
            latitude: self.south,
            height: 0.0,
        }
    }
    pub fn northwest(self) -> Cartographic {
        Cartographic {
            longitude: self.west,
            latitude: self.north,
            height: 0.0,
        }
    }
    pub fn southeast(self) -> Cartographic {
        Cartographic {
            longitude: self.east,
            latitude: self.south,
            height: 0.0,
        }
    }
    pub fn northeast(self) -> Cartographic {
        Cartographic {
            longitude: self.east,
            latitude: self.north,
            height: 0.0,
        }
    }

    pub fn from_cartographics(points: &[Cartographic]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }
        let (mut west, mut east, mut south, mut north) = (
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
        );
        let (mut west_idl, mut east_idl) = (f64::INFINITY, f64::NEG_INFINITY);
        for p in points {
            if !p.longitude.is_finite() || !p.latitude.is_finite() {
                return None;
            }
            west = west.min(p.longitude);
            east = east.max(p.longitude);
            south = south.min(p.latitude);
            north = north.max(p.latitude);
            let lon = if p.longitude >= 0.0 {
                p.longitude
            } else {
                p.longitude + TAU
            };
            west_idl = west_idl.min(lon);
            east_idl = east_idl.max(lon);
        }
        if east - west > east_idl - west_idl {
            west = if west_idl > PI {
                west_idl - TAU
            } else {
                west_idl
            };
            east = if east_idl > PI {
                east_idl - TAU
            } else {
                east_idl
            };
        }
        Self::radians(west, south, east, north).ok()
    }
    pub fn from_cartesians(points: &[DVec3], ellipsoid: Ellipsoid) -> Option<Self> {
        let positions: Option<Vec<_>> = points
            .iter()
            .map(|&p| ellipsoid.cartesian_to_cartographic(p))
            .collect();
        Self::from_cartographics(&positions?)
    }

    /// Cesium's five-sample rectangle for a bounding sphere, ignoring height.
    pub fn from_bounding_sphere(sphere: Sphere, ellipsoid: Ellipsoid) -> Option<Self> {
        if !sphere.center.is_finite() || !sphere.radius.is_finite() || sphere.radius < 0.0 {
            return None;
        }
        if sphere.center == DVec3::ZERO {
            return Some(Self::MAXIMUM);
        }
        let frame = ellipsoid.east_north_up_to_fixed_frame(sphere.center)?;
        let east = frame.x_axis.truncate().try_normalize()? * sphere.radius;
        let north = frame.y_axis.truncate().try_normalize()? * sphere.radius;
        let center = sphere.center;
        Self::from_cartesians(
            &[
                center + north,
                center - east,
                center - north,
                center + east,
                center,
            ],
            ellipsoid,
        )
    }
    pub fn contains(self, position: Cartographic) -> bool {
        let mut lon = position.longitude;
        let mut east = self.east;
        if east < self.west {
            east += TAU;
            if lon < 0.0 {
                lon += TAU;
            }
        }
        let equal = |a: f64, b: f64| (a - b).abs() <= 1e-14_f64.max(1e-14 * a.abs().max(b.abs()));
        (lon > self.west || equal(lon, self.west))
            && (lon < east || equal(lon, east))
            && position.latitude >= self.south
            && position.latitude <= self.north
    }
    fn aligned(self, other: Self) -> (f64, f64, f64, f64) {
        let (mut e, mut w, mut oe, mut ow) = (self.east, self.west, other.east, other.west);
        if e < w && oe > 0.0 {
            e += TAU;
        } else if oe < ow && e > 0.0 {
            oe += TAU;
        }
        if e < w && ow < 0.0 {
            ow += TAU;
        } else if oe < ow && w < 0.0 {
            w += TAU;
        }
        (w, e, ow, oe)
    }
    pub fn intersection(self, other: Self) -> Option<Self> {
        let (w, e, ow, oe) = self.aligned(other);
        let mut west = wrap(w.max(ow));
        let east = wrap(e.min(oe));
        if west == PI && east < PI {
            west = -PI;
        }
        if (self.west < self.east || other.west < other.east) && east <= west {
            return None;
        }
        let south = self.south.max(other.south);
        let north = self.north.min(other.north);
        if south >= north {
            return None;
        }
        Some(Self {
            west,
            south,
            east,
            north,
        })
    }
    pub fn simple_intersection(self, other: Self) -> Option<Self> {
        let result = Self {
            west: self.west.max(other.west),
            south: self.south.max(other.south),
            east: self.east.min(other.east),
            north: self.north.min(other.north),
        };
        (result.west < result.east && result.south < result.north).then_some(result)
    }
    pub fn union(self, other: Self) -> Self {
        let (w, e, ow, oe) = self.aligned(other);
        Self {
            west: wrap(w.min(ow)),
            south: self.south.min(other.south),
            east: wrap(e.max(oe)),
            north: self.north.max(other.north),
        }
    }
    /// Preserves Cesium's numeric min/max semantics, including for crossing rectangles.
    pub fn expand(self, p: Cartographic) -> Self {
        Self {
            west: self.west.min(p.longitude),
            south: self.south.min(p.latitude),
            east: self.east.max(p.longitude),
            north: self.north.max(p.latitude),
        }
    }
    /// Samples corners and relevant equatorial/axis extrema for bounding volumes.
    pub fn subsample(self, ellipsoid: Ellipsoid, height: f64) -> Option<Vec<DVec3>> {
        if !height.is_finite() {
            return None;
        }
        let mut result = Vec::new();
        for mut corner in [
            self.northwest(),
            self.northeast(),
            self.southeast(),
            self.southwest(),
        ] {
            corner.height = height;
            result.push(ellipsoid.cartographic_to_cartesian(corner)?);
        }
        let latitude = if self.north < 0.0 {
            self.north
        } else if self.south > 0.0 {
            self.south
        } else {
            0.0
        };
        for i in 1..8 {
            let point = Cartographic {
                longitude: -PI + i as f64 * FRAC_PI_2,
                latitude,
                height,
            };
            if self.contains(point) {
                result.push(ellipsoid.cartographic_to_cartesian(point)?);
            }
        }
        if latitude == 0.0 {
            for longitude in [self.west, self.east] {
                result.push(ellipsoid.cartographic_to_cartesian(Cartographic {
                    longitude,
                    latitude,
                    height,
                })?);
            }
        }
        Some(result)
    }
    pub fn subsection(
        self,
        west: f64,
        south: f64,
        east: f64,
        north: f64,
    ) -> Result<Self, MathError> {
        if ![west, south, east, north]
            .iter()
            .all(|x| (0.0..=1.0).contains(x))
            || west > east
            || south > north
        {
            return Err(MathError);
        }
        let lon = |t: f64| {
            if t == 1.0 {
                self.east
            } else if self.west <= self.east {
                self.west + t * self.width()
            } else {
                wrap(self.west + t * self.width())
            }
        };
        let lat = |t: f64| {
            if t == 1.0 {
                self.north
            } else {
                self.south + t * self.height()
            }
        };
        Ok(Self {
            west: lon(west),
            south: lat(south),
            east: lon(east),
            north: lat(north),
        })
    }
}
