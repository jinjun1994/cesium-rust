//! PolylineVolumeGeometry matching CesiumJS `Core/PolylineVolumeGeometry`.

use crate::bounding_rectangle::BoundingRectangle;
use crate::cartesian::{Cartesian2, Cartesian3};
use crate::core_math::RADIANS_PER_DEGREE;
use crate::ellipsoid::Ellipsoid;
use crate::geometry::{
    ComponentDatatype, CornerType, Geometry, GeometryAttribute, GeometryAttributes, PrimitiveType,
};
use crate::geometry_pipeline::GeometryPipeline;
use crate::polygon_geometry_library::PolygonGeometryLibrary;
use crate::polygon_pipeline::PolygonPipeline;
use crate::polyline_volume_geometry_library::PolylineVolumeGeometryLibrary;
use crate::vertex_format::VertexFormat;
use crate::winding_order::WindingOrder;
use crate::BoundingSphere;
use glam::{DVec2, DVec3};

/// A description of a polyline with a volume (a 2D shape extruded along a polyline).
#[derive(Clone, Debug, PartialEq)]
pub struct PolylineVolumeGeometry {
    pub polyline_positions: Vec<Cartesian3>,
    pub shape_positions: Vec<Cartesian2>,
    pub ellipsoid: Ellipsoid,
    pub corner_type: CornerType,
    pub vertex_format: VertexFormat,
    pub granularity: f64,
}

/// Options for creating a [`PolylineVolumeGeometry`].
#[derive(Clone, Debug)]
pub struct PolylineVolumeGeometryOptions {
    pub polyline_positions: Vec<Cartesian3>,
    pub shape_positions: Vec<Cartesian2>,
    pub ellipsoid: Option<Ellipsoid>,
    pub corner_type: Option<CornerType>,
    pub vertex_format: Option<VertexFormat>,
    pub granularity: Option<f64>,
}

impl PolylineVolumeGeometry {
    pub fn new(options: PolylineVolumeGeometryOptions) -> Self {
        Self {
            polyline_positions: options.polyline_positions,
            shape_positions: options.shape_positions,
            ellipsoid: options.ellipsoid.unwrap_or(Ellipsoid::WGS84),
            corner_type: options.corner_type.unwrap_or(CornerType::Rounded),
            vertex_format: options.vertex_format.unwrap_or(VertexFormat::DEFAULT),
            granularity: options.granularity.unwrap_or(RADIANS_PER_DEGREE),
        }
    }

    pub fn packed_length(&self) -> usize {
        1 + self.polyline_positions.len() * 3
            + 1
            + self.shape_positions.len() * 2
            + 3 // ellipsoid
            + VertexFormat::PACKED_LENGTH
            + 2
    }

    pub fn pack(&self, array: &mut Vec<f64>, starting_index: Option<usize>) {
        let mut index = starting_index.unwrap_or(0);
        let needed = index + self.packed_length();
        if array.len() < needed {
            array.resize(needed, 0.0);
        }

        array[index] = self.polyline_positions.len() as f64;
        index += 1;
        for p in &self.polyline_positions {
            array[index] = p.x;
            array[index + 1] = p.y;
            array[index + 2] = p.z;
            index += 3;
        }

        array[index] = self.shape_positions.len() as f64;
        index += 1;
        for p in &self.shape_positions {
            array[index] = p.x;
            array[index + 1] = p.y;
            index += 2;
        }

        let radii = self.ellipsoid.radii();
        array[index] = radii.x;
        array[index + 1] = radii.y;
        array[index + 2] = radii.z;
        index += 3;

        let _ = self.vertex_format.pack(array, index);
        index += VertexFormat::PACKED_LENGTH;

        array[index] = self.corner_type as i32 as f64;
        index += 1;
        array[index] = self.granularity;
    }

    pub fn unpack(array: &[f64], starting_index: Option<usize>) -> Option<Self> {
        let mut index = starting_index.unwrap_or(0);

        if index >= array.len() {
            return None;
        }
        let polyline_len = array[index] as usize;
        index += 1;

        let mut polyline_positions = Vec::with_capacity(polyline_len);
        for _ in 0..polyline_len {
            if index + 3 > array.len() {
                return None;
            }
            polyline_positions.push(Cartesian3::new(array[index], array[index + 1], array[index + 2]));
            index += 3;
        }

        if index >= array.len() {
            return None;
        }
        let shape_len = array[index] as usize;
        index += 1;

        let mut shape_positions = Vec::with_capacity(shape_len);
        for _ in 0..shape_len {
            if index + 2 > array.len() {
                return None;
            }
            shape_positions.push(Cartesian2::new(array[index], array[index + 1]));
            index += 2;
        }

        if index + 3 > array.len() {
            return None;
        }
        let ellipsoid = Ellipsoid::new(DVec3::new(array[index], array[index + 1], array[index + 2])).ok()?;
        index += 3;

        let vertex_format = VertexFormat::unpack(array, index).ok()?;
        index += VertexFormat::PACKED_LENGTH;

        if index + 2 > array.len() {
            return None;
        }
        let corner_type = match array[index] as i32 {
            1 => CornerType::Mitered,
            2 => CornerType::Beveled,
            _ => CornerType::Rounded,
        };
        index += 1;
        let granularity = array[index];

        Some(Self {
            polyline_positions,
            shape_positions,
            ellipsoid,
            corner_type,
            vertex_format,
            granularity,
        })
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        let clean_positions =
            PolygonGeometryLibrary::array_remove_duplicates(&self.polyline_positions, false);
        let mut shape_2d =
            PolylineVolumeGeometryLibrary::remove_duplicates_from_shape(&self.shape_positions);

        if clean_positions.len() < 2 || shape_2d.len() < 3 {
            return None;
        }

        if PolygonPipeline::compute_winding_order_2d(&shape_2d).ok() == Some(WindingOrder::Clockwise) {
            shape_2d.reverse();
        }

        let dvec2_work: Vec<DVec2> = shape_2d.iter().map(|p| DVec2::new(p.x, p.y)).collect();
        let bounding_rectangle = BoundingRectangle::from_points(&dvec2_work).ok()?;

        let computed_positions = PolylineVolumeGeometryLibrary::compute_positions(
            &clean_positions,
            &shape_2d,
            &bounding_rectangle,
            &self.ellipsoid,
            self.corner_type,
            self.granularity,
            true,
        );

        let shape_length = shape_2d.len();
        let vertex_count = computed_positions.len() / 3;
        let length = (vertex_count - shape_length * 2) / (shape_length * 2);

        let first_end_indices = PolygonPipeline::triangulate(&shape_2d, &[]);

        let indices_count = (length - 1) * shape_length * 6 + first_end_indices.len() * 2;
        let mut indices = Vec::with_capacity(indices_count);

        let offset = shape_length * 2;
        for i in 0..length - 1 {
            for j in 0..shape_length - 1 {
                let ll = (j * 2 + i * shape_length * 2) as u32;
                let lr = ll + offset as u32;
                let ul = ll + 1;
                let ur = ul + offset as u32;

                indices.extend_from_slice(&[ul, ll, ur, ur, ll, lr]);
            }
            let ll = (shape_length * 2 - 2 + i * shape_length * 2) as u32;
            let ul = ll + 1;
            let ur = ul + offset as u32;
            let lr = ll + offset as u32;

            indices.extend_from_slice(&[ul, ll, ur, ur, ll, lr]);
        }

        let mut attributes = GeometryAttributes {
            position: if self.vertex_format.position {
                Some(GeometryAttribute::new(
                    ComponentDatatype::Double,
                    3,
                    computed_positions.clone(),
                ))
            } else {
                None
            },
            ..Default::default()
        };

        if self.vertex_format.st || self.vertex_format.tangent || self.vertex_format.bitangent {
            let mut st = Vec::with_capacity(vertex_count * 2);
            let length_st = 1.0 / (length - 1) as f64;
            let height_st = 1.0 / bounding_rectangle.height;
            let height_offset = bounding_rectangle.height / 2.0;

            for i in 0..length {
                let s = i as f64 * length_st;
                let t0 = height_st * (shape_2d[0].y + height_offset);
                st.extend_from_slice(&[s, t0]);
                for pt in &shape_2d[1..] {
                    let t = height_st * (pt.y + height_offset);
                    st.extend_from_slice(&[s, t, s, t]);
                }
                let t_end = height_st * (shape_2d[0].y + height_offset);
                st.extend_from_slice(&[s, t_end]);
            }

            for pt in &shape_2d {
                let s = 0.0;
                let t = height_st * (pt.y + height_offset);
                st.extend_from_slice(&[s, t]);
            }

            for pt in &shape_2d {
                let s = (length - 1) as f64 * length_st;
                let t = height_st * (pt.y + height_offset);
                st.extend_from_slice(&[s, t]);
            }

            attributes.st = Some(GeometryAttribute::new(ComponentDatatype::Float, 2, st));
        }

        let end_offset = (vertex_count - shape_length * 2) as u32;
        for chunk in first_end_indices.as_chunks::<3>().0 {
            let v0 = chunk[0] + end_offset;
            let v1 = chunk[1] + end_offset;
            let v2 = chunk[2] + end_offset;
            let s_len = shape_length as u32;

            indices.extend_from_slice(&[v0, v1, v2, v2 + s_len, v1 + s_len, v0 + s_len]);
        }

        let mut geometry = Geometry {
            attributes,
            indices,
            primitive_type: PrimitiveType::Triangles,
            bounding_sphere: BoundingSphere::from_vertices(&computed_positions, DVec3::ZERO, 3).ok(),
        };

        if self.vertex_format.normal {
            let _ = GeometryPipeline::compute_normal(&mut geometry);
        }

        if self.vertex_format.tangent || self.vertex_format.bitangent {
            let _ = GeometryPipeline::compute_tangent_and_bitangent(&mut geometry);

            if !self.vertex_format.tangent {
                geometry.attributes.tangent = None;
            }
            if !self.vertex_format.bitangent {
                geometry.attributes.bitangent = None;
            }
            if !self.vertex_format.st {
                geometry.attributes.st = None;
            }
        }

        Some(geometry)
    }
}
