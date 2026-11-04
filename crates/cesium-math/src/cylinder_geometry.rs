//! CylinderGeometry and CylinderOutlineGeometry matching CesiumJS `Core/CylinderGeometry` and `Core/CylinderOutlineGeometry`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::{
    core_math::TWO_PI,
    geometry::{ComponentDatatype, Geometry, GeometryAttribute, GeometryAttributes, PrimitiveType},
    VertexFormat,
};

// ============================================================================
// CylinderGeometry
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CylinderGeometry {
    pub length: f64,
    pub top_radius: f64,
    pub bottom_radius: f64,
    pub slices: u32,
    pub vertex_format: VertexFormat,
}

impl CylinderGeometry {
    pub fn new(
        length: f64,
        top_radius: f64,
        bottom_radius: f64,
        slices: Option<u32>,
        vertex_format: Option<VertexFormat>,
    ) -> Self {
        Self {
            length,
            top_radius,
            bottom_radius,
            slices: slices.unwrap_or(128).max(3),
            vertex_format: vertex_format.unwrap_or(VertexFormat::DEFAULT),
        }
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        let length = self.length;
        let top_radius = self.top_radius;
        let bottom_radius = self.bottom_radius;
        let slices = self.slices;
        let vertex_format = self.vertex_format;

        if length <= 0.0 || top_radius < 0.0 || bottom_radius < 0.0 || (top_radius == 0.0 && bottom_radius == 0.0) {
            return None;
        }

        let top_z = length * 0.5;
        let bottom_z = -top_z;

        let two_slices = slices + slices;
        let three_slices = slices + two_slices;
        let num_vertices = (two_slices + two_slices) as usize;

        let mut positions = vec![0.0; num_vertices * 3];
        let mut index = 0;
        let mut tb_index = 0;
        let bottom_offset = (two_slices * 3) as usize;
        let top_offset = ((two_slices + slices) * 3) as usize;

        for i in 0..slices {
            let angle = (i as f64 / slices as f64) * TWO_PI;
            let (sin_a, cos_a) = angle.sin_cos();
            let bx = cos_a * bottom_radius;
            let by = sin_a * bottom_radius;
            let tx = cos_a * top_radius;
            let ty = sin_a * top_radius;

            positions[tb_index + bottom_offset] = bx;
            positions[tb_index + bottom_offset + 1] = by;
            positions[tb_index + bottom_offset + 2] = bottom_z;

            positions[tb_index + top_offset] = tx;
            positions[tb_index + top_offset + 1] = ty;
            positions[tb_index + top_offset + 2] = top_z;
            tb_index += 3;

            positions[index] = bx;
            positions[index + 1] = by;
            positions[index + 2] = bottom_z;
            positions[index + 3] = tx;
            positions[index + 4] = ty;
            positions[index + 5] = top_z;
            index += 6;
        }

        let mut attributes = GeometryAttributes::default();
        if vertex_format.position {
            attributes.position = Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                positions,
            ));
        }

        let num_indices = ((12 * slices) - 12) as usize;
        let mut indices = Vec::with_capacity(num_indices);
        let mut j = 0;
        for _ in 0..slices - 1 {
            indices.push(j);
            indices.push(j + 2);
            indices.push(j + 3);

            indices.push(j);
            indices.push(j + 3);
            indices.push(j + 1);

            j += 2;
        }

        indices.push(two_slices - 2);
        indices.push(0);
        indices.push(1);
        indices.push(two_slices - 2);
        indices.push(1);
        indices.push(two_slices - 1);

        for i in 1..slices - 1 {
            indices.push(two_slices + i + 1);
            indices.push(two_slices + i);
            indices.push(two_slices);
        }

        for i in 1..slices - 1 {
            indices.push(three_slices);
            indices.push(three_slices + i);
            indices.push(three_slices + i + 1);
        }

        Some(Geometry {
            attributes,
            indices,
            primitive_type: PrimitiveType::Triangles,
        })
    }
}

// ============================================================================
// CylinderOutlineGeometry
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CylinderOutlineGeometry {
    pub length: f64,
    pub top_radius: f64,
    pub bottom_radius: f64,
    pub slices: u32,
    pub number_of_vertical_lines: u32,
}

impl CylinderOutlineGeometry {
    pub fn new(
        length: f64,
        top_radius: f64,
        bottom_radius: f64,
        slices: Option<u32>,
        number_of_vertical_lines: Option<u32>,
    ) -> Self {
        Self {
            length,
            top_radius,
            bottom_radius,
            slices: slices.unwrap_or(128).max(3),
            number_of_vertical_lines: number_of_vertical_lines.unwrap_or(16),
        }
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        let length = self.length;
        let top_radius = self.top_radius;
        let bottom_radius = self.bottom_radius;
        let slices = self.slices;
        let number_of_vertical_lines = self.number_of_vertical_lines;

        if length <= 0.0 || top_radius < 0.0 || bottom_radius < 0.0 || (top_radius == 0.0 && bottom_radius == 0.0) {
            return None;
        }

        let top_z = length * 0.5;
        let bottom_z = -top_z;

        let num_vertices = (slices * 2) as usize;
        let mut positions = vec![0.0; num_vertices * 3];

        for i in 0..slices {
            let angle = (i as f64 / slices as f64) * TWO_PI;
            let (sin_a, cos_a) = angle.sin_cos();

            let bx = cos_a * bottom_radius;
            let by = sin_a * bottom_radius;
            let tx = cos_a * top_radius;
            let ty = sin_a * top_radius;

            let idx = (i * 3) as usize;
            positions[idx] = bx;
            positions[idx + 1] = by;
            positions[idx + 2] = bottom_z;

            let top_idx = ((i + slices) * 3) as usize;
            positions[top_idx] = tx;
            positions[top_idx + 1] = ty;
            positions[top_idx + 2] = top_z;
        }

        let mut indices = Vec::new();

        // Bottom circle
        for i in 0..slices - 1 {
            indices.push(i);
            indices.push(i + 1);
        }
        indices.push(slices - 1);
        indices.push(0);

        // Top circle
        for i in 0..slices - 1 {
            indices.push(slices + i);
            indices.push(slices + i + 1);
        }
        indices.push(slices + slices - 1);
        indices.push(slices);

        // Vertical lines
        if number_of_vertical_lines > 0 {
            let num_lines = number_of_vertical_lines.min(slices);
            let step = (slices as f64 / num_lines as f64).round() as u32;
            let mut i = 0;
            while i < slices {
                indices.push(i);
                indices.push(slices + i);
                i += step.max(1);
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
