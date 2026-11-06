//! EllipseGeometry and EllipseOutlineGeometry matching CesiumJS `Core/EllipseGeometry`,
//! `Core/EllipseOutlineGeometry`, and `Core/EllipseGeometryLibrary`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::{
    core_math::PI_OVER_TWO,
    geometry::{
        ComponentDatatype, Geometry, GeometryAttribute, GeometryAttributes, GeometryOffsetAttribute,
        PrimitiveType,
    },
    BoundingSphere, Cartesian3, Ellipsoid, Matrix3, Quaternion, VertexFormat,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EllipseGeometry {
    pub center: Cartesian3,
    pub semi_major_axis: f64,
    pub semi_minor_axis: f64,
    pub ellipsoid: Ellipsoid,
    pub height: f64,
    pub extruded_height: Option<f64>,
    pub granularity: f64,
    pub rotation: f64,
    pub st_rotation: f64,
    pub vertex_format: VertexFormat,
    pub offset_attribute: Option<GeometryOffsetAttribute>,
}

impl EllipseGeometry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        center: Cartesian3,
        semi_major_axis: f64,
        semi_minor_axis: f64,
        ellipsoid: Option<Ellipsoid>,
        height: Option<f64>,
        extruded_height: Option<f64>,
        granularity: Option<f64>,
        rotation: Option<f64>,
        st_rotation: Option<f64>,
        vertex_format: Option<VertexFormat>,
    ) -> Self {
        Self {
            center,
            semi_major_axis,
            semi_minor_axis,
            ellipsoid: ellipsoid.unwrap_or(Ellipsoid::WGS84),
            height: height.unwrap_or(0.0),
            extruded_height,
            granularity: granularity.unwrap_or(0.02),
            rotation: rotation.unwrap_or(0.0),
            st_rotation: st_rotation.unwrap_or(0.0),
            vertex_format: vertex_format.unwrap_or(VertexFormat::DEFAULT),
            offset_attribute: None,
        }
    }

    pub fn with_offset_attribute(mut self, offset_attribute: Option<GeometryOffsetAttribute>) -> Self {
        self.offset_attribute = offset_attribute;
        self
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        let center = self.center;
        let a = self.semi_major_axis;
        let b = self.semi_minor_axis;
        let gran = self.granularity;

        if a <= 0.0 || b <= 0.0 || gran <= 0.0 || a < b {
            return None;
        }

        let positions_res = compute_ellipse_positions(
            center,
            a,
            b,
            self.rotation,
            gran,
            true,
            false,
        );

        let positions = positions_res.positions?;
        let num_pts = positions_res.num_pts;

        let raised_positions = raise_positions_to_height(
            &positions,
            self.ellipsoid,
            self.height,
            self.extruded_height,
        );

        let mut attributes = GeometryAttributes::default();

        if self.vertex_format.position {
            attributes.position = Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                raised_positions.clone(),
            ));
        }

        let num_vertices = raised_positions.len() / 3;

        if self.vertex_format.normal {
            let mut normals = Vec::with_capacity(num_vertices * 3);
            for i in 0..num_vertices {
                let p = Cartesian3::new(
                    raised_positions[i * 3],
                    raised_positions[i * 3 + 1],
                    raised_positions[i * 3 + 2],
                );
                let n: Cartesian3 = self
                    .ellipsoid
                    .geodetic_surface_normal(p.into())
                    .unwrap_or(Cartesian3::UNIT_Z.into())
                    .into();
                normals.push(n.x);
                normals.push(n.y);
                normals.push(n.z);
            }
            attributes.normal = Some(GeometryAttribute::new(
                ComponentDatatype::Float,
                3,
                normals,
            ));
        }

        if self.vertex_format.st {
            let mut uvs = Vec::with_capacity(num_vertices * 2);
            for _ in 0..num_vertices {
                uvs.push(0.0);
                uvs.push(0.0);
            }
            attributes.st = Some(GeometryAttribute::new(
                ComponentDatatype::Float,
                2,
                uvs,
            ));
        }

        let indices = top_indices(num_pts);

        if let Some(offset_attr) = self.offset_attribute {
            let offset_val = match offset_attr {
                GeometryOffsetAttribute::None => 0.0,
                _ => 1.0,
            };
            let count = positions.len() / 3;
            attributes.apply_offset = Some(GeometryAttribute::new(
                ComponentDatatype::UnsignedByte,
                1,
                vec![offset_val; count],
            ));
        }

        Some(Geometry {
            attributes,
            indices,
            primitive_type: PrimitiveType::Triangles,
            bounding_sphere: Some(BoundingSphere::new(center, a)),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EllipseOutlineGeometry {
    pub center: Cartesian3,
    pub semi_major_axis: f64,
    pub semi_minor_axis: f64,
    pub ellipsoid: Ellipsoid,
    pub height: f64,
    pub extruded_height: Option<f64>,
    pub granularity: f64,
    pub rotation: f64,
    pub number_of_vertical_lines: Option<usize>,
    pub offset_attribute: Option<GeometryOffsetAttribute>,
}

impl EllipseOutlineGeometry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        center: Cartesian3,
        semi_major_axis: f64,
        semi_minor_axis: f64,
        ellipsoid: Option<Ellipsoid>,
        height: Option<f64>,
        extruded_height: Option<f64>,
        granularity: Option<f64>,
        rotation: Option<f64>,
        number_of_vertical_lines: Option<usize>,
    ) -> Self {
        Self {
            center,
            semi_major_axis,
            semi_minor_axis,
            ellipsoid: ellipsoid.unwrap_or(Ellipsoid::WGS84),
            height: height.unwrap_or(0.0),
            extruded_height,
            granularity: granularity.unwrap_or(0.02),
            rotation: rotation.unwrap_or(0.0),
            number_of_vertical_lines,
            offset_attribute: None,
        }
    }

    pub fn with_offset_attribute(mut self, offset_attribute: Option<GeometryOffsetAttribute>) -> Self {
        self.offset_attribute = offset_attribute;
        self
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        let center = self.center;
        let a = self.semi_major_axis;
        let b = self.semi_minor_axis;
        let gran = self.granularity;

        if a <= 0.0 || b <= 0.0 || gran <= 0.0 || a < b {
            return None;
        }

        let is_extruded = self.extruded_height.is_some()
            && (self.extruded_height.unwrap() - self.height).abs() > f64::EPSILON;

        let res = compute_ellipse_positions(
            center,
            a,
            b,
            self.rotation,
            gran,
            false,
            true,
        );

        let outer_positions = res.outer_positions?;

        let positions = raise_positions_to_height(
            &outer_positions,
            self.ellipsoid,
            self.height,
            if is_extruded { self.extruded_height } else { None },
        );

        let length = positions.len() / 3;

        let indices = if is_extruded {
            let ring_len = length / 2;
            let num_vert = self.number_of_vertical_lines.unwrap_or(16).min(ring_len);
            let mut idxs = Vec::with_capacity(length * 2 + num_vert * 2);

            for i in 0..ring_len {
                let next = (i + 1) % ring_len;
                // Top loop
                idxs.push(i as u32);
                idxs.push(next as u32);
                // Bottom loop
                idxs.push((i + ring_len) as u32);
                idxs.push((next + ring_len) as u32);
            }

            if num_vert > 0 {
                let num_side = (ring_len as f64 / num_vert as f64).round() as usize;
                let max_i = (num_side * num_vert).min(ring_len);
                let mut i = 0;
                while i < max_i {
                    idxs.push(i as u32);
                    idxs.push((i + ring_len) as u32);
                    i += num_side;
                }
            }

            idxs
        } else {
            let mut idxs = Vec::with_capacity(length * 2);
            for i in 0..length {
                idxs.push(i as u32);
                idxs.push(((i + 1) % length) as u32);
            }
            idxs
        };

        let mut attributes = GeometryAttributes {
            position: Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                positions,
            )),
            ..Default::default()
        };

        if let Some(offset_attr) = self.offset_attribute {
            let offset_val = match offset_attr {
                GeometryOffsetAttribute::None => 0.0,
                _ => 1.0,
            };
            attributes.apply_offset = Some(GeometryAttribute::new(
                ComponentDatatype::UnsignedByte,
                1,
                vec![offset_val; length],
            ));
        }

        Some(Geometry {
            attributes,
            indices,
            primitive_type: PrimitiveType::Lines,
            bounding_sphere: Some(BoundingSphere::new(center, a)),
        })
    }
}

// ============================================================================
// EllipseGeometryLibrary
// ============================================================================

struct EllipsePositions {
    positions: Option<Vec<f64>>,
    outer_positions: Option<Vec<f64>>,
    num_pts: usize,
}

#[allow(clippy::too_many_arguments)]
fn point_on_ellipsoid(
    theta: f64,
    rotation: f64,
    north_vec: Cartesian3,
    east_vec: Cartesian3,
    a_sqr: f64,
    ab: f64,
    b_sqr: f64,
    mag: f64,
    unit_pos: Cartesian3,
) -> Cartesian3 {
    let azimuth = theta + rotation;
    let rot_axis = (east_vec * azimuth.cos() + north_vec * azimuth.sin()).normalize();

    let cos_t = theta.cos();
    let sin_t = theta.sin();
    let radius = ab / (b_sqr * cos_t * cos_t + a_sqr * sin_t * sin_t).sqrt();
    let angle = radius / mag;

    let q = Quaternion::from_axis_angle(rot_axis, angle);
    let m = Matrix3::from_quaternion(&q);
    m.multiply_by_vector(unit_pos).normalize() * mag
}

fn compute_ellipse_positions(
    center: Cartesian3,
    semi_major_axis: f64,
    semi_minor_axis: f64,
    rotation: f64,
    granularity: f64,
    add_fill_positions: bool,
    add_edge_positions: bool,
) -> EllipsePositions {
    let gran = granularity * 8.0;
    let a_sqr = semi_minor_axis * semi_minor_axis;
    let b_sqr = semi_major_axis * semi_major_axis;
    let ab = semi_major_axis * semi_minor_axis;

    let mag = center.magnitude();
    let unit_pos = center.normalize();
    let mut east_vec = Cartesian3::UNIT_Z.cross(&center).normalize();
    if east_vec.magnitude() == 0.0 {
        east_vec = Cartesian3::UNIT_X;
    }
    let north_vec = unit_pos.cross(&east_vec).normalize();

    let mut num_pts = 1 + (PI_OVER_TWO / gran).ceil() as i32;
    let delta_theta = PI_OVER_TWO / (num_pts - 1) as f64;
    let theta_check = PI_OVER_TWO - num_pts as f64 * delta_theta;
    if theta_check < 0.0 {
        num_pts -= (theta_check.abs() / delta_theta).ceil() as i32;
    }
    let num_pts = num_pts.max(1) as usize;

    let size = 2 * (num_pts * (num_pts + 2));
    let mut positions = if add_fill_positions {
        Some(vec![0.0; size * 3])
    } else {
        None
    };
    let mut pos_idx = 0;

    let outer_len = num_pts * 4 * 3;
    let mut outer_positions = if add_edge_positions {
        Some(vec![0.0; outer_len])
    } else {
        None
    };
    let mut outer_right_index = if add_edge_positions { outer_len as isize - 1 } else { 0 };
    let mut outer_left_index = 0usize;

    // Eastern half
    let mut theta = PI_OVER_TWO;
    let mut position = point_on_ellipsoid(
        theta, rotation, north_vec, east_vec, a_sqr, ab, b_sqr, mag, unit_pos,
    );

    if let Some(pos) = positions.as_mut() {
        pos[pos_idx] = position.x;
        pos[pos_idx + 1] = position.y;
        pos[pos_idx + 2] = position.z;
        pos_idx += 3;
    }
    if let Some(outer) = outer_positions.as_mut() {
        outer[outer_right_index as usize] = position.z; outer_right_index -= 1;
        outer[outer_right_index as usize] = position.y; outer_right_index -= 1;
        outer[outer_right_index as usize] = position.x; outer_right_index -= 1;
    }

    theta = PI_OVER_TWO - delta_theta;
    for i in 1..=num_pts {
        position = point_on_ellipsoid(
            theta, rotation, north_vec, east_vec, a_sqr, ab, b_sqr, mag, unit_pos,
        );
        let reflected_position = point_on_ellipsoid(
            std::f64::consts::PI - theta, rotation, north_vec, east_vec, a_sqr, ab, b_sqr, mag, unit_pos,
        );

        if let Some(pos) = positions.as_mut() {
            pos[pos_idx] = position.x;
            pos[pos_idx + 1] = position.y;
            pos[pos_idx + 2] = position.z;
            pos_idx += 3;

            let num_interior = 2 * i + 2;
            for j in 1..(num_interior - 1) {
                let t = j as f64 / (num_interior - 1) as f64;
                let interior_pos = Cartesian3::lerp(position, reflected_position, t);
                pos[pos_idx] = interior_pos.x;
                pos[pos_idx + 1] = interior_pos.y;
                pos[pos_idx + 2] = interior_pos.z;
                pos_idx += 3;
            }

            pos[pos_idx] = reflected_position.x;
            pos[pos_idx + 1] = reflected_position.y;
            pos[pos_idx + 2] = reflected_position.z;
            pos_idx += 3;
        }

        if let Some(outer) = outer_positions.as_mut() {
            outer[outer_right_index as usize] = position.z; outer_right_index -= 1;
            outer[outer_right_index as usize] = position.y; outer_right_index -= 1;
            outer[outer_right_index as usize] = position.x; outer_right_index -= 1;

            outer[outer_left_index] = reflected_position.x; outer_left_index += 1;
            outer[outer_left_index] = reflected_position.y; outer_left_index += 1;
            outer[outer_left_index] = reflected_position.z; outer_left_index += 1;
        }

        theta = PI_OVER_TWO - (i as f64 + 1.0) * delta_theta;
    }

    // Western half
    for i in (2..=num_pts).rev() {
        theta = PI_OVER_TWO - (i as f64 - 1.0) * delta_theta;
        position = point_on_ellipsoid(
            -theta, rotation, north_vec, east_vec, a_sqr, ab, b_sqr, mag, unit_pos,
        );
        let reflected_position = point_on_ellipsoid(
            theta + std::f64::consts::PI, rotation, north_vec, east_vec, a_sqr, ab, b_sqr, mag, unit_pos,
        );

        if let Some(pos) = positions.as_mut() {
            pos[pos_idx] = position.x;
            pos[pos_idx + 1] = position.y;
            pos[pos_idx + 2] = position.z;
            pos_idx += 3;

            let num_interior = 2 * (i - 1) + 2;
            for j in 1..(num_interior - 1) {
                let t = j as f64 / (num_interior - 1) as f64;
                let interior_pos = Cartesian3::lerp(position, reflected_position, t);
                pos[pos_idx] = interior_pos.x;
                pos[pos_idx + 1] = interior_pos.y;
                pos[pos_idx + 2] = interior_pos.z;
                pos_idx += 3;
            }

            pos[pos_idx] = reflected_position.x;
            pos[pos_idx + 1] = reflected_position.y;
            pos[pos_idx + 2] = reflected_position.z;
            pos_idx += 3;
        }

        if let Some(outer) = outer_positions.as_mut() {
            outer[outer_right_index as usize] = position.z; outer_right_index -= 1;
            outer[outer_right_index as usize] = position.y; outer_right_index -= 1;
            outer[outer_right_index as usize] = position.x; outer_right_index -= 1;

            outer[outer_left_index] = reflected_position.x; outer_left_index += 1;
            outer[outer_left_index] = reflected_position.y; outer_left_index += 1;
            outer[outer_left_index] = reflected_position.z; outer_left_index += 1;
        }
    }

    theta = PI_OVER_TWO;
    position = point_on_ellipsoid(
        -theta, rotation, north_vec, east_vec, a_sqr, ab, b_sqr, mag, unit_pos,
    );

    if let Some(pos) = positions.as_mut() {
        pos[pos_idx] = position.x;
        pos[pos_idx + 1] = position.y;
        pos[pos_idx + 2] = position.z;
    }
    if let Some(outer) = outer_positions.as_mut() {
        outer[outer_right_index as usize] = position.z; outer_right_index -= 1;
        outer[outer_right_index as usize] = position.y; outer_right_index -= 1;
        outer[outer_right_index as usize] = position.x;
    }

    EllipsePositions {
        positions,
        outer_positions,
        num_pts,
    }
}

fn top_indices(num_pts: usize) -> Vec<u32> {
    let total_indices = 12 * (num_pts * (num_pts + 1)) - 6;
    let mut indices = Vec::with_capacity(total_indices);

    let mut prev_index = 0u32;
    let mut position_index = 1u32;

    for _ in 0..3 {
        indices.push(position_index); position_index += 1;
        indices.push(prev_index);
        indices.push(position_index);
    }

    for i in 2..=num_pts {
        position_index = (i * (i + 1) - 1) as u32;
        prev_index = ((i - 1) * i - 1) as u32;

        indices.push(position_index); position_index += 1;
        indices.push(prev_index);
        indices.push(position_index);

        let num_interior = 2 * i;
        for _ in 0..(num_interior - 1) {
            indices.push(position_index);
            indices.push(prev_index); prev_index += 1;
            indices.push(prev_index);

            indices.push(position_index); position_index += 1;
            indices.push(prev_index);
            indices.push(position_index);
        }

        indices.push(position_index); position_index += 1;
        indices.push(prev_index);
        indices.push(position_index);
    }

    // Center column
    let num_interior = num_pts * 2;
    position_index += 1;
    prev_index += 1;
    for _ in 0..(num_interior - 1) {
        indices.push(position_index);
        indices.push(prev_index); prev_index += 1;
        indices.push(prev_index);

        indices.push(position_index); position_index += 1;
        indices.push(prev_index);
        indices.push(position_index);
    }

    indices.push(position_index);
    indices.push(prev_index); prev_index += 1;
    indices.push(prev_index);

    indices.push(position_index); position_index += 1;
    indices.push(prev_index); prev_index += 1;
    indices.push(prev_index);

    // Left of north vector
    prev_index += 1;
    for i in (2..num_pts).rev() {
        indices.push(prev_index); prev_index += 1;
        indices.push(prev_index);
        indices.push(position_index);

        let num_interior = 2 * i;
        for _ in 0..(num_interior - 1) {
            indices.push(position_index);
            indices.push(prev_index); prev_index += 1;
            indices.push(prev_index);

            indices.push(position_index); position_index += 1;
            indices.push(prev_index);
            indices.push(position_index);
        }

        indices.push(prev_index); prev_index += 1;
        indices.push(prev_index); prev_index += 1;
        indices.push(position_index); position_index += 1;
    }

    for _ in 0..3 {
        indices.push(prev_index); prev_index += 1;
        indices.push(prev_index);
        indices.push(position_index);
    }

    indices
}

fn raise_positions_to_height(
    positions: &[f64],
    ellipsoid: Ellipsoid,
    height: f64,
    extruded_height: Option<f64>,
) -> Vec<f64> {
    let extrude = extruded_height.is_some();
    let count = positions.len() / 3;
    let size = if extrude { count * 2 } else { count };
    let mut final_positions = vec![0.0; size * 3];
    let bottom_offset = if extrude { count * 3 } else { 0 };

    for i in 0..count {
        let idx = i * 3;
        let pos = Cartesian3::new(positions[idx], positions[idx + 1], positions[idx + 2]);
        let surface_pos: Cartesian3 = ellipsoid
            .scale_to_geodetic_surface(pos.into())
            .unwrap_or(pos.into())
            .into();
        let normal: Cartesian3 = ellipsoid
            .geodetic_surface_normal(surface_pos.into())
            .unwrap_or(Cartesian3::UNIT_Z.into())
            .into();

        let top_pos = surface_pos + normal * height;
        final_positions[idx] = top_pos.x;
        final_positions[idx + 1] = top_pos.y;
        final_positions[idx + 2] = top_pos.z;

        if let Some(ext_h) = extruded_height {
            let bot_pos = surface_pos + normal * ext_h;
            final_positions[idx + bottom_offset] = bot_pos.x;
            final_positions[idx + 1 + bottom_offset] = bot_pos.y;
            final_positions[idx + 2 + bottom_offset] = bot_pos.z;
        }
    }

    final_positions
}
