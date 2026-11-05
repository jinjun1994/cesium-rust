//! Oriented bounding boxes adapted from CesiumJS (Apache-2.0).
use crate::{
    BoundingVolume, Cartographic, Ellipsoid, EllipsoidTangentPlane, GeoRectangle, MathError,
    Occluder, Plane, PlaneIntersection, RayInterval, Sphere,
};
use glam::{DMat3, DMat4, DVec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrientedBoundingBox {
    pub center: DVec3,
    pub half_axes: DMat3,
}

impl Default for OrientedBoundingBox {
    fn default() -> Self {
        Self {
            center: DVec3::ZERO,
            half_axes: DMat3::ZERO,
        }
    }
}

impl OrientedBoundingBox {
    pub const PACKED_LENGTH: usize = 12;

    /// Uses a sphere enclosing all box corners for conservative occlusion.
    pub fn is_occluded(self, occluder: Occluder) -> Result<bool, MathError> {
        let sphere = Sphere::from_oriented_box(self.center, self.half_axes)?;
        Ok(!occluder.is_sphere_visible(sphere))
    }

    /// Builds a box around an ellipsoid-of-revolution rectangle and heights.
    pub fn from_rectangle(
        rectangle: GeoRectangle,
        minimum_height: f64,
        maximum_height: f64,
        ellipsoid: Ellipsoid,
    ) -> Result<Self, MathError> {
        GeoRectangle::radians(
            rectangle.west,
            rectangle.south,
            rectangle.east,
            rectangle.north,
        )?;
        let radii = ellipsoid.radii();
        if !minimum_height.is_finite()
            || !maximum_height.is_finite()
            || minimum_height > maximum_height
            || rectangle.height() < 0.0
            || radii.min_element() <= 0.0
            || (radii.x - radii.y).abs() > 1e-15 * radii.x.max(radii.y)
        {
            return Err(MathError);
        }
        let cart = |longitude, latitude, height| {
            ellipsoid
                .cartographic_to_cartesian(Cartographic {
                    longitude,
                    latitude,
                    height,
                })
                .ok_or(MathError)
        };
        let center = rectangle.center();
        let from_extents = |origin: DVec3, axes: DMat3, min: DVec3, max: DVec3| {
            Self::new(
                origin + axes * ((min + max) * 0.5),
                axes * DMat3::from_diagonal((max - min) * 0.5),
            )
        };
        if rectangle.width() <= std::f64::consts::PI {
            let tangent = EllipsoidTangentPlane::new(
                cart(center.longitude, center.latitude, 0.0)?,
                ellipsoid,
            )?;
            let lat_center = if rectangle.south < 0.0 && rectangle.north > 0.0 {
                0.0
            } else {
                center.latitude
            };
            let nc = cart(center.longitude, rectangle.north, maximum_height)?;
            let nw = cart(rectangle.west, rectangle.north, maximum_height)?;
            let cw = cart(rectangle.west, lat_center, maximum_height)?;
            let sw = cart(rectangle.west, rectangle.south, maximum_height)?;
            let sc = cart(center.longitude, rectangle.south, maximum_height)?;
            let project = |p| {
                tangent
                    .project_point_to_nearest_on_plane(p)
                    .ok_or(MathError)
            };
            let (nc, nw, cw, sw, sc) = (
                project(nc)?,
                project(nw)?,
                project(cw)?,
                project(sw)?,
                project(sc)?,
            );
            let min_x = nw.x.min(cw.x).min(sw.x);
            let min_y = sw.y.min(sc.y);
            let max_y = nw.y.max(nc.y);
            let min_z = tangent
                .plane
                .point_distance(cart(rectangle.west, rectangle.north, minimum_height)?)
                .min(tangent.plane.point_distance(cart(
                    rectangle.west,
                    rectangle.south,
                    minimum_height,
                )?));
            return Ok(from_extents(
                tangent.origin,
                DMat3::from_cols(tangent.x_axis, tangent.y_axis, tangent.z_axis()),
                DVec3::new(min_x, min_y, min_z),
                DVec3::new(-min_x, max_y, maximum_height),
            ));
        }
        let above = rectangle.south > 0.0;
        let below = rectangle.north < 0.0;
        let lat = if above {
            rectangle.south
        } else if below {
            rectangle.north
        } else {
            0.0
        };
        let mut origin = cart(center.longitude, lat, maximum_height)?;
        origin.z = 0.0;
        let normal = if origin.x.abs() < 1e-10 && origin.y.abs() < 1e-10 {
            DVec3::X
        } else {
            origin.try_normalize().ok_or(MathError)?
        };
        let x_axis = normal.cross(DVec3::Z);
        let plane = Plane::from_point_normal(origin, normal)?;
        let horizon = cart(
            center.longitude + std::f64::consts::FRAC_PI_2,
            lat,
            maximum_height,
        )?;
        let max_x = plane.project_point(horizon).dot(x_axis);
        let max_y = cart(
            0.0,
            rectangle.north,
            if below {
                minimum_height
            } else {
                maximum_height
            },
        )?
        .z;
        let min_y = cart(
            0.0,
            rectangle.south,
            if above {
                minimum_height
            } else {
                maximum_height
            },
        )?
        .z;
        let min_z = plane.point_distance(cart(rectangle.east, lat, maximum_height)?);
        Ok(from_extents(
            origin,
            DMat3::from_cols(x_axis, DVec3::Z, normal),
            DVec3::new(-max_x, min_y, min_z),
            DVec3::new(max_x, max_y, 0.0),
        ))
    }
    /// Constructs a PCA-oriented box enclosing the supplied points.
    pub fn from_points(points: &[DVec3]) -> Result<Self, MathError> {
        if points.is_empty() {
            return Ok(Self::default());
        }
        if points.iter().any(|p| !p.is_finite()) {
            return Err(MathError);
        }
        let mean = points.iter().copied().sum::<DVec3>() / points.len() as f64;
        let mut covariance = DMat3::ZERO;
        for &point in points {
            let p = point - mean;
            covariance += DMat3::from_cols(p * p.x, p * p.y, p * p.z);
        }
        covariance *= 1.0 / points.len() as f64;
        if !covariance.is_finite() {
            return Err(MathError);
        }
        let mut axes = DMat3::IDENTITY;
        let tolerance = covariance
            .to_cols_array()
            .iter()
            .map(|x| x * x)
            .sum::<f64>()
            .sqrt()
            * 1e-20;
        // Cesium's Jacobi iteration picks the largest off-diagonal entry,
        // preserves eigenvector order, and performs at most ten sweeps.
        for _ in 0..30 {
            let a = covariance.to_cols_array();
            let off = (2.0 * (a[7] * a[7] + a[6] * a[6] + a[3] * a[3])).sqrt();
            if off <= tolerance {
                break;
            }
            let pairs = [(1, 2), (0, 2), (0, 1)];
            let mut index = 1;
            let mut largest = 0.0;
            for (i, &(p, q)) in pairs.iter().enumerate() {
                let magnitude = a[q * 3 + p].abs();
                if magnitude > largest {
                    largest = magnitude;
                    index = i;
                }
            }
            if largest <= 1e-15 {
                break;
            }
            let (p, q) = pairs[index];
            let tau = (a[q * 3 + q] - a[p * 3 + p]) / (2.0 * a[q * 3 + p]);
            let t = if tau < 0.0 {
                -1.0 / (-tau + (1.0 + tau * tau).sqrt())
            } else {
                1.0 / (tau + (1.0 + tau * tau).sqrt())
            };
            let c = 1.0 / (1.0 + t * t).sqrt();
            let s = t * c;
            let mut j = DMat3::IDENTITY.to_cols_array();
            j[p * 3 + p] = c;
            j[q * 3 + q] = c;
            j[q * 3 + p] = s;
            j[p * 3 + q] = -s;
            let rotation = DMat3::from_cols_array(&j);
            covariance = rotation.transpose() * covariance * rotation;
            axes *= rotation;
        }
        let mut min = DVec3::splat(f64::INFINITY);
        let mut max = DVec3::splat(f64::NEG_INFINITY);
        for &point in points {
            // Project offsets from the mean to retain precision for ECEF points.
            let local = axes.transpose() * (point - mean);
            min = min.min(local);
            max = max.max(local);
        }
        let center = mean + axes * ((min + max) * 0.5);
        let half_axes = axes * DMat3::from_diagonal((max - min) * 0.5);
        if !center.is_finite() || !half_axes.is_finite() {
            return Err(MathError);
        }
        Ok(Self::new(center, half_axes))
    }
    pub fn new(center: DVec3, half_axes: DMat3) -> Self {
        Self { center, half_axes }
    }
    pub fn from_transformation(transform: DMat4) -> Result<Self, MathError> {
        if !transform.is_finite() {
            return Err(MathError);
        }
        Ok(Self::new(
            transform.w_axis.truncate(),
            DMat3::from_cols(
                transform.x_axis.truncate(),
                transform.y_axis.truncate(),
                transform.z_axis.truncate(),
            ) * 0.5,
        ))
    }
    pub fn transformation(self) -> DMat4 {
        DMat4::from_cols(
            (self.half_axes.x_axis * 2.0).extend(0.0),
            (self.half_axes.y_axis * 2.0).extend(0.0),
            (self.half_axes.z_axis * 2.0).extend(0.0),
            self.center.extend(1.0),
        )
    }
    pub fn corners(self) -> [DVec3; 8] {
        let a = self.half_axes.x_axis;
        let b = self.half_axes.y_axis;
        let c = self.half_axes.z_axis;
        let o = self.center;
        [
            o - a - b - c,
            o - a - b + c,
            o - a + b - c,
            o - a + b + c,
            o + a - b - c,
            o + a - b + c,
            o + a + b - c,
            o + a + b + c,
        ]
    }
    pub fn intersect_plane(self, plane: Plane) -> PlaneIntersection {
        let normal = plane.normal;
        let radius = normal.dot(self.half_axes.x_axis).abs()
            + normal.dot(self.half_axes.y_axis).abs()
            + normal.dot(self.half_axes.z_axis).abs();
        let distance = plane.point_distance(self.center);
        if distance <= -radius {
            PlaneIntersection::Outside
        } else if distance >= radius {
            PlaneIntersection::Inside
        } else {
            PlaneIntersection::Intersecting
        }
    }
    /// Exact for mutually orthogonal half axes, including zero-length axes.
    /// Sheared boxes are rejected because their closest-point problem differs.
    pub fn distance_squared_to(self, point: DVec3) -> Option<f64> {
        if !self.center.is_finite() || !self.half_axes.is_finite() || !point.is_finite() {
            return None;
        }
        let axes = [
            self.half_axes.x_axis,
            self.half_axes.y_axis,
            self.half_axes.z_axis,
        ];
        for i in 0..3 {
            for j in i + 1..3 {
                let scale = axes[i].length() * axes[j].length();
                if scale > 0.0 && axes[i].dot(axes[j]).abs() > scale * 1e-12 {
                    return None;
                }
            }
        }
        let offset = point - self.center;
        let mut closest = DVec3::ZERO;
        for axis in axes {
            let length = axis.length();
            if length > 0.0 {
                let unit = axis / length;
                closest += unit * offset.dot(unit).clamp(-length, length);
            }
        }
        Some((offset - closest).length_squared())
    }
    pub fn compute_plane_distances(self, position: DVec3, direction: DVec3) -> RayInterval {
        let midpoint = direction.dot(self.center - position);
        let radius = direction.dot(self.half_axes.x_axis).abs()
            + direction.dot(self.half_axes.y_axis).abs()
            + direction.dot(self.half_axes.z_axis).abs();
        RayInterval::new(midpoint - radius, midpoint + radius)
    }
    pub fn pack(self, output: &mut [f64], offset: usize) -> Result<(), MathError> {
        let end = offset.checked_add(Self::PACKED_LENGTH).ok_or(MathError)?;
        let out = output.get_mut(offset..end).ok_or(MathError)?;
        out[..3].copy_from_slice(&self.center.to_array());
        out[3..].copy_from_slice(&self.half_axes.to_cols_array());
        Ok(())
    }
    pub fn unpack(input: &[f64], offset: usize) -> Result<Self, MathError> {
        let end = offset.checked_add(Self::PACKED_LENGTH).ok_or(MathError)?;
        let v = input.get(offset..end).ok_or(MathError)?;
        let axes: [f64; 9] = v[3..].try_into().map_err(|_| MathError)?;
        Ok(Self::new(
            DVec3::new(v[0], v[1], v[2]),
            DMat3::from_cols_array(&axes),
        ))
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        match other {
            Some(o) => self.center == o.center && self.half_axes == o.half_axes,
            None => false,
        }
    }
}

impl BoundingVolume for OrientedBoundingBox {
    fn intersect_plane(&self, plane: Plane) -> PlaneIntersection {
        (*self).intersect_plane(plane)
    }
}
