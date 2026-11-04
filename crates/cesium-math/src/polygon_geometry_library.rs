//! Polygon geometry library matching CesiumJS `Core/PolygonGeometryLibrary`.
//! Adapted from CesiumJS (Apache-2.0).

use glam::DVec3;

use std::collections::VecDeque;

use crate::cartesian::Cartesian3;
use crate::coplanar_polygon_geometry::PolygonHierarchy;
use crate::core_math::EPSILON10;
use crate::ellipsoid::Ellipsoid;
use crate::ellipsoid_rhumb_line::EllipsoidRhumbLine;
use crate::errors::DeveloperError;
use crate::geometry::ArcType;
use crate::intersections::line_segment_plane;

pub struct PolygonGeometryLibrary;

#[derive(Clone, Debug)]
struct EdgeOnPlane {
    position: usize,
    edge_type: i32,
    visited: bool,
    next: i32,
    theta: f64,
}

fn compute_equator_intersection_rhumb(
    start: Cartesian3,
    end: Cartesian3,
    ellipsoid: &Ellipsoid,
) -> Option<Cartesian3> {
    let c0 = ellipsoid.cartesian_to_cartographic(start.into())?;
    let c1 = ellipsoid.cartesian_to_cartographic(end.into())?;

    if c0.latitude.signum() == c1.latitude.signum() {
        return None;
    }

    let rhumb = EllipsoidRhumbLine::new(Some(c0), Some(c1), Some(*ellipsoid));
    let intersection = rhumb.find_intersection_with_latitude(0.0)?;

    let mut min_lon = c0.longitude.min(c1.longitude);
    let mut max_lon = c0.longitude.max(c1.longitude);

    if (max_lon - min_lon).abs() > std::f64::consts::PI {
        std::mem::swap(&mut min_lon, &mut max_lon);
    }

    if intersection.longitude < min_lon || intersection.longitude > max_lon {
        return None;
    }

    ellipsoid
        .cartographic_to_cartesian(intersection)
        .map(Cartesian3::from)
}

fn compute_equator_intersection(
    start: Cartesian3,
    end: Cartesian3,
    ellipsoid: &Ellipsoid,
    arc_type: ArcType,
) -> Option<Cartesian3> {
    if arc_type == ArcType::Rhumb {
        return compute_equator_intersection_rhumb(start, end, ellipsoid);
    }

    let intersection = line_segment_plane(start.into(), end.into(), DVec3::Z, 0.0)?;
    ellipsoid
        .scale_to_geodetic_surface(intersection)
        .map(Cartesian3::from)
}

fn compute_edges_on_plane(
    positions: &mut Vec<Cartesian3>,
    ellipsoid: &Ellipsoid,
    arc_type: ArcType,
) -> Vec<EdgeOnPlane> {
    let mut edges_on_plane = Vec::new();
    let mut i = 0;

    let get_longitude = |pos: Cartesian3| -> f64 {
        ellipsoid
            .cartesian_to_cartographic(pos.into())
            .map(|c| c.longitude)
            .unwrap_or(0.0)
    };

    let sign = |z: f64| -> i32 {
        if z.abs() < 1e-10 {
            0
        } else if z > 0.0 {
            1
        } else {
            -1
        }
    };

    while i < positions.len() {
        let start_point = positions[i];
        let end_point = positions[(i + 1) % positions.len()];

        let type_val = sign(start_point.z);
        let next_val = sign(end_point.z);

        if type_val == 0 {
            edges_on_plane.push(EdgeOnPlane {
                position: i,
                edge_type: type_val,
                visited: false,
                next: next_val,
                theta: get_longitude(start_point),
            });
        } else if next_val != 0 {
            if let Some(intersection) =
                compute_equator_intersection(start_point, end_point, ellipsoid, arc_type)
            {
                i += 1;
                positions.insert(i, intersection);
                edges_on_plane.push(EdgeOnPlane {
                    position: i,
                    edge_type: type_val,
                    visited: false,
                    next: next_val,
                    theta: get_longitude(intersection),
                });
            }
        }

        i += 1;
    }

    edges_on_plane
}

fn wire_polygon(
    polygons: &mut Vec<Vec<Cartesian3>>,
    mut polygon_index: usize,
    positions: &[Cartesian3],
    edges_on_plane: &mut [EdgeOnPlane],
    to_delete: usize,
    start_index: usize,
    above_plane: bool,
) -> usize {
    let mut polygon = Vec::new();
    let mut i = start_index;
    let mut polygons_to_wire = Vec::new();

    loop {
        let position = positions[i];
        polygon.push(position);

        let edge_opt = edges_on_plane.iter().position(|e| e.position == i);
        if edge_opt.is_none() {
            i += 1;
            if i >= positions.len() || i == start_index || polygon.len() >= positions.len() {
                break;
            }
            continue;
        }

        let edge_index = edge_opt.unwrap();
        let has_been_visited = edges_on_plane[edge_index].visited;
        let edge_type = edges_on_plane[edge_index].edge_type;
        let next = edges_on_plane[edge_index].next;

        edges_on_plane[edge_index].visited = true;

        if edge_type == 0 {
            if next == 0 {
                let prev_idx = if above_plane {
                    edge_index.wrapping_sub(1)
                } else {
                    edge_index + 1
                };
                if prev_idx < edges_on_plane.len() && edges_on_plane[prev_idx].position == i + 1 {
                    edges_on_plane[prev_idx].visited = true;
                } else {
                    i += 1;
                    if i >= positions.len() || i == start_index || polygon.len() >= positions.len() {
                        break;
                    }
                    continue;
                }
            }

            if (!has_been_visited && above_plane && next > 0)
                || (start_index == i && !above_plane && next < 0)
            {
                i += 1;
                if i >= positions.len() || i == start_index || polygon.len() >= positions.len() {
                    break;
                }
                continue;
            }
        }

        let follow_edge = if above_plane {
            edge_type >= 0
        } else {
            edge_type <= 0
        };
        if !follow_edge {
            i += 1;
            if i >= positions.len() || i == start_index || polygon.len() >= positions.len() {
                break;
            }
            continue;
        }

        if !has_been_visited {
            polygons_to_wire.push(i);
        }

        let next_edge_idx = if above_plane {
            edge_index.wrapping_add(1)
        } else {
            edge_index.wrapping_sub(1)
        };

        if next_edge_idx >= edges_on_plane.len() {
            i += 1;
            if i >= positions.len() || i == start_index || polygon.len() >= positions.len() {
                break;
            }
            continue;
        }

        i = edges_on_plane[next_edge_idx].position;

        if i >= positions.len() || i == start_index || polygon.len() >= positions.len() {
            break;
        }
    }

    if to_delete > 0 && polygon_index < polygons.len() {
        polygons[polygon_index] = polygon;
    } else {
        polygons.insert(polygon_index, polygon);
    }

    for index in polygons_to_wire {
        polygon_index = wire_polygon(
            polygons,
            polygon_index + 1,
            positions,
            edges_on_plane,
            0,
            index,
            !above_plane,
        );
    }

    polygon_index
}

impl PolygonGeometryLibrary {
    pub fn subdivide_line_count(p0: Cartesian3, p1: Cartesian3, min_distance: f64) -> usize {
        let distance = p0.distance(&p1);
        let n = distance / min_distance;
        let count_divide = (n.log2().ceil() as i32).max(0);
        2_usize.pow(count_divide as u32)
    }

    pub fn subdivide_line(
        p0: Cartesian3,
        p1: Cartesian3,
        min_distance: f64,
        result: Option<&mut Vec<f64>>,
    ) -> Vec<f64> {
        let num_vertices = Self::subdivide_line_count(p0, p1, min_distance);
        let length = p0.distance(&p1);
        let distance_between_vertices = length / (num_vertices as f64);

        let mut positions = if let Some(res) = result {
            res.clear();
            std::mem::take(res)
        } else {
            Vec::with_capacity(num_vertices * 3)
        };

        let diff = p1 - p0;
        for i in 0..num_vertices {
            let ratio = if length > 0.0 {
                (i as f64 * distance_between_vertices) / length
            } else {
                0.0
            };
            let pt = p0 + diff * ratio;
            positions.push(pt.x);
            positions.push(pt.y);
            positions.push(pt.z);
        }

        positions
    }

    pub fn subdivide_rhumb_line_count(
        ellipsoid: &Ellipsoid,
        p0: Cartesian3,
        p1: Cartesian3,
        min_distance: f64,
    ) -> usize {
        let c0 = match ellipsoid.cartesian_to_cartographic(p0.into()) {
            Some(c) => c,
            None => return 1,
        };
        let c1 = match ellipsoid.cartesian_to_cartographic(p1.into()) {
            Some(c) => c,
            None => return 1,
        };
        let rhumb = EllipsoidRhumbLine::new(Some(c0), Some(c1), Some(*ellipsoid));
        let dist = rhumb.surface_distance().unwrap_or(0.0);
        let n = dist / min_distance;
        let count_divide = (n.log2().ceil() as i32).max(0);
        2_usize.pow(count_divide as u32)
    }

    pub fn subdivide_rhumb_line(
        ellipsoid: &Ellipsoid,
        p0: Cartesian3,
        p1: Cartesian3,
        min_distance: f64,
    ) -> Vec<f64> {
        let c0 = match ellipsoid.cartesian_to_cartographic(p0.into()) {
            Some(c) => c,
            None => return vec![p0.x, p0.y, p0.z],
        };
        let c1 = match ellipsoid.cartesian_to_cartographic(p1.into()) {
            Some(c) => c,
            None => return vec![p0.x, p0.y, p0.z],
        };

        let rhumb = EllipsoidRhumbLine::new(Some(c0), Some(c1), Some(*ellipsoid));
        let surf_dist = rhumb.surface_distance().unwrap_or(0.0);

        if surf_dist <= min_distance {
            return vec![p0.x, p0.y, p0.z];
        }

        let num_vertices = Self::subdivide_rhumb_line_count(ellipsoid, p0, p1, min_distance);
        let dist_step = surf_dist / num_vertices as f64;

        let mut positions = Vec::with_capacity(num_vertices * 3);
        for i in 0..num_vertices {
            let c = rhumb
                .interpolate_using_surface_distance(i as f64 * dist_step)
                .unwrap_or(c0);
            let p = ellipsoid
                .cartographic_to_cartesian(c)
                .map(Cartesian3::from)
                .unwrap_or(p0);
            positions.push(p.x);
            positions.push(p.y);
            positions.push(p.z);
        }

        positions
    }

    pub fn split_polygons_on_equator(
        outer_rings: &[Vec<Cartesian3>],
        ellipsoid: &Ellipsoid,
        arc_type: ArcType,
    ) -> Vec<Vec<Cartesian3>> {
        let mut result: Vec<Vec<Cartesian3>> = outer_rings.to_vec();

        let mut current_polygon = 0;
        while current_polygon < result.len() {
            let outer_ring = result[current_polygon].clone();
            let mut positions = outer_ring.clone();

            if outer_ring.len() < 3 {
                current_polygon += 1;
                continue;
            }

            let mut edges_on_plane = compute_edges_on_plane(&mut positions, ellipsoid, arc_type);
            if positions.len() == outer_ring.len() || edges_on_plane.len() <= 1 {
                result[current_polygon] = positions;
                current_polygon += 1;
                continue;
            }

            edges_on_plane.sort_by(|a, b| {
                a.theta
                    .partial_cmp(&b.theta)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let north = positions[0].z >= 0.0;
            current_polygon = wire_polygon(
                &mut result,
                current_polygon,
                &positions,
                &mut edges_on_plane,
                1,
                0,
                north,
            );
        }

        result
    }

    pub fn compute_hierarchy_packed_length(hierarchy: &PolygonHierarchy) -> usize {
        let mut num_components = 0;
        let mut stack = vec![hierarchy];
        while let Some(h) = stack.pop() {
            num_components += 2;
            num_components += h.positions.len() * 3;
            for hole in &h.holes {
                stack.push(hole);
            }
        }
        num_components
    }

    pub fn pack_polygon_hierarchy(
        hierarchy: &PolygonHierarchy,
        array: &mut [f64],
        starting_index: &mut usize,
    ) {
        let mut stack = vec![hierarchy];
        while let Some(h) = stack.pop() {
            array[*starting_index] = h.positions.len() as f64;
            *starting_index += 1;
            array[*starting_index] = h.holes.len() as f64;
            *starting_index += 1;

            for pos in &h.positions {
                array[*starting_index] = pos.x;
                array[*starting_index + 1] = pos.y;
                array[*starting_index + 2] = pos.z;
                *starting_index += 3;
            }

            for hole in &h.holes {
                stack.push(hole);
            }
        }
    }

    pub fn unpack_polygon_hierarchy(
        array: &[f64],
        starting_index: &mut usize,
    ) -> Result<PolygonHierarchy, DeveloperError> {
        if *starting_index + 2 > array.len() {
            return Err(DeveloperError::new("Array too short to unpack hierarchy header"));
        }
        let positions_length = array[*starting_index] as usize;
        *starting_index += 1;
        let holes_length = array[*starting_index] as usize;
        *starting_index += 1;

        if *starting_index + positions_length * 3 > array.len() {
            return Err(DeveloperError::new("Array too short to unpack hierarchy positions"));
        }

        let mut positions = Vec::with_capacity(positions_length);
        for _ in 0..positions_length {
            let x = array[*starting_index];
            let y = array[*starting_index + 1];
            let z = array[*starting_index + 2];
            positions.push(Cartesian3::new(x, y, z));
            *starting_index += 3;
        }

        let mut holes = Vec::with_capacity(holes_length);
        for _ in 0..holes_length {
            let hole = Self::unpack_polygon_hierarchy(array, starting_index)?;
            holes.push(hole);
        }

        Ok(PolygonHierarchy { positions, holes })
    }

    pub fn array_remove_duplicates(
        values: &[Cartesian3],
        wrap_around: bool,
    ) -> Vec<Cartesian3> {
        if values.len() < 2 {
            return values.to_vec();
        }
        let mut cleaned: Option<Vec<Cartesian3>> = None;
        let mut v0 = values[0];

        for (i, &v1) in values.iter().enumerate().skip(1) {
            if v0.equals_epsilon(&v1, Some(EPSILON10), Some(EPSILON10)) {
                if cleaned.is_none() {
                    let mut list = Vec::with_capacity(values.len());
                    list.extend_from_slice(&values[..i]);
                    cleaned = Some(list);
                }
            } else {
                if let Some(ref mut list) = cleaned {
                    list.push(v1);
                }
                v0 = v1;
            }
        }

        let mut result = cleaned.unwrap_or_else(|| values.to_vec());

        if wrap_around
            && result.len() > 1
            && result[0].equals_epsilon(result.last().unwrap(), Some(EPSILON10), Some(EPSILON10))
        {
            result.pop();
        }

        result
    }

    pub fn polygon_outlines_from_hierarchy(
        hierarchy: &PolygonHierarchy,
        scale_to_ellipsoid_surface: bool,
        ellipsoid: &Ellipsoid,
    ) -> Vec<Vec<Cartesian3>> {
        let mut polygons = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(hierarchy);

        while let Some(outer_node) = queue.pop_front() {
            let mut outer_ring = outer_node.positions.clone();
            if scale_to_ellipsoid_surface {
                for pt in &mut outer_ring {
                    if let Some(scaled) = ellipsoid.scale_to_geodetic_surface((*pt).into()) {
                        *pt = Cartesian3::from(scaled);
                    }
                }
            }
            outer_ring = Self::array_remove_duplicates(&outer_ring, true);
            if outer_ring.len() < 3 {
                continue;
            }

            for hole in &outer_node.holes {
                let mut hole_positions = hole.positions.clone();
                if scale_to_ellipsoid_surface {
                    for pt in &mut hole_positions {
                        if let Some(scaled) = ellipsoid.scale_to_geodetic_surface((*pt).into()) {
                            *pt = Cartesian3::from(scaled);
                        }
                    }
                }
                hole_positions = Self::array_remove_duplicates(&hole_positions, true);
                if hole_positions.len() < 3 {
                    continue;
                }
                polygons.push(hole_positions);

                for child_hole in &hole.holes {
                    queue.push_back(child_hole);
                }
            }

            polygons.push(outer_ring);
        }

        polygons
    }

    pub fn scale_to_geodetic_height_extruded(
        positions: &mut [f64],
        max_height: f64,
        min_height: f64,
        ellipsoid: &Ellipsoid,
        per_position_height: bool,
    ) {
        let length = positions.len() / 2;
        let mut i = 0;
        while i < length {
            let p = Cartesian3::new(positions[i], positions[i + 1], positions[i + 2]);
            let p_dvec = p.into();
            let n1 = ellipsoid.geodetic_surface_normal(p_dvec).unwrap_or(DVec3::Z);
            let mut p2 = ellipsoid.scale_to_geodetic_surface(p_dvec).unwrap_or(p_dvec);
            let bottom = p2 + n1 * min_height;
            positions[i + length] = bottom.x;
            positions[i + 1 + length] = bottom.y;
            positions[i + 2 + length] = bottom.z;

            if per_position_height {
                p2 = p_dvec;
            }
            let top = p2 + n1 * max_height;
            positions[i] = top.x;
            positions[i + 1] = top.y;
            positions[i + 2] = top.z;

            i += 3;
        }
    }
}
