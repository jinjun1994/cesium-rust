//! Bounding sphere operations adapted from CesiumJS BoundingSphere (Apache-2.0).
use crate::{Cartographic, Ellipsoid, GeoRectangle, MathError, Sphere};
use glam::{DMat3, DMat4, DVec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaneIntersection {
    Outside,
    Intersecting,
    Inside,
}

impl PlaneIntersection {
    pub const OUTSIDE: Self = Self::Outside;
    pub const INTERSECTING: Self = Self::Intersecting;
    pub const INSIDE: Self = Self::Inside;
}

impl Sphere {
    /// Projects the eight corners of a tangent-frame cube around this sphere,
    /// then swizzles its projected center to `(height, x, y)` as Cesium does.
    /// At exact poles, an east-axis fallback avoids a zero-vector normalization.
    pub fn project_to_2d(
        self,
        ellipsoid: Ellipsoid,
        project: impl Fn(Cartographic) -> Option<DVec3>,
    ) -> Result<Self, MathError> {
        if !self.center.is_finite() || !self.radius.is_finite() || self.radius < 0.0 {
            return Err(MathError);
        }
        let normal = if self.center == DVec3::ZERO {
            DVec3::X
        } else {
            ellipsoid
                .geodetic_surface_normal(self.center)
                .ok_or(MathError)?
        };
        let east = DVec3::Z.cross(normal).try_normalize().unwrap_or(DVec3::Y);
        let north = normal.cross(east).try_normalize().ok_or(MathError)?;
        let (normal, east, north) = (
            normal * self.radius,
            east * self.radius,
            north * self.radius,
        );
        let mut projected = Vec::with_capacity(8);
        for direction in [1.0, -1.0] {
            for (ns, es) in [(1.0, 1.0), (1.0, -1.0), (-1.0, -1.0), (-1.0, 1.0)] {
                let point = self.center + normal * direction + north * ns + east * es;
                let cartographic = ellipsoid
                    .cartesian_to_cartographic(point)
                    .ok_or(MathError)?;
                projected.push(
                    project(cartographic)
                        .filter(|p| p.is_finite())
                        .ok_or(MathError)?,
                );
            }
        }
        let mut result = Self::from_points(&projected)?;
        result.center = DVec3::new(result.center.z, result.center.x, result.center.y);
        Ok(result)
    }
    /// Creates a sphere from the extrema sampled on a geographic rectangle.
    /// As in Cesium, a missing rectangle gives the zero sphere. Sampling is
    /// approximate and is not guaranteed to be tight for every ellipsoid.
    pub fn from_rectangle_3d(
        rectangle: Option<GeoRectangle>,
        ellipsoid: Ellipsoid,
        height: f64,
    ) -> Result<Self, MathError> {
        match rectangle {
            Some(r) => Self::from_points(&r.subsample(ellipsoid, height).ok_or(MathError)?),
            None => Self::from_points(&[]),
        }
    }

    /// Creates a sphere from the projected southwest/min-height and
    /// northeast/max-height corners, matching Cesium's 2D rectangle behavior.
    pub fn from_rectangle_with_heights_2d(
        rectangle: Option<GeoRectangle>,
        minimum_height: f64,
        maximum_height: f64,
        project: impl Fn(Cartographic) -> Option<DVec3>,
    ) -> Result<Self, MathError> {
        let Some(rectangle) = rectangle else {
            return Self::from_points(&[]);
        };
        if !minimum_height.is_finite() || !maximum_height.is_finite() {
            return Err(MathError);
        }
        let mut southwest = rectangle.southwest();
        southwest.height = minimum_height;
        let mut northeast = rectangle.northeast();
        northeast.height = maximum_height;
        let lower = project(southwest)
            .filter(|p| p.is_finite())
            .ok_or(MathError)?;
        let upper = project(northeast)
            .filter(|p| p.is_finite())
            .ok_or(MathError)?;
        let result = Self::from_corner_points(lower, upper);
        if !result.center.is_finite() || !result.radius.is_finite() {
            return Err(MathError);
        }
        Ok(result)
    }

    /// Bounding sphere of a box represented by three half-axis columns.
    /// Checking all diagonal signs also encloses boxes with sheared axes.
    pub fn from_oriented_box(center: DVec3, half_axes: DMat3) -> Result<Self, MathError> {
        if !center.is_finite() || !half_axes.is_finite() {
            return Err(MathError);
        }
        let (x, y, z) = (half_axes.x_axis, half_axes.y_axis, half_axes.z_axis);
        let radius = [
            (x + y + z).length(),
            (x + y - z).length(),
            (x - y + z).length(),
            (x - y - z).length(),
        ]
        .into_iter()
        .fold(0.0, f64::max);
        if !radius.is_finite() {
            return Err(MathError);
        }
        Ok(Self { center, radius })
    }

    /// Sphere enclosing the transformed unit cube, including sheared transforms.
    /// For orthogonal columns this equals Cesium's column-length formula.
    pub fn from_transformation(matrix: DMat4) -> Result<Self, MathError> {
        if !matrix.is_finite() {
            return Err(MathError);
        }
        let axes = DMat3::from_cols(
            matrix.x_axis.truncate() * 0.5,
            matrix.y_axis.truncate() * 0.5,
            matrix.z_axis.truncate() * 0.5,
        );
        Self::from_oriented_box(matrix.w_axis.truncate(), axes)
    }

    /// Ritter and axis-aligned-box candidate spheres; returns the tighter one.
    /// The empty input produces the zero sphere. Non-finite inputs are rejected.
    pub fn from_points(points: &[DVec3]) -> Result<Self, MathError> {
        if points.iter().any(|p| !p.is_finite()) {
            return Err(MathError);
        }
        let Some(&first) = points.first() else {
            return Ok(Self {
                center: DVec3::ZERO,
                radius: 0.0,
            });
        };
        let (mut minima, mut maxima) = ([first; 3], [first; 3]);
        for &p in &points[1..] {
            for axis in 0..3 {
                if p[axis] < minima[axis][axis] {
                    minima[axis] = p;
                }
                if p[axis] > maxima[axis][axis] {
                    maxima[axis] = p;
                }
            }
        }
        let mut axis = 0;
        let mut max_span = maxima[0].distance_squared(minima[0]);
        for candidate in 1..3 {
            let span = maxima[candidate].distance_squared(minima[candidate]);
            if span > max_span {
                max_span = span;
                axis = candidate;
            }
        }
        let mut ritter_center = (minima[axis] + maxima[axis]) * 0.5;
        let mut ritter_radius = maxima[axis].distance(ritter_center);
        let mut radius_squared = ritter_radius * ritter_radius;
        let min_box = DVec3::new(minima[0].x, minima[1].y, minima[2].z);
        let max_box = DVec3::new(maxima[0].x, maxima[1].y, maxima[2].z);
        let naive_center = (min_box + max_box) * 0.5;
        let mut naive_radius: f64 = 0.0;
        for &p in points {
            naive_radius = naive_radius.max(p.distance(naive_center));
            let dist_sq = p.distance_squared(ritter_center);
            if dist_sq > radius_squared {
                let dist = dist_sq.sqrt();
                ritter_radius = (ritter_radius + dist) * 0.5;
                radius_squared = ritter_radius * ritter_radius;
                let old_to_new = dist - ritter_radius;
                ritter_center = (ritter_radius * ritter_center + old_to_new * p) / dist;
            }
        }
        let result = if ritter_radius < naive_radius {
            Self {
                center: ritter_center,
                radius: ritter_radius,
            }
        } else {
            Self {
                center: naive_center,
                radius: naive_radius,
            }
        };
        if !result.center.is_finite() || !result.radius.is_finite() {
            return Err(MathError);
        }
        Ok(result)
    }

    /// Constructs from interleaved xyz values with a center offset.
    pub fn from_vertices(values: &[f64], center: DVec3, stride: usize) -> Result<Self, MathError> {
        if stride < 3
            || !center.is_finite()
            || (!values.len().is_multiple_of(stride) && values.len() % stride < 3)
        {
            return Err(MathError);
        }
        let points: Vec<_> = values
            .chunks(stride)
            .map(|v| center + DVec3::new(v[0], v[1], v[2]))
            .collect();
        Self::from_points(&points)
    }
    pub fn from_encoded_vertices(high: &[f32], low: &[f32]) -> Result<Self, MathError> {
        if high.len() != low.len() || !high.len().is_multiple_of(3) {
            return Err(MathError);
        }
        let points: Vec<_> = high
            .as_chunks::<3>()
            .0
            .iter()
            .zip(low.as_chunks::<3>().0.iter())
            .map(|(h, l)| {
                DVec3::new(
                    h[0] as f64 + l[0] as f64,
                    h[1] as f64 + l[1] as f64,
                    h[2] as f64 + l[2] as f64,
                )
            })
            .collect();
        Self::from_points(&points)
    }
    pub fn from_corner_points(corner: DVec3, opposite: DVec3) -> Self {
        let center = (corner + opposite) * 0.5;
        Self {
            center,
            radius: center.distance(opposite),
        }
    }
    pub fn from_ellipsoid(ellipsoid: Ellipsoid) -> Self {
        Self {
            center: DVec3::ZERO,
            radius: ellipsoid.maximum_radius(),
        }
    }
    pub fn union(self, other: Self) -> Self {
        let delta = other.center - self.center;
        let distance = delta.length();
        if self.radius >= distance + other.radius {
            return self;
        }
        if other.radius >= distance + self.radius {
            return other;
        }
        let radius = (self.radius + distance + other.radius) * 0.5;
        Self {
            center: self.center + delta * ((radius - self.radius) / distance),
            radius,
        }
    }
    /// Expands the radius while preserving the center, matching Cesium.
    pub fn expand(self, point: DVec3) -> Self {
        Self {
            center: self.center,
            radius: self.radius.max(self.center.distance(point)),
        }
    }
    pub fn intersect_plane(self, normal: DVec3, distance: f64) -> PlaneIntersection {
        let d = normal.dot(self.center) + distance;
        if d < -self.radius {
            PlaneIntersection::Outside
        } else if d < self.radius {
            PlaneIntersection::Intersecting
        } else {
            PlaneIntersection::Inside
        }
    }
    pub fn from_bounding_spheres(spheres: &[Self]) -> Result<Self, MathError> {
        if spheres
            .iter()
            .any(|s| !s.center.is_finite() || !s.radius.is_finite() || s.radius < 0.0)
        {
            return Err(MathError);
        }
        match spheres {
            [] => Self::from_points(&[]),
            [only] => Ok(*only),
            [a, b] => Ok(a.union(*b)),
            _ => {
                let centers: Vec<_> = spheres.iter().map(|s| s.center).collect();
                let mut result = Self::from_points(&centers)?;
                for s in spheres {
                    result.radius = result
                        .radius
                        .max(result.center.distance(s.center) + s.radius);
                }
                Ok(result)
            }
        }
    }

    pub const PACKED_LENGTH: usize = 4;

    pub fn pack(&self, output: &mut [f64], offset: usize) -> Result<(), MathError> {
        let end = offset.checked_add(Self::PACKED_LENGTH).ok_or(MathError)?;
        let target = output.get_mut(offset..end).ok_or(MathError)?;
        target.copy_from_slice(&[self.center.x, self.center.y, self.center.z, self.radius]);
        Ok(())
    }

    pub fn unpack(input: &[f64], offset: usize) -> Result<Self, MathError> {
        let end = offset.checked_add(Self::PACKED_LENGTH).ok_or(MathError)?;
        let values = input.get(offset..end).ok_or(MathError)?;
        Ok(Self {
            center: DVec3::new(values[0], values[1], values[2]),
            radius: values[3],
        })
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        match other {
            Some(o) => self.center == o.center && self.radius == o.radius,
            None => false,
        }
    }

    pub fn transform(&self, matrix: DMat4) -> Result<Self, MathError> {
        let center = matrix.transform_point3(self.center);
        if !center.is_finite() {
            return Err(MathError);
        }
        let col0 = matrix.col(0).truncate().length();
        let col1 = matrix.col(1).truncate().length();
        let col2 = matrix.col(2).truncate().length();
        let max_scale = col0.max(col1).max(col2);
        let radius = max_scale * self.radius;
        if !radius.is_finite() {
            return Err(MathError);
        }
        Ok(Self { center, radius })
    }

    pub fn transform_without_scale(&self, matrix: DMat4) -> Result<Self, MathError> {
        let center = matrix.transform_point3(self.center);
        if !center.is_finite() {
            return Err(MathError);
        }
        Ok(Self {
            center,
            radius: self.radius,
        })
    }

    pub fn volume(&self) -> f64 {
        (4.0 / 3.0) * std::f64::consts::PI * self.radius * self.radius * self.radius
    }

    pub fn from_oriented_bounding_box(obb: &crate::OrientedBoundingBox) -> Result<Self, MathError> {
        Self::from_oriented_box(obb.center, obb.half_axes)
    }

    pub fn intersect_plane_struct(&self, plane: &crate::Plane) -> PlaneIntersection {
        self.intersect_plane(plane.normal, plane.distance)
    }

    pub fn is_occluded(&self, occluder: &crate::Occluder) -> bool {
        !occluder.is_sphere_visible(*self)
    }
}
