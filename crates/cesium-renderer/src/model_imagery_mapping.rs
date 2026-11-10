//! ModelImageryMapping matching CesiumJS `Scene/Model/ModelImageryMapping.js` and `MappedPositions.js` (Apache-2.0).

use cesium_math::{
    AttributeType, BoundingRectangle, Cartesian2, Cartesian3, Cartographic, ComponentDatatype,
    DeveloperError, Ellipsoid, GeoRectangle, MapProjection, Matrix4,
};
use glam::DVec3;

use crate::model_components::Attribute;

/// A collection of cartographic positions (and their bounding rectangle) that
/// have been computed from cartesian positions, for a specific ellipsoid.
#[derive(Clone, Debug, PartialEq)]
pub struct MappedPositions {
    pub cartographic_positions: Vec<Cartographic>,
    pub num_positions: usize,
    pub cartographic_bounding_rectangle: GeoRectangle,
    pub ellipsoid: Ellipsoid,
}

impl MappedPositions {
    pub fn new(
        cartographic_positions: Vec<Cartographic>,
        num_positions: usize,
        cartographic_bounding_rectangle: GeoRectangle,
        ellipsoid: Ellipsoid,
    ) -> Result<Self, DeveloperError> {
        Ok(Self {
            cartographic_positions,
            num_positions,
            cartographic_bounding_rectangle,
            ellipsoid,
        })
    }
}

/// A class for computing the texture coordinates of imagery that is
/// supposed to be mapped on a ModelComponents::Primitive.
pub struct ModelImageryMapping;

impl ModelImageryMapping {
    /// Creates a typed array that contains texture coordinates for the given MappedPositions, using the given projection.
    pub fn create_texture_coordinates_for_mapped_positions(
        mapped_positions: Option<&MappedPositions>,
        projection: Option<&impl MapProjection>,
    ) -> Result<Vec<f32>, DeveloperError> {
        let mapped =
            mapped_positions.ok_or_else(|| DeveloperError::new("mappedPositions is required"))?;
        let proj = projection.ok_or_else(|| DeveloperError::new("projection is required"))?;

        Self::create_texture_coordinates(
            Some(&mapped.cartographic_positions),
            Some(mapped.num_positions as i32),
            Some(&mapped.cartographic_bounding_rectangle),
            Some(proj),
        )
    }

    /// Creates the Attribute for the texture coordinates for a primitive.
    pub fn create_texture_coordinates_attribute_for_mapped_positions(
        mapped_positions: Option<&MappedPositions>,
        projection: Option<&impl MapProjection>,
    ) -> Result<Attribute, DeveloperError> {
        let mapped =
            mapped_positions.ok_or_else(|| DeveloperError::new("mappedPositions is required"))?;
        let proj = projection.ok_or_else(|| DeveloperError::new("projection is required"))?;

        let tex_coords =
            Self::create_texture_coordinates_for_mapped_positions(Some(mapped), Some(proj))?;
        Self::create_tex_coord_attribute(Some(&tex_coords))
    }

    /// Creates a typed array that contains texture coordinates for a primitive with the given positions.
    pub fn create_texture_coordinates(
        cartographic_positions: Option<&[Cartographic]>,
        num_positions: Option<i32>,
        cartographic_bounding_rectangle: Option<&GeoRectangle>,
        projection: Option<&impl MapProjection>,
    ) -> Result<Vec<f32>, DeveloperError> {
        let positions = cartographic_positions
            .ok_or_else(|| DeveloperError::new("cartographicPositions is required"))?;
        let num_pos = num_positions.ok_or_else(|| DeveloperError::new("numPositions is required"))?;
        if num_pos < 0 {
            return Err(DeveloperError::new("numPositions must be greater than or equal to 0"));
        }
        let bounding_rect = cartographic_bounding_rectangle
            .ok_or_else(|| DeveloperError::new("cartographicBoundingRectangle is required"))?;
        let proj = projection.ok_or_else(|| DeveloperError::new("projection is required"))?;

        let bounding_rectangle =
            BoundingRectangle::from_rectangle(Some(bounding_rect), Some(proj));
        let projected = Self::create_projected_positions(Some(positions), Some(proj))?;
        let tex_coords = Self::compute_tex_coords(Some(&projected), Some(&bounding_rectangle))?;
        Self::create_typed_array_from_cartesians2(Some(num_pos), Some(&tex_coords))
    }

    /// Creates an iterable over Cartesian3 objects from the given typed array.
    pub fn create_iterable_cartesian3_from_typed_array(
        typed_array: Option<&[f32]>,
        stride: Option<usize>,
    ) -> Result<Vec<Cartesian3>, DeveloperError> {
        let arr = typed_array.ok_or_else(|| DeveloperError::new("typedArray is required"))?;
        let strd = stride.ok_or_else(|| DeveloperError::new("stride is required"))?;
        if strd < 3 {
            return Err(DeveloperError::new("stride must be at least 3"));
        }

        let num_elements = arr.len() / strd;
        let mut result = Vec::with_capacity(num_elements);
        for i in 0..num_elements {
            let x = arr[i * strd] as f64;
            let y = arr[i * strd + 1] as f64;
            let z = arr[i * strd + 2] as f64;
            result.push(Cartesian3::new(x, y, z));
        }
        Ok(result)
    }

    /// Creates a new vector that applies the given mapper to the given slice.
    pub fn map<T, R, F>(iterable: Option<&[T]>, mapper: Option<F>) -> Result<Vec<R>, DeveloperError>
    where
        F: Fn(&T) -> R,
    {
        let items = iterable.ok_or_else(|| DeveloperError::new("iterable is required"))?;
        let func = mapper.ok_or_else(|| DeveloperError::new("mapper is required"))?;
        Ok(items.iter().map(func).collect())
    }

    /// Computes the bounding rectangle of the given cartographic positions.
    pub fn compute_cartographic_bounding_rectangle(
        cartographic_positions: Option<&[Cartographic]>,
        result: Option<&mut GeoRectangle>,
    ) -> Result<GeoRectangle, DeveloperError> {
        let positions = cartographic_positions
            .ok_or_else(|| DeveloperError::new("cartographicPositions is required"))?;

        let mut north = f64::NEG_INFINITY;
        let mut south = f64::INFINITY;
        let mut east = f64::NEG_INFINITY;
        let mut west = f64::INFINITY;

        for pos in positions {
            north = north.max(pos.latitude);
            south = south.min(pos.latitude);
            east = east.max(pos.longitude);
            west = west.min(pos.longitude);
        }

        let rect = GeoRectangle::new(west, south, east, north);
        if let Some(res) = result {
            *res = rect;
        }
        Ok(rect)
    }

    /// Creates transformed Cartesian3 objects by applying the given matrix.
    pub fn transform_cartesians3(
        positions: Option<&[Cartesian3]>,
        matrix: Option<&Matrix4>,
    ) -> Result<Vec<Cartesian3>, DeveloperError> {
        let pos = positions.ok_or_else(|| DeveloperError::new("positions is required"))?;
        let mat = matrix.ok_or_else(|| DeveloperError::new("matrix is required"))?;

        let mut result = Vec::with_capacity(pos.len());
        for p in pos {
            result.push(mat.multiply_by_point(*p));
        }
        Ok(result)
    }

    /// Converts Cartesian3 positions to Cartographic positions using the ellipsoid.
    pub fn transform_to_cartographic(
        positions: Option<&[Cartesian3]>,
        ellipsoid: Option<&Ellipsoid>,
    ) -> Result<Vec<Cartographic>, DeveloperError> {
        let pos = positions.ok_or_else(|| DeveloperError::new("positions is required"))?;
        let ell = ellipsoid.ok_or_else(|| DeveloperError::new("ellipsoid is required"))?;

        let mut result = Vec::with_capacity(pos.len());
        for p in pos {
            let dvec = DVec3::new(p.x, p.y, p.z);
            let cart = ell.cartesian_to_cartographic(dvec).unwrap_or_default();
            result.push(cart);
        }
        Ok(result)
    }

    /// Applies projection to cartographic positions.
    pub fn create_projected_positions(
        cartographic_positions: Option<&[Cartographic]>,
        projection: Option<&impl MapProjection>,
    ) -> Result<Vec<Cartesian3>, DeveloperError> {
        let pos = cartographic_positions
            .ok_or_else(|| DeveloperError::new("cartographicPositions is required"))?;
        let proj = projection.ok_or_else(|| DeveloperError::new("projection is required"))?;

        let mut result = Vec::with_capacity(pos.len());
        for c in pos {
            let p = proj
                .project(*c)
                .map(|v| Cartesian3::new(v.x, v.y, v.z))
                .unwrap_or(Cartesian3::ZERO);
            result.push(p);
        }
        Ok(result)
    }

    /// Computes texture coordinates relative to a bounding rectangle.
    pub fn compute_tex_coords(
        positions: Option<&[Cartesian3]>,
        bounding_rectangle: Option<&BoundingRectangle>,
    ) -> Result<Vec<Cartesian2>, DeveloperError> {
        let pos = positions.ok_or_else(|| DeveloperError::new("positions is required"))?;
        let bounds =
            bounding_rectangle.ok_or_else(|| DeveloperError::new("boundingRectangle is required"))?;

        let inv_size_x = 1.0 / bounds.width;
        let inv_size_y = 1.0 / bounds.height;

        let mut result = Vec::with_capacity(pos.len());
        for p in pos {
            let u_raw = (p.x - bounds.x) * inv_size_x;
            let v_raw = (p.y - bounds.y) * inv_size_y;
            let u = u_raw.clamp(0.0, 1.0);
            let v = v_raw.clamp(0.0, 1.0);
            result.push(Cartesian2::new(u, v));
        }
        Ok(result)
    }

    /// Creates a typed array from Cartesian2 objects.
    pub fn create_typed_array_from_cartesians2(
        num_elements: Option<i32>,
        elements: Option<&[Cartesian2]>,
    ) -> Result<Vec<f32>, DeveloperError> {
        let num = num_elements.ok_or_else(|| DeveloperError::new("numElements is required"))?;
        if num < 0 {
            return Err(DeveloperError::new("numElements must be greater than or equal to 0"));
        }
        let elems = elements.ok_or_else(|| DeveloperError::new("elements is required"))?;

        let mut typed_array = vec![0.0f32; (num as usize) * 2];
        for (i, elem) in elems.iter().enumerate().take(num as usize) {
            typed_array[i * 2] = elem.x as f32;
            typed_array[i * 2 + 1] = elem.y as f32;
        }
        Ok(typed_array)
    }

    /// Creates a texture coordinates attribute from the given typed array.
    pub fn create_tex_coord_attribute(
        tex_coords_typed_array: Option<&[f32]>,
    ) -> Result<Attribute, DeveloperError> {
        let arr = tex_coords_typed_array
            .ok_or_else(|| DeveloperError::new("texCoordsTypedArray is required"))?;

        let byte_slice = bytemuck::cast_slice(arr).to_vec();
        Ok(Attribute {
            name: Some("Imagery Texture Coordinates".to_string()),
            semantic: Some("TEXCOORD".to_string()),
            set_index: Some(0),
            component_datatype: Some(ComponentDatatype::Float),
            attribute_type: Some(AttributeType::Vec2),
            normalized: false,
            count: arr.len() / 2,
            min: None,
            max: None,
            constant: Some(vec![0.0, 0.0]),
            quantization: None,
            typed_array: Some(byte_slice),
            buffer: None,
            byte_offset: 0,
            byte_stride: None,
        })
    }
}
