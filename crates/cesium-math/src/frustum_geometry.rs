//! FrustumGeometry and FrustumOutlineGeometry matching CesiumJS `Core/FrustumGeometry` and `Core/FrustumOutlineGeometry`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::{
    frustum::{OrthographicFrustum, PerspectiveFrustum},
    geometry::{ComponentDatatype, Geometry, GeometryAttribute, GeometryAttributes, PrimitiveType},
    BoundingSphere, Cartesian3, Matrix3, Quaternion, VertexFormat,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnyFrustum {
    Perspective(PerspectiveFrustum),
    Orthographic(OrthographicFrustum),
}

impl From<PerspectiveFrustum> for AnyFrustum {
    fn from(f: PerspectiveFrustum) -> Self {
        Self::Perspective(f)
    }
}

impl From<OrthographicFrustum> for AnyFrustum {
    fn from(f: OrthographicFrustum) -> Self {
        Self::Orthographic(f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrustumGeometry {
    pub frustum: AnyFrustum,
    pub origin: Cartesian3,
    pub orientation: Quaternion,
    pub vertex_format: VertexFormat,
    pub draw_near_plane: bool,
}

impl FrustumGeometry {
    pub fn new(
        frustum: impl Into<AnyFrustum>,
        origin: Cartesian3,
        orientation: Quaternion,
        vertex_format: Option<VertexFormat>,
    ) -> Self {
        Self {
            frustum: frustum.into(),
            origin,
            orientation,
            vertex_format: vertex_format.unwrap_or(VertexFormat::DEFAULT),
            draw_near_plane: true,
        }
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        let corners = compute_frustum_corners(self.frustum, self.origin, self.orientation)?;

        let n0 = corners[0];
        let n1 = corners[1];
        let n2 = corners[2];
        let n3 = corners[3];
        let f0 = corners[4];
        let f1 = corners[5];
        let f2 = corners[6];
        let f3 = corners[7];

        let faces: [[Cartesian3; 4]; 6] = [
            [n0, n1, n2, n3], // Near
            [f0, f1, f2, f3], // Far
            [f0, n0, n3, f3], // -X
            [f1, n1, n0, f0], // -Y
            [n1, f1, f2, n2], // +X
            [n2, f2, f3, n3], // +Y
        ];

        let active_faces = if self.draw_near_plane {
            &faces[..]
        } else {
            &faces[1..]
        };

        let num_faces = active_faces.len();
        let num_vertices = num_faces * 4;

        let mut positions = Vec::with_capacity(num_vertices * 3);
        let mut normals = if self.vertex_format.normal {
            Vec::with_capacity(num_vertices * 3)
        } else {
            Vec::new()
        };
        let mut tangents = if self.vertex_format.tangent {
            Vec::with_capacity(num_vertices * 3)
        } else {
            Vec::new()
        };
        let mut bitangents = if self.vertex_format.bitangent {
            Vec::with_capacity(num_vertices * 3)
        } else {
            Vec::new()
        };
        let mut st = if self.vertex_format.st {
            Vec::with_capacity(num_vertices * 2)
        } else {
            Vec::new()
        };

        for face in active_faces {
            let p0 = face[0];
            let p1 = face[1];
            let p2 = face[2];
            let p3 = face[3];

            for p in [p0, p1, p2, p3] {
                positions.push(p.x);
                positions.push(p.y);
                positions.push(p.z);
            }

            if self.vertex_format.normal || self.vertex_format.tangent || self.vertex_format.bitangent {
                let u = p1 - p0;
                let v = p3 - p0;
                let norm = u.cross(&v).normalize();
                let tan = u.normalize();
                let bitan = norm.cross(&tan).normalize();

                for _ in 0..4 {
                    if self.vertex_format.normal {
                        normals.push(norm.x);
                        normals.push(norm.y);
                        normals.push(norm.z);
                    }
                    if self.vertex_format.tangent {
                        tangents.push(tan.x);
                        tangents.push(tan.y);
                        tangents.push(tan.z);
                    }
                    if self.vertex_format.bitangent {
                        bitangents.push(bitan.x);
                        bitangents.push(bitan.y);
                        bitangents.push(bitan.z);
                    }
                }
            }

            if self.vertex_format.st {
                st.push(0.0); st.push(0.0);
                st.push(1.0); st.push(0.0);
                st.push(1.0); st.push(1.0);
                st.push(0.0); st.push(1.0);
            }
        }

        let mut indices = Vec::with_capacity(num_faces * 6);
        for i in 0..num_faces {
            let base = (i * 4) as u32;
            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 2);
            indices.push(base);
            indices.push(base + 2);
            indices.push(base + 3);
        }

        let mut attributes = GeometryAttributes::default();
        if self.vertex_format.position {
            attributes.position = Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                positions,
            ));
        }
        if self.vertex_format.normal {
            attributes.normal = Some(GeometryAttribute::new(
                ComponentDatatype::Float,
                3,
                normals,
            ));
        }
        if self.vertex_format.tangent {
            attributes.tangent = Some(GeometryAttribute::new(
                ComponentDatatype::Float,
                3,
                tangents,
            ));
        }
        if self.vertex_format.bitangent {
            attributes.bitangent = Some(GeometryAttribute::new(
                ComponentDatatype::Float,
                3,
                bitangents,
            ));
        }
        if self.vertex_format.st {
            attributes.st = Some(GeometryAttribute::new(
                ComponentDatatype::Float,
                2,
                st,
            ));
        }

        let bs = compute_bounding_sphere(&corners);
        Some(Geometry {
            attributes,
            indices,
            primitive_type: PrimitiveType::Triangles,
            bounding_sphere: Some(bs),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrustumOutlineGeometry {
    pub frustum: AnyFrustum,
    pub origin: Cartesian3,
    pub orientation: Quaternion,
    pub draw_near_plane: bool,
}

impl FrustumOutlineGeometry {
    pub fn new(
        frustum: impl Into<AnyFrustum>,
        origin: Cartesian3,
        orientation: Quaternion,
    ) -> Self {
        Self {
            frustum: frustum.into(),
            origin,
            orientation,
            draw_near_plane: true,
        }
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        let corners = compute_frustum_corners(self.frustum, self.origin, self.orientation)?;

        let mut positions = Vec::with_capacity(8 * 3);
        for c in &corners {
            positions.push(c.x);
            positions.push(c.y);
            positions.push(c.z);
        }

        let mut indices = Vec::with_capacity(24);

        // Near plane
        if self.draw_near_plane {
            indices.push(0); indices.push(1);
            indices.push(1); indices.push(2);
            indices.push(2); indices.push(3);
            indices.push(3); indices.push(0);
        }

        // Far plane
        indices.push(4); indices.push(5);
        indices.push(5); indices.push(6);
        indices.push(6); indices.push(7);
        indices.push(7); indices.push(4);

        // Connecting lines
        indices.push(0); indices.push(4);
        indices.push(1); indices.push(5);
        indices.push(2); indices.push(6);
        indices.push(3); indices.push(7);

        let attributes = GeometryAttributes {
            position: Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                positions,
            )),
            ..Default::default()
        };

        let bs = compute_bounding_sphere(&corners);
        Some(Geometry {
            attributes,
            indices,
            primitive_type: PrimitiveType::Lines,
            bounding_sphere: Some(bs),
        })
    }
}

fn compute_bounding_sphere(corners: &[Cartesian3; 8]) -> BoundingSphere {
    let mut center = Cartesian3::ZERO;
    for c in corners {
        center = center + *c;
    }
    center = center * (1.0 / 8.0);

    let mut max_radius_sq = 0.0f64;
    for c in corners {
        let dist_sq = c.distance_squared(&center);
        if dist_sq > max_radius_sq {
            max_radius_sq = dist_sq;
        }
    }

    BoundingSphere::new(center, max_radius_sq.sqrt())
}

fn compute_frustum_corners(
    frustum: AnyFrustum,
    origin: Cartesian3,
    orientation: Quaternion,
) -> Option<[Cartesian3; 8]> {
    let rot_matrix = Matrix3::from_quaternion(&orientation);

    match frustum {
        AnyFrustum::Perspective(p) => {
            if p.near <= 0.0 || p.near >= p.far || p.aspect_ratio <= 0.0 || p.fov <= 0.0 {
                return None;
            }

            let fovy = if p.aspect_ratio <= 1.0 {
                p.fov
            } else {
                2.0 * ((p.fov * 0.5).tan() / p.aspect_ratio).atan()
            };

            let half_tan_y = (fovy * 0.5).tan();
            let half_tan_x = half_tan_y * p.aspect_ratio;

            let h_near = p.near * half_tan_y;
            let w_near = p.near * half_tan_x;

            let h_far = p.far * half_tan_y;
            let w_far = p.far * half_tan_x;

            let local_corners = [
                Cartesian3::new(-w_near, -h_near, p.near),
                Cartesian3::new(w_near, -h_near, p.near),
                Cartesian3::new(w_near, h_near, p.near),
                Cartesian3::new(-w_near, h_near, p.near),
                Cartesian3::new(-w_far, -h_far, p.far),
                Cartesian3::new(w_far, -h_far, p.far),
                Cartesian3::new(w_far, h_far, p.far),
                Cartesian3::new(-w_far, h_far, p.far),
            ];

            let mut corners = [Cartesian3::ZERO; 8];
            for i in 0..8 {
                corners[i] = origin + rot_matrix.multiply_by_vector(local_corners[i]);
            }

            Some(corners)
        }
        AnyFrustum::Orthographic(o) => {
            if o.near <= 0.0 || o.near >= o.far || o.aspect_ratio <= 0.0 || o.width <= 0.0 {
                return None;
            }

            let half_w = o.width * 0.5;
            let half_h = half_w / o.aspect_ratio;

            let local_corners = [
                Cartesian3::new(-half_w, -half_h, o.near),
                Cartesian3::new(half_w, -half_h, o.near),
                Cartesian3::new(half_w, half_h, o.near),
                Cartesian3::new(-half_w, half_h, o.near),
                Cartesian3::new(-half_w, -half_h, o.far),
                Cartesian3::new(half_w, -half_h, o.far),
                Cartesian3::new(half_w, half_h, o.far),
                Cartesian3::new(-half_w, half_h, o.far),
            ];

            let mut corners = [Cartesian3::ZERO; 8];
            for i in 0..8 {
                corners[i] = origin + rot_matrix.multiply_by_vector(local_corners[i]);
            }

            Some(corners)
        }
    }
}
