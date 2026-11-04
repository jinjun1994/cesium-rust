//! PolygonOutlineGeometry matching CesiumJS `Core/PolygonOutlineGeometry`.
//! Adapted from CesiumJS (Apache-2.0).

use glam::DVec3;

use crate::cartesian::{Cartesian2, Cartesian3};
use crate::coplanar_polygon_geometry::PolygonHierarchy;
use crate::core_math::{chord_length, EPSILON2, RADIANS_PER_DEGREE};
use crate::ellipsoid::Ellipsoid;
use crate::ellipsoid_tangent_plane::EllipsoidTangentPlane;
use crate::errors::DeveloperError;
use crate::geometry::{
    ArcType, ComponentDatatype, Geometry, GeometryAttribute, GeometryAttributes,
    GeometryOffsetAttribute, PrimitiveType,
};
use crate::polygon_geometry_library::PolygonGeometryLibrary;
use crate::polygon_pipeline::PolygonPipeline;
use crate::winding_order::WindingOrder;
use crate::BoundingSphere;

#[derive(Clone, Debug, PartialEq)]
pub struct PolygonOutlineGeometryOptions {
    pub polygon_hierarchy: PolygonHierarchy,
    pub height: Option<f64>,
    pub extruded_height: Option<f64>,
    pub ellipsoid: Option<Ellipsoid>,
    pub granularity: Option<f64>,
    pub per_position_height: Option<bool>,
    pub arc_type: Option<ArcType>,
    pub offset_attribute: Option<GeometryOffsetAttribute>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PolygonOutlineGeometryFromPositionsOptions {
    pub positions: Vec<Cartesian3>,
    pub height: Option<f64>,
    pub extruded_height: Option<f64>,
    pub ellipsoid: Option<Ellipsoid>,
    pub granularity: Option<f64>,
    pub per_position_height: Option<bool>,
    pub arc_type: Option<ArcType>,
    pub offset_attribute: Option<GeometryOffsetAttribute>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PolygonOutlineGeometry {
    polygon_hierarchy: PolygonHierarchy,
    ellipsoid: Ellipsoid,
    height: f64,
    extruded_height: f64,
    granularity: f64,
    per_position_height: bool,
    per_position_height_extrude: bool,
    arc_type: ArcType,
    offset_attribute: Option<GeometryOffsetAttribute>,
    packed_length: usize,
}

struct RingGeometry {
    positions: Vec<f64>,
    indices: Vec<u32>,
    apply_offset: Option<Vec<f64>>,
}

fn create_geometry_from_positions(
    ellipsoid: &Ellipsoid,
    positions: &[Cartesian3],
    min_distance: f64,
    per_position_height: bool,
    arc_type: ArcType,
) -> Result<RingGeometry, DeveloperError> {
    let points_dvec: Vec<DVec3> = positions.iter().map(|p| (*p).into()).collect();
    let tangent_plane = EllipsoidTangentPlane::from_points(&points_dvec, *ellipsoid)
        .map_err(|e| DeveloperError::new(format!("Tangent plane error: {:?}", e)))?;
    let positions_2d = tangent_plane.project_points_onto_plane(&points_dvec);
    let positions_cart2: Vec<Cartesian2> = positions_2d
        .iter()
        .map(|p| Cartesian2::new(p.x, p.y))
        .collect();

    let mut positions_work = positions.to_vec();
    if let Ok(order) = PolygonPipeline::compute_winding_order_2d(&positions_cart2) {
        if order == WindingOrder::Clockwise {
            positions_work.reverse();
        }
    }

    let length = positions_work.len();
    let mut subdivided_positions = Vec::new();

    if !per_position_height {
        for i in 0..length {
            let p0 = positions_work[i];
            let p1 = positions_work[(i + 1) % length];
            if arc_type == ArcType::Geodesic {
                let temp = PolygonGeometryLibrary::subdivide_line(p0, p1, min_distance, None);
                subdivided_positions.extend_from_slice(&temp);
            } else if arc_type == ArcType::Rhumb {
                let temp =
                    PolygonGeometryLibrary::subdivide_rhumb_line(ellipsoid, p0, p1, min_distance);
                subdivided_positions.extend_from_slice(&temp);
            }
        }
    } else {
        for i in 0..length {
            let p0 = positions_work[i];
            let p1 = positions_work[(i + 1) % length];
            subdivided_positions.push(p0.x);
            subdivided_positions.push(p0.y);
            subdivided_positions.push(p0.z);
            subdivided_positions.push(p1.x);
            subdivided_positions.push(p1.y);
            subdivided_positions.push(p1.z);
        }
    }

    let num_verts = subdivided_positions.len() / 3;
    let mut indices = Vec::with_capacity(num_verts * 2);
    for i in 0..(num_verts - 1) {
        indices.push(i as u32);
        indices.push((i + 1) as u32);
    }
    indices.push((num_verts - 1) as u32);
    indices.push(0);

    Ok(RingGeometry {
        positions: subdivided_positions,
        indices,
        apply_offset: None,
    })
}

fn create_geometry_from_positions_extruded(
    ellipsoid: &Ellipsoid,
    positions: &[Cartesian3],
    min_distance: f64,
    per_position_height: bool,
    arc_type: ArcType,
) -> Result<RingGeometry, DeveloperError> {
    let points_dvec: Vec<DVec3> = positions.iter().map(|p| (*p).into()).collect();
    let tangent_plane = EllipsoidTangentPlane::from_points(&points_dvec, *ellipsoid)
        .map_err(|e| DeveloperError::new(format!("Tangent plane error: {:?}", e)))?;
    let positions_2d = tangent_plane.project_points_onto_plane(&points_dvec);
    let positions_cart2: Vec<Cartesian2> = positions_2d
        .iter()
        .map(|p| Cartesian2::new(p.x, p.y))
        .collect();

    let mut positions_work = positions.to_vec();
    if let Ok(order) = PolygonPipeline::compute_winding_order_2d(&positions_cart2) {
        if order == WindingOrder::Clockwise {
            positions_work.reverse();
        }
    }

    let length = positions_work.len();
    let mut corners = Vec::with_capacity(length);
    let mut subdivided_positions = Vec::new();

    if !per_position_height {
        for i in 0..length {
            corners.push((subdivided_positions.len() / 3) as u32);
            let p0 = positions_work[i];
            let p1 = positions_work[(i + 1) % length];
            if arc_type == ArcType::Geodesic {
                let temp = PolygonGeometryLibrary::subdivide_line(p0, p1, min_distance, None);
                subdivided_positions.extend_from_slice(&temp);
            } else if arc_type == ArcType::Rhumb {
                let temp =
                    PolygonGeometryLibrary::subdivide_rhumb_line(ellipsoid, p0, p1, min_distance);
                subdivided_positions.extend_from_slice(&temp);
            }
        }
    } else {
        for i in 0..length {
            corners.push((subdivided_positions.len() / 3) as u32);
            let p0 = positions_work[i];
            let p1 = positions_work[(i + 1) % length];
            subdivided_positions.push(p0.x);
            subdivided_positions.push(p0.y);
            subdivided_positions.push(p0.z);
            subdivided_positions.push(p1.x);
            subdivided_positions.push(p1.y);
            subdivided_positions.push(p1.z);
        }
    }

    let top_len = subdivided_positions.len() / 3;
    let bottom_slice = subdivided_positions.clone();
    subdivided_positions.extend_from_slice(&bottom_slice);

    let corners_len = corners.len();
    let mut indices = Vec::with_capacity((top_len * 2 + corners_len) * 2);

    for i in 0..top_len {
        indices.push(i as u32);
        indices.push(((i + 1) % top_len) as u32);
        indices.push((i + top_len) as u32);
        indices.push((((i + 1) % top_len) + top_len) as u32);
    }

    for &corner in &corners {
        indices.push(corner);
        indices.push(corner + top_len as u32);
    }

    Ok(RingGeometry {
        positions: subdivided_positions,
        indices,
        apply_offset: None,
    })
}

impl PolygonOutlineGeometry {
    pub fn new(options: PolygonOutlineGeometryOptions) -> Result<Self, DeveloperError> {
        let per_position_height = options.per_position_height.unwrap_or(false);
        if per_position_height && options.height.is_some() {
            return Err(DeveloperError::new(
                "Cannot use both options.perPositionHeight and options.height",
            ));
        }

        let arc_type = options.arc_type.unwrap_or(ArcType::Geodesic);
        if arc_type != ArcType::Geodesic && arc_type != ArcType::Rhumb {
            return Err(DeveloperError::new(
                "Invalid arcType. Valid options are ArcType.GEODESIC and ArcType.RHUMB.",
            ));
        }

        let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let granularity = options.granularity.unwrap_or(RADIANS_PER_DEGREE);
        let per_position_height_extrude =
            per_position_height && options.extruded_height.is_some();

        let mut height = options.height.unwrap_or(0.0);
        let mut extruded_height = options.extruded_height.unwrap_or(height);

        if !per_position_height_extrude {
            let h = height.max(extruded_height);
            extruded_height = height.min(extruded_height);
            height = h;
        }

        let packed_length = PolygonGeometryLibrary::compute_hierarchy_packed_length(
            &options.polygon_hierarchy,
        ) + 3
            + 8;

        Ok(Self {
            polygon_hierarchy: options.polygon_hierarchy,
            ellipsoid,
            height,
            extruded_height,
            granularity,
            per_position_height,
            per_position_height_extrude,
            arc_type,
            offset_attribute: options.offset_attribute,
            packed_length,
        })
    }

    pub fn from_positions(
        options: PolygonOutlineGeometryFromPositionsOptions,
    ) -> Result<Self, DeveloperError> {
        Self::new(PolygonOutlineGeometryOptions {
            polygon_hierarchy: PolygonHierarchy::new(options.positions),
            height: options.height,
            extruded_height: options.extruded_height,
            ellipsoid: options.ellipsoid,
            granularity: options.granularity,
            per_position_height: options.per_position_height,
            arc_type: options.arc_type,
            offset_attribute: options.offset_attribute,
        })
    }

    pub fn polygon_hierarchy(&self) -> &PolygonHierarchy {
        &self.polygon_hierarchy
    }

    pub fn ellipsoid(&self) -> &Ellipsoid {
        &self.ellipsoid
    }

    pub fn height(&self) -> f64 {
        self.height
    }

    pub fn extruded_height(&self) -> f64 {
        self.extruded_height
    }

    pub fn granularity(&self) -> f64 {
        self.granularity
    }

    pub fn per_position_height(&self) -> bool {
        self.per_position_height
    }

    pub fn arc_type(&self) -> ArcType {
        self.arc_type
    }

    pub fn offset_attribute(&self) -> Option<GeometryOffsetAttribute> {
        self.offset_attribute
    }

    pub fn packed_length(&self) -> usize {
        self.packed_length
    }

    pub fn pack(&self, array: &mut Vec<f64>, starting_index: Option<usize>) -> usize {
        let mut idx = starting_index.unwrap_or(0);
        let needed = idx + self.packed_length;
        if array.len() < needed {
            array.resize(needed, 0.0);
        }

        PolygonGeometryLibrary::pack_polygon_hierarchy(
            &self.polygon_hierarchy,
            array,
            &mut idx,
        );

        let radii = self.ellipsoid.radii();
        array[idx] = radii.x;
        array[idx + 1] = radii.y;
        array[idx + 2] = radii.z;
        idx += 3;

        array[idx] = self.height;
        array[idx + 1] = self.extruded_height;
        array[idx + 2] = self.granularity;
        array[idx + 3] = if self.per_position_height_extrude { 1.0 } else { 0.0 };
        array[idx + 4] = if self.per_position_height { 1.0 } else { 0.0 };
        array[idx + 5] = self.arc_type as u32 as f64;
        array[idx + 6] = match self.offset_attribute {
            Some(GeometryOffsetAttribute::None) => 0.0,
            Some(GeometryOffsetAttribute::Top) => 1.0,
            Some(GeometryOffsetAttribute::All) => 2.0,
            None => -1.0,
        };
        array[idx + 7] = self.packed_length as f64;

        idx + 8
    }

    pub fn unpack(
        array: &[f64],
        starting_index: Option<usize>,
        _result: Option<&mut Self>,
    ) -> Result<Self, DeveloperError> {
        let mut idx = starting_index.unwrap_or(0);
        let polygon_hierarchy =
            PolygonGeometryLibrary::unpack_polygon_hierarchy(array, &mut idx)?;

        if idx + 3 > array.len() {
            return Err(DeveloperError::new("Array too short to unpack ellipsoid"));
        }
        let ellipsoid = Ellipsoid::new(DVec3::new(array[idx], array[idx + 1], array[idx + 2]))
            .map_err(|e| DeveloperError::new(format!("{:?}", e)))?;
        idx += 3;

        if idx + 8 > array.len() {
            return Err(DeveloperError::new("Array too short to unpack polygon options"));
        }

        let height = array[idx];
        let extruded_height = array[idx + 1];
        let granularity = array[idx + 2];
        let per_position_height_extrude = array[idx + 3] == 1.0;
        let per_position_height = array[idx + 4] == 1.0;
        let arc_type = match array[idx + 5] as u32 {
            1 => ArcType::Geodesic,
            2 => ArcType::Rhumb,
            _ => ArcType::None,
        };
        let offset_code = array[idx + 6] as i32;
        let offset_attribute = match offset_code {
            0 => Some(GeometryOffsetAttribute::None),
            1 => Some(GeometryOffsetAttribute::Top),
            2 => Some(GeometryOffsetAttribute::All),
            _ => None,
        };
        let packed_length = array[idx + 7] as usize;

        Ok(Self {
            polygon_hierarchy,
            ellipsoid,
            height,
            extruded_height,
            granularity,
            per_position_height,
            per_position_height_extrude,
            arc_type,
            offset_attribute,
            packed_length,
        })
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        let polygons = PolygonGeometryLibrary::polygon_outlines_from_hierarchy(
            &self.polygon_hierarchy,
            !self.per_position_height,
            &self.ellipsoid,
        );

        if polygons.is_empty() {
            return None;
        }

        let min_distance =
            chord_length(self.granularity, self.ellipsoid.maximum_radius());
        let extrude = self.per_position_height_extrude
            || (self.height - self.extruded_height).abs() > EPSILON2;

        let mut ring_geometries = Vec::with_capacity(polygons.len());

        if extrude {
            for poly in &polygons {
                let mut ring = create_geometry_from_positions_extruded(
                    &self.ellipsoid,
                    poly,
                    min_distance,
                    self.per_position_height,
                    self.arc_type,
                )
                .ok()?;

                PolygonGeometryLibrary::scale_to_geodetic_height_extruded(
                    &mut ring.positions,
                    self.height,
                    self.extruded_height,
                    &self.ellipsoid,
                    self.per_position_height,
                );

                if let Some(offset_attr) = self.offset_attribute {
                    let size = ring.positions.len() / 3;
                    let mut offset = vec![0.0; size];
                    if offset_attr == GeometryOffsetAttribute::Top {
                        for item in offset.iter_mut().take(size / 2) {
                            *item = 1.0;
                        }
                    } else {
                        let val = if offset_attr == GeometryOffsetAttribute::None {
                            0.0
                        } else {
                            1.0
                        };
                        offset.fill(val);
                    }
                    ring.apply_offset = Some(offset);
                }

                ring_geometries.push(ring);
            }
        } else {
            for poly in &polygons {
                let mut ring = create_geometry_from_positions(
                    &self.ellipsoid,
                    poly,
                    min_distance,
                    self.per_position_height,
                    self.arc_type,
                )
                .ok()?;

                PolygonPipeline::scale_to_geodetic_height(
                    &mut ring.positions,
                    Some(self.height),
                    Some(&self.ellipsoid),
                    Some(!self.per_position_height),
                );

                if let Some(offset_attr) = self.offset_attribute {
                    let size = ring.positions.len() / 3;
                    let val = if offset_attr == GeometryOffsetAttribute::None {
                        0.0
                    } else {
                        1.0
                    };
                    ring.apply_offset = Some(vec![val; size]);
                }

                ring_geometries.push(ring);
            }
        }

        let mut combined_positions = Vec::new();
        let mut combined_indices = Vec::new();
        let mut combined_apply_offset = Vec::new();
        let mut current_offset = 0u32;
        let has_apply_offset = self.offset_attribute.is_some();

        for ring in ring_geometries {
            let num_verts = (ring.positions.len() / 3) as u32;
            combined_positions.extend_from_slice(&ring.positions);
            for idx in ring.indices {
                combined_indices.push(idx + current_offset);
            }
            if has_apply_offset {
                if let Some(offset) = ring.apply_offset {
                    combined_apply_offset.extend_from_slice(&offset);
                }
            }
            current_offset += num_verts;
        }

        let bs = BoundingSphere::from_vertices(
            &combined_positions,
            Cartesian3::ZERO.into(),
            3,
        )
        .ok()?;

        let mut attributes = GeometryAttributes {
            position: Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                combined_positions,
            )),
            ..Default::default()
        };

        if has_apply_offset {
            attributes.apply_offset = Some(GeometryAttribute::new(
                ComponentDatatype::UnsignedByte,
                1,
                combined_apply_offset,
            ));
        }

        Some(Geometry {
            attributes,
            indices: combined_indices,
            primitive_type: PrimitiveType::Lines,
            bounding_sphere: Some(bs),
        })
    }
}
