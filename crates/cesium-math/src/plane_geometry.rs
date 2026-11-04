//! PlaneGeometry and PlaneOutlineGeometry matching CesiumJS `Core/PlaneGeometry` and `Core/PlaneOutlineGeometry`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::{
    geometry::{ComponentDatatype, Geometry, GeometryAttribute, GeometryAttributes, PrimitiveType},
    VertexFormat,
};

// ============================================================================
// PlaneGeometry
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlaneGeometry {
    pub vertex_format: VertexFormat,
}

impl Default for PlaneGeometry {
    fn default() -> Self {
        Self {
            vertex_format: VertexFormat::DEFAULT,
        }
    }
}

impl PlaneGeometry {
    pub const fn new(vertex_format: Option<VertexFormat>) -> Self {
        Self {
            vertex_format: match vertex_format {
                Some(vf) => vf,
                None => VertexFormat::DEFAULT,
            },
        }
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        let vf = self.vertex_format;
        if !vf.position {
            return None;
        }

        let mut attributes = GeometryAttributes::default();

        let min_x = -0.5;
        let min_y = -0.5;
        let max_x = 0.5;
        let max_y = 0.5;

        let positions = vec![
            min_x, min_y, 0.0,
            max_x, min_y, 0.0,
            max_x, max_y, 0.0,
            min_x, max_y, 0.0,
        ];
        attributes.position = Some(GeometryAttribute::new(
            ComponentDatatype::Double,
            3,
            positions,
        ));

        if vf.normal {
            let normals = vec![
                0.0, 0.0, 1.0,
                0.0, 0.0, 1.0,
                0.0, 0.0, 1.0,
                0.0, 0.0, 1.0,
            ];
            attributes.normal = Some(GeometryAttribute::new(
                ComponentDatatype::Float,
                3,
                normals,
            ));
        }

        if vf.st {
            let tex_coords = vec![
                0.0, 0.0,
                1.0, 0.0,
                1.0, 1.0,
                0.0, 1.0,
            ];
            attributes.st = Some(GeometryAttribute::new(
                ComponentDatatype::Float,
                2,
                tex_coords,
            ));
        }

        if vf.tangent {
            let tangents = vec![
                1.0, 0.0, 0.0,
                1.0, 0.0, 0.0,
                1.0, 0.0, 0.0,
                1.0, 0.0, 0.0,
            ];
            attributes.tangent = Some(GeometryAttribute::new(
                ComponentDatatype::Float,
                3,
                tangents,
            ));
        }

        if vf.bitangent {
            let bitangents = vec![
                0.0, 1.0, 0.0,
                0.0, 1.0, 0.0,
                0.0, 1.0, 0.0,
                0.0, 1.0, 0.0,
            ];
            attributes.bitangent = Some(GeometryAttribute::new(
                ComponentDatatype::Float,
                3,
                bitangents,
            ));
        }

        let indices = vec![0, 1, 2, 0, 2, 3];

        Some(Geometry {
            attributes,
            indices,
            primitive_type: PrimitiveType::Triangles,
            bounding_sphere: None,
        })
    }
}

// ============================================================================
// PlaneOutlineGeometry
// ============================================================================

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlaneOutlineGeometry;

impl PlaneOutlineGeometry {
    pub const fn new() -> Self {
        Self
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        let min_x = -0.5;
        let min_y = -0.5;
        let max_x = 0.5;
        let max_y = 0.5;

        let positions = vec![
            min_x, min_y, 0.0,
            max_x, min_y, 0.0,
            max_x, max_y, 0.0,
            min_x, max_y, 0.0,
        ];
        let attributes = GeometryAttributes {
            position: Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                positions,
            )),
            ..Default::default()
        };

        let indices = vec![0, 1, 1, 2, 2, 3, 3, 0];

        Some(Geometry {
            attributes,
            indices,
            primitive_type: PrimitiveType::Lines,
            bounding_sphere: None,
        })
    }
}
