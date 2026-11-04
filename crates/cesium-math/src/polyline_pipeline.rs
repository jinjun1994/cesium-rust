//! Polyline geometry processing functions matching CesiumJS `Core/PolylinePipeline`.
//!
//! Includes arc generation (geodesic and rhumb lines), height subdivision,
//! and longitude wrapping at the 180 degree meridian (IDL).

use crate::cartesian::Cartesian3;
use crate::chord_length;
use crate::core_math::RADIANS_PER_DEGREE;
use crate::ellipsoid::Ellipsoid;
use crate::ellipsoid_geodesic::EllipsoidGeodesic;
use crate::ellipsoid_rhumb_line::EllipsoidRhumbLine;
use crate::intersections::line_segment_plane;
use crate::plane::Plane;
use crate::Cartographic;
use glam::{DMat4, DVec3};

/// Result of wrapping a polyline across the 180 degree meridian.
#[derive(Clone, Debug, PartialEq)]
pub struct PolylineSegments {
    /// Positions for all segments concatenated.
    pub positions: Vec<Cartesian3>,
    /// Number of positions in each segment.
    pub lengths: Vec<usize>,
}

/// Height specification for polyline points: either a uniform height or per-position heights.
#[derive(Clone, Debug, PartialEq)]
pub enum PolylineHeight {
    Uniform(f64),
    PerPosition(Vec<f64>),
}

impl PolylineHeight {
    pub fn get(&self, index: usize) -> f64 {
        match self {
            PolylineHeight::Uniform(h) => *h,
            PolylineHeight::PerPosition(v) => v.get(index).copied().unwrap_or(0.0),
        }
    }
}

impl From<f64> for PolylineHeight {
    fn from(h: f64) -> Self {
        PolylineHeight::Uniform(h)
    }
}

impl From<Vec<f64>> for PolylineHeight {
    fn from(v: Vec<f64>) -> Self {
        PolylineHeight::PerPosition(v)
    }
}

impl From<&[f64]> for PolylineHeight {
    fn from(v: &[f64]) -> Self {
        PolylineHeight::PerPosition(v.to_vec())
    }
}

/// Options for polyline arc generation.
#[derive(Clone, Debug, Default)]
pub struct PolylineArcOptions<'a> {
    pub positions: &'a [Cartesian3],
    pub height: Option<PolylineHeight>,
    pub granularity: Option<f64>,
    pub min_distance: Option<f64>,
    pub ellipsoid: Option<Ellipsoid>,
}

fn subdivide_heights(num_points: usize, h0: f64, h1: f64) -> Vec<f64> {
    if num_points == 0 {
        return Vec::new();
    }
    if (h0 - h1).abs() < f64::EPSILON {
        return vec![h0; num_points];
    }
    let d_height = h1 - h0;
    let height_per_vertex = d_height / num_points as f64;
    (0..num_points)
        .map(|i| h0 + i as f64 * height_per_vertex)
        .collect()
}

/// Polyline pipeline algorithms matching CesiumJS `Core/PolylinePipeline`.
pub struct PolylinePipeline;

impl PolylinePipeline {
    /// Calculates the number of points for subdividing a line segment based on chord length.
    pub fn number_of_points(p0: Cartesian3, p1: Cartesian3, min_distance: f64) -> usize {
        let distance = p0.distance(&p1);
        (distance / min_distance).ceil() as usize
    }

    /// Calculates the number of points for subdividing a rhumb line segment based on angular distance.
    pub fn number_of_points_rhumb_line(
        p0: Cartographic,
        p1: Cartographic,
        granularity: f64,
    ) -> usize {
        let d_lon = p0.longitude - p1.longitude;
        let d_lat = p0.latitude - p1.latitude;
        let radians_dist_sq = d_lon * d_lon + d_lat * d_lat;
        let count = (radians_dist_sq.sqrt() / granularity).ceil() as usize;
        count.max(1)
    }

    /// Extracts heights for an array of Cartesian positions with respect to the specified ellipsoid.
    pub fn extract_heights(positions: &[Cartesian3], ellipsoid: &Ellipsoid) -> Vec<f64> {
        positions
            .iter()
            .map(|p| {
                ellipsoid
                    .cartesian_to_cartographic((*p).into())
                    .map(|c| c.height)
                    .unwrap_or(0.0)
            })
            .collect()
    }

    /// Breaks a polyline into segments such that it does not cross the ±180 degree meridian (IDL).
    pub fn wrap_longitude(
        positions: &[Cartesian3],
        model_matrix: Option<DMat4>,
    ) -> PolylineSegments {
        if positions.is_empty() {
            return PolylineSegments {
                positions: Vec::new(),
                lengths: Vec::new(),
            };
        }

        let matrix = model_matrix.unwrap_or(DMat4::IDENTITY);
        let inverse_matrix = matrix.inverse();

        let origin = inverse_matrix.transform_point3(DVec3::ZERO);
        let xz_normal = inverse_matrix
            .transform_vector3(DVec3::Y)
            .normalize_or_zero();
        let xz_plane = Plane::from_point_normal(origin, xz_normal)
            .unwrap_or(Plane::new(DVec3::Y, 0.0).unwrap());

        let yz_normal = inverse_matrix
            .transform_vector3(DVec3::X)
            .normalize_or_zero();
        let yz_plane = Plane::from_point_normal(origin, yz_normal)
            .unwrap_or(Plane::new(DVec3::X, 0.0).unwrap());

        let mut cartesians = Vec::new();
        let mut segments = Vec::new();

        let mut count = 1;
        cartesians.push(positions[0]);
        let mut prev = positions[0];

        for &cur in &positions[1..] {
            let prev_v: DVec3 = prev.into();
            let cur_v: DVec3 = cur.into();

            if yz_plane.point_distance(prev_v) < 0.0 || yz_plane.point_distance(cur_v) < 0.0 {
                if let Some(intersection) =
                    line_segment_plane(prev_v, cur_v, xz_plane.normal, xz_plane.distance)
                {
                    let mut offset = xz_normal * 5.0e-9;
                    if xz_plane.point_distance(prev_v) < 0.0 {
                        offset = -offset;
                    }

                    cartesians.push(Cartesian3::from(intersection + offset));
                    segments.push(count + 1);

                    offset = -offset;
                    cartesians.push(Cartesian3::from(intersection + offset));
                    count = 1;
                }
            }

            cartesians.push(cur);
            count += 1;
            prev = cur;
        }

        segments.push(count);

        PolylineSegments {
            positions: cartesians,
            lengths: segments,
        }
    }

    /// Subdivides polyline along great-circle geodesics and raises points to the specified height.
    /// Returns packed Cartesian coordinates as `[x, y, z, x, y, z, ...]`.
    pub fn generate_arc(options: &PolylineArcOptions) -> Vec<f64> {
        let positions = options.positions;
        let length = positions.len();
        if length == 0 {
            return Vec::new();
        }

        let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let height = &options.height;

        if length == 1 {
            let p0 = ellipsoid
                .scale_to_geodetic_surface(positions[0].into())
                .unwrap_or(positions[0].into());
            let h = height.as_ref().map(|h| h.get(0)).unwrap_or(0.0);
            let mut p = p0;
            if h != 0.0 {
                if let Some(n) = ellipsoid.geodetic_surface_normal(p) {
                    p += n * h;
                }
            }
            return vec![p.x, p.y, p.z];
        }

        let min_distance = options.min_distance.unwrap_or_else(|| {
            let granularity = options.granularity.unwrap_or(RADIANS_PER_DEGREE);
            chord_length(granularity, ellipsoid.maximum_radius())
        });

        let mut num_points = 0;
        for i in 0..length - 1 {
            num_points += Self::number_of_points(positions[i], positions[i + 1], min_distance);
        }

        let array_length = (num_points + 1) * 3;
        let mut new_positions = Vec::with_capacity(array_length);

        for i in 0..length - 1 {
            let p0 = positions[i];
            let p1 = positions[i + 1];

            let h0 = height.as_ref().map(|h| h.get(i)).unwrap_or(0.0);
            let h1 = height.as_ref().map(|h| h.get(i + 1)).unwrap_or(0.0);

            let first = ellipsoid
                .scale_to_geodetic_surface(p0.into())
                .unwrap_or(p0.into());
            let last = ellipsoid
                .scale_to_geodetic_surface(p1.into())
                .unwrap_or(p1.into());

            let seg_points = Self::number_of_points(p0, p1, min_distance);
            let start = ellipsoid
                .cartesian_to_cartographic(first)
                .unwrap_or(Cartographic::ZERO);
            let end = ellipsoid
                .cartesian_to_cartographic(last)
                .unwrap_or(Cartographic::ZERO);

            let heights = subdivide_heights(seg_points, h0, h1);

            let geodesic = EllipsoidGeodesic::new(Some(start), Some(end), Some(ellipsoid));
            let surface_distance = geodesic.surface_distance().unwrap_or(0.0);
            let surface_distance_between_points = if seg_points > 0 {
                surface_distance / seg_points as f64
            } else {
                0.0
            };

            let mut start_h = start;
            start_h.height = h0;
            let cart = ellipsoid
                .cartographic_to_cartesian(start_h)
                .unwrap_or(DVec3::ZERO);
            new_positions.extend_from_slice(&[cart.x, cart.y, cart.z]);

            for (j, &h) in heights.iter().enumerate().take(seg_points).skip(1) {
                let mut carto = geodesic
                    .interpolate_using_surface_distance(j as f64 * surface_distance_between_points)
                    .unwrap_or(start);
                carto.height = h;
                let pt = ellipsoid
                    .cartographic_to_cartesian(carto)
                    .unwrap_or(DVec3::ZERO);
                new_positions.extend_from_slice(&[pt.x, pt.y, pt.z]);
            }
        }

        let last_point = positions[length - 1];
        let mut carto = ellipsoid
            .cartesian_to_cartographic(last_point.into())
            .unwrap_or(Cartographic::ZERO);
        carto.height = height.as_ref().map(|h| h.get(length - 1)).unwrap_or(0.0);
        let cart = ellipsoid
            .cartographic_to_cartesian(carto)
            .unwrap_or(DVec3::ZERO);
        new_positions.extend_from_slice(&[cart.x, cart.y, cart.z]);

        new_positions
    }

    /// Subdivides polyline along great-circle geodesics and returns Cartesian positions.
    pub fn generate_cartesian_arc(options: &PolylineArcOptions) -> Vec<Cartesian3> {
        let values = Self::generate_arc(options);
        values
            .as_chunks::<3>()
            .0
            .iter()
            .map(|chunk| Cartesian3::new(chunk[0], chunk[1], chunk[2]))
            .collect()
    }

    /// Subdivides polyline along rhumb lines and raises points to the specified height.
    /// Returns packed Cartesian coordinates as `[x, y, z, x, y, z, ...]`.
    pub fn generate_rhumb_arc(options: &PolylineArcOptions) -> Vec<f64> {
        let positions = options.positions;
        let length = positions.len();
        if length == 0 {
            return Vec::new();
        }

        let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let height = &options.height;

        if length == 1 {
            let p0 = ellipsoid
                .scale_to_geodetic_surface(positions[0].into())
                .unwrap_or(positions[0].into());
            let h = height.as_ref().map(|h| h.get(0)).unwrap_or(0.0);
            let mut p = p0;
            if h != 0.0 {
                if let Some(n) = ellipsoid.geodetic_surface_normal(p) {
                    p += n * h;
                }
            }
            return vec![p.x, p.y, p.z];
        }

        let granularity = options.granularity.unwrap_or(RADIANS_PER_DEGREE);

        let mut num_points = 0;
        let mut c0 = ellipsoid
            .cartesian_to_cartographic(positions[0].into())
            .unwrap_or(Cartographic::ZERO);

        for i in 0..length - 1 {
            let c1 = ellipsoid
                .cartesian_to_cartographic(positions[i + 1].into())
                .unwrap_or(Cartographic::ZERO);
            num_points += Self::number_of_points_rhumb_line(c0, c1, granularity);
            c0 = c1;
        }

        let array_length = (num_points + 1) * 3;
        let mut new_positions = Vec::with_capacity(array_length);

        for i in 0..length - 1 {
            let p0 = positions[i];
            let p1 = positions[i + 1];

            let h0 = height.as_ref().map(|h| h.get(i)).unwrap_or(0.0);
            let h1 = height.as_ref().map(|h| h.get(i + 1)).unwrap_or(0.0);

            let mut start = ellipsoid
                .cartesian_to_cartographic(p0.into())
                .unwrap_or(Cartographic::ZERO);
            let mut end = ellipsoid
                .cartesian_to_cartographic(p1.into())
                .unwrap_or(Cartographic::ZERO);

            let seg_points = Self::number_of_points_rhumb_line(start, end, granularity);
            start.height = 0.0;
            end.height = 0.0;
            let heights = subdivide_heights(seg_points, h0, h1);

            let rhumb = EllipsoidRhumbLine::new(Some(start), Some(end), Some(ellipsoid));
            let surface_distance = rhumb.surface_distance().unwrap_or(0.0);
            let surface_distance_between_points = if seg_points > 0 {
                surface_distance / seg_points as f64
            } else {
                0.0
            };

            let mut start_h = start;
            start_h.height = h0;
            let cart = ellipsoid
                .cartographic_to_cartesian(start_h)
                .unwrap_or(DVec3::ZERO);
            new_positions.extend_from_slice(&[cart.x, cart.y, cart.z]);

            for (j, &h) in heights.iter().enumerate().take(seg_points).skip(1) {
                let mut carto = rhumb
                    .interpolate_using_surface_distance(j as f64 * surface_distance_between_points)
                    .unwrap_or(start);
                carto.height = h;
                let pt = ellipsoid
                    .cartographic_to_cartesian(carto)
                    .unwrap_or(DVec3::ZERO);
                new_positions.extend_from_slice(&[pt.x, pt.y, pt.z]);
            }
        }

        let last_point = positions[length - 1];
        let mut carto = ellipsoid
            .cartesian_to_cartographic(last_point.into())
            .unwrap_or(Cartographic::ZERO);
        carto.height = height.as_ref().map(|h| h.get(length - 1)).unwrap_or(0.0);
        let cart = ellipsoid
            .cartographic_to_cartesian(carto)
            .unwrap_or(DVec3::ZERO);
        new_positions.extend_from_slice(&[cart.x, cart.y, cart.z]);

        new_positions
    }

    /// Subdivides polyline along rhumb lines and returns Cartesian positions.
    pub fn generate_cartesian_rhumb_arc(options: &PolylineArcOptions) -> Vec<Cartesian3> {
        let values = Self::generate_rhumb_arc(options);
        values
            .as_chunks::<3>()
            .0
            .iter()
            .map(|chunk| Cartesian3::new(chunk[0], chunk[1], chunk[2]))
            .collect()
    }
}
