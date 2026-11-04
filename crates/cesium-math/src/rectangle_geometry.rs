//! RectangleGeometry matching CesiumJS `Core/RectangleGeometry`.
//! Adapted from CesiumJS (Apache-2.0).

use std::f64::consts::{FRAC_PI_2, PI};

use glam::DVec3;

use crate::cartesian::Cartesian2;
use crate::core_math::{EPSILON10, EPSILON2, RADIANS_PER_DEGREE};
use crate::ellipsoid::Ellipsoid;
use crate::errors::DeveloperError;
use crate::geometry::{
    ComponentDatatype, Geometry, GeometryAttribute, GeometryAttributes, GeometryOffsetAttribute,
    PrimitiveType,
};
use crate::matrices::{Matrix2, Matrix3};
use crate::polygon_pipeline::PolygonPipeline;
use crate::quaternion::Quaternion;
use crate::rectangle::GeoRectangle;
use crate::rectangle_geometry_library::{RectangleGeometryLibrary, RectangleOptionsComputed};
use crate::vertex_format::VertexFormat;
use crate::BoundingSphere;

#[derive(Clone, Debug, PartialEq)]
pub struct RectangleGeometryOptions {
    pub rectangle: GeoRectangle,
    pub vertex_format: Option<VertexFormat>,
    pub ellipsoid: Option<Ellipsoid>,
    pub granularity: Option<f64>,
    pub height: Option<f64>,
    pub rotation: Option<f64>,
    pub st_rotation: Option<f64>,
    pub extruded_height: Option<f64>,
    pub shadow_volume: Option<bool>,
    pub offset_attribute: Option<GeometryOffsetAttribute>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RectangleGeometry {
    pub(crate) rectangle: GeoRectangle,
    pub(crate) vertex_format: VertexFormat,
    pub(crate) ellipsoid: Ellipsoid,
    pub(crate) granularity: f64,
    pub(crate) surface_height: f64,
    pub(crate) rotation: f64,
    pub(crate) st_rotation: f64,
    pub(crate) extruded_height: f64,
    pub(crate) shadow_volume: bool,
    pub(crate) offset_attribute: Option<GeometryOffsetAttribute>,
}

impl RectangleGeometry {
    pub const PACKED_LENGTH: usize =
        GeoRectangle::PACKED_LENGTH + Ellipsoid::PACKED_LENGTH + VertexFormat::PACKED_LENGTH + 7;

    pub fn new(options: RectangleGeometryOptions) -> Result<Self, DeveloperError> {
        let rectangle = options.rectangle;
        if rectangle.north < -FRAC_PI_2
            || rectangle.north > FRAC_PI_2
            || rectangle.south < -FRAC_PI_2
            || rectangle.south > FRAC_PI_2
            || rectangle.west < -PI
            || rectangle.west > PI
            || rectangle.east < -PI
            || rectangle.east > PI
        {
            return Err(DeveloperError::new(
                "Rectangle coordinates are out of valid range.",
            ));
        }

        if rectangle.north < rectangle.south {
            return Err(DeveloperError::new(
                "options.rectangle.north must be greater than or equal to options.rectangle.south",
            ));
        }

        let granularity = options.granularity.unwrap_or(RADIANS_PER_DEGREE);
        let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let rotation = options.rotation.unwrap_or(0.0);
        let st_rotation = options.st_rotation.unwrap_or(0.0);
        let vertex_format = options.vertex_format.unwrap_or_default();
        let height = options.height.unwrap_or(0.0);
        let extruded_height = options.extruded_height.unwrap_or(height);
        let surface_height = height.max(extruded_height);
        let min_extruded_height = height.min(extruded_height);
        let shadow_volume = options.shadow_volume.unwrap_or(false);

        Ok(Self {
            rectangle,
            vertex_format,
            ellipsoid,
            granularity,
            surface_height,
            rotation,
            st_rotation,
            extruded_height: min_extruded_height,
            shadow_volume,
            offset_attribute: options.offset_attribute,
        })
    }

    pub fn pack(&self, array: &mut [f64], starting_index: usize) -> Result<(), DeveloperError> {
        if array.len() < starting_index + Self::PACKED_LENGTH {
            return Err(DeveloperError::new("Array is too short"));
        }
        let mut idx = starting_index;
        self.rectangle.pack(array, idx)?;
        idx += GeoRectangle::PACKED_LENGTH;

        self.ellipsoid.pack(array, idx)?;
        idx += Ellipsoid::PACKED_LENGTH;

        self.vertex_format
            .pack(array, idx)
            .map_err(|_| DeveloperError::new("Failed to pack vertex format"))?;
        idx += VertexFormat::PACKED_LENGTH;

        array[idx] = self.granularity;
        array[idx + 1] = self.surface_height;
        array[idx + 2] = self.rotation;
        array[idx + 3] = self.st_rotation;
        array[idx + 4] = self.extruded_height;
        array[idx + 5] = if self.shadow_volume { 1.0 } else { 0.0 };
        array[idx + 6] = match self.offset_attribute {
            Some(GeometryOffsetAttribute::Top) => 1.0,
            Some(GeometryOffsetAttribute::All) => 2.0,
            Some(GeometryOffsetAttribute::None) => 0.0,
            None => -1.0,
        };
        Ok(())
    }

    pub fn unpack(array: &[f64], starting_index: usize) -> Result<Self, DeveloperError> {
        if array.len() < starting_index + Self::PACKED_LENGTH {
            return Err(DeveloperError::new("Array is too short"));
        }
        let mut idx = starting_index;
        let rectangle = GeoRectangle::unpack(array, idx)?;
        idx += GeoRectangle::PACKED_LENGTH;

        let ellipsoid = Ellipsoid::unpack(array, idx)?;
        idx += Ellipsoid::PACKED_LENGTH;

        let vertex_format = VertexFormat::unpack(array, idx)
            .map_err(|_| DeveloperError::new("Failed to unpack vertex format"))?;
        idx += VertexFormat::PACKED_LENGTH;

        let granularity = array[idx];
        let surface_height = array[idx + 1];
        let rotation = array[idx + 2];
        let st_rotation = array[idx + 3];
        let extruded_height = array[idx + 4];
        let shadow_volume = array[idx + 5] == 1.0;
        let offset_code = array[idx + 6] as i32;
        let offset_attribute = match offset_code {
            1 => Some(GeometryOffsetAttribute::Top),
            2 => Some(GeometryOffsetAttribute::All),
            0 => Some(GeometryOffsetAttribute::None),
            _ => None,
        };

        Ok(Self {
            rectangle,
            vertex_format,
            ellipsoid,
            granularity,
            surface_height,
            rotation,
            st_rotation,
            extruded_height,
            shadow_volume,
            offset_attribute,
        })
    }

    pub fn compute_rectangle(
        options: &RectangleGeometryOptions,
    ) -> Result<GeoRectangle, DeveloperError> {
        let rectangle = options.rectangle;
        if rectangle.north < rectangle.south {
            return Err(DeveloperError::new(
                "options.rectangle.north must be greater than or equal to options.rectangle.south",
            ));
        }
        let granularity = options.granularity.unwrap_or(RADIANS_PER_DEGREE);
        let rotation = options.rotation.unwrap_or(0.0);
        let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);

        compute_rectangle_internal(&rectangle, granularity, rotation, &ellipsoid)
    }

    pub fn rectangle(&self) -> GeoRectangle {
        compute_rectangle_internal(
            &self.rectangle,
            self.granularity,
            self.rotation,
            &self.ellipsoid,
        )
        .unwrap_or(self.rectangle)
    }

    pub fn texture_coordinate_rotation_points(&self) -> [f64; 6] {
        if self.st_rotation == 0.0 {
            return [0.0, 0.0, 0.0, 1.0, 1.0, 0.0];
        }

        let rotation = self.rotation - self.st_rotation;
        let unrotated = compute_rectangle_internal(
            &self.rectangle,
            self.granularity,
            rotation,
            &self.ellipsoid,
        )
        .unwrap_or(self.rectangle);

        let mut pts = [
            Cartesian2::new(unrotated.west, unrotated.south),
            Cartesian2::new(unrotated.west, unrotated.north),
            Cartesian2::new(unrotated.east, unrotated.south),
        ];

        let bounding_rectangle = self.rectangle();
        let to_desired = Matrix2::from_rotation(self.st_rotation);
        let center = bounding_rectangle.center();

        let width = bounding_rectangle.width();
        let height = bounding_rectangle.height();

        for pt in &mut pts {
            pt.x -= center.longitude;
            pt.y -= center.latitude;
            let rot = to_desired.multiply_by_vector(*pt);
            let mut x = rot.x + center.longitude;
            let mut y = rot.y + center.latitude;

            x = if width > 0.0 {
                (x - bounding_rectangle.west) / width
            } else {
                0.0
            };
            y = if height > 0.0 {
                (y - bounding_rectangle.south) / height
            } else {
                0.0
            };
            pt.x = x;
            pt.y = y;
        }

        [pts[0].x, pts[0].y, pts[1].x, pts[1].y, pts[2].x, pts[2].y]
    }

    pub fn create_geometry(geometry: &Self) -> Result<Option<Geometry>, DeveloperError> {
        let rectangle = geometry.rectangle;
        if (rectangle.north - rectangle.south).abs() <= EPSILON10
            || (rectangle.east - rectangle.west).abs() <= EPSILON10
        {
            return Ok(None);
        }

        let mut computed_options = RectangleGeometryLibrary::compute_options(
            &rectangle,
            geometry.granularity,
            geometry.rotation,
            geometry.st_rotation,
        )?;

        let tangent_rotation_matrix = if geometry.st_rotation != 0.0 || geometry.rotation != 0.0 {
            let center = rectangle.center();
            let axis = Ellipsoid::geodetic_surface_normal_cartographic(center);
            let quat = Quaternion::from_axis_angle(axis.into(), -geometry.st_rotation);
            Matrix3::from_quaternion(&quat)
        } else {
            Matrix3::IDENTITY
        };
        computed_options.tangent_rotation_matrix = tangent_rotation_matrix;

        let surface_height = geometry.surface_height;
        let extruded_height = geometry.extruded_height;
        let extrude = (surface_height - extruded_height).abs() > EPSILON2;

        let geo = if extrude {
            construct_extruded_rectangle(geometry, &computed_options)
        } else {
            let mut g = construct_rectangle(geometry, &computed_options);
            if let Some(ref mut pos_attr) = g.attributes.position {
                PolygonPipeline::scale_to_geodetic_height(
                    &mut pos_attr.values,
                    Some(surface_height),
                    Some(&geometry.ellipsoid),
                    Some(false),
                );
            }
            if let Some(offset_attr) = geometry.offset_attribute {
                let count = g.attributes.position.as_ref().map_or(0, |p| p.values.len() / 3);
                let val = match offset_attr {
                    GeometryOffsetAttribute::None => 0.0,
                    _ => 1.0,
                };
                g.attributes.apply_offset = Some(GeometryAttribute::new(
                    ComponentDatatype::UnsignedByte,
                    1,
                    vec![val; count],
                ));
            }
            let bs = BoundingSphere::from_rectangle_3d(
                Some(rectangle),
                geometry.ellipsoid,
                surface_height,
            )
            .ok();
            g.bounding_sphere = bs;
            g
        };

        Ok(Some(geo))
    }
}

fn compute_rectangle_internal(
    rectangle: &GeoRectangle,
    granularity: f64,
    rotation: f64,
    ellipsoid: &Ellipsoid,
) -> Result<GeoRectangle, DeveloperError> {
    if rotation == 0.0 {
        return Ok(*rectangle);
    }

    let computed_options =
        RectangleGeometryLibrary::compute_options(rectangle, granularity, rotation, 0.0)?;
    let height = computed_options.height;
    let width = computed_options.width;

    let (p0, _) = RectangleGeometryLibrary::compute_position(
        &computed_options,
        ellipsoid,
        false,
        0,
        0,
    );
    let (p1, _) = RectangleGeometryLibrary::compute_position(
        &computed_options,
        ellipsoid,
        false,
        0,
        width - 1,
    );
    let (p2, _) = RectangleGeometryLibrary::compute_position(
        &computed_options,
        ellipsoid,
        false,
        height - 1,
        0,
    );
    let (p3, _) = RectangleGeometryLibrary::compute_position(
        &computed_options,
        ellipsoid,
        false,
        height - 1,
        width - 1,
    );

    let cartesians = [
        DVec3::new(p0.x, p0.y, p0.z),
        DVec3::new(p1.x, p1.y, p1.z),
        DVec3::new(p2.x, p2.y, p2.z),
        DVec3::new(p3.x, p3.y, p3.z),
    ];

    GeoRectangle::from_cartesians(&cartesians, *ellipsoid)
        .ok_or_else(|| DeveloperError::new("Failed to compute bounding rectangle from cartesians"))
}

fn calculate_attributes(
    positions: &[f64],
    vertex_format: &VertexFormat,
    ellipsoid: &Ellipsoid,
    tangent_rotation_matrix: &Matrix3,
) -> GeometryAttributes {
    let length = positions.len();
    let num_verts = length / 3;

    let mut normals = if vertex_format.normal {
        Some(vec![0.0f64; length])
    } else {
        None
    };
    let mut tangents = if vertex_format.tangent {
        Some(vec![0.0f64; length])
    } else {
        None
    };
    let mut bitangents = if vertex_format.bitangent {
        Some(vec![0.0f64; length])
    } else {
        None
    };

    if vertex_format.normal || vertex_format.tangent || vertex_format.bitangent {
        for v in 0..num_verts {
            let i = v * 3;
            let p = DVec3::new(positions[i], positions[i + 1], positions[i + 2]);
            let normal = ellipsoid
                .geodetic_surface_normal(p)
                .unwrap_or(DVec3::Z);

            let mut tangent = DVec3::ZERO;
            let mut bitangent = DVec3::ZERO;
            if vertex_format.tangent || vertex_format.bitangent {
                let cross_z = DVec3::Z.cross(normal);
                tangent = tangent_rotation_matrix.multiply_by_vector(cross_z.into()).into();
                tangent = tangent.normalize_or_zero();
                if vertex_format.bitangent {
                    bitangent = normal.cross(tangent).normalize_or_zero();
                }
            }

            if let Some(ref mut n) = normals {
                n[i] = normal.x;
                n[i + 1] = normal.y;
                n[i + 2] = normal.z;
            }
            if let Some(ref mut t) = tangents {
                t[i] = tangent.x;
                t[i + 1] = tangent.y;
                t[i + 2] = tangent.z;
            }
            if let Some(ref mut b) = bitangents {
                b[i] = bitangent.x;
                b[i + 1] = bitangent.y;
                b[i + 2] = bitangent.z;
            }
        }
    }

    GeometryAttributes {
        position: Some(GeometryAttribute::new(
            ComponentDatatype::Double,
            3,
            positions.to_vec(),
        )),
        normal: normals.map(|values| {
            GeometryAttribute::new(ComponentDatatype::Float, 3, values)
        }),
        tangent: tangents.map(|values| {
            GeometryAttribute::new(ComponentDatatype::Float, 3, values)
        }),
        bitangent: bitangents.map(|values| {
            GeometryAttribute::new(ComponentDatatype::Float, 3, values)
        }),
        ..Default::default()
    }
}

fn calculate_attributes_wall(
    positions: &[f64],
    vertex_format: &VertexFormat,
    ellipsoid: &Ellipsoid,
) -> GeometryAttributes {
    let length = positions.len();

    let mut normals = if vertex_format.normal {
        Some(vec![0.0f64; length])
    } else {
        None
    };
    let mut tangents = if vertex_format.tangent {
        Some(vec![0.0f64; length])
    } else {
        None
    };
    let mut bitangents = if vertex_format.bitangent {
        Some(vec![0.0f64; length])
    } else {
        None
    };

    let mut recompute_normal = true;
    let mut normal = DVec3::ZERO;

    if vertex_format.normal || vertex_format.tangent || vertex_format.bitangent {
        for i in (0..length).step_by(6) {
            let p = DVec3::new(positions[i], positions[i + 1], positions[i + 2]);
            let p1_idx = (i + 6) % length;
            let mut p1 = DVec3::new(positions[p1_idx], positions[p1_idx + 1], positions[p1_idx + 2]);

            if recompute_normal {
                let p2_idx = (i + 3) % length;
                let mut p2 = DVec3::new(positions[p2_idx], positions[p2_idx + 1], positions[p2_idx + 2]);
                p1 -= p;
                p2 -= p;
                normal = p2.cross(p1).normalize_or_zero();
                recompute_normal = false;
            }

            if (p1 - p).length_squared() <= EPSILON10 * EPSILON10 {
                recompute_normal = true;
            }

            let mut bitangent = DVec3::ZERO;
            let mut tangent = DVec3::ZERO;
            if vertex_format.tangent || vertex_format.bitangent {
                bitangent = ellipsoid
                    .geodetic_surface_normal(p)
                    .unwrap_or(DVec3::Z);
                if vertex_format.tangent {
                    tangent = bitangent.cross(normal).normalize_or_zero();
                }
            }

            for j in 0..2 {
                let idx = i + j * 3;
                if let Some(ref mut n) = normals {
                    n[idx] = normal.x;
                    n[idx + 1] = normal.y;
                    n[idx + 2] = normal.z;
                }
                if let Some(ref mut t) = tangents {
                    t[idx] = tangent.x;
                    t[idx + 1] = tangent.y;
                    t[idx + 2] = tangent.z;
                }
                if let Some(ref mut b) = bitangents {
                    b[idx] = bitangent.x;
                    b[idx + 1] = bitangent.y;
                    b[idx + 2] = bitangent.z;
                }
            }
        }
    }

    GeometryAttributes {
        position: Some(GeometryAttribute::new(
            ComponentDatatype::Double,
            3,
            positions.to_vec(),
        )),
        normal: normals.map(|values| {
            GeometryAttribute::new(ComponentDatatype::Float, 3, values)
        }),
        tangent: tangents.map(|values| {
            GeometryAttribute::new(ComponentDatatype::Float, 3, values)
        }),
        bitangent: bitangents.map(|values| {
            GeometryAttribute::new(ComponentDatatype::Float, 3, values)
        }),
        ..Default::default()
    }
}

fn construct_rectangle(
    geometry: &RectangleGeometry,
    computed_options: &RectangleOptionsComputed,
) -> Geometry {
    let vertex_format = &geometry.vertex_format;
    let ellipsoid = &geometry.ellipsoid;
    let height = computed_options.height;
    let width = computed_options.width;
    let north_cap = computed_options.north_cap;
    let south_cap = computed_options.south_cap;

    let mut row_start = 0;
    let mut row_end = height;
    let mut row_height = height;
    let mut size = 0;
    if north_cap {
        row_start = 1;
        row_height -= 1;
        size += 1;
    }
    if south_cap {
        row_end -= 1;
        row_height -= 1;
        size += 1;
    }
    size += width * row_height;

    let mut positions = Vec::with_capacity(size * 3);
    let mut texture_coordinates = if vertex_format.st {
        Some(Vec::with_capacity(size * 2))
    } else {
        None
    };

    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = -f64::MAX;
    let mut max_y = -f64::MAX;

    for row in row_start..row_end {
        for col in 0..width {
            let (pos, st_opt) = RectangleGeometryLibrary::compute_position(
                computed_options,
                ellipsoid,
                vertex_format.st,
                row,
                col,
            );
            positions.extend_from_slice(&[pos.x, pos.y, pos.z]);

            if let (Some(st), Some(ref mut tc)) = (st_opt, &mut texture_coordinates) {
                tc.push(st.x);
                tc.push(st.y);
                min_x = min_x.min(st.x);
                min_y = min_y.min(st.y);
                max_x = max_x.max(st.x);
                max_y = max_y.max(st.y);
            }
        }
    }

    if north_cap {
        let (pos, st_opt) = RectangleGeometryLibrary::compute_position(
            computed_options,
            ellipsoid,
            vertex_format.st,
            0,
            0,
        );
        positions.extend_from_slice(&[pos.x, pos.y, pos.z]);
        if let (Some(st), Some(ref mut tc)) = (st_opt, &mut texture_coordinates) {
            tc.push(st.x);
            tc.push(st.y);
            min_x = st.x;
            min_y = st.y;
            max_x = st.x;
            max_y = st.y;
        }
    }

    if south_cap {
        let (pos, st_opt) = RectangleGeometryLibrary::compute_position(
            computed_options,
            ellipsoid,
            vertex_format.st,
            height - 1,
            0,
        );
        positions.extend_from_slice(&[pos.x, pos.y, pos.z]);
        if let (Some(st), Some(ref mut tc)) = (st_opt, &mut texture_coordinates) {
            tc.push(st.x);
            tc.push(st.y);
            min_x = min_x.min(st.x);
            min_y = min_y.min(st.y);
            max_x = max_x.max(st.x);
            max_y = max_y.max(st.y);
        }
    }

    if vertex_format.st && (min_x < 0.0 || min_y < 0.0 || max_x > 1.0 || max_y > 1.0) {
        if let Some(ref mut tc) = texture_coordinates {
            let dx = max_x - min_x;
            let dy = max_y - min_y;
            if dx > 0.0 && dy > 0.0 {
                for k in (0..tc.len()).step_by(2) {
                    tc[k] = (tc[k] - min_x) / dx;
                    tc[k + 1] = (tc[k + 1] - min_y) / dy;
                }
            }
        }
    }

    let mut attributes = calculate_attributes(
        &positions,
        vertex_format,
        ellipsoid,
        &computed_options.tangent_rotation_matrix,
    );

    let mut indices_size = 6 * (width - 1) * (row_height - 1);
    if north_cap {
        indices_size += 3 * (width - 1);
    }
    if south_cap {
        indices_size += 3 * (width - 1);
    }
    let mut indices = Vec::with_capacity(indices_size);
    let mut index = 0u32;

    for _ in 0..(row_height - 1) {
        for _ in 0..(width - 1) {
            let upper_left = index;
            let lower_left = upper_left + width as u32;
            let lower_right = lower_left + 1;
            let upper_right = upper_left + 1;

            indices.push(upper_left);
            indices.push(lower_left);
            indices.push(upper_right);
            indices.push(upper_right);
            indices.push(lower_left);
            indices.push(lower_right);

            index += 1;
        }
        index += 1;
    }

    if north_cap || south_cap {
        let mut north_index = (size - 1) as u32;
        let south_index = (size - 1) as u32;
        if north_cap && south_cap {
            north_index = (size - 2) as u32;
        }

        if north_cap {
            for cap_index in 0..(width - 1) as u32 {
                let p1 = cap_index;
                let p2 = p1 + 1;
                indices.push(north_index);
                indices.push(p1);
                indices.push(p2);
            }
        }
        if south_cap {
            let start = ((row_height - 1) * width) as u32;
            for i in 0..(width - 1) as u32 {
                let p1 = start + i;
                let p2 = p1 + 1;
                indices.push(p1);
                indices.push(south_index);
                indices.push(p2);
            }
        }
    }

    if vertex_format.st {
        if let Some(tc) = texture_coordinates {
            attributes.st = Some(GeometryAttribute::new(
                ComponentDatatype::Float,
                2,
                tc,
            ));
        }
    }

    Geometry {
        attributes,
        indices,
        primitive_type: PrimitiveType::Triangles,
        bounding_sphere: None,
    }
}

fn construct_extruded_rectangle(
    geometry: &RectangleGeometry,
    computed_options: &RectangleOptionsComputed,
) -> Geometry {
    let vertex_format = &geometry.vertex_format;
    let min_height = geometry.extruded_height;
    let max_height = geometry.surface_height;
    let ellipsoid = &geometry.ellipsoid;
    let height = computed_options.height;
    let width = computed_options.width;

    let top_bottom_geo = construct_rectangle(geometry, computed_options);

    let base_positions = top_bottom_geo.attributes.position.unwrap().values;
    let mut top_positions = base_positions.clone();
    PolygonPipeline::scale_to_geodetic_height(
        &mut top_positions,
        Some(max_height),
        Some(ellipsoid),
        Some(false),
    );

    let mut bottom_positions = base_positions;
    PolygonPipeline::scale_to_geodetic_height(
        &mut bottom_positions,
        Some(min_height),
        Some(ellipsoid),
        Some(true),
    );

    let length = top_positions.len();
    let new_length = length * 2;
    let mut positions = Vec::with_capacity(new_length);
    positions.extend_from_slice(&top_positions);
    positions.extend_from_slice(&bottom_positions);

    let mut normals = if vertex_format.normal {
        let top_normals = top_bottom_geo.attributes.normal.as_ref().unwrap().values.clone();
        let mut n = Vec::with_capacity(new_length);
        n.extend_from_slice(&top_normals);
        for val in top_normals {
            n.push(-val);
        }
        Some(n)
    } else {
        None
    };

    let mut tangents = if vertex_format.tangent {
        let top_tangents = top_bottom_geo.attributes.tangent.as_ref().unwrap().values.clone();
        let mut t = Vec::with_capacity(new_length);
        t.extend_from_slice(&top_tangents);
        for val in top_tangents {
            t.push(-val);
        }
        Some(t)
    } else {
        None
    };

    let mut bitangents = if vertex_format.bitangent {
        let top_bitangents = top_bottom_geo.attributes.bitangent.as_ref().unwrap().values.clone();
        let mut b = Vec::with_capacity(new_length);
        b.extend_from_slice(&top_bitangents);
        b.extend_from_slice(&top_bitangents);
        Some(b)
    } else {
        None
    };

    let top_st = top_bottom_geo.attributes.st.as_ref().map(|st| st.values.clone());
    let mut textures = if vertex_format.st {
        if let Some(ref st) = top_st {
            let mut tex = Vec::with_capacity(st.len() * 2);
            tex.extend_from_slice(st);
            tex.extend_from_slice(st);
            Some(tex)
        } else {
            None
        }
    } else {
        None
    };

    let mut apply_offset = None;
    if let Some(offset_attr) = geometry.offset_attribute {
        let size = (length / 3) * 2;
        let mut vals = vec![0.0; size];
        if offset_attr == GeometryOffsetAttribute::Top {
            for v in vals.iter_mut().take(size / 2) {
                *v = 1.0;
            }
        } else {
            let fill_v = if offset_attr == GeometryOffsetAttribute::None {
                0.0
            } else {
                1.0
            };
            vals.fill(fill_v);
        }
        apply_offset = Some(vals);
    }

    let indices_length = top_bottom_geo.indices.len();
    let pos_length = (length / 3) as u32;
    let mut indices = Vec::with_capacity(indices_length * 2);
    indices.extend_from_slice(&top_bottom_geo.indices);
    for i in (0..indices_length).step_by(3) {
        indices.push(top_bottom_geo.indices[i + 2] + pos_length);
        indices.push(top_bottom_geo.indices[i + 1] + pos_length);
        indices.push(top_bottom_geo.indices[i] + pos_length);
    }

    // 侧壁 walls 计算
    let north_cap = computed_options.north_cap;
    let south_cap = computed_options.south_cap;

    let mut row_height = height;
    let mut width_multiplier = 2;
    let mut perimeter_positions = 0;
    let mut corners = 4;
    let mut duplicate_corners = 4;
    if north_cap {
        width_multiplier -= 1;
        row_height -= 1;
        perimeter_positions += 1;
        corners -= 2;
        duplicate_corners -= 1;
    }
    if south_cap {
        width_multiplier -= 1;
        row_height -= 1;
        perimeter_positions += 1;
        corners -= 2;
        duplicate_corners -= 1;
    }
    perimeter_positions += width_multiplier * width + 2 * row_height - corners;

    let wall_count = (perimeter_positions + duplicate_corners) * 2;
    let mut wall_positions = Vec::with_capacity(wall_count * 3);
    let mut wall_textures = if vertex_format.st {
        Some(Vec::with_capacity(wall_count * 2))
    } else {
        None
    };
    let mut wall_offset_attr = if geometry.offset_attribute.is_some() {
        Some(Vec::with_capacity(wall_count))
    } else {
        None
    };

    let area = width * row_height;

    let add_wall_p = |wall_pos: &mut Vec<f64>, i: usize| {
        let three_i = i * 3;
        wall_pos.extend_from_slice(&[
            top_positions[three_i],
            top_positions[three_i + 1],
            top_positions[three_i + 2],
            bottom_positions[three_i],
            bottom_positions[three_i + 1],
            bottom_positions[three_i + 2],
        ]);
    };

    let add_wall_t = |wall_tex: &mut Option<Vec<f64>>, i: usize| {
        if let (Some(wt), Some(ref st)) = (wall_tex, &top_st) {
            let two_i = i * 2;
            wt.extend_from_slice(&[
                st[two_i],
                st[two_i + 1],
                st[two_i],
                st[two_i + 1],
            ]);
        }
    };

    let add_wall_offset = |wall_offset: &mut Option<Vec<f64>>| {
        if let Some(wo) = wall_offset {
            if geometry.offset_attribute == Some(GeometryOffsetAttribute::Top) {
                wo.push(1.0);
                wo.push(0.0);
            } else if geometry.offset_attribute == Some(GeometryOffsetAttribute::All) {
                wo.push(1.0);
                wo.push(1.0);
            } else {
                wo.push(0.0);
                wo.push(0.0);
            }
        }
    };

    // 1. 左侧边缘 (col = 0)
    for i in (0..area).step_by(width) {
        add_wall_p(&mut wall_positions, i);
        add_wall_t(&mut wall_textures, i);
        add_wall_offset(&mut wall_offset_attr);
    }

    // 2. 南侧边缘
    if !south_cap {
        for i in (area - width)..area {
            add_wall_p(&mut wall_positions, i);
            add_wall_t(&mut wall_textures, i);
            add_wall_offset(&mut wall_offset_attr);
        }
    } else {
        let south_index = if north_cap { area + 1 } else { area };
        for _ in 0..2 {
            add_wall_p(&mut wall_positions, south_index);
            add_wall_t(&mut wall_textures, south_index);
            add_wall_offset(&mut wall_offset_attr);
        }
    }

    // 3. 右侧边缘 (col = width - 1)
    if area > 0 {
        let mut i = area - 1;
        while i > 0 {
            add_wall_p(&mut wall_positions, i);
            add_wall_t(&mut wall_textures, i);
            add_wall_offset(&mut wall_offset_attr);
            if i < width {
                break;
            }
            i -= width;
        }
    }

    // 4. 北侧边缘
    if !north_cap {
        for i in (0..width).rev() {
            add_wall_p(&mut wall_positions, i);
            add_wall_t(&mut wall_textures, i);
            add_wall_offset(&mut wall_offset_attr);
        }
    } else {
        let north_index = area;
        for _ in 0..2 {
            add_wall_p(&mut wall_positions, north_index);
            add_wall_t(&mut wall_textures, north_index);
            add_wall_offset(&mut wall_offset_attr);
        }
    }

    let wall_attributes = calculate_attributes_wall(&wall_positions, vertex_format, ellipsoid);

    let wall_verts_count = wall_positions.len() / 3;
    let mut wall_indices = Vec::with_capacity(perimeter_positions * 6);
    for i in (0..(wall_verts_count - 1)).step_by(2) {
        let upper_left = i;
        let upper_right = (upper_left + 2) % wall_verts_count;
        let p1 = DVec3::new(
            wall_positions[upper_left * 3],
            wall_positions[upper_left * 3 + 1],
            wall_positions[upper_left * 3 + 2],
        );
        let p2 = DVec3::new(
            wall_positions[upper_right * 3],
            wall_positions[upper_right * 3 + 1],
            wall_positions[upper_right * 3 + 2],
        );
        if (p1 - p2).length_squared() <= EPSILON10 * EPSILON10 {
            continue;
        }
        let lower_left = (upper_left + 1) % wall_verts_count;
        let lower_right = (lower_left + 2) % wall_verts_count;

        wall_indices.push(upper_left as u32);
        wall_indices.push(lower_left as u32);
        wall_indices.push(upper_right as u32);
        wall_indices.push(upper_right as u32);
        wall_indices.push(lower_left as u32);
        wall_indices.push(lower_right as u32);
    }

    // 合并 top_bottom_geo 和 wall_geo
    let base_vert_count = (positions.len() / 3) as u32;
    for idx in wall_indices {
        indices.push(idx + base_vert_count);
    }
    positions.extend_from_slice(&wall_positions);

    if let (Some(ref mut n), Some(wn)) = (&mut normals, wall_attributes.normal) {
        n.extend_from_slice(&wn.values);
    }
    if let (Some(ref mut t), Some(wt)) = (&mut tangents, wall_attributes.tangent) {
        t.extend_from_slice(&wt.values);
    }
    if let (Some(ref mut b), Some(wb)) = (&mut bitangents, wall_attributes.bitangent) {
        b.extend_from_slice(&wb.values);
    }
    if let (Some(ref mut tex), Some(wt)) = (&mut textures, wall_textures) {
        tex.extend_from_slice(&wt);
    }
    if let (Some(ref mut ao), Some(wao)) = (&mut apply_offset, wall_offset_attr) {
        ao.extend_from_slice(&wao);
    }

    let top_bs =
        BoundingSphere::from_rectangle_3d(Some(geometry.rectangle), *ellipsoid, max_height).ok();
    let bottom_bs =
        BoundingSphere::from_rectangle_3d(Some(geometry.rectangle), *ellipsoid, min_height).ok();
    let bs = match (top_bs, bottom_bs) {
        (Some(t), Some(b)) => Some(t.union(b)),
        (Some(t), None) => Some(t),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };

    Geometry {
        attributes: GeometryAttributes {
            position: Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                positions,
            )),
            normal: normals.map(|v| GeometryAttribute::new(
                ComponentDatatype::Float,
                3,
                v,
            )),
            tangent: tangents.map(|v| GeometryAttribute::new(
                ComponentDatatype::Float,
                3,
                v,
            )),
            bitangent: bitangents.map(|v| GeometryAttribute::new(
                ComponentDatatype::Float,
                3,
                v,
            )),
            st: textures.map(|v| GeometryAttribute::new(
                ComponentDatatype::Float,
                2,
                v,
            )),
            apply_offset: apply_offset.map(|v| GeometryAttribute::new(
                ComponentDatatype::UnsignedByte,
                1,
                v,
            )),
            ..Default::default()
        },
        indices,
        primitive_type: PrimitiveType::Triangles,
        bounding_sphere: bs,
    }
}
