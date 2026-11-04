//! SimplePolylineGeometry matching CesiumJS `Core/SimplePolylineGeometry`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::{
    core_math::chord_length,
    geometry::{
        ArcType, ComponentDatatype, Geometry, GeometryAttribute, GeometryAttributes, PrimitiveType,
    },
    BoundingSphere, Cartesian3, Color, Ellipsoid,
};
use glam::DVec3;

#[derive(Clone, Debug, PartialEq)]
pub struct SimplePolylineGeometry {
    pub positions: Vec<Cartesian3>,
    pub colors: Option<Vec<Color>>,
    pub colors_per_vertex: bool,
    pub arc_type: ArcType,
    pub granularity: f64,
    pub ellipsoid: Ellipsoid,
}

#[derive(Clone, Debug, Default)]
pub struct SimplePolylineGeometryOptions {
    pub positions: Vec<Cartesian3>,
    pub colors: Option<Vec<Color>>,
    pub colors_per_vertex: Option<bool>,
    pub arc_type: Option<ArcType>,
    pub granularity: Option<f64>,
    pub ellipsoid: Option<Ellipsoid>,
}

impl SimplePolylineGeometry {
    pub fn new(options: SimplePolylineGeometryOptions) -> Self {
        Self {
            positions: options.positions,
            colors: options.colors,
            colors_per_vertex: options.colors_per_vertex.unwrap_or(false),
            arc_type: options.arc_type.unwrap_or(ArcType::Geodesic),
            granularity: options.granularity.unwrap_or_else(|| 1.0_f64.to_radians()),
            ellipsoid: options.ellipsoid.unwrap_or(Ellipsoid::WGS84),
        }
    }

    #[allow(clippy::needless_range_loop)]
    pub fn create_geometry(&self) -> Option<Geometry> {
        let length = self.positions.len();
        if length < 2 {
            return None;
        }

        if let Some(ref c) = self.colors {
            if self.colors_per_vertex && c.len() < length {
                return None;
            }
            if !self.colors_per_vertex && c.len() < length - 1 {
                return None;
            }
        }

        let per_segment_colors = self.colors.is_some() && !self.colors_per_vertex;
        let mut final_positions = Vec::new();
        let mut final_colors: Option<Vec<f64>> = if self.colors.is_some() {
            Some(Vec::new())
        } else {
            None
        };

        if self.arc_type == ArcType::None {
            if per_segment_colors {
                let colors = self.colors.as_ref().unwrap();
                for i in 0..length - 1 {
                    let p0 = self.positions[i];
                    let p1 = self.positions[i + 1];
                    let color = colors[i];

                    final_positions.extend_from_slice(&[p0.x, p0.y, p0.z, p1.x, p1.y, p1.z]);
                    if let Some(ref mut fc) = final_colors {
                        for _ in 0..2 {
                            fc.extend_from_slice(&[
                                color.red as f64,
                                color.green as f64,
                                color.blue as f64,
                                color.alpha as f64,
                            ]);
                        }
                    }
                }
            } else {
                for (i, &p) in self.positions.iter().enumerate() {
                    final_positions.extend_from_slice(&[p.x, p.y, p.z]);
                    if let Some(ref mut fc) = final_colors {
                        let color = self.colors.as_ref().unwrap()[i];
                        fc.extend_from_slice(&[
                            color.red as f64,
                            color.green as f64,
                            color.blue as f64,
                            color.alpha as f64,
                        ]);
                    }
                }
            }
        } else {
            // Geodesic or Rhumb subdivision
            let min_dist = chord_length(self.granularity, self.ellipsoid.maximum_radius());

            if per_segment_colors {
                let colors = self.colors.as_ref().unwrap();
                for i in 0..length - 1 {
                    let p0 = self.positions[i];
                    let p1 = self.positions[i + 1];
                    let color = colors[i];

                    let seg = subdivide_segment(p0, p1, min_dist, self.ellipsoid, self.arc_type);
                    for p in &seg {
                        final_positions.extend_from_slice(&[p.x, p.y, p.z]);
                        if let Some(ref mut fc) = final_colors {
                            fc.extend_from_slice(&[
                                color.red as f64,
                                color.green as f64,
                                color.blue as f64,
                                color.alpha as f64,
                            ]);
                        }
                    }
                }
            } else {
                let mut full_pts = Vec::new();
                for i in 0..length - 1 {
                    let p0 = self.positions[i];
                    let p1 = self.positions[i + 1];
                    let seg = subdivide_segment(p0, p1, min_dist, self.ellipsoid, self.arc_type);
                    let skip = if i > 0 { 1 } else { 0 };
                    full_pts.extend_from_slice(&seg[skip..]);
                }

                for p in &full_pts {
                    final_positions.extend_from_slice(&[p.x, p.y, p.z]);
                }

                if let Some(ref colors) = self.colors {
                    if let Some(ref mut fc) = final_colors {
                        if self.colors_per_vertex {
                            // If subdivision was 1:1, assign colors directly
                            let n_pts = full_pts.len();
                            for i in 0..n_pts {
                                let c = if i < colors.len() {
                                    colors[i]
                                } else {
                                    *colors.last().unwrap()
                                };
                                fc.extend_from_slice(&[
                                    c.red as f64,
                                    c.green as f64,
                                    c.blue as f64,
                                    c.alpha as f64,
                                ]);
                            }
                        }
                    }
                }
            }
        }

        let num_verts = final_positions.len() / 3;
        let mut indices = Vec::new();

        if per_segment_colors {
            for i in (0..num_verts as u32).step_by(2) {
                indices.push(i);
                indices.push(i + 1);
            }
        } else {
            for i in 0..(num_verts as u32 - 1) {
                indices.push(i);
                indices.push(i + 1);
            }
        }

        let mut attributes = GeometryAttributes {
            position: Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                final_positions,
            )),
            ..Default::default()
        };

        if let Some(colors) = final_colors {
            attributes.color = Some(GeometryAttribute::new(
                ComponentDatatype::UnsignedByte,
                4,
                colors,
            ));
        }

        let dvecs: Vec<DVec3> = self.positions.iter().map(|&p| p.into()).collect();
        let bs = BoundingSphere::from_points(&dvecs).ok();

        Some(Geometry {
            attributes,
            indices,
            primitive_type: PrimitiveType::Lines,
            bounding_sphere: bs,
        })
    }
}

fn subdivide_segment(
    p0: Cartesian3,
    p1: Cartesian3,
    min_distance: f64,
    ellipsoid: Ellipsoid,
    _arc_type: ArcType,
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
