//! CorridorGeometryLibrary matching CesiumJS `Core/CorridorGeometryLibrary`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::{
    core_math::EPSILON7,
    geometry::CornerType,
    Cartesian3, Ellipsoid, EllipsoidTangentPlane, Matrix3, Quaternion,
};
use glam::DVec3;

#[derive(Clone, Debug, PartialEq)]
pub struct CorridorCorner {
    pub left_positions: Option<Vec<f64>>,
    pub right_positions: Option<Vec<f64>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CorridorPositions {
    pub positions: Vec<Vec<f64>>,
    pub corners: Vec<CorridorCorner>,
    pub lefts: Option<Vec<f64>>,
    pub normals: Option<Vec<f64>>,
    pub end_positions: Option<[Vec<f64>; 2]>,
}

pub struct CorridorParameters<'a> {
    pub positions: &'a [Cartesian3],
    pub width: f64,
    pub ellipsoid: Ellipsoid,
    pub corner_type: CornerType,
    pub granularity: f64,
    pub save_attributes: bool,
}

pub struct CorridorGeometryLibrary;

impl CorridorGeometryLibrary {
    pub fn add_attribute(
        attribute: &mut [f64],
        value: Cartesian3,
        front: Option<usize>,
        back: Option<usize>,
    ) {
        if let Some(f) = front {
            attribute[f] = value.x;
            attribute[f + 1] = value.y;
            attribute[f + 2] = value.z;
        }
        if let Some(b) = back {
            attribute[b] = value.z;
            attribute[b - 1] = value.y;
            attribute[b - 2] = value.x;
        }
    }

    pub fn compute_positions(params: CorridorParameters) -> CorridorPositions {
        let granularity = params.granularity;
        let positions = params.positions;
        let ellipsoid = params.ellipsoid;
        let width = params.width / 2.0;
        let corner_type = params.corner_type;
        let save_attributes = params.save_attributes;

        let mut calculated_positions = Vec::new();
        let mut calculated_lefts = if save_attributes { Some(Vec::new()) } else { None };
        let mut calculated_normals = if save_attributes { Some(Vec::new()) } else { None };

        if positions.len() < 2 {
            return CorridorPositions {
                positions: calculated_positions,
                corners: Vec::new(),
                lefts: calculated_lefts,
                normals: calculated_normals,
                end_positions: None,
            };
        }

        let mut normal = Cartesian3::from(
            ellipsoid
                .geodetic_surface_normal(positions[0].into())
                .unwrap_or(DVec3::Z),
        );
        let mut forward = (positions[1] - positions[0]).normalize();
        let mut left = normal.cross(&forward).normalize();

        if let Some(ref mut l) = calculated_lefts {
            l.extend_from_slice(&[left.x, left.y, left.z]);
        }
        if let Some(ref mut n) = calculated_normals {
            n.extend_from_slice(&[normal.x, normal.y, normal.z]);
        }

        let mut previous_pos = positions[0];
        let mut position = positions[1];
        let mut backward = -forward;

        let mut corners = Vec::new();
        let length = positions.len();

        for i in 1..length - 1 {
            normal = Cartesian3::from(
                ellipsoid
                    .geodetic_surface_normal(position.into())
                    .unwrap_or(DVec3::Z),
            );
            let next_position = positions[i + 1];
            forward = (next_position - position).normalize();

            let forward_proj = (forward - normal * forward.dot(&normal)).normalize();
            let backward_proj = (backward - normal * backward.dot(&normal)).normalize();

            let do_corner = (forward_proj.dot(&backward_proj).abs() - 1.0).abs() > EPSILON7;

            if do_corner {
                let mut corner_direction = (forward + backward).normalize();
                corner_direction = corner_direction.cross(&normal);
                corner_direction = normal.cross(&corner_direction).normalize();

                let cross_mag = corner_direction.cross(&backward).magnitude();
                let scalar = width / cross_mag.max(0.25);
                let left_is_outside = angle_is_greater_than_pi(forward, backward, position, ellipsoid);
                corner_direction = corner_direction * scalar;

                if left_is_outside {
                    let right_pos = position + corner_direction;
                    let center = right_pos + left * width;
                    let mut left_pos = right_pos + left * (width * 2.0);

                    let subdivided = generate_arc(previous_pos, center, granularity, ellipsoid);
                    add_shifted_positions(&subdivided, left, width, &mut calculated_positions);

                    if let Some(ref mut l) = calculated_lefts {
                        l.extend_from_slice(&[left.x, left.y, left.z]);
                    }
                    if let Some(ref mut n) = calculated_normals {
                        n.extend_from_slice(&[normal.x, normal.y, normal.z]);
                    }

                    let start_point = left_pos;
                    left = normal.cross(&forward).normalize();
                    left_pos = right_pos + left * (width * 2.0);
                    previous_pos = right_pos + left * width;

                    if corner_type == CornerType::Rounded || corner_type == CornerType::Beveled {
                        let round = compute_round_corner(
                            right_pos,
                            start_point,
                            left_pos,
                            corner_type,
                            left_is_outside,
                        );
                        corners.push(CorridorCorner {
                            left_positions: Some(round),
                            right_positions: None,
                        });
                    } else {
                        let mitered = compute_mitered_corner(
                            position,
                            -corner_direction,
                            left_pos,
                            left_is_outside,
                        );
                        corners.push(CorridorCorner {
                            left_positions: Some(mitered),
                            right_positions: None,
                        });
                    }
                } else {
                    let left_pos = position + corner_direction;
                    let center = left_pos - left * width;
                    let mut right_pos = left_pos - left * (width * 2.0);

                    let subdivided = generate_arc(previous_pos, center, granularity, ellipsoid);
                    add_shifted_positions(&subdivided, left, width, &mut calculated_positions);

                    if let Some(ref mut l) = calculated_lefts {
                        l.extend_from_slice(&[left.x, left.y, left.z]);
                    }
                    if let Some(ref mut n) = calculated_normals {
                        n.extend_from_slice(&[normal.x, normal.y, normal.z]);
                    }

                    let start_point = right_pos;
                    left = normal.cross(&forward).normalize();
                    right_pos = left_pos - left * (width * 2.0);
                    previous_pos = left_pos - left * width;

                    if corner_type == CornerType::Rounded || corner_type == CornerType::Beveled {
                        let round = compute_round_corner(
                            left_pos,
                            start_point,
                            right_pos,
                            corner_type,
                            left_is_outside,
                        );
                        corners.push(CorridorCorner {
                            left_positions: None,
                            right_positions: Some(round),
                        });
                    } else {
                        let mitered = compute_mitered_corner(
                            position,
                            corner_direction,
                            right_pos,
                            left_is_outside,
                        );
                        corners.push(CorridorCorner {
                            left_positions: None,
                            right_positions: Some(mitered),
                        });
                    }
                }
                backward = -forward;
            }
            position = next_position;
        }

        normal = Cartesian3::from(
            ellipsoid
                .geodetic_surface_normal(position.into())
                .unwrap_or(DVec3::Z),
        );
        let subdivided = generate_arc(previous_pos, position, granularity, ellipsoid);
        add_shifted_positions(&subdivided, left, width, &mut calculated_positions);

        if let Some(ref mut l) = calculated_lefts {
            l.extend_from_slice(&[left.x, left.y, left.z]);
        }
        if let Some(ref mut n) = calculated_normals {
            n.extend_from_slice(&[normal.x, normal.y, normal.z]);
        }

        let end_positions = if corner_type == CornerType::Rounded {
            Some(add_end_caps(&calculated_positions))
        } else {
            None
        };

        CorridorPositions {
            positions: calculated_positions,
            corners,
            lefts: calculated_lefts,
            normals: calculated_normals,
            end_positions,
        }
    }
}

fn angle_is_greater_than_pi(
    forward: Cartesian3,
    backward: Cartesian3,
    position: Cartesian3,
    ellipsoid: Ellipsoid,
) -> bool {
    let Ok(tangent_plane) = EllipsoidTangentPlane::new(position.into(), ellipsoid) else {
        return false;
    };
    if let (Some(next), Some(prev)) = (
        tangent_plane.project_point_onto_plane((position + forward).into()),
        tangent_plane.project_point_onto_plane((position + backward).into()),
    ) {
        prev.x * next.y - prev.y * next.x >= 0.0
    } else {
        false
    }
}

fn generate_arc(
    p0: Cartesian3,
    p1: Cartesian3,
    granularity: f64,
    ellipsoid: Ellipsoid,
) -> Vec<Cartesian3> {
    let d0: DVec3 = p0.into();
    let d1: DVec3 = p1.into();
    let norm0 = d0.normalize();
    let norm1 = d1.normalize();
    let angle = norm0.dot(norm1).clamp(-1.0, 1.0).acos();
    let n = (angle / granularity).ceil().max(1.0) as usize;
    let mut result = Vec::with_capacity(n + 1);

    for i in 0..=n {
        let t = i as f64 / n as f64;
        let dir = if angle.abs() < 1e-10 {
            norm0
        } else {
            let sin_angle = angle.sin();
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

fn add_shifted_positions(
    positions: &[Cartesian3],
    left: Cartesian3,
    scalar: f64,
    calculated_positions: &mut Vec<Vec<f64>>,
) {
    let count = positions.len();
    let mut right_positions = Vec::with_capacity(count * 3);
    let mut left_positions = Vec::with_capacity(count * 3);
    let scaled_left = left * scalar;
    let scaled_right = -scaled_left;

    for &pos in positions {
        let r = pos + scaled_right;
        right_positions.push(r.x);
        right_positions.push(r.y);
        right_positions.push(r.z);
    }

    for &pos in positions.iter().rev() {
        let l = pos + scaled_left;
        left_positions.push(l.x);
        left_positions.push(l.y);
        left_positions.push(l.z);
    }

    calculated_positions.push(right_positions);
    calculated_positions.push(left_positions);
}

fn compute_round_corner(
    corner_point: Cartesian3,
    mut start_point: Cartesian3,
    end_point: Cartesian3,
    corner_type: CornerType,
    left_is_outside: bool,
) -> Vec<f64> {
    let v_start = (start_point - corner_point).normalize();
    let v_end = (end_point - corner_point).normalize();
    let angle = v_start.dot(&v_end).clamp(-1.0, 1.0).acos();
    let granularity = if corner_type == CornerType::Beveled {
        1
    } else {
        (angle / 5.0_f64.to_radians()).ceil() as usize + 1
    };

    let size = granularity * 3;
    let mut array = vec![0.0; size];
    array[size - 3] = end_point.x;
    array[size - 2] = end_point.y;
    array[size - 1] = end_point.z;

    let axis = if left_is_outside {
        -corner_point.normalize()
    } else {
        corner_point.normalize()
    };

    let q = Quaternion::from_axis_angle(axis, angle / granularity as f64);
    let m = Matrix3::from_quaternion(&q);

    let mut index = 0;
    for _ in 0..granularity {
        start_point = m.multiply_by_vector(start_point);
        array[index] = start_point.x;
        array[index + 1] = start_point.y;
        array[index + 2] = start_point.z;
        index += 3;
    }

    array
}

fn compute_mitered_corner(
    position: Cartesian3,
    mut left_corner_direction: Cartesian3,
    last_point: Cartesian3,
    left_is_outside: bool,
) -> Vec<f64> {
    let corner_point = if left_is_outside {
        position + left_corner_direction
    } else {
        left_corner_direction = -left_corner_direction;
        position + left_corner_direction
    };
    vec![
        corner_point.x,
        corner_point.y,
        corner_point.z,
        last_point.x,
        last_point.y,
        last_point.z,
    ]
}

fn add_end_caps(calculated_positions: &[Vec<f64>]) -> [Vec<f64>; 2] {
    let first_left = &calculated_positions[1];
    let start_point = Cartesian3::new(
        first_left[first_left.len() - 3],
        first_left[first_left.len() - 2],
        first_left[first_left.len() - 1],
    );
    let end_point = Cartesian3::new(
        calculated_positions[0][0],
        calculated_positions[0][1],
        calculated_positions[0][2],
    );
    let corner_point = (start_point + end_point) * 0.5;
    let first_cap = compute_round_corner(corner_point, start_point, end_point, CornerType::Rounded, false);

    let last_idx = calculated_positions.len() - 1;
    let right_edge = &calculated_positions[last_idx - 1];
    let left_edge = &calculated_positions[last_idx];
    let start_point2 = Cartesian3::new(
        right_edge[right_edge.len() - 3],
        right_edge[right_edge.len() - 2],
        right_edge[right_edge.len() - 1],
    );
    let end_point2 = Cartesian3::new(left_edge[0], left_edge[1], left_edge[2]);
    let corner_point2 = (start_point2 + end_point2) * 0.5;
    let last_cap = compute_round_corner(corner_point2, start_point2, end_point2, CornerType::Rounded, false);

    [first_cap, last_cap]
}
