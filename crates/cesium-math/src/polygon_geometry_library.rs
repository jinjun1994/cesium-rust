//! Polygon geometry library matching CesiumJS `Core/PolygonGeometryLibrary`.
//! Adapted from CesiumJS (Apache-2.0).

use glam::DVec3;

use crate::cartesian::Cartesian3;
use crate::core_math::negative_pi_to_pi;
use crate::ellipsoid::Ellipsoid;
use crate::ellipsoid_rhumb_line::EllipsoidRhumbLine;
use crate::geometry::ArcType;
use crate::intersections::line_segment_plane;
use crate::Cartographic;

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
    let heading = rhumb.heading().ok()?;

    // At latitude 0, sigma = 0
    let sigma1 = (0.5 * (std::f64::consts::FRAC_PI_2 + c0.latitude)).tan().ln();
    let delta_lon = heading.tan() * (0.0 - sigma1);
    let intersection_lon = negative_pi_to_pi(c0.longitude + delta_lon);

    let mut min_lon = c0.longitude.min(c1.longitude);
    let mut max_lon = c0.longitude.max(c1.longitude);

    if (max_lon - min_lon).abs() > std::f64::consts::PI {
        std::mem::swap(&mut min_lon, &mut max_lon);
    }

    if intersection_lon < min_lon || intersection_lon > max_lon {
        return None;
    }

    let inter_carto = Cartographic {
        longitude: intersection_lon,
        latitude: 0.0,
        height: 0.0,
    };
    ellipsoid.cartographic_to_cartesian(inter_carto).map(Cartesian3::from)
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
}
