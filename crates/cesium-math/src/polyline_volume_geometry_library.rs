//! Polyline volume geometry library matching CesiumJS `Core/PolylineVolumeGeometryLibrary`.

use crate::bounding_rectangle::BoundingRectangle;
use crate::cartesian::{Cartesian2, Cartesian3};
use crate::core_math::EPSILON7;
use crate::ellipsoid::Ellipsoid;
use crate::ellipsoid_tangent_plane::EllipsoidTangentPlane;
use crate::geometry::CornerType;
use crate::matrices::Matrix3;
use crate::polyline_pipeline::{PolylineArcOptions, PolylinePipeline};
use crate::quaternion::Quaternion;
use glam::{DMat3, DMat4, DQuat, DVec3};

fn scale_to_surface(positions: &mut [Cartesian3], ellipsoid: &Ellipsoid) -> Vec<f64> {
    let mut heights = Vec::with_capacity(positions.len());
    for pos in positions.iter_mut() {
        let carto = ellipsoid
            .cartesian_to_cartographic((*pos).into())
            .unwrap_or(crate::Cartographic::ZERO);
        heights.push(carto.height);
        if let Some(surf) = ellipsoid.scale_to_geodetic_surface((*pos).into()) {
            *pos = Cartesian3::from(surf);
        }
    }
    heights
}

fn subdivide_heights(p0: Cartesian3, p1: Cartesian3, h0: f64, h1: f64, granularity: f64) -> Vec<f64> {
    let angle_between = p0.angle_between(&p1);
    let num_points = (angle_between / granularity).ceil() as usize;
    let num_points = num_points.max(1);

    let mut heights = Vec::with_capacity(num_points + 1);
    if (h0 - h1).abs() < f64::EPSILON {
        for _ in 0..num_points {
            heights.push(h0);
        }
        heights.push(h1);
        return heights;
    }

    let d_height = h1 - h0;
    let height_per_vertex = d_height / num_points as f64;

    heights.push(h0);
    for i in 1..num_points {
        heights.push(h0 + i as f64 * height_per_vertex);
    }
    heights.push(h1);
    heights
}

fn compute_rotation_angle(
    start: Cartesian3,
    end: Cartesian3,
    position: Cartesian3,
    ellipsoid: &Ellipsoid,
) -> f64 {
    let Ok(tangent_plane) = EllipsoidTangentPlane::new(position.into(), *ellipsoid) else {
        return 0.0;
    };
    let Some(next_2d) = tangent_plane.project_point_onto_plane((position + start).into()) else {
        return 0.0;
    };
    let Some(prev_2d) = tangent_plane.project_point_onto_plane((position + end).into()) else {
        return 0.0;
    };

    let next = Cartesian2::new(next_2d.x, next_2d.y);
    let prev = Cartesian2::new(prev_2d.x, prev_2d.y);

    let angle = next.angle_between(&prev);
    if prev.x * next.y - prev.y * next.x >= 0.0 {
        -angle
    } else {
        angle
    }
}

#[allow(clippy::too_many_arguments)]
fn add_position(
    center: Cartesian3,
    left: Cartesian3,
    shape: &[f64],
    final_positions: &mut Vec<f64>,
    ellipsoid: &Ellipsoid,
    height: f64,
    x_scalar: f64,
    repeat: usize,
) {
    let transform = ellipsoid
        .east_north_up_to_fixed_frame(center.into())
        .unwrap_or(DMat4::IDENTITY);

    let negative_x = -DVec3::X;
    let west = transform.transform_vector3(negative_x).normalize_or_zero();
    let angle = compute_rotation_angle(Cartesian3::from(west), left, center, ellipsoid);

    let rotation_z = DMat3::from_rotation_z(angle);
    let local_transform = DMat4::from_rotation_translation(
        DQuat::from_mat3(&rotation_z),
        DVec3::new(0.0, 0.0, height),
    );
    let combined_transform = transform * local_transform;

    let scale = DMat3::from_diagonal(DVec3::new(x_scalar, 1.0, 1.0));

    for _ in 0..repeat {
        for chunk in shape.as_chunks::<3>().0 {
            let pt = DVec3::new(chunk[0], chunk[1], chunk[2]);
            let scaled = scale * pt;
            let final_pt = combined_transform.transform_point3(scaled);
            final_positions.extend_from_slice(&[final_pt.x, final_pt.y, final_pt.z]);
        }
    }
}

fn add_positions(
    centers: &[f64],
    left: Cartesian3,
    shape: &[f64],
    final_positions: &mut Vec<f64>,
    ellipsoid: &Ellipsoid,
    heights: &[f64],
    x_scalar: f64,
) {
    for (i, chunk) in centers.as_chunks::<3>().0.iter().enumerate() {
        let center = Cartesian3::new(chunk[0], chunk[1], chunk[2]);
        let h = heights.get(i).copied().unwrap_or(0.0);
        add_position(center, left, shape, final_positions, ellipsoid, h, x_scalar, 1);
    }
}

fn convert_shape_to_3d_duplicate(
    shape_2d: &[Cartesian2],
    bounding_rectangle: &BoundingRectangle,
) -> Vec<f64> {
    let length = shape_2d.len();
    let mut shape = Vec::with_capacity(length * 6);
    let x_offset = bounding_rectangle.x + bounding_rectangle.width / 2.0;
    let y_offset = bounding_rectangle.y + bounding_rectangle.height / 2.0;

    let p0 = shape_2d[0];
    shape.extend_from_slice(&[p0.x - x_offset, 0.0, p0.y - y_offset]);

    for p in &shape_2d[1..length] {
        let x = p.x - x_offset;
        let z = p.y - y_offset;
        shape.extend_from_slice(&[x, 0.0, z]);
        shape.extend_from_slice(&[x, 0.0, z]);
    }

    shape.extend_from_slice(&[p0.x - x_offset, 0.0, p0.y - y_offset]);
    shape
}

fn convert_shape_to_3d(
    shape_2d: &[Cartesian2],
    bounding_rectangle: &BoundingRectangle,
) -> Vec<f64> {
    let length = shape_2d.len();
    let mut shape = Vec::with_capacity(length * 3);
    let x_offset = bounding_rectangle.x + bounding_rectangle.width / 2.0;
    let y_offset = bounding_rectangle.y + bounding_rectangle.height / 2.0;

    for p in shape_2d {
        shape.extend_from_slice(&[p.x - x_offset, 0.0, p.y - y_offset]);
    }

    shape
}

#[allow(clippy::too_many_arguments)]
fn compute_round_corner(
    pivot: Cartesian3,
    start_point: Cartesian3,
    end_point: Cartesian3,
    corner_type: CornerType,
    left_is_outside: bool,
    ellipsoid: &Ellipsoid,
    final_positions: &mut Vec<f64>,
    shape: &[f64],
    height: f64,
    duplicate_points: bool,
) {
    let v_start = start_point - pivot;
    let v_end = end_point - pivot;
    let angle = v_start.angle_between(&v_end);

    let granularity = if corner_type == CornerType::Beveled {
        0
    } else {
        (angle / 5.0f64.to_radians()).ceil() as usize
    };

    let step_angle = angle / (granularity + 1) as f64;
    let axis = if left_is_outside { -pivot } else { pivot };
    let quat = Quaternion::from_axis_angle(axis, step_angle);
    let m = Matrix3::from_quaternion(&quat);

    let mut cur_point = start_point;
    if granularity > 0 {
        let repeat = if duplicate_points { 2 } else { 1 };
        for _ in 0..granularity {
            cur_point = m.multiply_by_vector(cur_point);

            let mut left = cur_point - pivot;
            left = left.normalize();
            if !left_is_outside {
                left = -left;
            }
            let surface_point = ellipsoid
                .scale_to_geodetic_surface(cur_point.into())
                .map(Cartesian3::from)
                .unwrap_or(cur_point);

            add_position(
                surface_point,
                left,
                shape,
                final_positions,
                ellipsoid,
                height,
                1.0,
                repeat,
            );
        }
    } else {
        let mut left = cur_point - pivot;
        left = left.normalize();
        if !left_is_outside {
            left = -left;
        }
        let surface_point = ellipsoid
            .scale_to_geodetic_surface(cur_point.into())
            .map(Cartesian3::from)
            .unwrap_or(cur_point);

        add_position(
            surface_point,
            left,
            shape,
            final_positions,
            ellipsoid,
            height,
            1.0,
            1,
        );

        let mut end_left = end_point - pivot;
        end_left = end_left.normalize();
        if !left_is_outside {
            end_left = -end_left;
        }
        let end_surface = ellipsoid
            .scale_to_geodetic_surface(end_point.into())
            .map(Cartesian3::from)
            .unwrap_or(end_point);

        add_position(
            end_surface,
            end_left,
            shape,
            final_positions,
            ellipsoid,
            height,
            1.0,
            1,
        );
    }
}

/// Polyline volume geometry algorithms matching CesiumJS `Core/PolylineVolumeGeometryLibrary`.
pub struct PolylineVolumeGeometryLibrary;

impl PolylineVolumeGeometryLibrary {
    /// Removes adjacent and wrapping duplicate points from a 2D shape.
    pub fn remove_duplicates_from_shape(shape_positions: &[Cartesian2]) -> Vec<Cartesian2> {
        let length = shape_positions.len();
        if length == 0 {
            return Vec::new();
        }
        let mut cleaned = Vec::new();
        let mut i0 = length - 1;
        for i1 in 0..length {
            let v0 = shape_positions[i0];
            let v1 = shape_positions[i1];
            if v0 != v1 {
                cleaned.push(v1);
            }
            i0 = i1;
        }
        cleaned
    }

    /// Determines whether the turn angle from forward to backward is greater than PI in the local tangent plane.
    pub fn angle_is_greater_than_pi(
        forward: Cartesian3,
        backward: Cartesian3,
        position: Cartesian3,
        ellipsoid: &Ellipsoid,
    ) -> bool {
        let Ok(tangent_plane) = EllipsoidTangentPlane::new(position.into(), *ellipsoid) else {
            return false;
        };
        let Some(next_2d) = tangent_plane.project_point_onto_plane((position + forward).into()) else {
            return false;
        };
        let Some(prev_2d) = tangent_plane.project_point_onto_plane((position + backward).into()) else {
            return false;
        };

        prev_2d.x * next_2d.y - prev_2d.y * next_2d.x >= 0.0
    }

    /// Computes full extruded 3D positions for the polyline volume.
    #[allow(clippy::too_many_arguments)]
    pub fn compute_positions(
        positions: &[Cartesian3],
        shape_2d: &[Cartesian2],
        bounding_rectangle: &BoundingRectangle,
        ellipsoid: &Ellipsoid,
        corner_type: CornerType,
        granularity: f64,
        duplicate_points: bool,
    ) -> Vec<f64> {
        let mut positions = positions.to_vec();
        let heights = scale_to_surface(&mut positions, ellipsoid);

        let shape_for_sides = if duplicate_points {
            convert_shape_to_3d_duplicate(shape_2d, bounding_rectangle)
        } else {
            convert_shape_to_3d(shape_2d, bounding_rectangle)
        };

        let shape_for_ends = if duplicate_points {
            Some(convert_shape_to_3d(shape_2d, bounding_rectangle))
        } else {
            None
        };

        let height_offset = bounding_rectangle.height / 2.0;
        let width = bounding_rectangle.width / 2.0;
        let length = positions.len();

        let mut final_positions = Vec::new();
        let mut ends = if duplicate_points { Some(Vec::new()) } else { None };

        let mut position = positions[0];
        let mut next_position = positions[1];

        let mut surface_normal = ellipsoid
            .geodetic_surface_normal(position.into())
            .map(Cartesian3::from)
            .unwrap_or(Cartesian3::UNIT_Z);

        let mut forward = (next_position - position).normalize();
        let mut left = surface_normal.cross(&forward).normalize();

        let mut h0 = heights[0];
        let mut h1 = heights[1];

        if duplicate_points {
            if let (Some(ref mut ends_vec), Some(ref ends_shape)) = (&mut ends, &shape_for_ends) {
                add_position(
                    position,
                    left,
                    ends_shape,
                    ends_vec,
                    ellipsoid,
                    h0 + height_offset,
                    1.0,
                    1,
                );
            }
        }

        let mut previous_position = position;
        position = next_position;
        let mut backward = -forward;

        for i in 1..length - 1 {
            let repeat = if duplicate_points { 2 } else { 1 };
            next_position = positions[i + 1];
            if position == next_position {
                continue;
            }

            forward = (next_position - position).normalize();
            surface_normal = ellipsoid
                .geodetic_surface_normal(position.into())
                .map(Cartesian3::from)
                .unwrap_or(Cartesian3::UNIT_Z);

            let f_dot_n = forward.dot(&surface_normal);
            let forward_proj = (forward - surface_normal * f_dot_n).normalize();

            let b_dot_n = backward.dot(&surface_normal);
            let backward_proj = (backward - surface_normal * b_dot_n).normalize();

            let do_corner = (forward_proj.dot(&backward_proj).abs() - 1.0).abs() > EPSILON7;

            if do_corner {
                let mut corner_direction = (forward + backward).normalize();
                corner_direction = corner_direction.cross(&surface_normal);
                corner_direction = surface_normal.cross(&corner_direction).normalize();

                let cross_mag = corner_direction.cross(&backward).magnitude();
                let scalar = 1.0 / cross_mag.max(0.25);

                let left_is_outside =
                    Self::angle_is_greater_than_pi(forward, backward, position, ellipsoid);

                if left_is_outside {
                    let pivot = position + corner_direction * (scalar * width);
                    let start = pivot + left * width;

                    let sub_heights = subdivide_heights(
                        previous_position,
                        start,
                        h0 + height_offset,
                        h1 + height_offset,
                        granularity,
                    );
                    let sub_positions = PolylinePipeline::generate_arc(&PolylineArcOptions {
                        positions: &[previous_position, start],
                        granularity: Some(granularity),
                        ellipsoid: Some(*ellipsoid),
                        ..Default::default()
                    });

                    add_positions(
                        &sub_positions,
                        left,
                        &shape_for_sides,
                        &mut final_positions,
                        ellipsoid,
                        &sub_heights,
                        1.0,
                    );

                    left = surface_normal.cross(&forward).normalize();
                    let end = pivot + left * width;

                    if corner_type == CornerType::Rounded || corner_type == CornerType::Beveled {
                        compute_round_corner(
                            pivot,
                            start,
                            end,
                            corner_type,
                            left_is_outside,
                            ellipsoid,
                            &mut final_positions,
                            &shape_for_sides,
                            h1 + height_offset,
                            duplicate_points,
                        );
                    } else {
                        corner_direction = -corner_direction;
                        add_position(
                            position,
                            corner_direction,
                            &shape_for_sides,
                            &mut final_positions,
                            ellipsoid,
                            h1 + height_offset,
                            scalar,
                            repeat,
                        );
                    }
                    previous_position = end;
                } else {
                    let pivot = position + corner_direction * (scalar * width);
                    let start = pivot - left * width;

                    let sub_heights = subdivide_heights(
                        previous_position,
                        start,
                        h0 + height_offset,
                        h1 + height_offset,
                        granularity,
                    );
                    let sub_positions = PolylinePipeline::generate_arc(&PolylineArcOptions {
                        positions: &[previous_position, start],
                        granularity: Some(granularity),
                        ellipsoid: Some(*ellipsoid),
                        ..Default::default()
                    });

                    add_positions(
                        &sub_positions,
                        left,
                        &shape_for_sides,
                        &mut final_positions,
                        ellipsoid,
                        &sub_heights,
                        1.0,
                    );

                    left = surface_normal.cross(&forward).normalize();
                    let end = pivot - left * width;

                    if corner_type == CornerType::Rounded || corner_type == CornerType::Beveled {
                        compute_round_corner(
                            pivot,
                            start,
                            end,
                            corner_type,
                            left_is_outside,
                            ellipsoid,
                            &mut final_positions,
                            &shape_for_sides,
                            h1 + height_offset,
                            duplicate_points,
                        );
                    } else {
                        add_position(
                            position,
                            corner_direction,
                            &shape_for_sides,
                            &mut final_positions,
                            ellipsoid,
                            h1 + height_offset,
                            scalar,
                            repeat,
                        );
                    }
                    previous_position = end;
                }
                backward = -forward;
            } else {
                add_position(
                    previous_position,
                    left,
                    &shape_for_sides,
                    &mut final_positions,
                    ellipsoid,
                    h0 + height_offset,
                    1.0,
                    1,
                );
                previous_position = position;
            }

            h0 = h1;
            h1 = heights[i + 1];
            position = next_position;
        }

        let sub_heights = subdivide_heights(
            previous_position,
            position,
            h0 + height_offset,
            h1 + height_offset,
            granularity,
        );
        let sub_positions = PolylinePipeline::generate_arc(&PolylineArcOptions {
            positions: &[previous_position, position],
            granularity: Some(granularity),
            ellipsoid: Some(*ellipsoid),
            ..Default::default()
        });

        add_positions(
            &sub_positions,
            left,
            &shape_for_sides,
            &mut final_positions,
            ellipsoid,
            &sub_heights,
            1.0,
        );

        if duplicate_points {
            if let (Some(ref mut ends_vec), Some(ref ends_shape)) = (&mut ends, &shape_for_ends) {
                add_position(
                    position,
                    left,
                    ends_shape,
                    ends_vec,
                    ellipsoid,
                    h1 + height_offset,
                    1.0,
                    1,
                );
            }
        }

        if let Some(ends_vec) = ends {
            final_positions.extend_from_slice(&ends_vec);
        }

        final_positions
    }
}
