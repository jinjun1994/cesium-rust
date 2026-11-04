//! BoxGeometry and BoxOutlineGeometry matching CesiumJS `Core/BoxGeometry` and `Core/BoxOutlineGeometry`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::{
    geometry::{ComponentDatatype, Geometry, GeometryAttribute, GeometryAttributes, PrimitiveType},
    AxisAlignedBoundingBox, Cartesian3, VertexFormat,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxGeometry {
    pub minimum: Cartesian3,
    pub maximum: Cartesian3,
    pub vertex_format: VertexFormat,
}

impl BoxGeometry {
    pub fn new(minimum: Cartesian3, maximum: Cartesian3, vertex_format: Option<VertexFormat>) -> Self {
        Self {
            minimum,
            maximum,
            vertex_format: vertex_format.unwrap_or(VertexFormat::DEFAULT),
        }
    }

    pub fn from_dimensions(dimensions: Cartesian3, vertex_format: Option<VertexFormat>) -> Self {
        let half = dimensions * 0.5;
        Self::new(-half, half, vertex_format)
    }

    pub fn from_axis_aligned_bounding_box(
        aabb: &AxisAlignedBoundingBox,
        vertex_format: Option<VertexFormat>,
    ) -> Self {
        Self::new(aabb.minimum.into(), aabb.maximum.into(), vertex_format)
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        let min = self.minimum;
        let max = self.maximum;

        if min.equals(&max) {
            return None;
        }

        let mut attributes = GeometryAttributes::default();
        let vertex_format = self.vertex_format;

        if vertex_format.position
            && (vertex_format.st
                || vertex_format.normal
                || vertex_format.tangent
                || vertex_format.bitangent)
        {
            // 24 vertices (6 faces x 4 vertices)
            let mut positions = vec![0.0; 6 * 4 * 3];

            // +z face
            let mut offset = 0;
            let faces = [
                // +z
                [min.x, min.y, max.z, max.x, min.y, max.z, max.x, max.y, max.z, min.x, max.y, max.z],
                // -z
                [min.x, min.y, min.z, max.x, min.y, min.z, max.x, max.y, min.z, min.x, max.y, min.z],
                // +x
                [max.x, min.y, min.z, max.x, max.y, min.z, max.x, max.y, max.z, max.x, min.y, max.z],
                // -x
                [min.x, min.y, min.z, min.x, max.y, min.z, min.x, max.y, max.z, min.x, min.y, max.z],
                // +y
                [min.x, max.y, min.z, max.x, max.y, min.z, max.x, max.y, max.z, min.x, max.y, max.z],
                // -y
                [min.x, min.y, min.z, max.x, min.y, min.z, max.x, min.y, max.z, min.x, min.y, max.z],
            ];

            for face in faces {
                positions[offset..offset + 12].copy_from_slice(&face);
                offset += 12;
            }

            attributes.position = Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                positions,
            ));

            if vertex_format.normal {
                let mut normals = vec![0.0; 6 * 4 * 3];
                let face_normals = [
                    [0.0, 0.0, 1.0],
                    [0.0, 0.0, -1.0],
                    [1.0, 0.0, 0.0],
                    [-1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [0.0, -1.0, 0.0],
                ];
                let mut n_idx = 0;
                for n in face_normals {
                    for _ in 0..4 {
                        normals[n_idx] = n[0];
                        normals[n_idx + 1] = n[1];
                        normals[n_idx + 2] = n[2];
                        n_idx += 3;
                    }
                }
                attributes.normal = Some(GeometryAttribute::new(
                    ComponentDatatype::Float,
                    3,
                    normals,
                ));
            }

            if vertex_format.st {
                let mut uvs = vec![0.0; 6 * 4 * 2];
                let mut uv_idx = 0;
                for _ in 0..6 {
                    uvs[uv_idx..uv_idx + 8].copy_from_slice(&[0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0]);
                    uv_idx += 8;
                }
                attributes.st = Some(GeometryAttribute::new(ComponentDatatype::Float, 2, uvs));
            }

            let mut indices = vec![0u32; 6 * 2 * 3];
            for i in 0..6 {
                let base = (i * 4) as u32;
                let idx = i * 6;
                indices[idx] = base;
                indices[idx + 1] = base + 1;
                indices[idx + 2] = base + 2;
                indices[idx + 3] = base;
                indices[idx + 4] = base + 2;
                indices[idx + 5] = base + 3;
            }

            Some(Geometry {
                attributes,
                indices,
                primitive_type: PrimitiveType::Triangles,
                bounding_sphere: None,
            })
        } else {
            // Optimized position only: 8 corners, 12 triangles
            let positions = vec![
                min.x, min.y, min.z, // 0
                max.x, min.y, min.z, // 1
                max.x, max.y, min.z, // 2
                min.x, max.y, min.z, // 3
                min.x, min.y, max.z, // 4
                max.x, min.y, max.z, // 5
                max.x, max.y, max.z, // 6
                min.x, max.y, max.z, // 7
            ];

            attributes.position = Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                positions,
            ));

            #[rustfmt::skip]
            let indices = vec![
                4, 5, 6, 4, 6, 7, // +z
                1, 0, 3, 1, 3, 2, // -z
                1, 2, 6, 1, 6, 5, // +x
                0, 4, 7, 0, 7, 3, // -x
                3, 7, 6, 3, 6, 2, // +y
                0, 1, 5, 0, 5, 4, // -y
            ];

            Some(Geometry {
                attributes,
                indices,
                primitive_type: PrimitiveType::Triangles,
                bounding_sphere: None,
            })
        }
    }
}

// ============================================================================
// BoxOutlineGeometry
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxOutlineGeometry {
    pub minimum: Cartesian3,
    pub maximum: Cartesian3,
}

impl BoxOutlineGeometry {
    pub fn new(minimum: Cartesian3, maximum: Cartesian3) -> Self {
        Self { minimum, maximum }
    }

    pub fn from_dimensions(dimensions: Cartesian3) -> Self {
        let half = dimensions * 0.5;
        Self::new(-half, half)
    }

    pub fn from_axis_aligned_bounding_box(aabb: &AxisAlignedBoundingBox) -> Self {
        Self::new(aabb.minimum.into(), aabb.maximum.into())
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        let min = self.minimum;
        let max = self.maximum;

        if min.equals(&max) {
            return None;
        }

        let positions = vec![
            min.x, min.y, min.z, // 0
            max.x, min.y, min.z, // 1
            max.x, max.y, min.z, // 2
            min.x, max.y, min.z, // 3
            min.x, min.y, max.z, // 4
            max.x, min.y, max.z, // 5
            max.x, max.y, max.z, // 6
            min.x, max.y, max.z, // 7
        ];

        let attributes = GeometryAttributes {
            position: Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                positions,
            )),
            ..Default::default()
        };

        #[rustfmt::skip]
        let indices = vec![
            4, 5, 5, 6, 6, 7, 7, 4, // top
            0, 1, 1, 2, 2, 3, 3, 0, // bottom
            0, 4, 1, 5, 2, 6, 3, 7, // vertical
        ];

        Some(Geometry {
            attributes,
            indices,
            primitive_type: PrimitiveType::Lines,
            bounding_sphere: None,
        })
    }
}
