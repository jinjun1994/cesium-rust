//! Polygon geometry processing functions matching CesiumJS `Core/PolygonPipeline`.
//! Adapted from CesiumJS (Apache-2.0).

use std::collections::HashMap;

use crate::cartesian::{Cartesian2, Cartesian3};
use crate::core_math::{chord_length, RADIANS_PER_DEGREE};
use crate::ellipsoid::Ellipsoid;
use crate::ellipsoid_rhumb_line::EllipsoidRhumbLine;
use crate::errors::DeveloperError;
use crate::geometry::{
    ComponentDatatype, Geometry, GeometryAttribute, GeometryAttributes, PrimitiveType,
};
use crate::winding_order::WindingOrder;

pub struct PolygonPipeline;

impl PolygonPipeline {
    pub fn compute_area_2d(positions: &[Cartesian2]) -> Result<f64, DeveloperError> {
        let length = positions.len();
        if length < 3 {
            return Err(DeveloperError::new("At least three positions are required."));
        }

        let mut area = 0.0;
        let mut i0 = length - 1;
        for i1 in 0..length {
            let v0 = positions[i0];
            let v1 = positions[i1];
            area += v0.x * v1.y - v1.x * v0.y;
            i0 = i1;
        }

        Ok(area * 0.5)
    }

    pub fn compute_winding_order_2d(
        positions: &[Cartesian2],
    ) -> Result<WindingOrder, DeveloperError> {
        let area = Self::compute_area_2d(positions)?;
        if area > 0.0 {
            Ok(WindingOrder::CounterClockwise)
        } else {
            Ok(WindingOrder::Clockwise)
        }
    }

    pub fn triangulate(positions: &[Cartesian2], holes: &[usize]) -> Vec<u32> {
        let mut flattened = Vec::with_capacity(positions.len() * 2);
        for p in positions {
            flattened.push(p.x);
            flattened.push(p.y);
        }

        earcutr::earcut(&flattened, holes, 2)
            .map(|indices| indices.into_iter().map(|idx| idx as u32).collect())
            .unwrap_or_default()
    }

    pub fn compute_subdivision(
        ellipsoid: &Ellipsoid,
        positions: &[Cartesian3],
        indices: &[u32],
        texcoords: Option<&[Cartesian2]>,
        granularity: Option<f64>,
    ) -> Result<Geometry, DeveloperError> {
        let gran = granularity.unwrap_or(RADIANS_PER_DEGREE);
        if indices.len() < 3 {
            return Err(DeveloperError::new(
                "indices.length must be greater than or equal to 3.",
            ));
        }
        if !indices.len().is_multiple_of(3) {
            return Err(DeveloperError::new("indices.length must be divisible by 3."));
        }
        if gran <= 0.0 {
            return Err(DeveloperError::new("granularity must be greater than zero."));
        }

        let has_texcoords = texcoords.is_some();
        let mut triangles: Vec<u32> = indices.to_vec();

        let mut subdivided_positions: Vec<f64> = Vec::with_capacity(positions.len() * 3);
        for item in positions {
            subdivided_positions.push(item.x);
            subdivided_positions.push(item.y);
            subdivided_positions.push(item.z);
        }

        let mut subdivided_texcoords: Vec<f64> = if let Some(tc) = texcoords {
            let mut list = Vec::with_capacity(tc.len() * 2);
            for t in tc {
                list.push(t.x);
                list.push(t.y);
            }
            list
        } else {
            Vec::new()
        };

        let mut subdivided_indices: Vec<u32> = Vec::new();
        let mut edges: HashMap<(u32, u32), u32> = HashMap::new();

        let radius = ellipsoid.maximum_radius();
        let min_distance = chord_length(gran, radius);
        let min_distance_sqrd = min_distance * min_distance;

        while let Some(i2) = triangles.pop() {
            let i1 = triangles.pop().unwrap();
            let i0 = triangles.pop().unwrap();

            let v0 = Cartesian3::new(
                subdivided_positions[i0 as usize * 3],
                subdivided_positions[i0 as usize * 3 + 1],
                subdivided_positions[i0 as usize * 3 + 2],
            );
            let v1 = Cartesian3::new(
                subdivided_positions[i1 as usize * 3],
                subdivided_positions[i1 as usize * 3 + 1],
                subdivided_positions[i1 as usize * 3 + 2],
            );
            let v2 = Cartesian3::new(
                subdivided_positions[i2 as usize * 3],
                subdivided_positions[i2 as usize * 3 + 1],
                subdivided_positions[i2 as usize * 3 + 2],
            );

            let (t0, t1, t2) = if has_texcoords {
                (
                    Cartesian2::new(
                        subdivided_texcoords[i0 as usize * 2],
                        subdivided_texcoords[i0 as usize * 2 + 1],
                    ),
                    Cartesian2::new(
                        subdivided_texcoords[i1 as usize * 2],
                        subdivided_texcoords[i1 as usize * 2 + 1],
                    ),
                    Cartesian2::new(
                        subdivided_texcoords[i2 as usize * 2],
                        subdivided_texcoords[i2 as usize * 2 + 1],
                    ),
                )
            } else {
                (Cartesian2::ZERO, Cartesian2::ZERO, Cartesian2::ZERO)
            };

            let s0 = v0.normalize() * radius;
            let s1 = v1.normalize() * radius;
            let s2 = v2.normalize() * radius;

            let g0 = (s0 - s1).magnitude_squared();
            let g1 = (s1 - s2).magnitude_squared();
            let g2 = (s2 - s0).magnitude_squared();

            let max = g0.max(g1).max(g2);

            if max > min_distance_sqrd {
                if g0 == max {
                    let edge = (i0.min(i1), i0.max(i1));
                    let i = match edges.get(&edge) {
                        Some(&idx) => idx,
                        None => {
                            let mid = (v0 + v1) * 0.5;
                            subdivided_positions.push(mid.x);
                            subdivided_positions.push(mid.y);
                            subdivided_positions.push(mid.z);
                            let new_idx = (subdivided_positions.len() / 3 - 1) as u32;
                            edges.insert(edge, new_idx);
                            if has_texcoords {
                                let mid_tc = (t0 + t1) * 0.5;
                                subdivided_texcoords.push(mid_tc.x);
                                subdivided_texcoords.push(mid_tc.y);
                            }
                            new_idx
                        }
                    };
                    triangles.push(i0);
                    triangles.push(i);
                    triangles.push(i2);

                    triangles.push(i);
                    triangles.push(i1);
                    triangles.push(i2);
                } else if g1 == max {
                    let edge = (i1.min(i2), i1.max(i2));
                    let i = match edges.get(&edge) {
                        Some(&idx) => idx,
                        None => {
                            let mid = (v1 + v2) * 0.5;
                            subdivided_positions.push(mid.x);
                            subdivided_positions.push(mid.y);
                            subdivided_positions.push(mid.z);
                            let new_idx = (subdivided_positions.len() / 3 - 1) as u32;
                            edges.insert(edge, new_idx);
                            if has_texcoords {
                                let mid_tc = (t1 + t2) * 0.5;
                                subdivided_texcoords.push(mid_tc.x);
                                subdivided_texcoords.push(mid_tc.y);
                            }
                            new_idx
                        }
                    };
                    triangles.push(i1);
                    triangles.push(i);
                    triangles.push(i0);

                    triangles.push(i);
                    triangles.push(i2);
                    triangles.push(i0);
                } else {
                    let edge = (i2.min(i0), i2.max(i0));
                    let i = match edges.get(&edge) {
                        Some(&idx) => idx,
                        None => {
                            let mid = (v2 + v0) * 0.5;
                            subdivided_positions.push(mid.x);
                            subdivided_positions.push(mid.y);
                            subdivided_positions.push(mid.z);
                            let new_idx = (subdivided_positions.len() / 3 - 1) as u32;
                            edges.insert(edge, new_idx);
                            if has_texcoords {
                                let mid_tc = (t2 + t0) * 0.5;
                                subdivided_texcoords.push(mid_tc.x);
                                subdivided_texcoords.push(mid_tc.y);
                            }
                            new_idx
                        }
                    };
                    triangles.push(i2);
                    triangles.push(i);
                    triangles.push(i1);

                    triangles.push(i);
                    triangles.push(i0);
                    triangles.push(i1);
                }
            } else {
                subdivided_indices.push(i0);
                subdivided_indices.push(i1);
                subdivided_indices.push(i2);
            }
        }

        let mut attributes = GeometryAttributes {
            position: Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                subdivided_positions,
            )),
            ..Default::default()
        };

        if has_texcoords {
            attributes.st = Some(GeometryAttribute::new(
                ComponentDatatype::Float,
                2,
                subdivided_texcoords,
            ));
        }

        Ok(Geometry {
            attributes,
            indices: subdivided_indices,
            primitive_type: PrimitiveType::Triangles,
            bounding_sphere: None,
        })
    }

    pub fn compute_rhumb_line_subdivision(
        ellipsoid: &Ellipsoid,
        positions: &[Cartesian3],
        indices: &[u32],
        texcoords: Option<&[Cartesian2]>,
        granularity: Option<f64>,
    ) -> Result<Geometry, DeveloperError> {
        let gran = granularity.unwrap_or(RADIANS_PER_DEGREE);
        if indices.len() < 3 {
            return Err(DeveloperError::new(
                "indices.length must be greater than or equal to 3.",
            ));
        }
        if !indices.len().is_multiple_of(3) {
            return Err(DeveloperError::new("indices.length must be divisible by 3."));
        }
        if gran <= 0.0 {
            return Err(DeveloperError::new("granularity must be greater than zero."));
        }

        let has_texcoords = texcoords.is_some();
        let mut triangles: Vec<u32> = indices.to_vec();

        let mut subdivided_positions: Vec<f64> = Vec::with_capacity(positions.len() * 3);
        for item in positions {
            subdivided_positions.push(item.x);
            subdivided_positions.push(item.y);
            subdivided_positions.push(item.z);
        }

        let mut subdivided_texcoords: Vec<f64> = if let Some(tc) = texcoords {
            let mut list = Vec::with_capacity(tc.len() * 2);
            for t in tc {
                list.push(t.x);
                list.push(t.y);
            }
            list
        } else {
            Vec::new()
        };

        let mut subdivided_indices: Vec<u32> = Vec::new();
        let mut edges: HashMap<(u32, u32), u32> = HashMap::new();

        let radius = ellipsoid.maximum_radius();
        let min_distance = chord_length(gran, radius);

        let mut rhumb0 = EllipsoidRhumbLine::new(None, None, Some(*ellipsoid));
        let mut rhumb1 = EllipsoidRhumbLine::new(None, None, Some(*ellipsoid));
        let mut rhumb2 = EllipsoidRhumbLine::new(None, None, Some(*ellipsoid));

        while let Some(i2) = triangles.pop() {
            let i1 = triangles.pop().unwrap();
            let i0 = triangles.pop().unwrap();

            let v0 = Cartesian3::new(
                subdivided_positions[i0 as usize * 3],
                subdivided_positions[i0 as usize * 3 + 1],
                subdivided_positions[i0 as usize * 3 + 2],
            );
            let v1 = Cartesian3::new(
                subdivided_positions[i1 as usize * 3],
                subdivided_positions[i1 as usize * 3 + 1],
                subdivided_positions[i1 as usize * 3 + 2],
            );
            let v2 = Cartesian3::new(
                subdivided_positions[i2 as usize * 3],
                subdivided_positions[i2 as usize * 3 + 1],
                subdivided_positions[i2 as usize * 3 + 2],
            );

            let (t0, t1, t2) = if has_texcoords {
                (
                    Cartesian2::new(
                        subdivided_texcoords[i0 as usize * 2],
                        subdivided_texcoords[i0 as usize * 2 + 1],
                    ),
                    Cartesian2::new(
                        subdivided_texcoords[i1 as usize * 2],
                        subdivided_texcoords[i1 as usize * 2 + 1],
                    ),
                    Cartesian2::new(
                        subdivided_texcoords[i2 as usize * 2],
                        subdivided_texcoords[i2 as usize * 2 + 1],
                    ),
                )
            } else {
                (Cartesian2::ZERO, Cartesian2::ZERO, Cartesian2::ZERO)
            };

            let c0 = ellipsoid
                .cartesian_to_cartographic(v0.into())
                .ok_or_else(|| DeveloperError::new("Invalid cartesian position"))?;
            let c1 = ellipsoid
                .cartesian_to_cartographic(v1.into())
                .ok_or_else(|| DeveloperError::new("Invalid cartesian position"))?;
            let c2 = ellipsoid
                .cartesian_to_cartographic(v2.into())
                .ok_or_else(|| DeveloperError::new("Invalid cartesian position"))?;

            let _ = rhumb0.set_end_points(c0, c1);
            let g0 = rhumb0.surface_distance().unwrap_or(0.0);

            let _ = rhumb1.set_end_points(c1, c2);
            let g1 = rhumb1.surface_distance().unwrap_or(0.0);

            let _ = rhumb2.set_end_points(c2, c0);
            let g2 = rhumb2.surface_distance().unwrap_or(0.0);

            let max = g0.max(g1).max(g2);

            if max > min_distance {
                if g0 == max {
                    let edge = (i0.min(i1), i0.max(i1));
                    let i = match edges.get(&edge) {
                        Some(&idx) => idx,
                        None => {
                            let mid = rhumb0.interpolate_using_fraction(0.5)?;
                            let mid_height = (c0.height + c1.height) * 0.5;
                            let mid_cart = Cartesian3::from_radians(
                                mid.longitude,
                                mid.latitude,
                                Some(mid_height),
                                Some(ellipsoid),
                            )
                            .unwrap();
                            subdivided_positions.push(mid_cart.x);
                            subdivided_positions.push(mid_cart.y);
                            subdivided_positions.push(mid_cart.z);
                            let new_idx = (subdivided_positions.len() / 3 - 1) as u32;
                            edges.insert(edge, new_idx);
                            if has_texcoords {
                                let mid_tc = (t0 + t1) * 0.5;
                                subdivided_texcoords.push(mid_tc.x);
                                subdivided_texcoords.push(mid_tc.y);
                            }
                            new_idx
                        }
                    };
                    triangles.push(i0);
                    triangles.push(i);
                    triangles.push(i2);

                    triangles.push(i);
                    triangles.push(i1);
                    triangles.push(i2);
                } else if g1 == max {
                    let edge = (i1.min(i2), i1.max(i2));
                    let i = match edges.get(&edge) {
                        Some(&idx) => idx,
                        None => {
                            let mid = rhumb1.interpolate_using_fraction(0.5)?;
                            let mid_height = (c1.height + c2.height) * 0.5;
                            let mid_cart = Cartesian3::from_radians(
                                mid.longitude,
                                mid.latitude,
                                Some(mid_height),
                                Some(ellipsoid),
                            )
                            .unwrap();
                            subdivided_positions.push(mid_cart.x);
                            subdivided_positions.push(mid_cart.y);
                            subdivided_positions.push(mid_cart.z);
                            let new_idx = (subdivided_positions.len() / 3 - 1) as u32;
                            edges.insert(edge, new_idx);
                            if has_texcoords {
                                let mid_tc = (t1 + t2) * 0.5;
                                subdivided_texcoords.push(mid_tc.x);
                                subdivided_texcoords.push(mid_tc.y);
                            }
                            new_idx
                        }
                    };
                    triangles.push(i1);
                    triangles.push(i);
                    triangles.push(i0);

                    triangles.push(i);
                    triangles.push(i2);
                    triangles.push(i0);
                } else {
                    let edge = (i2.min(i0), i2.max(i0));
                    let i = match edges.get(&edge) {
                        Some(&idx) => idx,
                        None => {
                            let mid = rhumb2.interpolate_using_fraction(0.5)?;
                            let mid_height = (c2.height + c0.height) * 0.5;
                            let mid_cart = Cartesian3::from_radians(
                                mid.longitude,
                                mid.latitude,
                                Some(mid_height),
                                Some(ellipsoid),
                            )
                            .unwrap();
                            subdivided_positions.push(mid_cart.x);
                            subdivided_positions.push(mid_cart.y);
                            subdivided_positions.push(mid_cart.z);
                            let new_idx = (subdivided_positions.len() / 3 - 1) as u32;
                            edges.insert(edge, new_idx);
                            if has_texcoords {
                                let mid_tc = (t2 + t0) * 0.5;
                                subdivided_texcoords.push(mid_tc.x);
                                subdivided_texcoords.push(mid_tc.y);
                            }
                            new_idx
                        }
                    };
                    triangles.push(i2);
                    triangles.push(i);
                    triangles.push(i1);

                    triangles.push(i);
                    triangles.push(i0);
                    triangles.push(i1);
                }
            } else {
                subdivided_indices.push(i0);
                subdivided_indices.push(i1);
                subdivided_indices.push(i2);
            }
        }

        let mut attributes = GeometryAttributes {
            position: Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                subdivided_positions,
            )),
            ..Default::default()
        };

        if has_texcoords {
            attributes.st = Some(GeometryAttribute::new(
                ComponentDatatype::Float,
                2,
                subdivided_texcoords,
            ));
        }

        Ok(Geometry {
            attributes,
            indices: subdivided_indices,
            primitive_type: PrimitiveType::Triangles,
            bounding_sphere: None,
        })
    }

    pub fn scale_to_geodetic_height(
        positions: &mut [f64],
        height: Option<f64>,
        ellipsoid: Option<&Ellipsoid>,
        scale_to_surface: Option<bool>,
    ) {
        let e = ellipsoid.copied().unwrap_or(Ellipsoid::WGS84);
        let h = height.unwrap_or(0.0);
        let do_scale = scale_to_surface.unwrap_or(true);

        let len = positions.len();
        for i in (0..len).step_by(3) {
            if i + 2 >= len {
                break;
            }
            let mut p = Cartesian3::new(positions[i], positions[i + 1], positions[i + 2]);
            if do_scale {
                if let Some(surf) = e.scale_to_geodetic_surface(p.into()) {
                    p = Cartesian3::from(surf);
                }
            }
            if h != 0.0 {
                if let Some(n) = e.geodetic_surface_normal(p.into()) {
                    p = p + Cartesian3::from(n) * h;
                }
            }
            positions[i] = p.x;
            positions[i + 1] = p.y;
            positions[i + 2] = p.z;
        }
    }
}
