//! PolylineGeometry matching CesiumJS `Core/PolylineGeometry`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::{
    core_math::{chord_length, EPSILON7},
    geometry::{
        ArcType, ComponentDatatype, Geometry, GeometryAttribute, GeometryAttributes, PrimitiveType,
    },
    BoundingSphere, Cartesian3, Color, Ellipsoid, VertexFormat,
};
use glam::DVec3;

#[derive(Clone, Debug, PartialEq)]
pub struct PolylineGeometry {
    pub positions: Vec<Cartesian3>,
    pub width: f64,
    pub colors: Option<Vec<Color>>,
    pub colors_per_vertex: bool,
    pub arc_type: ArcType,
    pub granularity: f64,
    pub vertex_format: VertexFormat,
    pub ellipsoid: Ellipsoid,
}

#[derive(Clone, Debug, Default)]
pub struct PolylineGeometryOptions {
    pub positions: Vec<Cartesian3>,
    pub width: Option<f64>,
    pub colors: Option<Vec<Color>>,
    pub colors_per_vertex: Option<bool>,
    pub arc_type: Option<ArcType>,
    pub granularity: Option<f64>,
    pub vertex_format: Option<VertexFormat>,
    pub ellipsoid: Option<Ellipsoid>,
}

impl PolylineGeometry {
    pub fn new(options: PolylineGeometryOptions) -> Self {
        Self {
            positions: options.positions,
            width: options.width.unwrap_or(1.0),
            colors: options.colors,
            colors_per_vertex: options.colors_per_vertex.unwrap_or(false),
            arc_type: options.arc_type.unwrap_or(ArcType::Geodesic),
            granularity: options.granularity.unwrap_or_else(|| 1.0_f64.to_radians()),
            vertex_format: options.vertex_format.unwrap_or(VertexFormat::DEFAULT),
            ellipsoid: options.ellipsoid.unwrap_or(Ellipsoid::WGS84),
        }
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        if self.width <= 0.0 {
            return None;
        }

        // Deduplicate adjacent identical points
        let mut positions = Vec::with_capacity(self.positions.len());
        let mut removed_indices = Vec::new();

        for (i, &p) in self.positions.iter().enumerate() {
            if let Some(&last) = positions.last() {
                if p.equals_epsilon(&last, Some(EPSILON7), Some(EPSILON7)) {
                    removed_indices.push(i);
                    continue;
                }
            }
            positions.push(p);
        }

        if positions.len() < 2 {
            return None;
        }

        let mut colors = self.colors.clone();
        if let Some(ref mut cols) = colors {
            if !removed_indices.is_empty() {
                let mut filtered = Vec::new();
                for (i, c) in cols.iter().enumerate() {
                    if !removed_indices.contains(&i) {
                        filtered.push(*c);
                    }
                }
                *cols = filtered;
            }

            if self.colors_per_vertex && cols.len() < positions.len() {
                return None;
            }
            if !self.colors_per_vertex && cols.len() < positions.len() - 1 {
                return None;
            }
        }

        // Subdivide if geodesic or rhumb
        if self.arc_type != ArcType::None {
            let min_dist = chord_length(self.granularity, self.ellipsoid.maximum_radius());
            let mut subdivided = Vec::new();
            let mut new_colors = colors.as_ref().map(|_| Vec::new());

            for i in 0..positions.len() - 1 {
                let p0 = positions[i];
                let p1 = positions[i + 1];
                let seg = subdivide_segment(p0, p1, min_dist, self.ellipsoid);
                let skip = if i > 0 { 1 } else { 0 };

                if let (Some(ref cols), Some(ref mut nc)) = (&colors, &mut new_colors) {
                    let seg_count = seg.len() - skip;
                    if self.colors_per_vertex {
                        let c0 = cols[i];
                        let c1 = cols[i + 1];
                        for (s_idx, _) in seg.iter().enumerate().skip(skip) {
                            let t = s_idx as f32 / (seg.len() - 1) as f32;
                            nc.push(Color::new(
                                c0.red + (c1.red - c0.red) * t,
                                c0.green + (c1.green - c0.green) * t,
                                c0.blue + (c1.blue - c0.blue) * t,
                                c0.alpha + (c1.alpha - c0.alpha) * t,
                            ));
                        }
                    } else {
                        let c = cols[i];
                        for _ in 0..seg_count {
                            nc.push(c);
                        }
                    }
                }

                subdivided.extend_from_slice(&seg[skip..]);
            }

            positions = subdivided;
            if let Some(nc) = new_colors {
                colors = Some(nc);
            }
        }

        let positions_length = positions.len();
        if positions_length < 2 {
            return None;
        }

        let size = positions_length * 4 - 4;
        let mut final_positions = Vec::with_capacity(size * 3);
        let mut prev_positions = Vec::with_capacity(size * 3);
        let mut next_positions = Vec::with_capacity(size * 3);
        let mut expand_and_width = Vec::with_capacity(size * 2);
        let mut st = if self.vertex_format.st {
            Some(Vec::with_capacity(size * 2))
        } else {
            None
        };
        let mut final_colors = if colors.is_some() {
            Some(Vec::with_capacity(size * 4))
        } else {
            None
        };

        for j in 0..positions_length {
            let prev_pos = if j == 0 {
                positions[0] + (positions[0] - positions[1])
            } else {
                positions[j - 1]
            };

            let cur_pos = positions[j];

            let next_pos = if j == positions_length - 1 {
                positions[positions_length - 1]
                    + (positions[positions_length - 1] - positions[positions_length - 2])
            } else {
                positions[j + 1]
            };

            let (color0, color1) = if let Some(ref cols) = colors {
                let c0 = if j != 0 && !self.colors_per_vertex {
                    cols[j - 1]
                } else {
                    cols[j]
                };
                let c1 = if j != positions_length - 1 {
                    cols[j]
                } else {
                    *cols.last().unwrap()
                };
                (c0, c1)
            } else {
                (Color::WHITE, Color::WHITE)
            };

            let start_k = if j == 0 { 2 } else { 0 };
            let end_k = if j == positions_length - 1 { 2 } else { 4 };

            for k in start_k..end_k {
                final_positions.extend_from_slice(&[cur_pos.x, cur_pos.y, cur_pos.z]);
                prev_positions.extend_from_slice(&[prev_pos.x, prev_pos.y, prev_pos.z]);
                next_positions.extend_from_slice(&[next_pos.x, next_pos.y, next_pos.z]);

                let direction = if k < 2 { -1.0 } else { 1.0 };
                let expand = (2 * (k % 2) - 1) as f64;
                expand_and_width.push(expand);
                expand_and_width.push(direction * self.width);

                if let Some(ref mut st_arr) = st {
                    st_arr.push(j as f64 / (positions_length - 1) as f64);
                    st_arr.push(expand.max(0.0));
                }

                if let Some(ref mut fc) = final_colors {
                    let col = if k < 2 { color0 } else { color1 };
                    fc.extend_from_slice(&[
                        col.red as f64,
                        col.green as f64,
                        col.blue as f64,
                        col.alpha as f64,
                    ]);
                }
            }
        }

        let num_indices = positions_length * 6 - 6;
        let mut indices = Vec::with_capacity(num_indices);
        let mut idx = 0;
        for _ in 0..(positions_length - 1) {
            indices.push(idx);
            indices.push(idx + 2);
            indices.push(idx + 1);

            indices.push(idx + 1);
            indices.push(idx + 2);
            indices.push(idx + 3);

            idx += 4;
        }

        let mut attributes = GeometryAttributes {
            position: Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                final_positions,
            )),
            prev_position: Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                prev_positions,
            )),
            next_position: Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                next_positions,
            )),
            expand_and_width: Some(GeometryAttribute::new(
                ComponentDatatype::Float,
                2,
                expand_and_width,
            )),
            ..Default::default()
        };

        if let Some(st_vals) = st {
            attributes.st = Some(GeometryAttribute::new(ComponentDatatype::Float, 2, st_vals));
        }

        if let Some(col_vals) = final_colors {
            attributes.color = Some(GeometryAttribute::new(
                ComponentDatatype::UnsignedByte,
                4,
                col_vals,
            ));
        }

        let dvecs: Vec<DVec3> = positions.iter().map(|&p| p.into()).collect();
        let bs = BoundingSphere::from_points(&dvecs).ok();

        Some(Geometry {
            attributes,
            indices,
            primitive_type: PrimitiveType::Triangles,
            bounding_sphere: bs,
        })
    }
}

fn subdivide_segment(
    p0: Cartesian3,
    p1: Cartesian3,
    min_distance: f64,
    ellipsoid: Ellipsoid,
) -> Vec<Cartesian3> {
    let d0: DVec3 = p0.into();
    let d1: DVec3 = p1.into();
    let norm0 = d0.normalize();
    let norm1 = d1.normalize();
    let angle = norm0.dot(norm1).clamp(-1.0, 1.0).acos();
    let dist = angle * ellipsoid.maximum_radius();

    let n = (dist / min_distance).ceil().max(1.0) as usize;
    if n <= 1 {
        return vec![p0, p1];
    }

    let mut result = Vec::with_capacity(n + 1);
    for i in 0..=n {
        let t = i as f64 / n as f64;
        let sin_angle = angle.sin();
        let dir = if sin_angle.abs() < 1e-10 {
            norm0
        } else {
            let a = ((1.0 - t) * angle).sin() / sin_angle;
            let b = (t * angle).sin() / sin_angle;
            (norm0 * a + norm1 * b).normalize()
        };

        if let Some(surf) = ellipsoid.scale_to_geodetic_surface(dir) {
            result.push(Cartesian3::from(surf));
        } else {
            result.push(Cartesian3::from(dir * ellipsoid.maximum_radius()));
        }
    }
    result
}
