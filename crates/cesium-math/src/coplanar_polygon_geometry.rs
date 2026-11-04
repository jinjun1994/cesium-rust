//! CoplanarPolygonGeometry and CoplanarPolygonOutlineGeometry matching
//! CesiumJS `Core/CoplanarPolygonGeometry` and `Core/CoplanarPolygonOutlineGeometry`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::{
    core_math::EPSILON10,
    geometry::{ComponentDatatype, Geometry, GeometryAttribute, GeometryAttributes, PrimitiveType},
    BoundingSphere, Cartesian3, Ellipsoid, OrientedBoundingBox, VertexFormat,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PolygonHierarchy {
    pub positions: Vec<Cartesian3>,
    pub holes: Vec<PolygonHierarchy>,
}

impl PolygonHierarchy {
    pub fn new(positions: Vec<Cartesian3>) -> Self {
        Self {
            positions,
            holes: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CoplanarPolygonGeometry {
    pub polygon_hierarchy: PolygonHierarchy,
    pub vertex_format: VertexFormat,
    pub st_rotation: f64,
    pub ellipsoid: Ellipsoid,
}

impl CoplanarPolygonGeometry {
    pub fn new(
        polygon_hierarchy: PolygonHierarchy,
        vertex_format: Option<VertexFormat>,
        st_rotation: Option<f64>,
        ellipsoid: Option<Ellipsoid>,
    ) -> Self {
        Self {
            polygon_hierarchy,
            vertex_format: vertex_format.unwrap_or(VertexFormat::DEFAULT),
            st_rotation: st_rotation.unwrap_or(0.0),
            ellipsoid: ellipsoid.unwrap_or(Ellipsoid::WGS84),
        }
    }

    pub fn from_positions(
        positions: Vec<Cartesian3>,
        vertex_format: Option<VertexFormat>,
        st_rotation: Option<f64>,
        ellipsoid: Option<Ellipsoid>,
    ) -> Self {
        Self::new(
            PolygonHierarchy::new(positions),
            vertex_format,
            st_rotation,
            ellipsoid,
        )
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        let cleaned = clean_positions(&self.polygon_hierarchy.positions)?;
        if cleaned.len() < 3 {
            return None;
        }

        let dvecs: Vec<glam::DVec3> = cleaned.iter().map(|&p| p.into()).collect();
        let obb = OrientedBoundingBox::from_points(&dvecs).ok()?;
        let half_axes = obb.half_axes;

        let x_axis = half_axes.x_axis;
        let y_axis = half_axes.y_axis;
        let z_axis = half_axes.z_axis;

        let x_mag = x_axis.length();
        let y_mag = y_axis.length();
        let z_mag = z_axis.length();

        let min_mag = x_mag.min(y_mag).min(z_mag);
        if (x_mag == 0.0 && (y_mag == 0.0 || z_mag == 0.0)) || (y_mag == 0.0 && z_mag == 0.0) {
            return None;
        }

        let mut plane_axis1 = x_axis;
        let mut plane_axis2 = y_axis;

        if min_mag == y_mag || min_mag == z_mag {
            plane_axis1 = x_axis;
        }
        if min_mag == x_mag {
            plane_axis1 = y_axis;
        } else if min_mag == z_mag {
            plane_axis2 = y_axis;
        }
        if min_mag == x_mag || min_mag == y_mag {
            plane_axis2 = z_axis;
        }

        let axis1 = Cartesian3::from(plane_axis1.normalize());
        let axis2 = Cartesian3::from(plane_axis2.normalize());
        let mut normal = axis1.cross(&axis2).normalize();

        // Orient normal to match ellipsoid geodetic surface normal at center
        let center = Cartesian3::from(obb.center);
        if let Some(surf_norm) = self.ellipsoid.geodetic_surface_normal(obb.center) {
            let surf_norm_c3 = Cartesian3::from(surf_norm);
            if normal.dot(&surf_norm_c3) < 0.0 {
                normal = -normal;
            }
        }

        let n = cleaned.len();
        let mut points_2d = Vec::with_capacity(n);
        for &p in &cleaned {
            let d = p - center;
            let u = d.dot(&axis1);
            let v = d.dot(&axis2);
            points_2d.push([u, v]);
        }

        let indices = triangulate_2d(&points_2d);
        if indices.len() < 3 {
            return None;
        }

        let mut positions = Vec::with_capacity(n * 3);
        for &p in &cleaned {
            positions.push(p.x);
            positions.push(p.y);
            positions.push(p.z);
        }

        let mut attributes = GeometryAttributes {
            position: Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                positions,
            )),
            ..Default::default()
        };

        if self.vertex_format.normal {
            let mut normals = Vec::with_capacity(n * 3);
            for _ in 0..n {
                normals.push(normal.x);
                normals.push(normal.y);
                normals.push(normal.z);
            }
            attributes.normal = Some(GeometryAttribute::new(
                ComponentDatatype::Float,
                3,
                normals,
            ));
        }

        if self.vertex_format.tangent {
            let mut tangents = Vec::with_capacity(n * 3);
            for _ in 0..n {
                tangents.push(axis1.x);
                tangents.push(axis1.y);
                tangents.push(axis1.z);
            }
            attributes.tangent = Some(GeometryAttribute::new(
                ComponentDatatype::Float,
                3,
                tangents,
            ));
        }

        if self.vertex_format.bitangent {
            let bitan = normal.cross(&axis1).normalize();
            let mut bitangents = Vec::with_capacity(n * 3);
            for _ in 0..n {
                bitangents.push(bitan.x);
                bitangents.push(bitan.y);
                bitangents.push(bitan.z);
            }
            attributes.bitangent = Some(GeometryAttribute::new(
                ComponentDatatype::Float,
                3,
                bitangents,
            ));
        }

        if self.vertex_format.st {
            let mut min_u = f64::INFINITY;
            let mut max_u = f64::NEG_INFINITY;
            let mut min_v = f64::INFINITY;
            let mut max_v = f64::NEG_INFINITY;
            for &[u, v] in &points_2d {
                min_u = min_u.min(u);
                max_u = max_u.max(u);
                min_v = min_v.min(v);
                max_v = max_v.max(v);
            }
            let range_u = (max_u - min_u).max(1e-6);
            let range_v = (max_v - min_v).max(1e-6);

            let mut st = Vec::with_capacity(n * 2);
            for &[u, v] in &points_2d {
                st.push((u - min_u) / range_u);
                st.push((v - min_v) / range_v);
            }
            attributes.st = Some(GeometryAttribute::new(
                ComponentDatatype::Float,
                2,
                st,
            ));
        }

        let radius = obb.half_axes.x_axis.length().max(obb.half_axes.y_axis.length());
        let bs = BoundingSphere::new(center, radius);
        Some(Geometry {
            attributes,
            indices,
            primitive_type: PrimitiveType::Triangles,
            bounding_sphere: Some(bs),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CoplanarPolygonOutlineGeometry {
    pub polygon_hierarchy: PolygonHierarchy,
}

impl CoplanarPolygonOutlineGeometry {
    pub fn new(polygon_hierarchy: PolygonHierarchy) -> Self {
        Self { polygon_hierarchy }
    }

    pub fn from_positions(positions: Vec<Cartesian3>) -> Self {
        Self::new(PolygonHierarchy::new(positions))
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        let cleaned = clean_positions(&self.polygon_hierarchy.positions)?;
        if cleaned.len() < 3 {
            return None;
        }

        let dvecs: Vec<glam::DVec3> = cleaned.iter().map(|&p| p.into()).collect();
        let obb = OrientedBoundingBox::from_points(&dvecs).ok()?;
        let x_mag = obb.half_axes.x_axis.length();
        let y_mag = obb.half_axes.y_axis.length();
        let z_mag = obb.half_axes.z_axis.length();

        if (x_mag == 0.0 && (y_mag == 0.0 || z_mag == 0.0)) || (y_mag == 0.0 && z_mag == 0.0) {
            return None;
        }

        let n = cleaned.len();
        let mut positions = Vec::with_capacity(n * 3);
        for &p in &cleaned {
            positions.push(p.x);
            positions.push(p.y);
            positions.push(p.z);
        }

        let mut indices = Vec::with_capacity(n * 2);
        for i in 0..n {
            indices.push(i as u32);
            indices.push(((i + 1) % n) as u32);
        }

        let attributes = GeometryAttributes {
            position: Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                positions,
            )),
            ..Default::default()
        };

        let radius = obb.half_axes.x_axis.length().max(obb.half_axes.y_axis.length());
        let bs = BoundingSphere::new(Cartesian3::from(obb.center), radius);
        Some(Geometry {
            attributes,
            indices,
            primitive_type: PrimitiveType::Lines,
            bounding_sphere: Some(bs),
        })
    }
}

fn clean_positions(positions: &[Cartesian3]) -> Option<Vec<Cartesian3>> {
    if positions.len() < 3 {
        return None;
    }
    let mut cleaned = Vec::with_capacity(positions.len());
    for &p in positions {
        if let Some(&last) = cleaned.last() {
            if p.equals_epsilon(&last, Some(EPSILON10), Some(EPSILON10)) {
                continue;
            }
        }
        cleaned.push(p);
    }
    if let (Some(&first), Some(&last)) = (cleaned.first(), cleaned.last()) {
        if cleaned.len() > 1 && first.equals_epsilon(&last, Some(EPSILON10), Some(EPSILON10)) {
            cleaned.pop();
        }
    }
    if cleaned.len() < 3 {
        None
    } else {
        Some(cleaned)
    }
}

fn point_in_triangle(p: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> bool {
    let sign1 = (p[0] - b[0]) * (a[1] - b[1]) - (a[0] - b[0]) * (p[1] - b[1]);
    let sign2 = (p[0] - c[0]) * (b[1] - c[1]) - (b[0] - c[0]) * (p[1] - c[1]);
    let sign3 = (p[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (p[1] - a[1]);

    let has_neg = sign1 < 0.0 || sign2 < 0.0 || sign3 < 0.0;
    let has_pos = sign1 > 0.0 || sign2 > 0.0 || sign3 > 0.0;

    !(has_neg && has_pos)
}

fn triangulate_2d(points: &[[f64; 2]]) -> Vec<u32> {
    let n = points.len();
    if n < 3 {
        return Vec::new();
    }
    if n == 3 {
        return vec![0, 1, 2];
    }
    if n == 4 {
        return vec![0, 1, 2, 0, 2, 3];
    }

    let mut indices = Vec::with_capacity((n - 2) * 3);
    let mut remaining: Vec<u32> = (0..n as u32).collect();

    let area: f64 = (0..n)
        .map(|i| {
            let j = (i + 1) % n;
            points[i][0] * points[j][1] - points[j][0] * points[i][1]
        })
        .sum();
    let ccw = area > 0.0;

    let mut count = 2 * remaining.len();
    while remaining.len() > 2 && count > 0 {
        count -= 1;
        let len = remaining.len();
        let mut ear_found = false;
        for i in 0..len {
            let prev = remaining[(i + len - 1) % len] as usize;
            let curr = remaining[i] as usize;
            let next = remaining[(i + 1) % len] as usize;

            let a = points[prev];
            let b = points[curr];
            let c = points[next];

            let cross = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
            let is_convex = if ccw { cross > 0.0 } else { cross < 0.0 };
            if !is_convex {
                continue;
            }

            let mut has_point_inside = false;
            for &idx in &remaining {
                let p_idx = idx as usize;
                if p_idx == prev || p_idx == curr || p_idx == next {
                    continue;
                }
                let p = points[p_idx];
                if point_in_triangle(p, a, b, c) {
                    has_point_inside = true;
                    break;
                }
            }

            if !has_point_inside {
                indices.push(prev as u32);
                indices.push(curr as u32);
                indices.push(next as u32);
                remaining.remove(i);
                ear_found = true;
                break;
            }
        }
        if !ear_found && remaining.len() >= 3 {
            for i in 1..(remaining.len() - 1) {
                indices.push(remaining[0]);
                indices.push(remaining[i]);
                indices.push(remaining[i + 1]);
            }
            break;
        }
    }
    indices
}
