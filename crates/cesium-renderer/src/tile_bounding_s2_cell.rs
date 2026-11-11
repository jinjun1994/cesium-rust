use std::collections::HashMap;

use cesium_math::{
    Cartesian3, Cartographic, Color, ComponentDatatype, CoplanarPolygonOutlineGeometry,
    DeveloperError, Ellipsoid, GeometryInstance, GeometryInstanceAttribute, Matrix4, Plane,
    PlaneIntersection, S2Cell, Sphere,
};

use crate::scene_primitive::{Primitive, PrimitiveAppearance, PrimitiveOptions};

/// Options for creating a `TileBoundingS2Cell`.
#[derive(Clone, Debug)]
pub struct TileBoundingS2CellOptions {
    pub token: String,
    pub minimum_height: Option<f64>,
    pub maximum_height: Option<f64>,
    pub ellipsoid: Option<Ellipsoid>,
    pub compute_bounding_volumes: Option<bool>,
}

/// Camera state for `TileBoundingS2Cell::distance_to_camera`.
#[derive(Clone, Debug)]
pub struct TileBoundingS2CellCamera {
    pub position_wc: Cartesian3,
}

impl TileBoundingS2CellCamera {
    pub fn new(position_wc: Cartesian3) -> Self {
        Self { position_wc }
    }
}

/// Frame state passed to `TileBoundingS2Cell::distance_to_camera`.
pub struct TileBoundingS2CellFrameState {
    pub camera: TileBoundingS2CellCamera,
}

/// A tile bounding volume specified as an S2 cell token with minimum and maximum heights.
#[derive(Clone, Debug)]
pub struct TileBoundingS2Cell {
    pub s2_cell: S2Cell,
    pub token: String,
    pub minimum_height: f64,
    pub maximum_height: f64,
    pub ellipsoid: Ellipsoid,
    pub center: Cartesian3,
    pub bounding_planes: [Plane; 6],
    pub vertices: [Cartesian3; 8],
    pub plane_vertices: [[Cartesian3; 4]; 6],
    pub edge_normals: [[Cartesian3; 4]; 6],
    bounding_sphere: Option<Sphere>,
}

fn to_cartesian3(ellipsoid: &Ellipsoid, c: Cartographic) -> Cartesian3 {
    ellipsoid
        .cartographic_to_cartesian(c)
        .unwrap_or(glam::DVec3::ZERO)
        .into()
}

fn compute_bounding_planes(
    s2_cell: &S2Cell,
    minimum_height: f64,
    maximum_height: f64,
    ellipsoid: &Ellipsoid,
) -> Result<[Plane; 6], DeveloperError> {
    let center_point = s2_cell.get_center(Some(ellipsoid));
    let center_normal: Cartesian3 = ellipsoid
        .geodetic_surface_normal(center_point.into())
        .unwrap_or(glam::DVec3::Z)
        .into();

    let mut center_carto = ellipsoid
        .cartesian_to_cartographic(center_point.into())
        .unwrap_or_else(|| Cartographic::new(0.0, 0.0, 0.0));
    center_carto.height = maximum_height;
    let top_pos = to_cartesian3(ellipsoid, center_carto);
    let top_plane = Plane::from_point_normal(top_pos.into(), center_normal.into())
        .map_err(|e| DeveloperError::new(format!("Failed to create top plane: {e:?}")))?;

    // Bottom plane
    let mut max_distance = 0.0_f64;
    let mut s2_vertices = [Cartesian3::ZERO; 4];
    for (i, v_out) in s2_vertices.iter_mut().enumerate() {
        let v = s2_cell
            .get_vertex(i, Some(ellipsoid))
            .map_err(|e| DeveloperError::new(format!("Failed to get S2 cell vertex: {e}")))?;
        *v_out = v;
        let mut carto = ellipsoid
            .cartesian_to_cartographic(v.into())
            .unwrap_or_else(|| Cartographic::new(0.0, 0.0, 0.0));
        carto.height = minimum_height;
        let p_min = to_cartesian3(ellipsoid, carto);
        let dist = Cartesian3::from(top_plane.normal).dot(&p_min) + top_plane.distance;
        if dist < max_distance {
            max_distance = dist;
        }
    }

    let bottom_normal = -Cartesian3::from(top_plane.normal);
    let bottom_plane = Plane::new(
        bottom_normal.into(),
        -top_plane.distance + max_distance,
    )
    .map_err(|e| DeveloperError::new(format!("Failed to create bottom plane: {e:?}")))?;

    // Side planes
    let mut planes = [
        top_plane,
        bottom_plane,
        top_plane,
        top_plane,
        top_plane,
        top_plane,
    ];

    for i in 0..4 {
        let v = s2_vertices[i];
        let next_v = s2_vertices[(i + 1) % 4];
        let geodetic_normal: Cartesian3 = ellipsoid
            .geodetic_surface_normal(v.into())
            .unwrap_or(glam::DVec3::Z)
            .into();
        let side = next_v - v;
        let side_normal = side.cross(&geodetic_normal).normalize();
        let side_plane = Plane::from_point_normal(v.into(), side_normal.into())
            .map_err(|e| DeveloperError::new(format!("Failed to create side plane: {e:?}")))?;
        planes[2 + i] = side_plane;
    }

    Ok(planes)
}

fn compute_intersection(p0: &Plane, p1: &Plane, p2: &Plane) -> Cartesian3 {
    let n0 = Cartesian3::from(p0.normal);
    let n1 = Cartesian3::from(p1.normal);
    let n2 = Cartesian3::from(p2.normal);

    let x0 = n0 * (-p0.distance);
    let x1 = n1 * (-p1.distance);
    let x2 = n2 * (-p2.distance);

    let f0 = n1.cross(&n2) * x0.dot(&n0);
    let f1 = n2.cross(&n0) * x1.dot(&n1);
    let f2 = n0.cross(&n1) * x2.dot(&n2);

    let det = n0.dot(&n1.cross(&n2));
    let s = f0 + f1 + f2;
    Cartesian3::new(s.x / det, s.y / det, s.z / det)
}

fn compute_vertices(bounding_planes: &[Plane; 6]) -> [Cartesian3; 8] {
    let mut vertices = [Cartesian3::ZERO; 8];
    for i in 0..4 {
        vertices[i] = compute_intersection(
            &bounding_planes[0],
            &bounding_planes[2 + ((i + 3) % 4)],
            &bounding_planes[2 + (i % 4)],
        );
        vertices[i + 4] = compute_intersection(
            &bounding_planes[1],
            &bounding_planes[2 + ((i + 3) % 4)],
            &bounding_planes[2 + (i % 4)],
        );
    }
    vertices
}

fn compute_edge_normals(plane: &Plane, vertices: &[Cartesian3; 4]) -> [Cartesian3; 4] {
    let mut edge_normals = [Cartesian3::ZERO; 4];
    let plane_normal = Cartesian3::from(plane.normal);
    for i in 0..4 {
        let edge = vertices[(i + 1) % 4] - vertices[i];
        let edge_normal = plane_normal.cross(&edge).normalize();
        edge_normals[i] = edge_normal;
    }
    edge_normals
}

fn closest_point_line_segment(p: Cartesian3, l0: Cartesian3, l1: Cartesian3) -> Cartesian3 {
    let d = l1 - l0;
    let p_l0 = p - l0;
    let t = d.dot(&p_l0);
    if t <= 0.0 {
        return l0;
    }
    let d_mag = d.magnitude_squared();
    if t >= d_mag {
        return l1;
    }
    let frac = t / d_mag;
    l0 * (1.0 - frac) + l1 * frac
}

fn closest_point_polygon(
    p: Cartesian3,
    vertices: &[Cartesian3; 4],
    edge_normals: &[Cartesian3; 4],
) -> Cartesian3 {
    let mut min_distance = f64::MAX;
    let mut closest_point = None;

    for i in 0..4 {
        let edge_plane_normal = edge_normals[i];
        let edge_plane_distance = -edge_plane_normal.dot(&vertices[i]);
        let dist = edge_plane_normal.dot(&p) + edge_plane_distance;
        if dist < 0.0 {
            continue;
        }

        let closest_on_edge = closest_point_line_segment(p, vertices[i], vertices[(i + 1) % 4]);
        let d = p.distance(&closest_on_edge);
        if d < min_distance {
            min_distance = d;
            closest_point = Some(closest_on_edge);
        }
    }

    closest_point.unwrap_or(p)
}

fn project_point_onto_plane(plane: &Plane, point: Cartesian3) -> Cartesian3 {
    let normal = Cartesian3::from(plane.normal);
    let dist = normal.dot(&point) + plane.distance;
    point - normal * dist
}

impl TileBoundingS2Cell {
    pub fn new(options: TileBoundingS2CellOptions) -> Result<Self, DeveloperError> {
        let s2_cell = S2Cell::from_token(&options.token)
            .map_err(|e| DeveloperError::new(format!("Invalid S2 token: {e}")))?;
        let minimum_height = options.minimum_height.unwrap_or(0.0);
        let maximum_height = options.maximum_height.unwrap_or(0.0);
        let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);

        let bounding_planes =
            compute_bounding_planes(&s2_cell, minimum_height, maximum_height, &ellipsoid)?;
        let vertices = compute_vertices(&bounding_planes);

        let mut edge_normals = [[Cartesian3::ZERO; 4]; 6];
        let top_vertices = [vertices[0], vertices[1], vertices[2], vertices[3]];
        let bottom_vertices = [vertices[4], vertices[5], vertices[6], vertices[7]];

        let mut top_normals = compute_edge_normals(&bounding_planes[0], &top_vertices);
        for normal in &mut top_normals {
            *normal = -*normal;
        }
        edge_normals[0] = top_normals;
        edge_normals[1] = compute_edge_normals(&bounding_planes[1], &bottom_vertices);

        let mut plane_vertices = [
            top_vertices,
            bottom_vertices,
            [Cartesian3::ZERO; 4],
            [Cartesian3::ZERO; 4],
            [Cartesian3::ZERO; 4],
            [Cartesian3::ZERO; 4],
        ];

        for i in 0..4 {
            let side_v = [
                vertices[i % 4],
                vertices[(i + 1) % 4],
                vertices[4 + ((i + 1) % 4)],
                vertices[4 + i],
            ];
            plane_vertices[2 + i] = side_v;
            edge_normals[2 + i] = compute_edge_normals(&bounding_planes[2 + i], &side_v);
        }

        let center_raw = s2_cell.get_center(Some(&ellipsoid));
        let mut center_carto = ellipsoid
            .cartesian_to_cartographic(center_raw.into())
            .unwrap_or_else(|| Cartographic::new(0.0, 0.0, 0.0));
        center_carto.height = (maximum_height + minimum_height) * 0.5;
        let center = to_cartesian3(&ellipsoid, center_carto);

        let bounding_sphere = if options.compute_bounding_volumes.unwrap_or(true) {
            let pts: Vec<glam::DVec3> = vertices.iter().map(|v| (*v).into()).collect();
            Sphere::from_points(&pts).ok()
        } else {
            None
        };

        Ok(Self {
            s2_cell,
            token: options.token,
            minimum_height,
            maximum_height,
            ellipsoid,
            center,
            bounding_planes,
            vertices,
            plane_vertices,
            edge_normals,
            bounding_sphere,
        })
    }

    pub fn bounding_volume(&self) -> &Self {
        self
    }

    pub fn bounding_sphere(&self) -> Option<&Sphere> {
        self.bounding_sphere.as_ref()
    }

    pub fn distance_to_camera(&self, frame_state: &TileBoundingS2CellFrameState) -> f64 {
        let point = frame_state.camera.position_wc;

        let mut selected_plane_indices = Vec::with_capacity(6);
        let mut vertices_list = Vec::with_capacity(6);
        let mut edge_normals_list = Vec::with_capacity(6);

        let top_dist =
            Cartesian3::from(self.bounding_planes[0].normal).dot(&point) + self.bounding_planes[0].distance;
        let bottom_dist =
            Cartesian3::from(self.bounding_planes[1].normal).dot(&point) + self.bounding_planes[1].distance;

        if top_dist > 0.0 {
            selected_plane_indices.push(0);
            vertices_list.push(self.plane_vertices[0]);
            edge_normals_list.push(self.edge_normals[0]);
        } else if bottom_dist > 0.0 {
            selected_plane_indices.push(1);
            vertices_list.push(self.plane_vertices[1]);
            edge_normals_list.push(self.edge_normals[1]);
        }

        for i in 0..4 {
            let side_plane_index = 2 + i;
            let side_dist = Cartesian3::from(self.bounding_planes[side_plane_index].normal).dot(&point)
                + self.bounding_planes[side_plane_index].distance;
            if side_dist > 0.0 {
                selected_plane_indices.push(side_plane_index);
                vertices_list.push(self.plane_vertices[side_plane_index]);
                edge_normals_list.push(self.edge_normals[side_plane_index]);
            }
        }

        if selected_plane_indices.is_empty() {
            return 0.0;
        }

        if selected_plane_indices.len() == 1 {
            let plane_idx = selected_plane_indices[0];
            let plane = &self.bounding_planes[plane_idx];
            let proj = project_point_onto_plane(plane, point);
            let face_point = closest_point_polygon(proj, &vertices_list[0], &edge_normals_list[0]);
            return face_point.distance(&point);
        }

        if selected_plane_indices.len() == 2 {
            if selected_plane_indices[0] == 0 {
                let e0_idx = 4 * selected_plane_indices[0] + (selected_plane_indices[1] - 2);
                let e1_idx =
                    4 * selected_plane_indices[0] + ((selected_plane_indices[1] - 2 + 1) % 4);
                let edge = [self.vertices[e0_idx], self.vertices[e1_idx]];
                let face_point = closest_point_line_segment(point, edge[0], edge[1]);
                return face_point.distance(&point);
            }

            let mut min_dist_sq = f64::MAX;
            for i in 0..2 {
                let plane_idx = selected_plane_indices[i];
                let plane = &self.bounding_planes[plane_idx];
                let proj = project_point_onto_plane(plane, point);
                let face_point = closest_point_polygon(
                    proj,
                    &vertices_list[i],
                    &self.edge_normals[plane_idx],
                );
                let d_sq = face_point.distance_squared(&point);
                if d_sq < min_dist_sq {
                    min_dist_sq = d_sq;
                }
            }
            return min_dist_sq.sqrt();
        }

        if selected_plane_indices.len() > 3 {
            let plane = &self.bounding_planes[1];
            let proj = project_point_onto_plane(plane, point);
            let face_point = closest_point_polygon(
                proj,
                &self.plane_vertices[1],
                &self.edge_normals[1],
            );
            return face_point.distance(&point);
        }

        // Case III: 3 selected planes
        let skip = if selected_plane_indices[1] == 2 && selected_plane_indices[2] == 5 {
            0
        } else {
            1
        };

        if selected_plane_indices[0] == 0 {
            let v_idx = (selected_plane_indices[1] - 2 + skip) % 4;
            return point.distance(&self.vertices[v_idx]);
        }

        let v_idx = 4 + ((selected_plane_indices[1] - 2 + skip) % 4);
        point.distance(&self.vertices[v_idx])
    }

    pub fn intersect_plane(&self, plane: Plane) -> PlaneIntersection {
        let mut plus_count = 0;
        let mut neg_count = 0;
        for vertex in &self.vertices {
            let dist = Cartesian3::from(plane.normal).dot(vertex) + plane.distance;
            if dist < 0.0 {
                neg_count += 1;
            } else {
                plus_count += 1;
            }
        }
        if plus_count == self.vertices.len() {
            PlaneIntersection::Inside
        } else if neg_count == self.vertices.len() {
            PlaneIntersection::Outside
        } else {
            PlaneIntersection::Intersecting
        }
    }

    pub fn create_debug_volume(&self, color: Color) -> Result<Primitive, DeveloperError> {
        let mut instances = Vec::with_capacity(6);

        let make_instance = |positions: &[Cartesian3; 4]| -> Result<GeometryInstance, DeveloperError> {
            let outline_geom = CoplanarPolygonOutlineGeometry::from_positions(positions.to_vec());
            let geometry = outline_geom
                .create_geometry()
                .ok_or_else(|| DeveloperError::new("Failed to create coplanar polygon geometry"))?;

            let mut attributes = HashMap::new();
            attributes.insert(
                "color".to_string(),
                GeometryInstanceAttribute {
                    component_datatype: ComponentDatatype::Float,
                    components_per_attribute: 4,
                    normalize: false,
                    value: vec![
                        color.red as f64,
                        color.green as f64,
                        color.blue as f64,
                        color.alpha as f64,
                    ],
                },
            );

            Ok(GeometryInstance::new(
                geometry,
                Some(Matrix4::IDENTITY),
                Some("outline".to_string()),
                Some(attributes),
            ))
        };

        // Side instances
        for i in 0..4 {
            instances.push(make_instance(&self.plane_vertices[2 + i])?);
        }
        // Bottom and Top instances
        instances.push(make_instance(&self.plane_vertices[1])?);
        instances.push(make_instance(&self.plane_vertices[0])?);

        Ok(Primitive::new(PrimitiveOptions {
            geometry_instances: Some(instances),
            appearance: Some(PrimitiveAppearance {
                closed: false,
                translucent: false,
            }),
            asynchronous: Some(false),
            ..Default::default()
        }))
    }
}
