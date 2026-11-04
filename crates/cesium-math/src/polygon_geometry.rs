//! PolygonGeometry matching CesiumJS `Core/PolygonGeometry`.
//! Adapted from CesiumJS (Apache-2.0).

use std::f64::consts::{FRAC_PI_2, PI};

use glam::DVec3;

use crate::cartesian::{Cartesian2, Cartesian3};
use crate::coplanar_polygon_geometry::PolygonHierarchy;
use crate::core_math::{EPSILON10, EPSILON2, RADIANS_PER_DEGREE, TWO_PI};
use crate::ellipsoid::Ellipsoid;
use crate::ellipsoid_tangent_plane::EllipsoidTangentPlane;
use crate::errors::DeveloperError;
use crate::geometry::{
    ArcType, ComponentDatatype, Geometry, GeometryAttribute, GeometryAttributes,
    GeometryOffsetAttribute, PrimitiveType,
};
use crate::polygon_geometry_library::PolygonGeometryLibrary;
use crate::polygon_pipeline::PolygonPipeline;
use crate::rectangle::GeoRectangle;
use crate::stereographic::Stereographic;
use crate::vertex_format::VertexFormat;
use crate::winding_order::WindingOrder;
use crate::BoundingSphere;

#[derive(Clone, Debug, PartialEq)]
pub struct PolygonGeometryOptions {
    pub polygon_hierarchy: PolygonHierarchy,
    pub height: Option<f64>,
    pub extruded_height: Option<f64>,
    pub vertex_format: Option<VertexFormat>,
    pub ellipsoid: Option<Ellipsoid>,
    pub granularity: Option<f64>,
    pub per_position_height: Option<bool>,
    pub close_top: Option<bool>,
    pub close_bottom: Option<bool>,
    pub arc_type: Option<ArcType>,
    pub st_rotation: Option<f64>,
    pub offset_attribute: Option<GeometryOffsetAttribute>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PolygonGeometryFromPositionsOptions {
    pub positions: Vec<Cartesian3>,
    pub height: Option<f64>,
    pub extruded_height: Option<f64>,
    pub vertex_format: Option<VertexFormat>,
    pub ellipsoid: Option<Ellipsoid>,
    pub granularity: Option<f64>,
    pub per_position_height: Option<bool>,
    pub close_top: Option<bool>,
    pub close_bottom: Option<bool>,
    pub arc_type: Option<ArcType>,
    pub st_rotation: Option<f64>,
    pub offset_attribute: Option<GeometryOffsetAttribute>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PolygonGeometry {
    polygon_hierarchy: PolygonHierarchy,
    vertex_format: VertexFormat,
    ellipsoid: Ellipsoid,
    granularity: f64,
    height: f64,
    extruded_height: f64,
    per_position_height: bool,
    per_position_height_extrude: bool,
    close_top: bool,
    close_bottom: bool,
    arc_type: ArcType,
    st_rotation: f64,
    offset_attribute: Option<GeometryOffsetAttribute>,
    packed_length: usize,
    rectangle: GeoRectangle,
}

struct PolygonAngleTracker {
    north_angle: f64,
    south_angle: f64,
    west_over_idl: f64,
    east_over_idl: f64,
}

fn expand_rectangle(
    polar: &Stereographic,
    last_polar: &Stereographic,
    ellipsoid: &Ellipsoid,
    arc_type: ArcType,
    tracker: &mut PolygonAngleTracker,
    result: &mut GeoRectangle,
) {
    let longitude = polar.longitude();
    let lon_adjusted = if longitude >= 0.0 {
        longitude
    } else {
        longitude + TWO_PI
    };
    tracker.west_over_idl = tracker.west_over_idl.min(lon_adjusted);
    tracker.east_over_idl = tracker.east_over_idl.max(lon_adjusted);

    result.west = result.west.min(longitude);
    result.east = result.east.max(longitude);

    let latitude = polar.get_latitude(Some(ellipsoid));
    let mut segment_latitude = latitude;

    result.south = result.south.min(latitude);
    result.north = result.north.max(latitude);

    if arc_type != ArcType::Rhumb {
        let segment = last_polar.position - polar.position;
        let denom = segment.dot(&segment);
        if denom > 0.0 {
            let t = last_polar.position.dot(&segment) / denom;
            if t > 0.0 && t < 1.0 {
                let projected = last_polar.position - segment * t;
                let mut closest_polar = *last_polar;
                closest_polar.position = projected;
                let adjusted_lat = closest_polar.get_latitude(Some(ellipsoid));
                result.south = result.south.min(adjusted_lat);
                result.north = result.north.max(adjusted_lat);

                if latitude.abs() > adjusted_lat.abs() {
                    segment_latitude = adjusted_lat;
                }
            }
        }
    }

    let direction = last_polar.x() * polar.y() - polar.x() * last_polar.y();
    let mut angle = if direction > 0.0 {
        1.0
    } else if direction < 0.0 {
        -1.0
    } else {
        0.0
    };

    if angle != 0.0 {
        angle *= last_polar.position.angle_between(&polar.position);
    }

    if segment_latitude >= 0.0 {
        tracker.north_angle += angle;
    }
    if segment_latitude <= 0.0 {
        tracker.south_angle += angle;
    }
}

impl PolygonGeometry {
    pub fn compute_rectangle_from_positions(
        positions: &[Cartesian3],
        ellipsoid: Option<&Ellipsoid>,
        arc_type: Option<ArcType>,
    ) -> Result<GeoRectangle, DeveloperError> {
        let ellipsoid = ellipsoid.copied().unwrap_or(Ellipsoid::WGS84);
        let arc_type = arc_type.unwrap_or(ArcType::Geodesic);

        if positions.len() < 3 {
            return Ok(GeoRectangle::default());
        }

        let mut result = GeoRectangle {
            west: f64::INFINITY,
            east: f64::NEG_INFINITY,
            south: f64::INFINITY,
            north: f64::NEG_INFINITY,
        };

        let mut tracker = PolygonAngleTracker {
            north_angle: 0.0,
            south_angle: 0.0,
            west_over_idl: f64::INFINITY,
            east_over_idl: f64::NEG_INFINITY,
        };

        let mut last_polar = Stereographic::from_cartesian(positions[0])?;
        for &pos in positions.iter().skip(1) {
            let polar = Stereographic::from_cartesian(pos)?;
            expand_rectangle(
                &polar,
                &last_polar,
                &ellipsoid,
                arc_type,
                &mut tracker,
                &mut result,
            );
            last_polar = polar;
        }

        let first_polar = Stereographic::from_cartesian(positions[0])?;
        expand_rectangle(
            &first_polar,
            &last_polar,
            &ellipsoid,
            arc_type,
            &mut tracker,
            &mut result,
        );

        if result.east - result.west > tracker.east_over_idl - tracker.west_over_idl {
            result.west = tracker.west_over_idl;
            result.east = tracker.east_over_idl;

            if result.east > PI {
                result.east -= TWO_PI;
            }
            if result.west > PI {
                result.west -= TWO_PI;
            }
        }

        if (tracker.north_angle.abs() - TWO_PI).abs() < EPSILON10 {
            result.north = FRAC_PI_2;
            result.east = PI;
            result.west = -PI;
        }

        if (tracker.south_angle.abs() - TWO_PI).abs() < EPSILON10 {
            result.south = -FRAC_PI_2;
            result.east = PI;
            result.west = -PI;
        }

        Ok(result)
    }

    pub fn new(options: PolygonGeometryOptions) -> Result<Self, DeveloperError> {
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

        let vertex_format = options.vertex_format.unwrap_or(VertexFormat::DEFAULT);
        let close_top = options.close_top.unwrap_or(true);
        let close_bottom = options.close_bottom.unwrap_or(true);
        let st_rotation = options.st_rotation.unwrap_or(0.0);

        let rectangle = Self::compute_rectangle_from_positions(
            &options.polygon_hierarchy.positions,
            Some(&ellipsoid),
            Some(arc_type),
        )?;

        let packed_length = PolygonGeometryLibrary::compute_hierarchy_packed_length(
            &options.polygon_hierarchy,
        ) + 3 // ellipsoid
            + 6 // vertex_format
            + 12;

        Ok(Self {
            polygon_hierarchy: options.polygon_hierarchy,
            vertex_format,
            ellipsoid,
            granularity,
            height,
            extruded_height,
            per_position_height,
            per_position_height_extrude,
            close_top,
            close_bottom,
            arc_type,
            st_rotation,
            offset_attribute: options.offset_attribute,
            packed_length,
            rectangle,
        })
    }

    pub fn from_positions(
        options: PolygonGeometryFromPositionsOptions,
    ) -> Result<Self, DeveloperError> {
        Self::new(PolygonGeometryOptions {
            polygon_hierarchy: PolygonHierarchy::new(options.positions),
            height: options.height,
            extruded_height: options.extruded_height,
            vertex_format: options.vertex_format,
            ellipsoid: options.ellipsoid,
            granularity: options.granularity,
            per_position_height: options.per_position_height,
            close_top: options.close_top,
            close_bottom: options.close_bottom,
            arc_type: options.arc_type,
            st_rotation: options.st_rotation,
            offset_attribute: options.offset_attribute,
        })
    }

    pub fn polygon_hierarchy(&self) -> &PolygonHierarchy {
        &self.polygon_hierarchy
    }

    pub fn vertex_format(&self) -> &VertexFormat {
        &self.vertex_format
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

    pub fn close_top(&self) -> bool {
        self.close_top
    }

    pub fn close_bottom(&self) -> bool {
        self.close_bottom
    }

    pub fn arc_type(&self) -> ArcType {
        self.arc_type
    }

    pub fn st_rotation(&self) -> f64 {
        self.st_rotation
    }

    pub fn offset_attribute(&self) -> Option<GeometryOffsetAttribute> {
        self.offset_attribute
    }

    pub fn rectangle(&self) -> &GeoRectangle {
        &self.rectangle
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

        array[idx] = if self.vertex_format.position { 1.0 } else { 0.0 };
        array[idx + 1] = if self.vertex_format.normal { 1.0 } else { 0.0 };
        array[idx + 2] = if self.vertex_format.st { 1.0 } else { 0.0 };
        array[idx + 3] = if self.vertex_format.tangent { 1.0 } else { 0.0 };
        array[idx + 4] = if self.vertex_format.bitangent { 1.0 } else { 0.0 };
        array[idx + 5] = if self.vertex_format.color { 1.0 } else { 0.0 };
        idx += 6;

        array[idx] = self.height;
        array[idx + 1] = self.extruded_height;
        array[idx + 2] = self.granularity;
        array[idx + 3] = self.st_rotation;
        array[idx + 4] = if self.per_position_height_extrude { 1.0 } else { 0.0 };
        array[idx + 5] = if self.per_position_height { 1.0 } else { 0.0 };
        array[idx + 6] = if self.close_top { 1.0 } else { 0.0 };
        array[idx + 7] = if self.close_bottom { 1.0 } else { 0.0 };
        array[idx + 8] = self.arc_type as u32 as f64;
        array[idx + 9] = match self.offset_attribute {
            Some(GeometryOffsetAttribute::None) => 0.0,
            Some(GeometryOffsetAttribute::Top) => 1.0,
            Some(GeometryOffsetAttribute::All) => 2.0,
            None => -1.0,
        };
        array[idx + 10] = 0.0; // shadowVolume
        array[idx + 11] = self.packed_length as f64;

        idx + 12
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

        if idx + 6 > array.len() {
            return Err(DeveloperError::new("Array too short to unpack vertex format"));
        }
        let vertex_format = VertexFormat {
            position: array[idx] == 1.0,
            normal: array[idx + 1] == 1.0,
            st: array[idx + 2] == 1.0,
            tangent: array[idx + 3] == 1.0,
            bitangent: array[idx + 4] == 1.0,
            color: array[idx + 5] == 1.0,
        };
        idx += 6;

        if idx + 12 > array.len() {
            return Err(DeveloperError::new("Array too short to unpack polygon options"));
        }

        let height = array[idx];
        let extruded_height = array[idx + 1];
        let granularity = array[idx + 2];
        let st_rotation = array[idx + 3];
        let per_position_height_extrude = array[idx + 4] == 1.0;
        let per_position_height = array[idx + 5] == 1.0;
        let close_top = array[idx + 6] == 1.0;
        let close_bottom = array[idx + 7] == 1.0;
        let arc_type = match array[idx + 8] as u32 {
            1 => ArcType::Geodesic,
            2 => ArcType::Rhumb,
            _ => ArcType::None,
        };
        let offset_code = array[idx + 9] as i32;
        let offset_attribute = match offset_code {
            0 => Some(GeometryOffsetAttribute::None),
            1 => Some(GeometryOffsetAttribute::Top),
            2 => Some(GeometryOffsetAttribute::All),
            _ => None,
        };
        let packed_length = array[idx + 11] as usize;

        let rectangle = Self::compute_rectangle_from_positions(
            &polygon_hierarchy.positions,
            Some(&ellipsoid),
            Some(arc_type),
        )?;

        Ok(Self {
            polygon_hierarchy,
            vertex_format,
            ellipsoid,
            granularity,
            height,
            extruded_height,
            per_position_height,
            per_position_height_extrude,
            close_top,
            close_bottom,
            arc_type,
            st_rotation,
            offset_attribute,
            packed_length,
            rectangle,
        })
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        let mut outer_positions = self.polygon_hierarchy.positions.clone();
        if !self.per_position_height {
            for pt in &mut outer_positions {
                if let Some(scaled) = self.ellipsoid.scale_to_geodetic_surface((*pt).into()) {
                    *pt = Cartesian3::from(scaled);
                }
            }
        }
        outer_positions = PolygonGeometryLibrary::array_remove_duplicates(&outer_positions, true);
        if outer_positions.len() < 3 {
            return None;
        }

        let mut cleaned_holes = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        for hole in &self.polygon_hierarchy.holes {
            queue.push_back(hole);
        }
        while let Some(hole) = queue.pop_front() {
            let mut hole_positions = hole.positions.clone();
            if !self.per_position_height {
                for pt in &mut hole_positions {
                    if let Some(scaled) = self.ellipsoid.scale_to_geodetic_surface((*pt).into()) {
                        *pt = Cartesian3::from(scaled);
                    }
                }
            }
            hole_positions = PolygonGeometryLibrary::array_remove_duplicates(&hole_positions, true);
            if hole_positions.len() >= 3 {
                cleaned_holes.push(hole_positions);
            }
            for child in &hole.holes {
                queue.push_back(child);
            }
        }

        let points_dvec: Vec<DVec3> = outer_positions.iter().map(|p| (*p).into()).collect();
        let tangent_plane = EllipsoidTangentPlane::from_points(&points_dvec, self.ellipsoid).ok()?;
        let outer_2d = tangent_plane.project_points_onto_plane(&points_dvec);
        let outer_cart2: Vec<Cartesian2> = outer_2d
            .iter()
            .map(|p| Cartesian2::new(p.x, p.y))
            .collect();

        let mut outer_3d_work = outer_positions.clone();
        let mut outer_2d_work = outer_cart2;
        if let Ok(order) = PolygonPipeline::compute_winding_order_2d(&outer_2d_work) {
            if order == WindingOrder::Clockwise {
                outer_3d_work.reverse();
                outer_2d_work.reverse();
            }
        }

        let mut all_2d = outer_2d_work;
        let mut all_3d = outer_3d_work;
        let mut hole_indices = Vec::new();

        for hole in cleaned_holes {
            let hole_dvec: Vec<DVec3> = hole.iter().map(|p| (*p).into()).collect();
            let hole_2d = tangent_plane.project_points_onto_plane(&hole_dvec);
            let mut hole_cart2: Vec<Cartesian2> =
                hole_2d.iter().map(|p| Cartesian2::new(p.x, p.y)).collect();
            let mut hole_3d_work = hole;
            if let Ok(order) = PolygonPipeline::compute_winding_order_2d(&hole_cart2) {
                if order == WindingOrder::CounterClockwise {
                    hole_3d_work.reverse();
                    hole_cart2.reverse();
                }
            }
            hole_indices.push(all_2d.len());
            all_2d.extend(hole_cart2);
            all_3d.extend(hole_3d_work);
        }

        let indices = PolygonPipeline::triangulate(&all_2d, &hole_indices);
        if indices.len() < 3 {
            return None;
        }

        let subdivided = if self.arc_type == ArcType::Rhumb {
            PolygonPipeline::compute_rhumb_line_subdivision(
                &self.ellipsoid,
                &all_3d,
                &indices,
                None,
                Some(self.granularity),
            )
            .ok()?
        } else {
            PolygonPipeline::compute_subdivision(
                &self.ellipsoid,
                &all_3d,
                &indices,
                None,
                Some(self.granularity),
            )
            .ok()?
        };

        let mut flat_positions = subdivided.attributes.position?.values;
        let original_top_indices = subdivided.indices;
        let top_vert_count = (flat_positions.len() / 3) as u32;

        let extrude = self.per_position_height_extrude
            || (self.height - self.extruded_height).abs() > EPSILON2;

        let mut final_indices = Vec::new();

        if extrude {
            if self.close_top {
                final_indices.extend_from_slice(&original_top_indices);
            }

            let bottom_slice = flat_positions.clone();
            flat_positions.extend_from_slice(&bottom_slice);

            if self.close_bottom {
                for chunk in original_top_indices.as_chunks::<3>().0 {
                    let i0 = chunk[0] + top_vert_count;
                    let i1 = chunk[1] + top_vert_count;
                    let i2 = chunk[2] + top_vert_count;
                    final_indices.push(i0);
                    final_indices.push(i2);
                    final_indices.push(i1);
                }
            }

            // 侧壁 walls: 连接外环顶底
            let mut wall_corners = Vec::new();
            let boundary_len = all_3d.len();
            for i in 0..boundary_len {
                wall_corners.push(i as u32);
            }
            // 简单侧壁构建
            for i in 0..boundary_len {
                let curr = i as u32;
                let next = ((i + 1) % boundary_len) as u32;
                let curr_b = curr + top_vert_count;
                let next_b = next + top_vert_count;
                final_indices.push(curr);
                final_indices.push(curr_b);
                final_indices.push(next_b);

                final_indices.push(curr);
                final_indices.push(next_b);
                final_indices.push(next);
            }

            PolygonGeometryLibrary::scale_to_geodetic_height_extruded(
                &mut flat_positions,
                self.height,
                self.extruded_height,
                &self.ellipsoid,
                self.per_position_height,
            );
        } else {
            final_indices = original_top_indices;
            PolygonPipeline::scale_to_geodetic_height(
                &mut flat_positions,
                Some(self.height),
                Some(&self.ellipsoid),
                Some(!self.per_position_height),
            );
        }

        let total_verts = flat_positions.len() / 3;
        let mut attributes = GeometryAttributes {
            position: Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                flat_positions.clone(),
            )),
            ..Default::default()
        };

        if self.vertex_format.normal {
            let mut normals = vec![0.0; total_verts * 3];
            for i in (0..flat_positions.len()).step_by(3) {
                let p = DVec3::new(flat_positions[i], flat_positions[i + 1], flat_positions[i + 2]);
                let norm = self.ellipsoid.geodetic_surface_normal(p).unwrap_or(DVec3::Z);
                normals[i] = norm.x;
                normals[i + 1] = norm.y;
                normals[i + 2] = norm.z;
            }
            attributes.normal = Some(GeometryAttribute::new(
                ComponentDatatype::Float,
                3,
                normals,
            ));
        }

        if self.vertex_format.st {
            let mut sts = Vec::with_capacity(total_verts * 2);
            let west = self.rectangle.west;
            let east = self.rectangle.east;
            let south = self.rectangle.south;
            let north = self.rectangle.north;
            let width = if east >= west { east - west } else { east - west + TWO_PI };
            let height = (north - south).max(1e-10);

            for i in (0..flat_positions.len()).step_by(3) {
                let p = DVec3::new(flat_positions[i], flat_positions[i + 1], flat_positions[i + 2]);
                if let Some(carto) = self.ellipsoid.cartesian_to_cartographic(p) {
                    let mut u = (carto.longitude - west) / width;
                    if u < 0.0 {
                        u += 1.0;
                    }
                    let v = ((carto.latitude - south) / height).clamp(0.0, 1.0);
                    sts.push(u);
                    sts.push(v);
                } else {
                    sts.push(0.0);
                    sts.push(0.0);
                }
            }
            attributes.st = Some(GeometryAttribute::new(
                ComponentDatatype::Float,
                2,
                sts,
            ));
        }

        if let Some(offset_attr) = self.offset_attribute {
            let mut apply_offset = vec![0.0; total_verts];
            if extrude {
                if offset_attr == GeometryOffsetAttribute::Top {
                    for item in apply_offset.iter_mut().take(top_vert_count as usize) {
                        *item = 1.0;
                    }
                } else {
                    let val = if offset_attr == GeometryOffsetAttribute::None {
                        0.0
                    } else {
                        1.0
                    };
                    apply_offset.fill(val);
                }
            } else {
                let val = if offset_attr == GeometryOffsetAttribute::None {
                    0.0
                } else {
                    1.0
                };
                apply_offset.fill(val);
            }
            attributes.apply_offset = Some(GeometryAttribute::new(
                ComponentDatatype::UnsignedByte,
                1,
                apply_offset,
            ));
        }

        let bs = BoundingSphere::from_vertices(
            &flat_positions,
            Cartesian3::ZERO.into(),
            3,
        )
        .ok()?;

        Some(Geometry {
            attributes,
            indices: final_indices,
            primitive_type: PrimitiveType::Triangles,
            bounding_sphere: Some(bs),
        })
    }
}
