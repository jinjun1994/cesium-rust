//! PolylineVolumeOutlineGeometry matching CesiumJS `Core/PolylineVolumeOutlineGeometry`.

use crate::bounding_rectangle::BoundingRectangle;
use crate::cartesian::{Cartesian2, Cartesian3};
use crate::core_math::RADIANS_PER_DEGREE;
use crate::ellipsoid::Ellipsoid;
use crate::geometry::{
    ComponentDatatype, CornerType, Geometry, GeometryAttribute, GeometryAttributes, PrimitiveType,
};
use crate::polygon_geometry_library::PolygonGeometryLibrary;
use crate::polygon_pipeline::PolygonPipeline;
use crate::polyline_volume_geometry_library::PolylineVolumeGeometryLibrary;
use crate::winding_order::WindingOrder;
use crate::BoundingSphere;
use glam::{DVec2, DVec3};

/// A description of the outline of a polyline with a volume.
#[derive(Clone, Debug, PartialEq)]
pub struct PolylineVolumeOutlineGeometry {
    pub polyline_positions: Vec<Cartesian3>,
    pub shape_positions: Vec<Cartesian2>,
    pub ellipsoid: Ellipsoid,
    pub corner_type: CornerType,
    pub granularity: f64,
}

/// Options for creating a [`PolylineVolumeOutlineGeometry`].
#[derive(Clone, Debug)]
pub struct PolylineVolumeOutlineGeometryOptions {
    pub polyline_positions: Vec<Cartesian3>,
    pub shape_positions: Vec<Cartesian2>,
    pub ellipsoid: Option<Ellipsoid>,
    pub corner_type: Option<CornerType>,
    pub granularity: Option<f64>,
}

impl PolylineVolumeOutlineGeometry {
    pub fn new(options: PolylineVolumeOutlineGeometryOptions) -> Self {
        Self {
            polyline_positions: options.polyline_positions,
            shape_positions: options.shape_positions,
            ellipsoid: options.ellipsoid.unwrap_or(Ellipsoid::WGS84),
            corner_type: options.corner_type.unwrap_or(CornerType::Rounded),
            granularity: options.granularity.unwrap_or(RADIANS_PER_DEGREE),
        }
    }

    pub fn packed_length(&self) -> usize {
        1 + self.polyline_positions.len() * 3
            + 1
            + self.shape_positions.len() * 2
            + 3 // ellipsoid
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
            false,
        );

        let shape_length = shape_2d.len();
        let position_length = computed_positions.len() / 3;
        let shape_count = position_length / shape_length;

        let indices_count = 2 * shape_length * (shape_count + 1);
        let mut indices = Vec::with_capacity(indices_count);

        let s_len = shape_length as u32;

        // Front face loop
        let offset = 0;
        for j in 0..shape_length - 1 {
            let idx = (j + offset) as u32;
            indices.push(idx);
            indices.push(idx + 1);
        }
        indices.push((shape_length - 1 + offset) as u32);
        indices.push(offset as u32);

        // Back face loop
        let end_face_offset = (shape_count - 1) * shape_length;
        for j in 0..shape_length - 1 {
            let idx = (j + end_face_offset) as u32;
            indices.push(idx);
            indices.push(idx + 1);
        }
        indices.push((shape_length - 1 + end_face_offset) as u32);
        indices.push(end_face_offset as u32);

        // Longitudinal edge lines
        for i in 0..shape_count - 1 {
            let first_offset = (shape_length * i) as u32;
            let second_offset = first_offset + s_len;
            for j in 0..shape_length {
                let j_u = j as u32;
                indices.push(j_u + first_offset);
                indices.push(j_u + second_offset);
            }
        }

        let attributes = GeometryAttributes {
            position: Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                computed_positions.clone(),
            )),
            ..Default::default()
        };

        Some(Geometry {
            attributes,
            indices,
            primitive_type: PrimitiveType::Lines,
            bounding_sphere: BoundingSphere::from_vertices(&computed_positions, DVec3::ZERO, 3).ok(),
        })
    }
}
