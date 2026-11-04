//! Geometry processing pipeline matching CesiumJS `Core/GeometryPipeline`.

use crate::cartesian::Cartesian3;
use crate::core_math::EPSILON10;
use crate::geometry::{ComponentDatatype, Geometry, GeometryAttribute, PrimitiveType};

/// Geometry pipeline functions.
pub struct GeometryPipeline;

impl GeometryPipeline {
    /// Computes per-vertex normals for a triangle mesh by accumulating face normals.
    pub fn compute_normal(geometry: &mut Geometry) {
        if geometry.primitive_type != PrimitiveType::Triangles {
            return;
        }

        let Some(pos_attr) = &geometry.attributes.position else {
            return;
        };

        let num_vertices = pos_attr.values.len() / 3;
        if num_vertices == 0 {
            return;
        }

        let num_indices = geometry.indices.len();
        if num_indices < 3 || !num_indices.is_multiple_of(3) {
            return;
        }

        let vertices = &pos_attr.values;
        let indices = &geometry.indices;

        let mut normals = vec![Cartesian3::ZERO; num_vertices];

        for chunk in indices.as_chunks::<3>().0 {
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;

            if i0 >= num_vertices || i1 >= num_vertices || i2 >= num_vertices {
                continue;
            }

            let v0 = Cartesian3::new(vertices[i0 * 3], vertices[i0 * 3 + 1], vertices[i0 * 3 + 2]);
            let v1 = Cartesian3::new(vertices[i1 * 3], vertices[i1 * 3 + 1], vertices[i1 * 3 + 2]);
            let v2 = Cartesian3::new(vertices[i2 * 3], vertices[i2 * 3 + 1], vertices[i2 * 3 + 2]);

            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let face_normal = edge1.cross(&edge2);

            normals[i0] = normals[i0] + face_normal;
            normals[i1] = normals[i1] + face_normal;
            normals[i2] = normals[i2] + face_normal;
        }

        let mut normal_values = Vec::with_capacity(num_vertices * 3);
        for mut norm in normals {
            if norm.equals_epsilon(&Cartesian3::ZERO, Some(EPSILON10), Some(EPSILON10)) {
                norm = Cartesian3::UNIT_Z;
            } else {
                norm = norm.normalize();
            }
            normal_values.extend_from_slice(&[norm.x, norm.y, norm.z]);
        }

        geometry.attributes.normal = Some(GeometryAttribute::new(
            ComponentDatatype::Float,
            3,
            normal_values,
        ));
    }

    /// Computes per-vertex tangents and bitangents for a triangle mesh.
    pub fn compute_tangent_and_bitangent(geometry: &mut Geometry) {
        if geometry.primitive_type != PrimitiveType::Triangles {
            return;
        }

        let (Some(pos_attr), Some(norm_attr), Some(st_attr)) = (
            &geometry.attributes.position,
            &geometry.attributes.normal,
            &geometry.attributes.st,
        ) else {
            return;
        };

        let num_vertices = pos_attr.values.len() / 3;
        let num_indices = geometry.indices.len();
        if num_vertices == 0 || num_indices < 3 || !num_indices.is_multiple_of(3) {
            return;
        }

        let vertices = &pos_attr.values;
        let normals = &norm_attr.values;
        let st = &st_attr.values;
        let indices = &geometry.indices;

        let mut tan1 = vec![Cartesian3::ZERO; num_vertices];

        for chunk in indices.as_chunks::<3>().0 {
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;

            if i0 >= num_vertices || i1 >= num_vertices || i2 >= num_vertices {
                continue;
            }

            let v0 = Cartesian3::new(vertices[i0 * 3], vertices[i0 * 3 + 1], vertices[i0 * 3 + 2]);
            let v1 = Cartesian3::new(vertices[i1 * 3], vertices[i1 * 3 + 1], vertices[i1 * 3 + 2]);
            let v2 = Cartesian3::new(vertices[i2 * 3], vertices[i2 * 3 + 1], vertices[i2 * 3 + 2]);

            let w0_u = st[i0 * 2];
            let w0_v = st[i0 * 2 + 1];
            let w1_u = st[i1 * 2];
            let w1_v = st[i1 * 2 + 1];
            let w2_u = st[i2 * 2];
            let w2_v = st[i2 * 2 + 1];

            let x1 = v1.x - v0.x;
            let x2 = v2.x - v0.x;
            let y1 = v1.y - v0.y;
            let y2 = v2.y - v0.y;
            let z1 = v1.z - v0.z;
            let z2 = v2.z - v0.z;

            let s1 = w1_u - w0_u;
            let s2 = w2_u - w0_u;
            let t1 = w1_v - w0_v;
            let t2 = w2_v - w0_v;

            let denom = s1 * t2 - s2 * t1;
            let r = if denom.abs() > 1e-12 { 1.0 / denom } else { 0.0 };

            let sdir = Cartesian3::new(
                (t2 * x1 - t1 * x2) * r,
                (t2 * y1 - t1 * y2) * r,
                (t2 * z1 - t1 * z2) * r,
            );

            tan1[i0] = tan1[i0] + sdir;
            tan1[i1] = tan1[i1] + sdir;
            tan1[i2] = tan1[i2] + sdir;
        }

        let mut tangent_values = Vec::with_capacity(num_vertices * 3);
        let mut bitangent_values = Vec::with_capacity(num_vertices * 3);

        for i in 0..num_vertices {
            let n = Cartesian3::new(normals[i * 3], normals[i * 3 + 1], normals[i * 3 + 2]);
            let t = tan1[i];

            let scalar = n.dot(&t);
            let mut tangent = (t - n * scalar).normalize();
            if tangent.equals_epsilon(&Cartesian3::ZERO, Some(EPSILON10), Some(EPSILON10)) {
                tangent = Cartesian3::UNIT_X;
            }

            let bitangent = n.cross(&tangent).normalize();

            tangent_values.extend_from_slice(&[tangent.x, tangent.y, tangent.z]);
            bitangent_values.extend_from_slice(&[bitangent.x, bitangent.y, bitangent.z]);
        }

        geometry.attributes.tangent = Some(GeometryAttribute::new(
            ComponentDatatype::Float,
            3,
            tangent_values,
        ));
        geometry.attributes.bitangent = Some(GeometryAttribute::new(
            ComponentDatatype::Float,
            3,
            bitangent_values,
        ));
    }
}
