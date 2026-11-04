//! RectangleOutlineGeometry matching CesiumJS `Core/RectangleOutlineGeometry`.
//! Adapted from CesiumJS (Apache-2.0).

use std::f64::consts::{FRAC_PI_2, PI};

use crate::core_math::{EPSILON10, EPSILON2, RADIANS_PER_DEGREE};
use crate::ellipsoid::Ellipsoid;
use crate::errors::DeveloperError;
use crate::geometry::{
    ComponentDatatype, Geometry, GeometryAttribute, GeometryAttributes, GeometryOffsetAttribute,
    PrimitiveType,
};
use crate::polygon_pipeline::PolygonPipeline;
use crate::rectangle::GeoRectangle;
use crate::rectangle_geometry_library::{RectangleGeometryLibrary, RectangleOptionsComputed};
use crate::BoundingSphere;

#[derive(Clone, Debug, PartialEq)]
pub struct RectangleOutlineGeometryOptions {
    pub rectangle: GeoRectangle,
    pub ellipsoid: Option<Ellipsoid>,
    pub granularity: Option<f64>,
    pub height: Option<f64>,
    pub rotation: Option<f64>,
    pub extruded_height: Option<f64>,
    pub offset_attribute: Option<GeometryOffsetAttribute>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RectangleOutlineGeometry {
    pub(crate) rectangle: GeoRectangle,
    pub(crate) ellipsoid: Ellipsoid,
    pub(crate) granularity: f64,
    pub(crate) surface_height: f64,
    pub(crate) rotation: f64,
    pub(crate) extruded_height: f64,
    pub(crate) offset_attribute: Option<GeometryOffsetAttribute>,
}

impl RectangleOutlineGeometry {
    pub const PACKED_LENGTH: usize = GeoRectangle::PACKED_LENGTH + Ellipsoid::PACKED_LENGTH + 5;

    pub fn new(options: RectangleOutlineGeometryOptions) -> Result<Self, DeveloperError> {
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
                "options.rectangle.north must be greater than options.rectangle.south",
            ));
        }

        let granularity = options.granularity.unwrap_or(RADIANS_PER_DEGREE);
        let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let rotation = options.rotation.unwrap_or(0.0);

        let height = options.height.unwrap_or(0.0);
        let extruded_height = options.extruded_height.unwrap_or(height);
        let surface_height = height.max(extruded_height);
        let min_extruded_height = height.min(extruded_height);

        Ok(Self {
            rectangle,
            ellipsoid,
            granularity,
            surface_height,
            rotation,
            extruded_height: min_extruded_height,
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

        array[idx] = self.granularity;
        array[idx + 1] = self.surface_height;
        array[idx + 2] = self.rotation;
        array[idx + 3] = self.extruded_height;
        array[idx + 4] = match self.offset_attribute {
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

        let granularity = array[idx];
        let surface_height = array[idx + 1];
        let rotation = array[idx + 2];
        let extruded_height = array[idx + 3];
        let offset_code = array[idx + 4] as i32;
        let offset_attribute = match offset_code {
            1 => Some(GeometryOffsetAttribute::Top),
            2 => Some(GeometryOffsetAttribute::All),
            0 => Some(GeometryOffsetAttribute::None),
            _ => None,
        };

        Ok(Self {
            rectangle,
            ellipsoid,
            granularity,
            surface_height,
            rotation,
            extruded_height,
            offset_attribute,
        })
    }

    pub fn create_geometry(geometry: &Self) -> Result<Option<Geometry>, DeveloperError> {
        let rectangle = geometry.rectangle;
        if (rectangle.north - rectangle.south).abs() <= EPSILON10
            || (rectangle.east - rectangle.west).abs() <= EPSILON10
        {
            return Ok(None);
        }

        let computed_options = RectangleGeometryLibrary::compute_options(
            &rectangle,
            geometry.granularity,
            geometry.rotation,
            0.0,
        )?;

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
                    GeometryOffsetAttribute::None => 0u8,
                    _ => 1u8,
                };
                g.attributes.apply_offset = Some(GeometryAttribute::new(
                    ComponentDatatype::UnsignedByte,
                    1,
                    vec![val as f64; count],
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

fn construct_rectangle(
    geometry: &RectangleOutlineGeometry,
    computed_options: &RectangleOptionsComputed,
) -> Geometry {
    let ellipsoid = geometry.ellipsoid;
    let height = computed_options.height;
    let width = computed_options.width;
    let north_cap = computed_options.north_cap;
    let south_cap = computed_options.south_cap;

    let mut row_height = height;
    let mut width_multiplier = 2;
    let mut size = 0;
    let mut corners = 4;
    if north_cap {
        width_multiplier -= 1;
        row_height -= 1;
        size += 1;
        corners -= 2;
    }
    if south_cap {
        width_multiplier -= 1;
        row_height -= 1;
        size += 1;
        corners -= 2;
    }
    size += width_multiplier * width + 2 * row_height - corners;

    let mut positions = Vec::with_capacity(size * 3);

    let row = 0;
    if north_cap {
        let (pos, _) = RectangleGeometryLibrary::compute_position(
            computed_options,
            &ellipsoid,
            false,
            row,
            0,
        );
        positions.extend_from_slice(&[pos.x, pos.y, pos.z]);
    } else {
        for col in 0..width {
            let (pos, _) = RectangleGeometryLibrary::compute_position(
                computed_options,
                &ellipsoid,
                false,
                row,
                col,
            );
            positions.extend_from_slice(&[pos.x, pos.y, pos.z]);
        }
    }

    let col = width - 1;
    for row in 1..height {
        let (pos, _) = RectangleGeometryLibrary::compute_position(
            computed_options,
            &ellipsoid,
            false,
            row,
            col,
        );
        positions.extend_from_slice(&[pos.x, pos.y, pos.z]);
    }

    let row = height - 1;
    if !south_cap {
        for col in (0..=(width - 2)).rev() {
            let (pos, _) = RectangleGeometryLibrary::compute_position(
                computed_options,
                &ellipsoid,
                false,
                row,
                col,
            );
            positions.extend_from_slice(&[pos.x, pos.y, pos.z]);
        }
    }

    let col = 0;
    if height >= 2 {
        for row in (1..=(height - 2)).rev() {
            let (pos, _) = RectangleGeometryLibrary::compute_position(
                computed_options,
                &ellipsoid,
                false,
                row,
                col,
            );
            positions.extend_from_slice(&[pos.x, pos.y, pos.z]);
        }
    }

    let num_positions = positions.len() / 3;
    let mut indices = Vec::with_capacity(num_positions * 2);
    for i in 0..(num_positions - 1) {
        indices.push(i as u32);
        indices.push((i + 1) as u32);
    }
    indices.push((num_positions - 1) as u32);
    indices.push(0);

    Geometry {
        attributes: GeometryAttributes {
            position: Some(GeometryAttribute::new(
                ComponentDatatype::Double,
                3,
                positions,
            )),
            ..Default::default()
        },
        indices,
        primitive_type: PrimitiveType::Lines,
        bounding_sphere: None,
    }
}

fn construct_extruded_rectangle(
    geometry: &RectangleOutlineGeometry,
    computed_options: &RectangleOptionsComputed,
) -> Geometry {
    let surface_height = geometry.surface_height;
    let extruded_height = geometry.extruded_height;
    let ellipsoid = geometry.ellipsoid;
    let mut geo = construct_rectangle(geometry, computed_options);

    let height = computed_options.height;
    let width = computed_options.width;

    let base_positions = geo.attributes.position.unwrap().values;
    let mut top_positions = base_positions.clone();
    PolygonPipeline::scale_to_geodetic_height(
        &mut top_positions,
        Some(surface_height),
        Some(&ellipsoid),
        Some(false),
    );

    let mut bottom_positions = base_positions;
    PolygonPipeline::scale_to_geodetic_height(
        &mut bottom_positions,
        Some(extruded_height),
        Some(&ellipsoid),
        Some(true),
    );

    let length = top_positions.len();
    let mut positions = Vec::with_capacity(length * 2);
    positions.extend_from_slice(&top_positions);
    positions.extend_from_slice(&bottom_positions);

    let north_cap = computed_options.north_cap;
    let south_cap = computed_options.south_cap;
    let mut corners = 4;
    if north_cap {
        corners -= 1;
    }
    if south_cap {
        corners -= 1;
    }

    let num_per_level = length / 3;
    let mut indices = Vec::with_capacity((num_per_level + corners) * 4);
    for i in 0..(num_per_level - 1) {
        indices.push(i as u32);
        indices.push((i + 1) as u32);
        indices.push((i + num_per_level) as u32);
        indices.push((i + num_per_level + 1) as u32);
    }
    indices.push((num_per_level - 1) as u32);
    indices.push(0);
    indices.push((num_per_level * 2 - 1) as u32);
    indices.push(num_per_level as u32);

    // 柱边 corners
    indices.push(0);
    indices.push(num_per_level as u32);

    let bottom_corner = if north_cap {
        height - 1
    } else {
        let top_right_corner = width - 1;
        indices.push(top_right_corner as u32);
        indices.push((top_right_corner + num_per_level) as u32);
        width + height - 2
    };

    indices.push(bottom_corner as u32);
    indices.push((bottom_corner + num_per_level) as u32);

    if !south_cap {
        let bottom_left_corner = width + bottom_corner - 1;
        indices.push(bottom_left_corner as u32);
        indices.push((bottom_left_corner + num_per_level) as u32);
    }

    let mut apply_offset = None;
    if let Some(offset_attr) = geometry.offset_attribute {
        let total_verts = positions.len() / 3;
        let mut vals = vec![0.0; total_verts];
        if offset_attr == GeometryOffsetAttribute::Top {
            for v in vals.iter_mut().take(num_per_level) {
                *v = 1.0;
            }
        } else {
            let fill_val = if offset_attr == GeometryOffsetAttribute::None {
                0.0
            } else {
                1.0
            };
            vals.fill(fill_val);
        }
        apply_offset = Some(GeometryAttribute::new(
            ComponentDatatype::UnsignedByte,
            1,
            vals,
        ));
    }

    let top_bs =
        BoundingSphere::from_rectangle_3d(Some(geometry.rectangle), ellipsoid, surface_height).ok();
    let bottom_bs =
        BoundingSphere::from_rectangle_3d(Some(geometry.rectangle), ellipsoid, extruded_height).ok();
    let bs = match (top_bs, bottom_bs) {
        (Some(t), Some(b)) => Some(t.union(b)),
        (Some(t), None) => Some(t),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };

    geo.attributes.position = Some(GeometryAttribute::new(
        ComponentDatatype::Double,
        3,
        positions,
    ));
    geo.attributes.apply_offset = apply_offset;
    geo.indices = indices;
    geo.primitive_type = PrimitiveType::Lines;
    geo.bounding_sphere = bs;

    geo
}
