//! EllipsoidGeometry and EllipsoidOutlineGeometry matching CesiumJS `Core/EllipsoidGeometry` and `Core/EllipsoidOutlineGeometry`.
//! Adapted from CesiumJS (Apache-2.0).

#![allow(clippy::too_many_arguments)]

use crate::{
    core_math::{PI, TWO_PI},
    geometry::{ComponentDatatype, Geometry, GeometryAttribute, GeometryAttributes, PrimitiveType},
    Cartesian3, VertexFormat,
};

// ============================================================================
// EllipsoidGeometry
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EllipsoidGeometry {
    pub radii: Cartesian3,
    pub inner_radii: Option<Cartesian3>,
    pub minimum_clock: f64,
    pub maximum_clock: f64,
    pub minimum_cone: f64,
    pub maximum_cone: f64,
    pub stack_partitions: i32,
    pub slice_partitions: i32,
    pub vertex_format: VertexFormat,
}

impl Default for EllipsoidGeometry {
    fn default() -> Self {
        Self {
            radii: Cartesian3::new(1.0, 1.0, 1.0),
            inner_radii: None,
            minimum_clock: 0.0,
            maximum_clock: TWO_PI,
            minimum_cone: 0.0,
            maximum_cone: PI,
            stack_partitions: 64,
            slice_partitions: 64,
            vertex_format: VertexFormat::DEFAULT,
        }
    }
}

impl EllipsoidGeometry {
    pub fn new(
        radii: Option<Cartesian3>,
        inner_radii: Option<Cartesian3>,
        minimum_clock: Option<f64>,
        maximum_clock: Option<f64>,
        minimum_cone: Option<f64>,
        maximum_cone: Option<f64>,
        stack_partitions: Option<i32>,
        slice_partitions: Option<i32>,
        vertex_format: Option<VertexFormat>,
    ) -> Self {
        Self {
            radii: radii.unwrap_or(Cartesian3::new(1.0, 1.0, 1.0)),
            inner_radii,
            minimum_clock: minimum_clock.unwrap_or(0.0),
            maximum_clock: maximum_clock.unwrap_or(TWO_PI),
            minimum_cone: minimum_cone.unwrap_or(0.0),
            maximum_cone: maximum_cone.unwrap_or(PI),
            stack_partitions: stack_partitions.unwrap_or(64),
            slice_partitions: slice_partitions.unwrap_or(64),
            vertex_format: vertex_format.unwrap_or(VertexFormat::DEFAULT),
        }
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        let radii = self.radii;
        if radii.x <= 0.0 || radii.y <= 0.0 || radii.z <= 0.0 {
            return None;
        }

        let inner_radii = self.inner_radii.unwrap_or(radii);
        if inner_radii.x <= 0.0 || inner_radii.y <= 0.0 || inner_radii.z <= 0.0 {
            return None;
        }

        let minimum_clock = self.minimum_clock;
        let maximum_clock = self.maximum_clock;
        let minimum_cone = self.minimum_cone;
        let maximum_cone = self.maximum_cone;
        let vertex_format = self.vertex_format;

        let mut slice_partitions = (self.slice_partitions + 1) as f64;
        let mut stack_partitions = (self.stack_partitions + 1) as f64;

        slice_partitions = ((slice_partitions * (maximum_clock - minimum_clock).abs()) / TWO_PI).round();
        stack_partitions = ((stack_partitions * (maximum_cone - minimum_cone).abs()) / PI).round();

        let slice_partitions = (slice_partitions as i32).max(2);
        let stack_partitions = (stack_partitions as i32).max(2);

        let mut phis = Vec::with_capacity((stack_partitions + 2) as usize);
        phis.push(minimum_cone);
        for i in 0..stack_partitions {
            phis.push(minimum_cone + (i as f64 * (maximum_cone - minimum_cone)) / (stack_partitions - 1) as f64);
        }
        phis.push(maximum_cone);

        let mut thetas = Vec::with_capacity((slice_partitions + 2) as usize);
        thetas.push(minimum_clock);
        for j in 0..slice_partitions {
            thetas.push(minimum_clock + (j as f64 * (maximum_clock - minimum_clock)) / (slice_partitions - 1) as f64);
        }
        thetas.push(maximum_clock);

        let num_phis = phis.len();
        let num_thetas = thetas.len();

        let sin_phi: Vec<f64> = phis.iter().map(|p| p.sin()).collect();
        let cos_phi: Vec<f64> = phis.iter().map(|p| p.cos()).collect();
        let sin_theta: Vec<f64> = thetas.iter().map(|t| t.sin()).collect();
        let cos_theta: Vec<f64> = thetas.iter().map(|t| t.cos()).collect();

        let vertex_count = num_thetas * num_phis;
        let mut positions = Vec::with_capacity(vertex_count * 3);

        for i in 0..num_phis {
            for j in 0..num_thetas {
                positions.push(radii.x * sin_phi[i] * cos_theta[j]);
                positions.push(radii.y * sin_phi[i] * sin_theta[j]);
                positions.push(radii.z * cos_phi[i]);
            }
        }

        let mut indices = Vec::new();
        for i in 1..(num_phis - 2) {
            let top_offset = (i * num_thetas) as u32;
            let bottom_offset = ((i + 1) * num_thetas) as u32;

            for j in 1..(num_thetas - 2) {
                let j = j as u32;
                indices.push(bottom_offset + j);
                indices.push(bottom_offset + j + 1);
                indices.push(top_offset + j + 1);

                indices.push(bottom_offset + j);
                indices.push(top_offset + j + 1);
                indices.push(top_offset + j);
            }
        }

        let mut attributes = GeometryAttributes::default();
        if vertex_format.position {
            attributes.position = Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                positions,
            ));
        }

        if vertex_format.normal {
            let mut normals = Vec::with_capacity(vertex_count * 3);
            for i in 0..num_phis {
                for j in 0..num_thetas {
                    let n = Cartesian3::new(
                        sin_phi[i] * cos_theta[j] / radii.x,
                        sin_phi[i] * sin_theta[j] / radii.y,
                        cos_phi[i] / radii.z,
                    ).normalize();
                    normals.push(n.x);
                    normals.push(n.y);
                    normals.push(n.z);
                }
            }
            attributes.normal = Some(GeometryAttribute::new(
                ComponentDatatype::Float,
                3,
                normals,
            ));
        }

        if vertex_format.st {
            let mut uvs = Vec::with_capacity(vertex_count * 2);
            for i in 0..num_phis {
                for j in 0..num_thetas {
                    let n = Cartesian3::new(
                        sin_phi[i] * cos_theta[j] / radii.x,
                        sin_phi[i] * sin_theta[j] / radii.y,
                        cos_phi[i] / radii.z,
                    ).normalize();
                    let u = (-n.y).atan2(-n.x) / TWO_PI + 0.5;
                    let v = n.z.asin() / PI + 0.5;
                    uvs.push(u);
                    uvs.push(v);
                }
            }
            attributes.st = Some(GeometryAttribute::new(ComponentDatatype::Float, 2, uvs));
        }

        Some(Geometry {
            attributes,
            indices,
            primitive_type: PrimitiveType::Triangles,
        })
    }
}

// ============================================================================
// EllipsoidOutlineGeometry
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EllipsoidOutlineGeometry {
    pub radii: Cartesian3,
    pub inner_radii: Option<Cartesian3>,
    pub minimum_clock: f64,
    pub maximum_clock: f64,
    pub minimum_cone: f64,
    pub maximum_cone: f64,
    pub stack_partitions: i32,
    pub slice_partitions: i32,
    pub subdivisions: i32,
}

impl Default for EllipsoidOutlineGeometry {
    fn default() -> Self {
        Self {
            radii: Cartesian3::new(1.0, 1.0, 1.0),
            inner_radii: None,
            minimum_clock: 0.0,
            maximum_clock: TWO_PI,
            minimum_cone: 0.0,
            maximum_cone: PI,
            stack_partitions: 10,
            slice_partitions: 8,
            subdivisions: 200,
        }
    }
}

impl EllipsoidOutlineGeometry {
    pub fn new(
        radii: Option<Cartesian3>,
        inner_radii: Option<Cartesian3>,
        minimum_clock: Option<f64>,
        maximum_clock: Option<f64>,
        minimum_cone: Option<f64>,
        maximum_cone: Option<f64>,
        stack_partitions: Option<i32>,
        slice_partitions: Option<i32>,
        subdivisions: Option<i32>,
    ) -> Self {
        Self {
            radii: radii.unwrap_or(Cartesian3::new(1.0, 1.0, 1.0)),
            inner_radii,
            minimum_clock: minimum_clock.unwrap_or(0.0),
            maximum_clock: maximum_clock.unwrap_or(TWO_PI),
            minimum_cone: minimum_cone.unwrap_or(0.0),
            maximum_cone: maximum_cone.unwrap_or(PI),
            stack_partitions: stack_partitions.unwrap_or(10),
            slice_partitions: slice_partitions.unwrap_or(8),
            subdivisions: subdivisions.unwrap_or(200),
        }
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        let radii = self.radii;
        if radii.x <= 0.0 || radii.y <= 0.0 || radii.z <= 0.0 {
            return None;
        }

        let minimum_clock = self.minimum_clock;
        let maximum_clock = self.maximum_clock;
        let minimum_cone = self.minimum_cone;
        let maximum_cone = self.maximum_cone;
        let subdivisions = self.subdivisions;

        let mut slice_partitions = (self.slice_partitions + 1) as f64;
        let mut stack_partitions = (self.stack_partitions + 1) as f64;

        slice_partitions = ((slice_partitions * (maximum_clock - minimum_clock).abs()) / TWO_PI).round();
        stack_partitions = ((stack_partitions * (maximum_cone - minimum_cone).abs()) / PI).round();

        let slice_partitions = (slice_partitions as i32).max(2);
        let stack_partitions = (stack_partitions as i32).max(2);

        let vertex_count = (subdivisions * (stack_partitions + slice_partitions)) as usize;
        let mut positions = Vec::with_capacity(vertex_count * 3);

        // Sin/cos phi for stacks
        let mut sin_phi = Vec::with_capacity(stack_partitions as usize);
        let mut cos_phi = Vec::with_capacity(stack_partitions as usize);
        for i in 0..stack_partitions {
            let phi = minimum_cone + (i as f64 * (maximum_cone - minimum_cone)) / (stack_partitions - 1) as f64;
            sin_phi.push(phi.sin());
            cos_phi.push(phi.cos());
        }

        // Sin/cos theta for subdivisions (latitude lines)
        let mut sin_theta = Vec::with_capacity(subdivisions as usize);
        let mut cos_theta = Vec::with_capacity(subdivisions as usize);
        for i in 0..subdivisions {
            let theta = minimum_clock + (i as f64 * (maximum_clock - minimum_clock)) / (subdivisions - 1) as f64;
            sin_theta.push(theta.sin());
            cos_theta.push(theta.cos());
        }

        // Latitude lines on outer surface
        for i in 0..stack_partitions as usize {
            for j in 0..subdivisions as usize {
                positions.push(radii.x * sin_phi[i] * cos_theta[j]);
                positions.push(radii.y * sin_phi[i] * sin_theta[j]);
                positions.push(radii.z * cos_phi[i]);
            }
        }

        // Sin/cos phi for subdivisions (longitude lines)
        let mut sub_sin_phi = Vec::with_capacity(subdivisions as usize);
        let mut sub_cos_phi = Vec::with_capacity(subdivisions as usize);
        for i in 0..subdivisions {
            let phi = minimum_cone + (i as f64 * (maximum_cone - minimum_cone)) / (subdivisions - 1) as f64;
            sub_sin_phi.push(phi.sin());
            sub_cos_phi.push(phi.cos());
        }

        // Sin/cos theta for slice partitions
        let mut slice_sin_theta = Vec::with_capacity(slice_partitions as usize);
        let mut slice_cos_theta = Vec::with_capacity(slice_partitions as usize);
        for i in 0..slice_partitions {
            let theta = minimum_clock + (i as f64 * (maximum_clock - minimum_clock)) / (slice_partitions - 1) as f64;
            slice_sin_theta.push(theta.sin());
            slice_cos_theta.push(theta.cos());
        }

        // Longitude lines on outer surface
        for i in 0..subdivisions as usize {
            for j in 0..slice_partitions as usize {
                positions.push(radii.x * sub_sin_phi[i] * slice_cos_theta[j]);
                positions.push(radii.y * sub_sin_phi[i] * slice_sin_theta[j]);
                positions.push(radii.z * sub_cos_phi[i]);
            }
        }

        // Create line indices
        let mut indices = Vec::new();

        // Latitude lines
        for i in 0..stack_partitions as u32 {
            let top_offset = i * subdivisions as u32;
            for j in 0..(subdivisions - 1) as u32 {
                indices.push(top_offset + j);
                indices.push(top_offset + j + 1);
            }
        }

        // Longitude lines
        let offset = (stack_partitions * subdivisions) as u32;
        for i in 0..slice_partitions as u32 {
            for j in 0..(subdivisions - 1) as u32 {
                indices.push(offset + i + j * slice_partitions as u32);
                indices.push(offset + i + (j + 1) * slice_partitions as u32);
            }
        }

        let attributes = GeometryAttributes {
            position: Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                positions,
            )),
            ..Default::default()
        };

        Some(Geometry {
            attributes,
            indices,
            primitive_type: PrimitiveType::Lines,
        })
    }
}
